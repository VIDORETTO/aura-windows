//! App-server lifecycle (002 TK-001, AC-009/AC-010).
//!
//! * Lazy start: the process starts on the first [`AppServerSupervisor::ensure_running`].
//! * Idle stop: after `idle` without active turns or pending requests.
//! * Crash restart: backoff 1 s, 2 s, 4 s; more than `max_restarts` crashes
//!   within `window` moves to `Failed` until [`AppServerSupervisor::reset`].

use crate::events::AppServerState;
use crate::launcher::Launcher;
use aura_core::jsonrpc::{Incoming, Peer, RpcError};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::process::Child;
use tokio::sync::{Mutex, broadcast, mpsc};
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub idle: Duration,
    pub backoff: Vec<Duration>,
    pub max_restarts: usize,
    pub window: Duration,
    pub handshake_timeout: Duration,
    pub client_name: String,
    pub client_title: String,
    pub client_version: String,
    pub experimental_api: bool,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            idle: Duration::from_secs(15 * 60),
            backoff: vec![
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(4),
            ],
            max_restarts: 3,
            window: Duration::from_secs(60),
            handshake_timeout: Duration::from_secs(20),
            client_name: "aura_desktop".into(),
            client_title: "Aura".into(),
            client_version: env!("CARGO_PKG_VERSION").into(),
            experimental_api: false,
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum SupervisorError {
    #[error("could not start app-server: {0}")]
    Spawn(String),
    #[error("handshake failed: {0}")]
    Handshake(RpcError),
    #[error("app-server keeps crashing; restart Aura or check diagnostics")]
    CrashLoop,
    #[error("app-server unavailable")]
    Unavailable,
}

/// Signals for the conversation service.
#[derive(Debug, Clone, PartialEq)]
pub enum SupervisorSignal {
    State(AppServerState),
    /// The process died unexpectedly; in-flight turns must be failed.
    Crashed,
}

struct Live {
    peer: Peer,
    child: Option<Child>,
    generation: u64,
}

struct State {
    live: Option<Live>,
    status: AppServerState,
    crashes: VecDeque<Instant>,
    active_turns: usize,
    pending_requests: usize,
    last_activity: Instant,
    restarting: bool,
}

struct Inner {
    launcher: Arc<dyn Launcher>,
    cfg: SupervisorConfig,
    state: Mutex<State>,
    signals: broadcast::Sender<SupervisorSignal>,
    incoming_tx: mpsc::UnboundedSender<Incoming>,
    generation: AtomicU64,
    launches: AtomicU64,
}

#[derive(Clone)]
pub struct AppServerSupervisor {
    inner: Arc<Inner>,
}

