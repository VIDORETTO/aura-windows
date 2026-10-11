//! Protect web inputs on the normalized Responses stream before Codex sees them.
use crate::{
    tool_results::{ToolCallProtector, tool_name},
    upstream::ByteStream,
};
use bytes::Bytes;
use futures::StreamExt;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

const MAX_FRAME: usize = 8 * 1024 * 1024;
const MAX_ARGUMENTS: usize = 64 * 1024;
const MAX_CALLS: usize = 32;

struct Call {
    tool: Option<&'static str>,
    call_id: String,
    index: Value,
    original: Option<String>,
    protected: Option<String>,
}
struct Filter {
    scope: Option<Arc<dyn ToolCallProtector>>,
    calls: HashMap<String, Call>,
    reasoning: HashMap<String, PrivateReasoning>,
}
struct PrivateReasoning {
    index: Value,
    original: Option<Value>,
    protected: Option<Value>,
}
impl Filter {
    fn protects_reasoning(&self) -> bool {
        self.scope
            .as_ref()
            .is_some_and(|scope| scope.protects_reasoning())
    }

    fn reasoning_item(
        &mut self,
        item: &mut Value,
        index: &Value,
        complete: bool,
    ) -> Result<(), ()> {
        let id = item["id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or(())?
            .to_owned();
        if self.calls.contains_key(&id) {
            return Err(());
        }
        if !self.reasoning.contains_key(&id) {
            if self.reasoning.len() >= MAX_CALLS {
                return Err(());
            }
            self.reasoning.insert(
                id.clone(),
                PrivateReasoning {
                    index: index.clone(),
                    original: None,
                    protected: None,
                },
            );
        }
        let record = self.reasoning.get_mut(&id).ok_or(())?;
        if record.index != *index {
            return Err(());
        }
        *item = if complete {
            if let Some(original) = &record.original {
                if original != item {
                    return Err(());
                }
            } else {
                let safe = self
                    .scope
                    .as_ref()
                    .ok_or(())?
                    .protect_reasoning(item)
                    .map_err(|_| ())?;
                record.original = Some(item.clone());
                record.protected = Some(safe);
            }
            record.protected.clone().ok_or(())?
        } else {
            serde_json::json!({"type":"reasoning","id":id,"summary":[],"encrypted_content":null})
        };
        Ok(())
    }

    fn item(&mut self, item: &mut Value, index: &Value, complete: bool) -> Result<(), ()> {
        if item["type"] == "reasoning" && self.protects_reasoning() {
            return self.reasoning_item(item, index, complete);
        }
        if item["type"] != "function_call" {
            return Ok(());
        }
        let tool = tool_name(item);
        let Some(id) = item["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
        else {
            return if tool.is_some() { Err(()) } else { Ok(()) };
        };
        let call_id = item["call_id"].as_str().unwrap_or_default();
        if self.reasoning.contains_key(&id) {
            return Err(());
        }
        if !self.calls.contains_key(&id) {
            if self.calls.len() >= MAX_CALLS {
                return Err(());
            }
            if self
                .calls
                .values()
                .any(|call| call.call_id == call_id || call.index == *index)
            {
                return Err(());
            }
            self.calls.insert(
                id.clone(),
                Call {
                    tool,
                    call_id: call_id.into(),
                    index: index.clone(),
                    original: None,
                    protected: None,
                },
            );
        }
        let call = self.calls.get_mut(&id).ok_or(())?;
        if call.tool != tool || call.call_id != call_id || call.index != *index {
            return Err(());
        }
        let Some(tool) = tool else {
            return Ok(());
        };
        if call_id.is_empty() {
            return Err(());
        }
        let text = item["arguments"].as_str().ok_or(())?;
        let safe = if complete {
            Self::arguments(&self.scope, call, tool, text)?.to_owned()
        } else {
            String::new()
        };
        // Private call snapshots have only protocol fields, no reflected extras.
        let object = item.as_object_mut().ok_or(())?;
        object.retain(|key, _| {
            matches!(
                key.as_str(),
                "type" | "id" | "status" | "call_id" | "namespace" | "name" | "arguments"
            )
        });
        item["arguments"] = Value::String(safe);
        Ok(())
    }
    fn arguments<'a>(
        scope: &Option<Arc<dyn ToolCallProtector>>,
        call: &'a mut Call,
        tool: &str,
        text: &str,
    ) -> Result<&'a str, ()> {
        if text.len() > MAX_ARGUMENTS {
            return Err(());
        }
        if let Some(original) = &call.original {
            if original != text {
                return Err(());
            }
        } else {
            let safe = scope
                .as_ref()
                .ok_or(())?
                .protect(tool, text)
                .map_err(|_| ())?;
            call.original = Some(text.into());
            call.protected = Some(safe);
        }
        call.protected.as_deref().ok_or(())
    }
    fn event(&mut self, block: &[u8]) -> Result<Option<Bytes>, ()> {
        let block = std::str::from_utf8(block).map_err(|_| ())?;
        let mut event = None;
        let mut data = Vec::new();
        for line in block.lines() {
            let Some((field, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.strip_prefix(' ').unwrap_or(value);
            match field {
                "event" => event = Some(value),
                "data" => data.push(value),
                _ => {}
            }
        }
        if data.is_empty() {
            return Ok(None);
        }
        let data = data.join("\n");
        if data == "[DONE]" {
            return Ok(Some(Bytes::from_static(b"data: [DONE]\n\n")));
        }
        let mut value: Value = serde_json::from_str(&data).map_err(|_| ())?;
        let kind = value["type"].as_str().or(event).ok_or(())?.to_owned();
        if event.is_some_and(|event| event != kind) {
            return Err(());
        }
        if self.protects_reasoning()
            && (kind.starts_with("response.reasoning")
                || value["item_id"]
                    .as_str()
                    .is_some_and(|id| self.reasoning.contains_key(id)))
        {
            return Ok(None);
        }
        match kind.as_str() {
            "response.output_item.added" | "response.output_item.done" => {
                let index = value["output_index"].clone();
                self.item(&mut value["item"], &index, kind.ends_with(".done"))?;
            }
            "response.function_call_arguments.delta" | "response.function_call_arguments.done" => {
                let id = value["item_id"].as_str().ok_or(())?;
                let call = self.calls.get_mut(id).ok_or(())?;
                if value["output_index"] != call.index {
                    return Err(());
                }
                if let Some(tool) = call.tool {
                    let (field, safe) = if kind.ends_with(".delta") {
                        ("delta", String::new())
                    } else {
                        let text = value["arguments"].as_str().ok_or(())?;
                        (
                            "arguments",
                            Self::arguments(&self.scope, call, tool, text)?.to_owned(),
                        )
                    };
                    value.as_object_mut().ok_or(())?.retain(|key, _| {
                        key == field
                            || matches!(
                                key.as_str(),
                                "type" | "sequence_number" | "item_id" | "output_index"
                            )
                    });
                    value[field] = Value::String(safe);
                }
            }
            "error" | "response.failed" => return Err(()),
            _ => {}
        }
        // Completed (and resumable created) snapshots can repeat full calls.
        if let Some(output) = value
            .get_mut("response")
            .and_then(|response| response.get_mut("output"))
            .and_then(Value::as_array_mut)
        {
            for (index, item) in output.iter_mut().enumerate() {
                self.item(item, &Value::from(index), kind == "response.completed")?;
            }
        }
        Ok(Some(Bytes::from(format!(
            "event: {kind}\ndata: {value}\n\n"
        ))))
    }
}

/// Errors contain no upstream bytes. A bounded channel propagates cancellation.
pub(crate) fn protect(
    mut input: ByteStream,
    scope: Option<Arc<dyn ToolCallProtector>>,
) -> ByteStream {
    let (tx, rx) = tokio::sync::mpsc::channel(8);
    tokio::spawn(async move {
        let mut filter = Filter {
            scope,
            calls: HashMap::new(),
            reasoning: HashMap::new(),
        };
        let mut pending = Vec::new();
        let mut failed = false;
        'chunks: while let Some(chunk) =
            tokio::select! { _ = tx.closed() => return, chunk = input.next() => chunk }
        {
            let Ok(chunk) = chunk else {
                failed = true;
                break;
            };
            for byte in chunk {
                pending.push(byte);
                if pending.len() > MAX_FRAME {
                    failed = true;
                    break 'chunks;
                }
                if pending.ends_with(b"\n\n") || pending.ends_with(b"\r\n\r\n") {
                    match filter.event(&pending) {
                        Ok(Some(frame)) => {
                            if tx.send(Ok(frame)).await.is_err() {
                                return;
                            }
                        }
                        Ok(None) => {}
                        Err(()) => {
                            failed = true;
                            break 'chunks;
                        }
                    }
                    pending.clear();
                }
            }
        }
        if !failed && !pending.is_empty() {
            match filter.event(&pending) {
                Ok(Some(frame)) => {
                    if tx.send(Ok(frame)).await.is_err() {
                        return;
                    }
                }
                Ok(None) => {}
                Err(()) => failed = true,
            }
        }
        if failed {
            let _ = tx
                .send(Err(std::io::Error::other("Web model response unavailable")))
                .await;
        }
    });
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    }))
}
