//! Builds a valid Responses API event stream (OT-003 of 003): every stream
//! starts with `response.created` and ends with exactly one
//! `response.completed` or `response.failed`, with each output item opened
//! (`output_item.added`) and closed (`output_item.done`) in order.
//!
//! The event set matches what the Codex client consumes
//! (`codex-rs/codex-api/src/sse/responses.rs`).

use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: i64,
    pub cached_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
}

#[derive(Debug)]
struct OpenText {
    id: String,
    index: usize,
    text: String,
}

#[derive(Debug)]
struct OpenReasoning {
    id: String,
    index: usize,
    text: String,
}

#[derive(Debug)]
struct OpenTool {
    id: String,
    index: usize,
    call_id: String,
    name: String,
    args: String,
    custom: bool,
}

pub struct Emitter {
    response_id: String,
    model: String,
    seq: u64,
    next_index: usize,
    text: Option<OpenText>,
    reasoning: Option<OpenReasoning>,
    tools: BTreeMap<u64, OpenTool>,
    done_items: Vec<(usize, Value)>,
    custom_tools: HashSet<String>,
    /// chat name → (namespace, name) for namespaced tools.
    namespaced: std::collections::HashMap<String, (String, String)>,
    finished: bool,
}

/// One SSE frame ready to be written.
pub fn frame(kind: &str, data: &Value) -> String {
    format!("event: {kind}\ndata: {data}\n\n")
}

fn rid(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}

impl Emitter {
    pub fn new(model: &str, custom_tools: HashSet<String>) -> Self {
        Self {
            response_id: rid("resp"),
            model: model.to_string(),
            seq: 0,
            next_index: 0,
            text: None,
            reasoning: None,
            tools: BTreeMap::new(),
            done_items: Vec::new(),
            custom_tools,
            namespaced: Default::default(),
            finished: false,
        }
    }

    /// Maps flattened chat tool names back to Codex namespaces on output.
    pub fn with_namespaces(
        mut self,
        namespaced: std::collections::HashMap<String, (String, String)>,
    ) -> Self {
        self.namespaced = namespaced;
        self
    }

