//! Responses ⇄ Chat Completions (003 TK-003..TK-005).

use super::emitter::{Emitter, Usage};
use super::request::{Msg, Normalized, Part};
use serde_json::{Map, Value, json};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatQuirks {
    /// Provider rejects `parallel_tool_calls`.
    pub no_parallel_tool_calls: bool,
    /// Provider accepts `reasoning_effort`.
    pub reasoning_effort: bool,
    /// Provider rejects `stream_options`.
    pub no_stream_options: bool,
}

pub fn to_chat_request(n: &Normalized, quirks: &ChatQuirks) -> Value {
    let mut messages = Vec::new();
    if !n.system.is_empty() {
        messages.push(json!({"role": "system", "content": n.system}));
    }
    for m in &n.messages {
        match m {
            Msg::User(parts) => {
                let only_text = parts.iter().all(|p| matches!(p, Part::Text(_)));
                let content = if only_text {
                    Value::String(
                        parts
                            .iter()
                            .filter_map(|p| {
                                if let Part::Text(t) = p {
                                    Some(t.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                } else {
                    Value::Array(
                        parts
                            .iter()
                            .map(|p| match p {
                                Part::Text(t) => json!({"type": "text", "text": t}),
                                Part::Image { url, detail } => json!({"type": "image_url", "image_url": {
                                    "url": url, "detail": detail.clone().unwrap_or_else(|| "auto".into())}}),
                            })
                            .collect(),
                    )
                };
                messages.push(json!({"role": "user", "content": content}));
            }
            Msg::Assistant { text, tool_calls } => {
                let mut msg = json!({"role": "assistant", "content": if text.is_empty() { Value::Null } else { json!(text) }});
                if !tool_calls.is_empty() {
                    msg["tool_calls"] = Value::Array(
                        tool_calls
                            .iter()
                            .map(|c| json!({"id": c.call_id, "type": "function", "function": {"name": c.name, "arguments": c.arguments}}))
                            .collect(),
                    );
                }
                messages.push(msg);
            }
            Msg::ToolResult { call_id, content } => {
                messages.push(json!({"role": "tool", "tool_call_id": call_id, "content": content}));
            }
        }
    }
    let mut body = Map::new();
    body.insert("model".into(), json!(n.model));
    body.insert("messages".into(), Value::Array(messages));
    body.insert("stream".into(), json!(true));
    if !quirks.no_stream_options {
        body.insert("stream_options".into(), json!({"include_usage": true}));
    }
    if !n.tools.is_empty() {
        body.insert(
            "tools".into(),
            Value::Array(
                n.tools
                    .iter()
                    .map(|t| json!({"type": "function", "function": {"name": t.name, "description": t.description, "parameters": t.parameters}}))
                    .collect(),
            ),
        );
        if let Some(choice) = &n.tool_choice {
            body.insert("tool_choice".into(), choice.clone());
        }
        if let (Some(p), false) = (n.parallel_tool_calls, quirks.no_parallel_tool_calls) {
            body.insert("parallel_tool_calls".into(), json!(p));
        }
    }
    if let (Some(e), true) = (&n.effort, quirks.reasoning_effort) {
        body.insert("reasoning_effort".into(), json!(e));
    }
    Value::Object(body)
}

/// Converts `chat.completion.chunk` payloads into Responses frames.
pub struct ChatStreamTranslator {
    pub emitter: Emitter,
    usage: Option<Usage>,
    saw_finish: bool,
}

impl ChatStreamTranslator {
    pub fn new(emitter: Emitter) -> Self {
        Self {
            emitter,
            usage: None,
            saw_finish: false,
        }
    }

    pub fn start(&mut self) -> Vec<String> {
        self.emitter.created()
    }

    /// Handles one SSE `data:` payload.
    pub fn push(&mut self, data: &str) -> Vec<String> {
        if data.trim() == "[DONE]" {
            return self.finish();
        }
        let Ok(chunk) = serde_json::from_str::<Value>(data) else {
            return vec![];
        };
        if let Some(err) = chunk.get("error").filter(|e| !e.is_null()) {
            let code = err["code"]
                .as_str()
                .or(err["type"].as_str())
                .unwrap_or("upstream_error")
                .to_string();
            let msg = err["message"]
                .as_str()
                .unwrap_or("upstream error")
                .to_string();
            return self.emitter.fail(&code, &msg);
        }
        let mut out = Vec::new();
        if let Some(u) = chunk.get("usage").filter(|u| !u.is_null()) {
            self.usage = Some(Usage {
                input_tokens: u["prompt_tokens"].as_i64().unwrap_or(0),
                cached_tokens: u["prompt_tokens_details"]["cached_tokens"]
                    .as_i64()
                    .unwrap_or(0),
                output_tokens: u["completion_tokens"].as_i64().unwrap_or(0),
                reasoning_tokens: u["completion_tokens_details"]["reasoning_tokens"]
                    .as_i64()
                    .unwrap_or(0),
            });
        }
        for choice in chunk["choices"].as_array().into_iter().flatten() {
            let delta = &choice["delta"];
            for key in ["reasoning_content", "reasoning"] {
                if let Some(r) = delta[key].as_str() {
                    out.extend(self.emitter.reasoning_delta(r));
                }
            }
            if let Some(t) = delta["content"].as_str() {
                out.extend(self.emitter.text_delta(t));
            }
            for (i, tc) in delta["tool_calls"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                let key = tc["index"].as_u64().unwrap_or(i as u64);
                out.extend(self.emitter.tool_call(
                    key,
                    tc["id"].as_str(),
                    tc["function"]["name"].as_str(),
                    tc["function"]["arguments"].as_str().unwrap_or(""),
                ));
            }
            if choice["finish_reason"].as_str().is_some() {
                self.saw_finish = true;
            }
        }
        out
    }

    /// End of upstream stream: completes, or fails when no finish was seen.
    pub fn finish(&mut self) -> Vec<String> {
        if self.emitter.is_finished() {
            return vec![];
        }
        if self.saw_finish || self.usage.is_some() {
            self.emitter.complete(self.usage.clone())
        } else {
            self.emitter.fail(
                "stream_disconnected",
                "upstream stream ended before finish_reason",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::emitter::{parse_frames, validate_sequence};
    use super::super::request::normalize;
    use super::*;
    use std::collections::HashSet;

    fn run(fixture: &str, custom: &[&str]) -> Vec<Value> {
        let mut t = ChatStreamTranslator::new(Emitter::new(
            "m",
            custom.iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
        ));
        let mut out = t.start();
        let mut p = crate::sse::SseParser::new();
        let mut events = p.push(fixture.as_bytes());
        events.extend(p.finish());
        for e in events {
            out.extend(t.push(&e.data));
        }
        out.extend(t.finish());
        let frames = parse_frames(&out.concat());
        validate_sequence(&frames).unwrap();
        frames
    }

    #[test]
    fn text_stream() {
        let frames = run(include_str!("../../tests/fixtures/chat/text.sse"), &[]);
        let deltas: String = frames
            .iter()
            .filter(|f| f["type"] == "response.output_text.delta")
            .map(|f| f["delta"].as_str().unwrap())
            .collect();
        assert_eq!(deltas, "Olá mundo");
        let done = frames.last().unwrap();
        assert_eq!(done["type"], "response.completed");
        assert_eq!(done["response"]["usage"]["input_tokens"], 120);
        assert_eq!(done["response"]["usage"]["output_tokens"], 30);
    }

    #[test]
    fn parallel_tool_calls_with_fragmented_arguments() {
        let frames = run(
            include_str!("../../tests/fixtures/chat/tools-parallel.sse"),
            &[],
        );
        let out = frames.last().unwrap()["response"]["output"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0]["name"], "get_weather");
        assert_eq!(out[0]["arguments"], "{\"city\":\"SP\"}");
        assert_eq!(out[0]["call_id"], "call_a");
        assert_eq!(out[1]["name"], "get_time");
        assert_eq!(out[1]["call_id"], "call_b");
    }

    #[test]
    fn deepseek_reasoning_then_text() {
        let frames = run(include_str!("../../tests/fixtures/chat/reasoning.sse"), &[]);
        let out = frames.last().unwrap()["response"]["output"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(out[0]["type"], "reasoning");
        assert_eq!(out[0]["summary"][0]["text"], "Vou somar.");
        assert_eq!(out[1]["content"][0]["text"], "4");
    }

    #[test]
    fn cut_stream_fails() {
        let frames = run(
            "data: {\"choices\":[{\"delta\":{\"content\":\"Olá\"}}]}\n\n",
            &[],
        );
        assert_eq!(frames.last().unwrap()["type"], "response.failed");
    }

    #[test]
    fn custom_tool_round_trip() {
        let frames = run(
            include_str!("../../tests/fixtures/chat/custom-tool.sse"),
            &["apply_patch"],
        );
        let item = &frames.last().unwrap()["response"]["output"][0];
        assert_eq!(item["type"], "custom_tool_call");
        assert_eq!(item["input"], "*** Begin Patch\n*** End Patch");
    }

    #[test]
    fn request_translation() {
        let req = json!({
            "model": "llama-x", "instructions": "Sys",
            "input": [
                {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "oi"},
                    {"type": "input_image", "image_url": "data:image/png;base64,AAAA"}]},
                {"type": "function_call", "call_id": "c1", "name": "f", "arguments": "{}"},
                {"type": "function_call_output", "call_id": "c1", "output": "ok"}],
            "tools": [{"type": "function", "name": "f", "description": "d", "parameters": {"type": "object"}},
                      {"type": "custom", "name": "apply_patch", "description": "p"}],
            "tool_choice": "auto", "parallel_tool_calls": true});
        let body = to_chat_request(
            &normalize(&req),
            &ChatQuirks {
                no_parallel_tool_calls: true,
                ..Default::default()
            },
        );
        assert_eq!(
            body["messages"][0],
            json!({"role": "system", "content": "Sys"})
        );
        assert_eq!(
            body["messages"][1]["content"][1],
            json!({"type": "image_url", "image_url": {"url": "data:image/png;base64,AAAA", "detail": "auto"}})
        );
        assert_eq!(body["messages"][2]["tool_calls"][0]["id"], "c1");
        assert_eq!(
            body["messages"][3],
            json!({"role": "tool", "tool_call_id": "c1", "content": "ok"})
        );
        assert_eq!(
            body["tools"][1]["function"]["parameters"]["required"],
            json!(["input"])
        );
        assert!(body.get("parallel_tool_calls").is_none());
        assert_eq!(body["stream_options"], json!({"include_usage": true}));
    }
}
