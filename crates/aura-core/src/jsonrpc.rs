//! Newline-delimited JSON-RPC 2.0 peer (the `"jsonrpc"` header is optional on
//! the wire, as in the Codex app-server protocol).
//!
//! One [`Peer`] multiplexes outgoing requests, outgoing notifications and
//! incoming requests/notifications over any async byte stream. It is used for
//! the Codex app-server (stdio) and for `aura-worker` (stdio).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, mpsc, oneshot};

#[derive(Debug, Clone, Error, PartialEq, Serialize, Deserialize)]
pub enum RpcError {
    #[error("remote error {code}: {message}")]
    Remote {
        code: i64,
        message: String,
        data: Option<Value>,
    },
    #[error("connection closed")]
    Closed,
    #[error("request timed out")]
    Timeout,
    #[error("invalid message: {0}")]
    Decode(String),
}

impl RpcError {
    pub fn remote(code: i64, message: impl Into<String>) -> Self {
        RpcError::Remote {
            code,
            message: message.into(),
            data: None,
        }
    }
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL: i64 = -32603;
}

#[derive(Debug)]
pub enum Incoming {
    Notification {
        method: String,
        params: Value,
    },
    Request {
        method: String,
        params: Value,
        responder: Responder,
    },
}

/// Replies to one incoming request. Dropping it without replying sends an
/// internal error so the other side never hangs.
#[derive(Debug)]
pub struct Responder {
    id: Value,
    out: mpsc::UnboundedSender<String>,
    done: bool,
}

impl Responder {
    pub fn id(&self) -> &Value {
        &self.id
    }
    pub fn ok(mut self, result: Value) {
        self.done = true;
        let _ = self
            .out
            .send(json!({"id": self.id, "result": result}).to_string());
    }
    /// Forgets the request without replying (the other side already
    /// considers it resolved).
    pub fn dismiss(mut self) {
        self.done = true;
    }
    pub fn err(mut self, code: i64, message: impl Into<String>) {
        self.done = true;
        let msg = json!({"id": self.id, "error": {"code": code, "message": message.into()}});
        let _ = self.out.send(msg.to_string());
    }
}

impl Drop for Responder {
    fn drop(&mut self) {
        if !self.done {
            let msg = json!({"id": self.id, "error": {"code": RpcError::INTERNAL, "message": "request dropped"}});
            let _ = self.out.send(msg.to_string());
        }
    }
}

type Pending = Arc<Mutex<HashMap<i64, oneshot::Sender<Result<Value, RpcError>>>>>;

/// Cheap to clone handle for sending requests and notifications.
#[derive(Clone)]
pub struct Peer {
    out: mpsc::UnboundedSender<String>,
    pending: Pending,
    next_id: Arc<AtomicI64>,
    include_jsonrpc_header: bool,
    tasks: Arc<[tokio::task::AbortHandle; 2]>,
}

impl Peer {
    /// Spawns reader/writer tasks. Returns the peer and the stream of incoming
    /// messages; the stream ends when the connection closes.
    pub fn spawn<R, W>(reader: R, writer: W) -> (Peer, mpsc::UnboundedReceiver<Incoming>)
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        Self::spawn_with(reader, writer, false)
    }

    pub fn spawn_with<R, W>(
        reader: R,
        mut writer: W,
        include_jsonrpc_header: bool,
    ) -> (Peer, mpsc::UnboundedReceiver<Incoming>)
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
        let (in_tx, in_rx) = mpsc::unbounded_channel::<Incoming>();
        let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

        let writer_task = tokio::spawn(async move {
            while let Some(line) = out_rx.recv().await {
                if writer.write_all(line.as_bytes()).await.is_err()
                    || writer.write_all(b"\n").await.is_err()
                    || writer.flush().await.is_err()
                {
                    break;
                }
            }
        });

        let reader_pending = pending.clone();
        let reply_tx = out_tx.clone();
        let reader_task = tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                dispatch(&line, &reader_pending, &in_tx, &reply_tx).await;
            }
            let mut map = reader_pending.lock().await;
            for (_, tx) in map.drain() {
                let _ = tx.send(Err(RpcError::Closed));
            }
        });

        (
            Peer {
                out: out_tx,
                pending,
                next_id: Arc::new(AtomicI64::new(1)),
                include_jsonrpc_header,
                tasks: Arc::new([writer_task.abort_handle(), reader_task.abort_handle()]),
            },
            in_rx,
        )
    }

    fn frame(&self, mut v: Value) -> String {
        if self.include_jsonrpc_header {
            v["jsonrpc"] = json!("2.0");
        }
        v.to_string()
    }

    pub fn is_closed(&self) -> bool {
        self.out.is_closed() || self.tasks.iter().any(|t| t.is_finished())
    }

    /// Closes the connection: both directions stop and pending requests fail.
    pub fn close(&self) {
        for t in self.tasks.iter() {
            t.abort();
        }
        let pending = self.pending.clone();
        tokio::spawn(async move {
            for (_, tx) in pending.lock().await.drain() {
                let _ = tx.send(Err(RpcError::Closed));
            }
        });
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);
        let msg = self.frame(json!({"id": id, "method": method, "params": params}));
        if self.out.send(msg).is_err() {
            self.pending.lock().await.remove(&id);
            return Err(RpcError::Closed);
        }
        rx.await.unwrap_or(Err(RpcError::Closed))
    }

    pub async fn request_timeout(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, RpcError> {
        match tokio::time::timeout(timeout, self.request(method, params)).await {
            Ok(r) => r,
            Err(_) => Err(RpcError::Timeout),
        }
    }

    pub fn notify(&self, method: &str, params: Value) -> Result<(), RpcError> {
        let msg = self.frame(json!({"method": method, "params": params}));
        self.out.send(msg).map_err(|_| RpcError::Closed)
    }
}

