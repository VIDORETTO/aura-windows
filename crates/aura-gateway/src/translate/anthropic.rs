//! Responses ⇄ Anthropic Messages (003 TK-006).

use super::emitter::{Emitter, Usage};
use super::request::{Msg, Normalized, Part, split_data_url};
use serde_json::{Value, json};
use std::collections::HashMap;

pub const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnthropicOptions {
    pub max_tokens: u32,
    pub thinking: bool,
}

impl Default for AnthropicOptions {
    fn default() -> Self {
        Self {
            max_tokens: 8192,
            thinking: false,
        }
    }
}

fn thinking_budget(effort: Option<&str>) -> Option<u32> {
    match effort {
        Some("low") | Some("minimal") => Some(2048),
        Some("medium") => Some(8192),
        Some("high") | Some("xhigh") => Some(24576),
        _ => None,
    }
}

fn image_block(url: &str) -> Value {
    match split_data_url(url) {
        Some((media, data)) => {
            json!({"type": "image", "source": {"type": "base64", "media_type": media, "data": data}})
        }
        None => json!({"type": "image", "source": {"type": "url", "url": url}}),
    }
}

pub fn to_messages_request(n: &Normalized, opts: &AnthropicOptions) -> Value {
    // Build (role, blocks) then merge consecutive roles (the API requires
    // alternation).
    let mut turns: Vec<(&'static str, Vec<Value>)> = Vec::new();
    let mut push = |role: &'static str, blocks: Vec<Value>| {
        if blocks.is_empty() {
            return;
        }
        match turns.last_mut() {
            Some((r, b)) if *r == role => b.extend(blocks),
            _ => turns.push((role, blocks)),
        }
    };
    for m in &n.messages {
        match m {
            Msg::User(parts) => push(
                "user",
                parts
                    .iter()
                    .map(|p| match p {
                        Part::Text(t) => json!({"type": "text", "text": t}),
                        Part::Image { url, .. } => image_block(url),
                    })
                    .collect(),
            ),
            Msg::Assistant { text, tool_calls } => {
                let mut blocks = Vec::new();
                if !text.is_empty() {
                    blocks.push(json!({"type": "text", "text": text}));
                }
                for c in tool_calls {
                    let input =
                        serde_json::from_str::<Value>(&c.arguments).unwrap_or_else(|_| json!({}));
                    blocks.push(json!({"type": "tool_use", "id": c.call_id, "name": c.name, "input": input}));
                }
                push("assistant", blocks);
            }
            Msg::ToolResult { call_id, content } => push(
                "user",
                vec![json!({"type": "tool_result", "tool_use_id": call_id, "content": content})],
            ),
        }
    }
    if turns.first().is_some_and(|(r, _)| *r == "assistant") {
        turns.insert(
            0,
            (
                "user",
                vec![json!({"type": "text", "text": "(continuação)"})],
            ),
        );
    }
    let messages: Vec<Value> = turns
        .into_iter()
        .map(|(role, content)| json!({"role": role, "content": content}))
        .collect();

    let mut body = json!({
        "model": n.model,
        "max_tokens": opts.max_tokens,
        "messages": messages,
        "stream": true,
    });
    if !n.system.is_empty() {
        body["system"] = json!(n.system);
    }
    if !n.tools.is_empty() {
        body["tools"] = Value::Array(
            n.tools
                .iter()
                .map(|t| json!({"name": t.name, "description": t.description, "input_schema": t.parameters}))
                .collect(),
        );
        if n.parallel_tool_calls == Some(false) {
            body["tool_choice"] = json!({"type": "auto", "disable_parallel_tool_use": true});
        }
    }
    if opts.thinking
        && let Some(budget) = thinking_budget(n.effort.as_deref())
    {
        body["thinking"] = json!({"type": "enabled", "budget_tokens": budget});
        body["max_tokens"] = json!(opts.max_tokens.max(budget + 1024));
    }
    body
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Block {
    Text,
    Tool,
    Thinking,
    Other,
}

pub struct AnthropicStreamTranslator {
    pub emitter: Emitter,
    blocks: HashMap<u64, Block>,
    usage: Usage,
    stopped: bool,
}

impl AnthropicStreamTranslator {
    pub fn new(emitter: Emitter) -> Self {
        Self {
            emitter,
            blocks: HashMap::new(),
            usage: Usage::default(),
            stopped: false,
        }
    }

    pub fn start(&mut self) -> Vec<String> {
        self.emitter.created()
    }

    pub fn push(&mut self, data: &str) -> Vec<String> {
        let Ok(ev) = serde_json::from_str::<Value>(data) else {
            return vec![];
        };
        match ev["type"].as_str().unwrap_or_default() {
            "message_start" => {
                let u = &ev["message"]["usage"];
                self.usage.input_tokens = u["input_tokens"].as_i64().unwrap_or(0)
                    + u["cache_read_input_tokens"].as_i64().unwrap_or(0)
                    + u["cache_creation_input_tokens"].as_i64().unwrap_or(0);
                self.usage.cached_tokens = u["cache_read_input_tokens"].as_i64().unwrap_or(0);
                vec![]
            }
            "content_block_start" => {
                let index = ev["index"].as_u64().unwrap_or(0);
                let block = &ev["content_block"];
                let kind = match block["type"].as_str() {
                    Some("text") => Block::Text,
                    Some("tool_use") => Block::Tool,
                    Some("thinking") => Block::Thinking,
                    _ => Block::Other,
                };
                self.blocks.insert(index, kind);
                match kind {
                    Block::Tool => self.emitter.tool_call(
                        index,
                        block["id"].as_str(),
                        block["name"].as_str(),
                        "",
                    ),
                    Block::Text => self
                        .emitter
                        .text_delta(block["text"].as_str().unwrap_or("")),
                    _ => vec![],
                }
            }
            "content_block_delta" => {
                let index = ev["index"].as_u64().unwrap_or(0);
                let d = &ev["delta"];
                match d["type"].as_str() {
                    Some("text_delta") => self.emitter.text_delta(d["text"].as_str().unwrap_or("")),
                    Some("thinking_delta") => self
                        .emitter
                        .reasoning_delta(d["thinking"].as_str().unwrap_or("")),
                    Some("input_json_delta") => self.emitter.tool_call(
                        index,
                        None,
                        None,
                        d["partial_json"].as_str().unwrap_or(""),
                    ),
                    _ => vec![],
                }
            }
            "message_delta" => {
                if let Some(o) = ev["usage"]["output_tokens"].as_i64() {
                    self.usage.output_tokens = o;
                }
                if ev["delta"]["stop_reason"].as_str().is_some() {
                    self.stopped = true;
                }
                vec![]
            }
            "message_stop" => self.emitter.complete(Some(self.usage.clone())),
            "error" => {
                let err = &ev["error"];
                let code = match err["type"].as_str() {
                    Some("overloaded_error") => "server_overloaded",
                    Some("rate_limit_error") => "rate_limit_exceeded",
                    Some("invalid_request_error") => "invalid_request_error",
                    Some(other) => other,
                    None => "upstream_error",
                };
                self.emitter
                    .fail(code, err["message"].as_str().unwrap_or("upstream error"))
            }
            _ => vec![],
        }
    }

    pub fn finish(&mut self) -> Vec<String> {
        if self.emitter.is_finished() {
            return vec![];
        }
        if self.stopped {
            self.emitter.complete(Some(self.usage.clone()))
        } else {
            self.emitter.fail(
                "stream_disconnected",
                "upstream stream ended before message_stop",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::emitter::{parse_frames, validate_sequence};
    use super::super::request::normalize;
    use super::*;

    fn run(fixture: &str) -> Vec<Value> {
        let mut t = AnthropicStreamTranslator::new(Emitter::new("claude-x", Default::default()));
        let mut out = t.start();
        let mut p = crate::sse::SseParser::new();
        for e in p.push(fixture.as_bytes()) {
            out.extend(t.push(&e.data));
        }
        out.extend(t.finish());
        let frames = parse_frames(&out.concat());
        validate_sequence(&frames).unwrap();
        frames
    }

    #[test]
    fn thinking_text_and_parallel_tool_use() {
        let frames = run(include_str!(
            "../../tests/fixtures/anthropic/tools-thinking.sse"
        ));
        let done = frames.last().unwrap();
        assert_eq!(done["type"], "response.completed");
        let out = done["response"]["output"].as_array().unwrap();
        assert_eq!(out[0]["type"], "reasoning");
        assert_eq!(out[1]["content"][0]["text"], "Vou verificar.");
        assert_eq!(out[2]["call_id"], "toolu_1");
        assert_eq!(out[2]["arguments"], "{\"city\": \"SP\"}");
        assert_eq!(out[3]["name"], "get_time");
        assert_eq!(done["response"]["usage"]["input_tokens"], 50);
        assert_eq!(done["response"]["usage"]["output_tokens"], 42);
    }

    #[test]
    fn overloaded_error_fails_the_response() {
        let frames = run(include_str!(
            "../../tests/fixtures/anthropic/overloaded.sse"
        ));
        assert_eq!(
            frames.last().unwrap()["response"]["error"]["code"],
            "server_overloaded"
        );
    }

    #[test]
    fn request_has_system_alternation_tools_and_images() {
        let req = json!({
            "model": "claude-x", "instructions": "Sys",
            "input": [
                {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "oi"},
                    {"type": "input_image", "image_url": "data:image/png;base64,QUJD"}]},
                {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "e isso"}]},
                {"type": "function_call", "call_id": "t1", "name": "f", "arguments": "{\"a\":1}"},
                {"type": "function_call_output", "call_id": "t1", "output": "ok"}],
            "tools": [{"type": "function", "name": "f", "description": "d", "parameters": {"type": "object"}}],
            "reasoning": {"effort": "medium"}});
        let body = to_messages_request(
            &normalize(&req),
            &AnthropicOptions {
                max_tokens: 4096,
                thinking: true,
            },
        );
        assert_eq!(body["system"], "Sys");
        let msgs = body["messages"].as_array().unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0]["content"].as_array().unwrap().len(), 3);
        assert_eq!(
            msgs[0]["content"][1]["source"],
            json!({"type": "base64", "media_type": "image/png", "data": "QUJD"})
        );
        assert_eq!(
            msgs[1]["content"][0],
            json!({"type": "tool_use", "id": "t1", "name": "f", "input": {"a": 1}})
        );
        assert_eq!(msgs[2]["content"][0]["type"], "tool_result");
        assert_eq!(body["tools"][0]["input_schema"], json!({"type": "object"}));
        assert_eq!(body["thinking"]["budget_tokens"], 8192);
        assert_eq!(body["max_tokens"], 9216);
    }
}