impl AppServerSupervisor {
    /// `incoming_tx` receives every notification and server request from the
    /// app-server (whatever process generation it comes from).
    pub fn new(
        launcher: Arc<dyn Launcher>,
        cfg: SupervisorConfig,
        incoming_tx: mpsc::UnboundedSender<Incoming>,
    ) -> Self {
        let (signals, _) = broadcast::channel(64);
        let sup = Self {
            inner: Arc::new(Inner {
                launcher,
                cfg,
                state: Mutex::new(State {
                    live: None,
                    status: AppServerState::Stopped,
                    crashes: VecDeque::new(),
                    active_turns: 0,
                    pending_requests: 0,
                    last_activity: Instant::now(),
                    restarting: false,
                }),
                signals,
                incoming_tx,
                generation: AtomicU64::new(0),
                launches: AtomicU64::new(0),
            }),
        };
        sup.spawn_idle_watchdog();
        sup
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SupervisorSignal> {
        self.inner.signals.subscribe()
    }

    pub async fn status(&self) -> AppServerState {
        self.inner.state.lock().await.status.clone()
    }

    /// Number of processes launched so far (diagnostics and tests).
    pub fn launch_count(&self) -> u64 {
        self.inner.launches.load(Ordering::Relaxed)
    }

    fn set_status(&self, st: &mut State, status: AppServerState) {
        st.status = status.clone();
        let _ = self.inner.signals.send(SupervisorSignal::State(status));
    }

    /// Returns a connected, initialized peer, starting the process if needed.
    pub async fn ensure_running(&self) -> Result<Peer, SupervisorError> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            {
                let mut st = self.inner.state.lock().await;
                st.last_activity = Instant::now();
                if let Some(live) = &st.live
                    && !live.peer.is_closed()
                {
                    return Ok(live.peer.clone());
                }
                if let AppServerState::Failed { .. } = st.status {
                    return Err(SupervisorError::CrashLoop);
                }
                if !st.restarting {
                    return self.start_locked(&mut st).await;
                }
            }
            // A crash restart is in progress; wait for it.
            if Instant::now() >= deadline {
                return Err(SupervisorError::Unavailable);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn start_locked(&self, st: &mut State) -> Result<Peer, SupervisorError> {
        self.set_status(st, AppServerState::Starting);
        let conn = match self.inner.launcher.launch() {
            Ok(c) => c,
            Err(e) => {
                self.set_status(st, AppServerState::Stopped);
                return Err(SupervisorError::Spawn(e.to_string()));
            }
        };
        self.inner.launches.fetch_add(1, Ordering::Relaxed);
        let generation = self.inner.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let peer = conn.peer.clone();

        // Forward incoming traffic; detect process exit when the stream ends.
        let mut incoming = conn.incoming;
        let tx = self.inner.incoming_tx.clone();
        let me = self.clone();
        tokio::spawn(async move {
            while let Some(msg) = incoming.recv().await {
                let _ = tx.send(msg);
            }
            me.on_exit(generation).await;
        });

        let init = json!({
            "clientInfo": {
                "name": self.inner.cfg.client_name,
                "title": self.inner.cfg.client_title,
                "version": self.inner.cfg.client_version,
            },
            "capabilities": { "experimentalApi": self.inner.cfg.experimental_api },
        });
        let result = peer
            .request_timeout("initialize", init, self.inner.cfg.handshake_timeout)
            .await;
        let version = match result {
            Ok(v) => v
                .get("userAgent")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            Err(e) => {
                self.set_status(st, AppServerState::Stopped);
                return Err(SupervisorError::Handshake(e));
            }
        };
        let _ = peer.notify("initialized", json!({}));
        st.live = Some(Live {
            peer: peer.clone(),
            child: conn.child,
            generation,
        });
        st.last_activity = Instant::now();
        self.set_status(st, AppServerState::Ready { version });
        Ok(peer)
    }

    /// Boxed so the start → exit → restart cycle has a nameable `Send` future.
    fn on_exit(&self, generation: u64) -> futures::future::BoxFuture<'static, ()> {
        let me = self.clone();
        Box::pin(async move { me.on_exit_inner(generation).await })
    }

    async fn on_exit_inner(&self, generation: u64) {
        let delay = {
            let mut st = self.inner.state.lock().await;
            let is_current = st.live.as_ref().is_some_and(|l| l.generation == generation);
            if !is_current {
                return; // intentional stop or stale generation
            }
            st.live = None;
            st.active_turns = 0;
            st.pending_requests = 0;
            let _ = self.inner.signals.send(SupervisorSignal::Crashed);
            let now = Instant::now();
            st.crashes.push_back(now);
            while st
                .crashes
                .front()
                .is_some_and(|t| now.duration_since(*t) > self.inner.cfg.window)
            {
                st.crashes.pop_front();
            }
            let attempt = st.crashes.len();
            if attempt > self.inner.cfg.max_restarts {
                self.set_status(
                    &mut st,
                    AppServerState::Failed {
                        reason: "crash loop".into(),
                    },
                );
                return;
            }
            st.restarting = true;
            self.set_status(
                &mut st,
                AppServerState::Restarting {
                    attempt: attempt as u32,
                },
            );
            let idx = (attempt - 1).min(self.inner.cfg.backoff.len().saturating_sub(1));
            self.inner
                .cfg
                .backoff
                .get(idx)
                .copied()
                .unwrap_or(Duration::from_secs(1))
        };
        tokio::time::sleep(delay).await;
        let mut st = self.inner.state.lock().await;
        st.restarting = false;
        if st.live.is_none()
            && let Err(e) = self.start_locked(&mut st).await
        {
            tracing::warn!("app-server restart failed: {e}");
        }
    }

    /// Clears a `Failed` state (user clicked "reconectar").
    pub async fn reset(&self) {
        let mut st = self.inner.state.lock().await;
        st.crashes.clear();
        if matches!(st.status, AppServerState::Failed { .. }) {
            self.set_status(&mut st, AppServerState::Stopped);
        }
    }

    pub async fn touch(&self) {
        self.inner.state.lock().await.last_activity = Instant::now();
    }

    pub async fn turn_started(&self) {
        let mut st = self.inner.state.lock().await;
        st.active_turns += 1;
        st.last_activity = Instant::now();
    }

    pub async fn turn_finished(&self) {
        let mut st = self.inner.state.lock().await;
        st.active_turns = st.active_turns.saturating_sub(1);
        st.last_activity = Instant::now();
    }

    pub async fn set_pending_requests(&self, n: usize) {
        self.inner.state.lock().await.pending_requests = n;
    }

    /// Stops the process intentionally (idle or app exit).
    pub async fn stop(&self) {
        let mut st = self.inner.state.lock().await;
        if let Some(mut live) = st.live.take() {
            live.peer.close();
            if let Some(child) = live.child.as_mut() {
                let _ = child.start_kill();
            }
        }
        st.restarting = false;
        self.set_status(&mut st, AppServerState::Stopped);
    }

    fn spawn_idle_watchdog(&self) {
        let weak = Arc::downgrade(&self.inner);
        let tick =
            (self.inner.cfg.idle / 4).clamp(Duration::from_millis(50), Duration::from_secs(30));
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tick).await;
                let Some(inner) = weak.upgrade() else { break };
                let sup = AppServerSupervisor { inner };
                let should_stop = {
                    let st = sup.inner.state.lock().await;
                    st.live.is_some()
                        && st.active_turns == 0
                        && st.pending_requests == 0
                        && st.last_activity.elapsed() >= sup.inner.cfg.idle
                };
                if should_stop {
                    tracing::info!("stopping idle app-server");
                    sup.stop().await;
                }
            }
        });
    }
}