async fn dispatch(
    line: &str,
    pending: &Pending,
    incoming: &mpsc::UnboundedSender<Incoming>,
    out: &mpsc::UnboundedSender<String>,
) {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        tracing::warn!(target: "jsonrpc", "ignoring non-JSON line ({} bytes)", line.len());
        return;
    };
    let method = v.get("method").and_then(Value::as_str).map(str::to_string);
    let id = v.get("id").cloned().filter(|id| !id.is_null());
    match (method, id) {
        (Some(method), Some(id)) => {
            let params = v.get("params").cloned().unwrap_or(Value::Null);
            let responder = Responder {
                id,
                out: out.clone(),
                done: false,
            };
            let _ = incoming.send(Incoming::Request {
                method,
                params,
                responder,
            });
        }
        (Some(method), None) => {
            let params = v.get("params").cloned().unwrap_or(Value::Null);
            let _ = incoming.send(Incoming::Notification { method, params });
        }
        (None, Some(id)) => {
            let Some(id) = id.as_i64() else { return };
            let Some(tx) = pending.lock().await.remove(&id) else {
                return;
            };
            let result = if let Some(err) = v.get("error") {
                Err(RpcError::Remote {
                    code: err
                        .get("code")
                        .and_then(Value::as_i64)
                        .unwrap_or(RpcError::INTERNAL),
                    message: err
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    data: err.get("data").cloned(),
                })
            } else {
                Ok(v.get("result").cloned().unwrap_or(Value::Null))
            };
            let _ = tx.send(result);
        }
        (None, None) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two peers connected back to back.
    fn pair() -> (
        (Peer, mpsc::UnboundedReceiver<Incoming>),
        (Peer, mpsc::UnboundedReceiver<Incoming>),
    ) {
        let (a, b) = tokio::io::duplex(64 * 1024);
        let (ar, aw) = tokio::io::split(a);
        let (br, bw) = tokio::io::split(b);
        (Peer::spawn(ar, aw), Peer::spawn(br, bw))
    }

    #[tokio::test]
    async fn request_is_correlated_with_its_response() {
        let ((client, _), (_server, mut server_in)) = pair();
        tokio::spawn(async move {
            while let Some(Incoming::Request {
                method,
                params,
                responder,
            }) = server_in.recv().await
            {
                responder.ok(json!({"echo": method, "n": params["n"]}));
            }
        });
        let (r1, r2) = tokio::join!(
            client.request("a", json!({"n": 1})),
            client.request("b", json!({"n": 2}))
        );
        assert_eq!(r1.unwrap(), json!({"echo": "a", "n": 1}));
        assert_eq!(r2.unwrap(), json!({"echo": "b", "n": 2}));
    }

    #[tokio::test]
    async fn remote_errors_are_surfaced() {
        let ((client, _), (_server, mut server_in)) = pair();
        tokio::spawn(async move {
            if let Some(Incoming::Request { responder, .. }) = server_in.recv().await {
                responder.err(-32600, "bad");
            }
        });
        let err = client.request("x", Value::Null).await.unwrap_err();
        assert_eq!(
            err,
            RpcError::Remote {
                code: -32600,
                message: "bad".into(),
                data: None
            }
        );
    }

    #[tokio::test]
    async fn notifications_and_server_requests_reach_the_client() {
        let ((client, mut client_in), (server, _server_in)) = pair();
        server
            .notify("turn/started", json!({"turn": {"id": "t1"}}))
            .unwrap();
        let s2 = server.clone();
        let asked =
            tokio::spawn(async move { s2.request("item/tool/call", json!({"q": 1})).await });
        let mut got_notification = false;
        for _ in 0..2 {
            match client_in.recv().await.unwrap() {
                Incoming::Notification { method, .. } => {
                    assert_eq!(method, "turn/started");
                    got_notification = true;
                }
                Incoming::Request {
                    method, responder, ..
                } => {
                    assert_eq!(method, "item/tool/call");
                    responder.ok(json!("done"));
                }
            }
        }
        assert!(got_notification);
        assert_eq!(asked.await.unwrap().unwrap(), json!("done"));
        drop(client);
    }

    #[tokio::test]
    async fn pending_requests_fail_when_the_connection_closes() {
        let (a, b) = tokio::io::duplex(1024);
        let (ar, aw) = tokio::io::split(a);
        let (client, _in) = Peer::spawn(ar, aw);
        let fut = tokio::spawn(async move { client.request("x", Value::Null).await });
        tokio::time::sleep(Duration::from_millis(20)).await;
        drop(b);
        assert_eq!(fut.await.unwrap().unwrap_err(), RpcError::Closed);
    }

    #[tokio::test]
    async fn dropped_responder_answers_with_internal_error() {
        let ((client, _), (_server, mut server_in)) = pair();
        tokio::spawn(async move {
            if let Some(Incoming::Request { responder, .. }) = server_in.recv().await {
                drop(responder);
            }
        });
        let err = client.request("x", Value::Null).await.unwrap_err();
        assert!(matches!(
            err,
            RpcError::Remote {
                code: RpcError::INTERNAL,
                ..
            }
        ));
    }
}