    /// `function_call` item fields for a (possibly namespaced) tool name.
    fn call_item(
        &self,
        id: &str,
        status: &str,
        call_id: &str,
        name: &str,
        arguments: &str,
    ) -> Value {
        match self.namespaced.get(name) {
            Some((ns, inner)) => {
                json!({"id": id, "type": "function_call", "status": status, "call_id": call_id,
                "name": inner, "namespace": ns, "arguments": arguments})
            }
            None => {
                json!({"id": id, "type": "function_call", "status": status, "call_id": call_id, "name": name, "arguments": arguments})
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn ev(&mut self, kind: &str, mut data: Value) -> String {
        data["type"] = json!(kind);
        data["sequence_number"] = json!(self.seq);
        self.seq += 1;
        frame(kind, &data)
    }

    fn response_obj(&self, status: &str) -> Value {
        json!({"id": self.response_id, "object": "response", "status": status, "model": self.model, "output": []})
    }

    pub fn created(&mut self) -> Vec<String> {
        let r = self.response_obj("in_progress");
        vec![self.ev("response.created", json!({"response": r}))]
    }

    fn close_text(&mut self, out: &mut Vec<String>) {
        if let Some(t) = self.text.take() {
            let item = json!({"id": t.id, "type": "message", "role": "assistant", "status": "completed",
                              "content": [{"type": "output_text", "text": t.text, "annotations": []}]});
            out.push(self.ev("response.output_text.done", json!({"item_id": t.id, "output_index": t.index, "content_index": 0, "text": t.text})));
            out.push(self.ev(
                "response.output_item.done",
                json!({"output_index": t.index, "item": item.clone()}),
            ));
            self.done_items.push((t.index, item));
        }
    }

    fn close_reasoning(&mut self, out: &mut Vec<String>) {
        if let Some(r) = self.reasoning.take() {
            out.push(self.ev(
                "response.reasoning_summary_text.done",
                json!({"item_id": r.id, "output_index": r.index, "summary_index": 0, "text": r.text}),
            ));
            let item = json!({"id": r.id, "type": "reasoning", "summary": [{"type": "summary_text", "text": r.text}], "encrypted_content": null});
            out.push(self.ev(
                "response.output_item.done",
                json!({"output_index": r.index, "item": item.clone()}),
            ));
            self.done_items.push((r.index, item));
        }
    }

    fn close_tools(&mut self, out: &mut Vec<String>) {
        let tools = std::mem::take(&mut self.tools);
        for (_, t) in tools {
            let item = if t.custom {
                let input = serde_json::from_str::<Value>(&t.args)
                    .ok()
                    .and_then(|v| v.get("input").and_then(Value::as_str).map(str::to_string))
                    .unwrap_or(t.args.clone());
                json!({"id": t.id, "type": "custom_tool_call", "status": "completed", "call_id": t.call_id, "name": t.name, "input": input})
            } else {
                out.push(self.ev(
                    "response.function_call_arguments.done",
                    json!({"item_id": t.id, "output_index": t.index, "arguments": t.args}),
                ));
                self.call_item(&t.id, "completed", &t.call_id, &t.name, &t.args)
            };
            out.push(self.ev(
                "response.output_item.done",
                json!({"output_index": t.index, "item": item.clone()}),
            ));
            self.done_items.push((t.index, item));
        }
    }

    pub fn reasoning_delta(&mut self, delta: &str) -> Vec<String> {
        let mut out = Vec::new();
        if delta.is_empty() {
            return out;
        }
        if self.reasoning.is_none() {
            self.close_text(&mut out);
            let id = rid("rs");
            let index = self.next_index;
            self.next_index += 1;
            let item = json!({"id": id, "type": "reasoning", "summary": []});
            out.push(self.ev(
                "response.output_item.added",
                json!({"output_index": index, "item": item}),
            ));
            out.push(self.ev(
                "response.reasoning_summary_part.added",
                json!({"item_id": id, "output_index": index, "summary_index": 0, "part": {"type": "summary_text", "text": ""}}),
            ));
            self.reasoning = Some(OpenReasoning {
                id,
                index,
                text: String::new(),
            });
        }
        let (id, index) = {
            let r = self.reasoning.as_mut().expect("open");
            r.text.push_str(delta);
            (r.id.clone(), r.index)
        };
        out.push(self.ev(
            "response.reasoning_summary_text.delta",
            json!({"item_id": id, "output_index": index, "summary_index": 0, "delta": delta}),
        ));
        out
    }

    pub fn text_delta(&mut self, delta: &str) -> Vec<String> {
        let mut out = Vec::new();
        if delta.is_empty() {
            return out;
        }
        if self.text.is_none() {
            self.close_reasoning(&mut out);
            let id = rid("msg");
            let index = self.next_index;
            self.next_index += 1;
            let item = json!({"id": id, "type": "message", "role": "assistant", "status": "in_progress", "content": []});
            out.push(self.ev(
                "response.output_item.added",
                json!({"output_index": index, "item": item}),
            ));
            out.push(self.ev(
                "response.content_part.added",
                json!({"item_id": id, "output_index": index, "content_index": 0, "part": {"type": "output_text", "text": "", "annotations": []}}),
            ));
            self.text = Some(OpenText {
                id,
                index,
                text: String::new(),
            });
        }
        let (id, index) = {
            let t = self.text.as_mut().expect("open");
            t.text.push_str(delta);
            (t.id.clone(), t.index)
        };
        out.push(self.ev(
            "response.output_text.delta",
            json!({"item_id": id, "output_index": index, "content_index": 0, "delta": delta}),
        ));
        out
    }

    /// Starts (or continues) the tool call with upstream index `key`.
    pub fn tool_call(
        &mut self,
        key: u64,
        call_id: Option<&str>,
        name: Option<&str>,
        args_delta: &str,
    ) -> Vec<String> {
        let mut out = Vec::new();
        if !self.tools.contains_key(&key) {
            self.close_reasoning(&mut out);
            self.close_text(&mut out);
            let index = self.next_index;
            self.next_index += 1;
            let name = name.unwrap_or_default().to_string();
            let custom = self.custom_tools.contains(&name);
            let call_id = call_id.map(str::to_string).unwrap_or_else(|| rid("call"));
            let id = rid("fc");
            let item = if custom {
                json!({"id": id, "type": "custom_tool_call", "status": "in_progress", "call_id": call_id, "name": name, "input": ""})
            } else {
                self.call_item(&id, "in_progress", &call_id, &name, "")
            };
            out.push(self.ev(
                "response.output_item.added",
                json!({"output_index": index, "item": item}),
            ));
            self.tools.insert(
                key,
                OpenTool {
                    id,
                    index,
                    call_id,
                    name,
                    args: String::new(),
                    custom,
                },
            );
        } else if let (Some(n), Some(t)) = (name, self.tools.get_mut(&key))
            && t.name.is_empty()
        {
            t.name = n.to_string();
        }
        if !args_delta.is_empty() {
            let (id, index, custom) = {
                let t = self.tools.get_mut(&key).expect("open");
                t.args.push_str(args_delta);
                (t.id.clone(), t.index, t.custom)
            };
            if !custom {
                out.push(self.ev(
                    "response.function_call_arguments.delta",
                    json!({"item_id": id, "output_index": index, "delta": args_delta}),
                ));
            }
        }
        out
    }

    pub fn complete(&mut self, usage: Option<Usage>) -> Vec<String> {
        let mut out = Vec::new();
        if self.finished {
            return out;
        }
        self.close_reasoning(&mut out);
        self.close_text(&mut out);
        self.close_tools(&mut out);
        self.done_items.sort_by_key(|(i, _)| *i);
        let mut r = self.response_obj("completed");
        r["output"] = Value::Array(self.done_items.iter().map(|(_, v)| v.clone()).collect());
        let u = usage.unwrap_or_default();
        r["usage"] = json!({
            "input_tokens": u.input_tokens,
            "input_tokens_details": {"cached_tokens": u.cached_tokens},
            "output_tokens": u.output_tokens,
            "output_tokens_details": {"reasoning_tokens": u.reasoning_tokens},
            "total_tokens": u.input_tokens + u.output_tokens,
        });
        out.push(self.ev("response.completed", json!({"response": r})));
        self.finished = true;
        out
    }

    pub fn fail(&mut self, code: &str, message: &str) -> Vec<String> {
        if self.finished {
            return vec![];
        }
        self.finished = true;
        let mut r = self.response_obj("failed");
        r["error"] = json!({"code": code, "message": message});
        vec![self.ev("response.failed", json!({"response": r}))]
    }
}

/// Validates a Responses event sequence (used by tests and by the
/// diagnostics self-check).
pub fn validate_sequence(frames: &[Value]) -> Result<(), String> {
    let kinds: Vec<&str> = frames
        .iter()
        .map(|f| f["type"].as_str().unwrap_or(""))
        .collect();
    if kinds.first() != Some(&"response.created") {
        return Err("stream must start with response.created".into());
    }
    let terminal = kinds
        .iter()
        .filter(|k| **k == "response.completed" || **k == "response.failed")
        .count();
    if terminal != 1
        || !matches!(
            kinds.last(),
            Some(&"response.completed") | Some(&"response.failed")
        )
    {
        return Err("stream must end with exactly one terminal event".into());
    }
    let mut open: Vec<u64> = Vec::new();
    for f in frames {
        match f["type"].as_str() {
            Some("response.output_item.added") => {
                open.push(f["output_index"].as_u64().unwrap_or(u64::MAX))
            }
            Some("response.output_item.done") => {
                let idx = f["output_index"].as_u64().unwrap_or(u64::MAX);
                let pos = open
                    .iter()
                    .position(|i| *i == idx)
                    .ok_or(format!("done without added for {idx}"))?;
                open.remove(pos);
            }
            _ => {}
        }
    }
    if !open.is_empty() && kinds.last() == Some(&"response.completed") {
        return Err(format!("items left open: {open:?}"));
    }
    Ok(())
}

/// Parses frames produced by [`frame`] back into JSON values.
pub fn parse_frames(text: &str) -> Vec<Value> {
    let mut p = crate::sse::SseParser::new();
    let mut evs = p.push(text.as_bytes());
    evs.extend(p.finish());
    evs.into_iter()
        .filter_map(|e| serde_json::from_str(&e.data).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasoning_then_text_then_parallel_tools() {
        let mut e = Emitter::new("m", HashSet::new());
        let mut s = e.created();
        s.extend(e.reasoning_delta("pensando"));
        s.extend(e.text_delta("Olá"));
        s.extend(e.tool_call(0, Some("c1"), Some("get_weather"), "{\"city\":"));
        s.extend(e.tool_call(1, Some("c2"), Some("get_time"), "{}"));
        s.extend(e.tool_call(0, None, None, "\"SP\"}"));
        s.extend(e.complete(Some(Usage {
            input_tokens: 10,
            output_tokens: 5,
            ..Default::default()
        })));
        let frames = parse_frames(&s.concat());
        validate_sequence(&frames).unwrap();
        let done = frames.last().unwrap();
        let output = done["response"]["output"].as_array().unwrap();
        assert_eq!(output[0]["type"], "reasoning");
        assert_eq!(output[1]["content"][0]["text"], "Olá");
        assert_eq!(output[2]["arguments"], "{\"city\":\"SP\"}");
        assert_eq!(output[3]["call_id"], "c2");
        assert_eq!(done["response"]["usage"]["total_tokens"], 15);
    }

    #[test]
    fn namespaced_tool_calls_carry_the_namespace() {
        let mut ns = std::collections::HashMap::new();
        ns.insert(
            "mcp__aura__screen_text".to_string(),
            ("mcp__aura".to_string(), "screen_text".to_string()),
        );
        let mut e = Emitter::new("m", HashSet::new()).with_namespaces(ns);
        let mut s = e.created();
        s.extend(e.tool_call(0, Some("c1"), Some("mcp__aura__screen_text"), "{}"));
        s.extend(e.complete(None));
        let frames = parse_frames(&s.concat());
        validate_sequence(&frames).unwrap();
        let item = &frames.last().unwrap()["response"]["output"][0];
        assert_eq!(item["type"], "function_call");
        assert_eq!(item["name"], "screen_text");
        assert_eq!(item["namespace"], "mcp__aura");
    }

    #[test]
    fn custom_tool_is_reconverted() {
        let mut e = Emitter::new("m", ["apply_patch".to_string()].into_iter().collect());
        let mut s = e.created();
        s.extend(e.tool_call(
            0,
            Some("c1"),
            Some("apply_patch"),
            "{\"input\":\"*** Begin Patch\"}",
        ));
        s.extend(e.complete(None));
        let frames = parse_frames(&s.concat());
        validate_sequence(&frames).unwrap();
        let item = &frames.last().unwrap()["response"]["output"][0];
        assert_eq!(item["type"], "custom_tool_call");
        assert_eq!(item["input"], "*** Begin Patch");
    }

    #[test]
    fn failure_is_terminal() {
        let mut e = Emitter::new("m", HashSet::new());
        let mut s = e.created();
        s.extend(e.text_delta("parcial"));
        s.extend(e.fail("stream_disconnected", "upstream closed"));
        assert!(e.complete(None).is_empty());
        let frames = parse_frames(&s.concat());
        validate_sequence(&frames).unwrap();
        assert_eq!(
            frames.last().unwrap()["response"]["error"]["code"],
            "stream_disconnected"
        );
    }
}
