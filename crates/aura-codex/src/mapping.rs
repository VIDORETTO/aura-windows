//! Pure mapping from app-server notifications to [`ConversationEvent`]s.

use crate::errors::turn_error_from;
use crate::events::{
    ConversationEvent as E, FileChangeSummary, ItemStatus, PlanStep, ToolKind, TurnStatus,
};
use serde_json::Value;

fn s(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn status_of(item: &Value, completed: bool) -> ItemStatus {
    match item
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "failed" => ItemStatus::Failed,
        "declined" => ItemStatus::Declined,
        "completed" => ItemStatus::Completed,
        "inProgress" => ItemStatus::InProgress,
        _ if completed => ItemStatus::Completed,
        _ => ItemStatus::InProgress,
    }
}

/// Counts `+`/`-` lines of a unified diff (ignores headers).
pub fn diff_stats(diff: &str) -> (u32, u32) {
    let mut added = 0;
    let mut removed = 0;
    for line in diff.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if line.starts_with('+') {
            added += 1;
        } else if line.starts_with('-') {
            removed += 1;
        }
    }
    (added, removed)
}

pub fn file_changes(item: &Value) -> Vec<FileChangeSummary> {
    item.get("changes")
        .and_then(Value::as_array)
        .map(|changes| {
            changes
                .iter()
                .map(|c| {
                    let diff = s(c, "diff");
                    let (added, removed) = diff_stats(&diff);
                    FileChangeSummary {
                        path: s(c, "path"),
                        added,
                        removed,
                        diff,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn map_item(thread_id: &str, item: &Value, completed: bool) -> Vec<E> {
    let item_id = s(item, "id");
    let ty = s(item, "type");
    let status = status_of(item, completed);
    let tool = |kind: ToolKind, title: String, detail: Option<String>| E::ToolCall {
        thread_id: thread_id.to_string(),
        item_id: item_id.clone(),
        kind,
        title,
        status,
        detail,
    };
    match ty.as_str() {
        "agentMessage" if completed => vec![E::MessageCompleted {
            thread_id: thread_id.to_string(),
            item_id: item_id.clone(),
            text: s(item, "text"),
        }],
        "plan" if completed => vec![E::PlanProposed {
            thread_id: thread_id.to_string(),
            item_id: item_id.clone(),
            text: s(item, "text"),
        }],
        "commandExecution" => {
            let output = item
                .get("aggregatedOutput")
                .and_then(Value::as_str)
                .map(str::to_string);
            vec![tool(ToolKind::Command, s(item, "command"), output)]
        }
        "fileChange" => vec![E::FileChanges {
            thread_id: thread_id.to_string(),
            item_id: item_id.clone(),
            changes: file_changes(item),
            status,
        }],
        "mcpToolCall" => {
            let title = format!("{}.{}", s(item, "server"), s(item, "tool"));
            let detail = item
                .get("error")
                .filter(|e| !e.is_null())
                .map(|e| e.to_string());
            vec![tool(ToolKind::Mcp, title, detail)]
        }
        "dynamicToolCall" => vec![tool(ToolKind::Dynamic, s(item, "tool"), None)],
        "webSearch" => vec![tool(ToolKind::WebSearch, s(item, "query"), None)],
        "imageView" => vec![tool(ToolKind::ImageView, s(item, "path"), None)],
        "collabToolCall" => vec![tool(ToolKind::Collab, s(item, "tool"), None)],
        "contextCompaction" if completed => vec![E::Compacted {
            thread_id: thread_id.to_string(),
        }],
        _ => vec![],
    }
}

/// Maps one notification. Unknown methods map to nothing.
pub fn map_notification(method: &str, params: &Value) -> Vec<E> {
    let thread_id = s(params, "threadId");
    match method {
        "turn/started" => {
            let turn_id = params.get("turn").map(|t| s(t, "id")).unwrap_or_default();
            vec![E::TurnStarted { thread_id, turn_id }]
        }
        "item/agentMessage/delta" => vec![E::MessageDelta {
            thread_id,
            item_id: s(params, "itemId"),
            delta: s(params, "delta"),
        }],
        "item/reasoning/summaryTextDelta" | "item/reasoning/textDelta" => vec![E::ReasoningDelta {
            thread_id,
            item_id: s(params, "itemId"),
            delta: s(params, "delta"),
        }],
        "item/started" => params
            .get("item")
            .map(|i| map_item(&thread_id, i, false))
            .unwrap_or_default(),
        "item/completed" => params
            .get("item")
            .map(|i| map_item(&thread_id, i, true))
            .unwrap_or_default(),
        "turn/plan/updated" => {
            let steps = params
                .get("plan")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|p| PlanStep {
                            step: s(p, "step"),
                            status: s(p, "status"),
                        })
                        .collect()
                })
                .unwrap_or_default();
            vec![E::PlanUpdated {
                thread_id,
                turn_id: s(params, "turnId"),
                explanation: params
                    .get("explanation")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                steps,
            }]
        }
        "turn/diff/updated" => vec![E::DiffUpdated {
            thread_id,
            turn_id: s(params, "turnId"),
            diff: s(params, "diff"),
        }],
        "thread/tokenUsage/updated" => {
            let usage = params.get("tokenUsage");
            let used = usage
                .and_then(|u| u.get("last"))
                .and_then(|l| l.get("totalTokens"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let window = usage
                .and_then(|u| u.get("modelContextWindow"))
                .and_then(Value::as_i64);
            vec![E::TokenUsage {
                thread_id,
                used,
                window,
            }]
        }
        "turn/completed" => {
            let turn = params.get("turn").cloned().unwrap_or(Value::Null);
            let status = match s(&turn, "status").as_str() {
                "completed" => TurnStatus::Completed,
                "interrupted" => TurnStatus::Interrupted,
                _ => TurnStatus::Failed,
            };
            let error = turn
                .get("error")
                .filter(|e| !e.is_null())
                .map(turn_error_from);
            vec![E::TurnCompleted {
                thread_id,
                turn_id: s(&turn, "id"),
                status,
                error,
            }]
        }
        "serverRequest/resolved" => {
            let request_id = match params.get("requestId") {
                Some(Value::String(v)) => v.clone(),
                Some(other) => other.to_string(),
                None => String::new(),
            };
            vec![E::RequestResolved { request_id }]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::TurnError;
    use serde_json::json;

    #[test]
    fn streaming_turn_maps_to_events() {
        let seq = [
            (
                "turn/started",
                json!({"threadId": "t", "turn": {"id": "u1", "status": "inProgress"}}),
            ),
            (
                "item/started",
                json!({"threadId": "t", "turnId": "u1", "item": {"type": "agentMessage", "id": "m1", "text": ""}}),
            ),
            (
                "item/agentMessage/delta",
                json!({"threadId": "t", "turnId": "u1", "itemId": "m1", "delta": "Olá"}),
            ),
            (
                "item/agentMessage/delta",
                json!({"threadId": "t", "turnId": "u1", "itemId": "m1", "delta": " mundo"}),
            ),
            (
                "item/completed",
                json!({"threadId": "t", "turnId": "u1", "item": {"type": "agentMessage", "id": "m1", "text": "Olá mundo"}}),
            ),
            (
                "turn/completed",
                json!({"threadId": "t", "turn": {"id": "u1", "status": "completed", "error": null}}),
            ),
        ];
        let events: Vec<_> = seq
            .iter()
            .flat_map(|(m, p)| map_notification(m, p))
            .collect();
        assert_eq!(
            events,
            vec![
                E::TurnStarted {
                    thread_id: "t".into(),
                    turn_id: "u1".into()
                },
                E::MessageDelta {
                    thread_id: "t".into(),
                    item_id: "m1".into(),
                    delta: "Olá".into()
                },
                E::MessageDelta {
                    thread_id: "t".into(),
                    item_id: "m1".into(),
                    delta: " mundo".into()
                },
                E::MessageCompleted {
                    thread_id: "t".into(),
                    item_id: "m1".into(),
                    text: "Olá mundo".into()
                },
                E::TurnCompleted {
                    thread_id: "t".into(),
                    turn_id: "u1".into(),
                    status: TurnStatus::Completed,
                    error: None
                },
            ]
        );
    }

    #[test]
    fn failed_turn_carries_error_category() {
        let p = json!({"threadId": "t", "turn": {"id": "u", "status": "failed",
            "error": {"message": "x", "codexErrorInfo": "contextWindowExceeded"}}});
        assert_eq!(
            map_notification("turn/completed", &p),
            vec![E::TurnCompleted {
                thread_id: "t".into(),
                turn_id: "u".into(),
                status: TurnStatus::Failed,
                error: Some(TurnError::ContextTooLong)
            }]
        );
    }

    #[test]
    fn file_change_items_have_stats() {
        let p = json!({"threadId": "t", "item": {"type": "fileChange", "id": "f", "status": "completed", "changes": [
            {"path": "a.md", "kind": "update", "diff": "--- a/a.md\n+++ b/a.md\n+x\n+y\n+z\n-w\n"},
            {"path": "b.txt", "kind": "add", "diff": "+1\n+2\n"}]}});
        let E::FileChanges { changes, .. } = &map_notification("item/completed", &p)[0] else {
            panic!()
        };
        assert_eq!((changes[0].added, changes[0].removed), (3, 1));
        assert_eq!((changes[1].added, changes[1].removed), (2, 0));
    }

    #[test]
    fn plan_usage_and_compaction() {
        let plan = json!({"threadId": "t", "turnId": "u", "plan": [
            {"step": "a", "status": "completed"}, {"step": "b", "status": "inProgress"}, {"step": "c", "status": "pending"}]});
        let E::PlanUpdated { steps, .. } = &map_notification("turn/plan/updated", &plan)[0] else {
            panic!()
        };
        assert_eq!(steps.len(), 3);
        let usage = json!({"threadId": "t", "turnId": "u", "tokenUsage": {"last": {"totalTokens": 8000}, "total": {}, "modelContextWindow": 200000}});
        assert_eq!(
            map_notification("thread/tokenUsage/updated", &usage),
            vec![E::TokenUsage {
                thread_id: "t".into(),
                used: 8000,
                window: Some(200000)
            }]
        );
        let compact = json!({"threadId": "t", "item": {"type": "contextCompaction", "id": "c"}});
        assert_eq!(
            map_notification("item/completed", &compact),
            vec![E::Compacted {
                thread_id: "t".into()
            }]
        );
    }

    #[test]
    fn mcp_tool_call_title() {
        let p = json!({"threadId": "t", "item": {"type": "mcpToolCall", "id": "i", "server": "aura", "tool": "screen_capture", "status": "completed"}});
        let E::ToolCall { title, kind, .. } = &map_notification("item/completed", &p)[0] else {
            panic!()
        };
        assert_eq!(title, "aura.screen_capture");
        assert_eq!(*kind, ToolKind::Mcp);
    }
}
