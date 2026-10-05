//! Server-initiated requests (approvals, user input, elicitation) and their
//! replies (AC-018..AC-021 of 002). OT-004: nothing here answers on its own;
//! a reply only happens through [`PendingRequests::respond`].

use crate::events::{ApprovalKind, ConversationEvent};
use crate::mapping::file_changes;
use aura_core::jsonrpc::Responder;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Decision {
    Accept,
    AcceptForSession,
    Decline,
    Cancel,
    /// Answers for `item/tool/requestUserInput` / elicitation forms.
    Answer {
        content: Value,
    },
    /// Subset of requested permissions (`item/permissions/requestApproval`).
    Grant {
        permissions: Value,
        session: bool,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ApprovalError {
    #[error("request not found or already answered")]
    NotFound,
    #[error("decision not valid for this request")]
    InvalidDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestType {
    Command,
    FileChange,
    Permissions,
    UserInput,
    Elicitation,
}

struct Pending {
    ty: RequestType,
    thread_id: Option<String>,
    responder: Responder,
    /// What YOLO (018) answers: permissions only, never questions or forms.
    yolo: Option<Decision>,
}

#[derive(Default)]
pub struct PendingRequests {
    inner: Mutex<HashMap<String, Pending>>,
}

impl PendingRequests {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn ids(&self) -> Vec<String> {
        self.inner.lock().unwrap().keys().cloned().collect()
    }

    /// Registers a server request. Returns the event for the UI, or gives the
    /// responder back when the method is not an approval-type request.
    pub fn register(
        &self,
        method: &str,
        params: &Value,
        responder: Responder,
    ) -> Result<ConversationEvent, Responder> {
        let request_id = match responder.id() {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let thread_id = params
            .get("threadId")
            .and_then(Value::as_str)
            .map(str::to_string);
        let str_field = |k: &str| params.get(k).and_then(Value::as_str).map(str::to_string);
        let (ty, event) = match method {
            "item/commandExecution/requestApproval" => (
                RequestType::Command,
                ConversationEvent::ApprovalRequested {
                    thread_id: thread_id.clone().unwrap_or_default(),
                    request_id: request_id.clone(),
                    kind: ApprovalKind::Command,
                    command: str_field("command"),
                    cwd: str_field("cwd"),
                    reason: str_field("reason"),
                    changes: vec![],
                    options: available(
                        params,
                        &["accept", "acceptForSession", "decline", "cancel"],
                    ),
                },
            ),
            "item/fileChange/requestApproval" => (
                RequestType::FileChange,
                ConversationEvent::ApprovalRequested {
                    thread_id: thread_id.clone().unwrap_or_default(),
                    request_id: request_id.clone(),
                    kind: ApprovalKind::FileChange,
                    command: None,
                    cwd: str_field("grantRoot"),
                    reason: str_field("reason"),
                    changes: params
                        .get("changes")
                        .map(|_| file_changes(params))
                        .unwrap_or_default(),
                    options: vec![
                        "accept".into(),
                        "acceptForSession".into(),
                        "decline".into(),
                        "cancel".into(),
                    ],
                },
            ),
            "item/permissions/requestApproval" => (
                RequestType::Permissions,
                ConversationEvent::UserInputRequested {
                    thread_id: thread_id.clone(),
                    request_id: request_id.clone(),
                    prompt: params.clone(),
                    auto_resolve_ms: None,
                    source: "permissions".into(),
                },
            ),
            "item/tool/requestUserInput" | "tool/requestUserInput" => (
                RequestType::UserInput,
                ConversationEvent::UserInputRequested {
                    thread_id: thread_id.clone(),
                    request_id: request_id.clone(),
                    prompt: params.clone(),
                    auto_resolve_ms: params.get("autoResolutionMs").and_then(Value::as_u64),
                    source: "agent".into(),
                },
            ),
            "mcpServer/elicitation/request" => (
                RequestType::Elicitation,
                ConversationEvent::UserInputRequested {
                    thread_id: thread_id.clone(),
                    request_id: request_id.clone(),
                    prompt: params.clone(),
                    auto_resolve_ms: None,
                    source: str_field("serverName").unwrap_or_else(|| "mcp".into()),
                },
            ),
            _ => return Err(responder),
        };
        let yolo = match ty {
            RequestType::Command | RequestType::FileChange => Some(Decision::Accept),
            RequestType::Permissions => Some(Decision::Grant {
                permissions: params.get("permissions").cloned().unwrap_or(json!({})),
                session: true,
            }),
            RequestType::Elicitation
                if params["_meta"]["codex_approval_kind"].as_str() == Some("mcp_tool_call") =>
            {
                Some(Decision::Answer { content: json!({}) })
            }
            RequestType::Elicitation | RequestType::UserInput => None,
        };
        self.inner.lock().unwrap().insert(
            request_id,
            Pending {
                ty,
                thread_id,
                responder,
                yolo,
            },
        );
        Ok(event)
    }

    /// The answer YOLO gives to a pending request, if it is a permission
    /// (command, file change, permissions, MCP tool call) and not a question.
    pub fn yolo_decision(&self, request_id: &str) -> Option<Decision> {
        self.inner
            .lock()
            .unwrap()
            .get(request_id)
            .and_then(|p| p.yolo.clone())
    }

    /// Sends the user's decision. Consumes the pending request.
    pub fn respond(&self, request_id: &str, decision: Decision) -> Result<(), ApprovalError> {
        let mut map = self.inner.lock().unwrap();
        let ty = map.get(request_id).ok_or(ApprovalError::NotFound)?.ty;
        let payload = match (ty, &decision) {
            (RequestType::Command | RequestType::FileChange, d) => {
                json!({"decision": decision_str(d)?})
            }
            (
                RequestType::Permissions,
                Decision::Grant {
                    permissions,
                    session,
                },
            ) => {
                json!({"permissions": permissions, "scope": if *session { "session" } else { "turn" }})
            }
            (RequestType::Permissions, Decision::Decline | Decision::Cancel) => {
                json!({"permissions": {}, "scope": "turn"})
            }
            (RequestType::UserInput, Decision::Answer { content }) => content.clone(),
            (RequestType::Elicitation, Decision::Answer { content }) => {
                json!({"action": "accept", "content": content})
            }
            (RequestType::Elicitation, Decision::Decline) => {
                json!({"action": "decline", "content": null})
            }
            (RequestType::Elicitation, Decision::Cancel) => {
                json!({"action": "cancel", "content": null})
            }
            _ => return Err(ApprovalError::InvalidDecision),
        };
        let pending = map.remove(request_id).expect("checked above");
        pending.responder.ok(payload);
        Ok(())
    }

    /// Called on `serverRequest/resolved` (answered elsewhere, auto-resolved,
    /// or cleared by the turn ending). Drops without replying twice.
    pub fn resolved(&self, request_id: &str) {
        if let Some(p) = self.inner.lock().unwrap().remove(request_id) {
            p.responder.dismiss();
        }
    }

    /// Pending requests belonging to a thread (for re-rendering on reopen).
    pub fn for_thread(&self, thread_id: &str) -> Vec<String> {
        self.inner
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, p)| p.thread_id.as_deref() == Some(thread_id))
            .map(|(id, _)| id.clone())
            .collect()
    }
}

fn decision_str(d: &Decision) -> Result<&'static str, ApprovalError> {
    Ok(match d {
        Decision::Accept => "accept",
        Decision::AcceptForSession => "acceptForSession",
        Decision::Decline => "decline",
        Decision::Cancel => "cancel",
        _ => return Err(ApprovalError::InvalidDecision),
    })
}

fn available(params: &Value, default: &[&str]) -> Vec<String> {
    params
        .get("availableDecisions")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.iter().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aura_core::jsonrpc::{Incoming, Peer};

    /// Returns a server peer whose requests arrive at the client side as
    /// `Incoming::Request` with a real `Responder`.
    async fn server_request(
        method: &str,
        params: Value,
    ) -> (tokio::task::JoinHandle<Value>, Responder, Value) {
        let (a, b) = tokio::io::duplex(1 << 16);
        let (ar, aw) = tokio::io::split(a);
        let (br, bw) = tokio::io::split(b);
        let (server, _s_in) = Peer::spawn(ar, aw);
        let (_client, mut c_in) = Peer::spawn(br, bw);
        let m = method.to_string();
        let answer = tokio::spawn(async move { server.request(&m, params).await.unwrap() });
        match c_in.recv().await.unwrap() {
            Incoming::Request {
                responder, params, ..
            } => (answer, responder, params),
            _ => panic!("expected request"),
        }
    }

    #[tokio::test]
    async fn command_approval_round_trip() {
        let params = json!({"threadId": "t", "turnId": "u", "itemId": "i", "command": "pip install requests",
                             "cwd": "C:\\w\\x", "reason": "instalar dependência"});
        let (answer, responder, params) =
            server_request("item/commandExecution/requestApproval", params).await;
        let pending = PendingRequests::new();
        let event = pending
            .register("item/commandExecution/requestApproval", &params, responder)
            .unwrap();
        let ConversationEvent::ApprovalRequested {
            request_id,
            command,
            cwd,
            reason,
            ..
        } = event
        else {
            panic!()
        };
        assert_eq!(command.as_deref(), Some("pip install requests"));
        assert_eq!(cwd.as_deref(), Some("C:\\w\\x"));
        assert_eq!(reason.as_deref(), Some("instalar dependência"));
        pending
            .respond(&request_id, Decision::AcceptForSession)
            .unwrap();
        assert_eq!(
            answer.await.unwrap(),
            json!({"decision": "acceptForSession"})
        );
        assert_eq!(
            pending.respond(&request_id, Decision::Accept),
            Err(ApprovalError::NotFound)
        );
    }

    #[tokio::test]
    async fn elicitation_form_and_invalid_decisions() {
        let (answer, responder, params) = server_request(
            "mcpServer/elicitation/request",
            json!({"serverName": "x", "mode": "form"}),
        )
        .await;
        let pending = PendingRequests::new();
        let ConversationEvent::UserInputRequested {
            request_id, source, ..
        } = pending
            .register("mcpServer/elicitation/request", &params, responder)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(source, "x");
        assert_eq!(
            pending.respond(&request_id, Decision::Accept),
            Err(ApprovalError::InvalidDecision)
        );
        pending
            .respond(
                &request_id,
                Decision::Answer {
                    content: json!({"nome": "Ana"}),
                },
            )
            .unwrap();
        assert_eq!(
            answer.await.unwrap(),
            json!({"action": "accept", "content": {"nome": "Ana"}})
        );
    }

    #[tokio::test]
    async fn unknown_methods_are_returned() {
        let (_answer, responder, params) = server_request("item/tool/call", json!({})).await;
        let pending = PendingRequests::new();
        assert!(
            pending
                .register("item/tool/call", &params, responder)
                .is_err()
        );
        assert!(pending.is_empty());
    }
}
