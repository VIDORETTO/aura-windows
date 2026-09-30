//! Pending agent consent requests: the MCP tool call waits while the Overlay
//! shows "Permitir uma vez / Permitir nesta Conversa / Negar".

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConsentAnswer {
    Once,
    Conversation,
    Deny,
}

/// How long a tool call waits for the user before failing with `timeout`.
pub const CONSENT_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Default)]
pub struct ConsentBroker {
    pending: Mutex<HashMap<String, oneshot::Sender<ConsentAnswer>>>,
}

impl ConsentBroker {
    pub fn register(&self, id: &str) -> oneshot::Receiver<ConsentAnswer> {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().unwrap().insert(id.to_string(), tx);
        rx
    }

    /// Returns false when the request is unknown or already answered.
    pub fn answer(&self, id: &str, answer: ConsentAnswer) -> bool {
        self.pending
            .lock()
            .unwrap()
            .remove(id)
            .is_some_and(|tx| tx.send(answer).is_ok())
    }

    pub fn forget(&self, id: &str) {
        self.pending.lock().unwrap().remove(id);
    }

    pub fn pending_ids(&self) -> Vec<String> {
        self.pending.lock().unwrap().keys().cloned().collect()
    }
}
