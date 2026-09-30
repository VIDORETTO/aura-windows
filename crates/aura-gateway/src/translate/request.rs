//! Normalizes a Responses API request (as sent by Codex) into a
//! provider-neutral conversation that the Chat Completions and Anthropic
//! builders consume.

use serde_json::{Value, json};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    Text(String),
    Image { url: String, detail: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    User(Vec<Part>),
    Assistant {
        text: String,
        tool_calls: Vec<ToolCall>,
    },
    ToolResult {
        call_id: String,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub custom: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Normalized {
    pub model: String,
    pub system: String,
    pub messages: Vec<Msg>,
    pub tools: Vec<Tool>,
    pub tool_choice: Option<Value>,
    pub parallel_tool_calls: Option<bool>,
    pub effort: Option<String>,
    pub custom_tools: HashSet<String>,
    /// Flattened namespace tools: chat name → (namespace, name). Codex sends
    /// MCP tools as `{"type":"namespace","name":"mcp__aura","tools":[..]}` and
    /// expects `function_call` items with `namespace` set.
    pub namespaced: std::collections::HashMap<String, (String, String)>,
    /// Hosted tools that cannot be translated (e.g. `web_search`).
    pub dropped_tools: Vec<String>,
}

fn text_of_output(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| p["text"].as_str().or(p["output"].as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn custom_tool_schema(description: &str) -> Value {
    json!({
        "type": "object",
        "properties": {"input": {"type": "string", "description": description}},
        "required": ["input"],
        "additionalProperties": false
    })
}

/// Chat-safe unique name for a namespaced tool (`mcp__aura__screen_text`).
pub fn flat_name(namespace: &str, name: &str) -> String {
    let joined = format!("{}__{name}", namespace.trim_end_matches('_'));
    // Chat providers limit names to 64 chars of `[a-zA-Z0-9_-]`.
    let clean: String = joined
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if clean.len() <= 64 {
        clean
    } else {
        clean[clean.len() - 64..].to_string()
    }
}

fn push_tool(out: &mut Normalized, t: &Value) {
    push_tool_in(out, t, None);
}

fn push_tool_in(out: &mut Normalized, t: &Value, namespace: Option<&str>) {
    match t["type"].as_str() {
        Some("function") => {
            let inner = t["name"]
                .as_str()
                .or(t["function"]["name"].as_str())
                .unwrap_or_default()
                .to_string();
            let name = match namespace {
                Some(ns) => {
                    let flat = flat_name(ns, &inner);
                    out.namespaced.insert(flat.clone(), (ns.to_string(), inner));
                    flat
                }
                None => inner,
            };
            out.tools.push(Tool {
                description: t["description"].as_str().unwrap_or_default().to_string(),
                parameters: t
                    .get("parameters")
                    .cloned()
                    .unwrap_or_else(|| json!({"type": "object", "properties": {}})),
                name,
                custom: false,
            });
        }
        Some("custom") => {
            let name = t["name"].as_str().unwrap_or_default().to_string();
            out.custom_tools.insert(name.clone());
            out.tools.push(Tool {
                description: t["description"].as_str().unwrap_or_default().to_string(),
                parameters: custom_tool_schema(
                    "Raw input for this tool, exactly as the tool expects it.",
                ),
                name,
                custom: true,
            });
        }
        Some("namespace") => {
            let ns = t["name"].as_str().unwrap_or_default().to_string();
            for inner in t["tools"].as_array().into_iter().flatten() {
                push_tool_in(
                    out,
                    inner,
                    Some(&ns).filter(|s| !s.is_empty()).map(|s| s.as_str()),
                );
            }
        }
        Some(other) => out.dropped_tools.push(other.to_string()),
        None => {}
    }
}

pub fn normalize(req: &Value) -> Normalized {
    let mut out = Normalized {
        model: req["model"].as_str().unwrap_or_default().to_string(),
        tool_choice: req.get("tool_choice").cloned().filter(|v| !v.is_null()),
        parallel_tool_calls: req["parallel_tool_calls"].as_bool(),
        effort: req["reasoning"]["effort"].as_str().map(str::to_string),
        ..Default::default()
    };
    let mut system = Vec::new();
    if let Some(i) = req["instructions"].as_str().filter(|s| !s.is_empty()) {
        system.push(i.to_string());
    }
    for t in req["tools"].as_array().into_iter().flatten() {
        push_tool(&mut out, t);
    }

    let items: Vec<Value> = match &req["input"] {
        Value::String(s) => vec![json!({"type": "message", "role": "user", "content": s})],
        Value::Array(a) => a.clone(),
        _ => vec![],
    };
    for item in items {
        let ty = item["type"].as_str().unwrap_or("message");
        match ty {
            "message" => {
                let role = item["role"].as_str().unwrap_or("user");
                let parts: Vec<Part> = match &item["content"] {
                    Value::String(s) => vec![Part::Text(s.clone())],
                    Value::Array(parts) => parts
                        .iter()
                        .filter_map(|p| match p["type"].as_str() {
                            Some("input_text") | Some("output_text") | Some("text") => {
                                p["text"].as_str().map(|t| Part::Text(t.to_string()))
                            }
                            Some("input_image") => p["image_url"].as_str().map(|u| Part::Image {
                                url: u.to_string(),
                                detail: p["detail"].as_str().map(str::to_string),
                            }),
                            _ => None,
                        })
                        .collect(),
                    _ => vec![],
                };
                match role {
                    "system" | "developer" => {
                        let t: Vec<String> = parts
                            .iter()
                            .filter_map(|p| {
                                if let Part::Text(t) = p {
                                    Some(t.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if !t.is_empty() {
                            system.push(t.join("\n"));
                        }
                    }
                    "assistant" => {
                        let text: String = parts
                            .iter()
                            .filter_map(|p| {
                                if let Part::Text(t) = p {
                                    Some(t.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("");
                        push_assistant(&mut out.messages, Some(text), None);
                    }
                    _ => {
                        if !parts.is_empty() {
                            out.messages.push(Msg::User(parts));
                        }
                    }
                }
            }
            "function_call" => push_assistant(
                &mut out.messages,
                None,
                Some(ToolCall {
                    call_id: item["call_id"].as_str().unwrap_or_default().to_string(),
                    name: match item["namespace"].as_str().filter(|s| !s.is_empty()) {
                        Some(ns) => flat_name(ns, item["name"].as_str().unwrap_or_default()),
                        None => item["name"].as_str().unwrap_or_default().to_string(),
                    },
                    arguments: item["arguments"].as_str().unwrap_or("{}").to_string(),
                }),
            ),
            "custom_tool_call" => push_assistant(
                &mut out.messages,
                None,
                Some(ToolCall {
                    call_id: item["call_id"].as_str().unwrap_or_default().to_string(),
                    name: item["name"].as_str().unwrap_or_default().to_string(),
                    arguments: json!({"input": item["input"].as_str().unwrap_or_default()})
                        .to_string(),
                }),
            ),
            "function_call_output" | "custom_tool_call_output" => {
                out.messages.push(Msg::ToolResult {
                    call_id: item["call_id"].as_str().unwrap_or_default().to_string(),
                    content: text_of_output(&item["output"]),
                })
            }
            _ => {} // reasoning, web_search_call, etc. are not replayed
        }
    }
    out.system = system.join("\n\n");
    out
}

/// Appends text/tool calls to the last assistant message when consecutive.
fn push_assistant(msgs: &mut Vec<Msg>, text: Option<String>, call: Option<ToolCall>) {
    if let Some(Msg::Assistant {
        text: t,
        tool_calls,
    }) = msgs.last_mut()
    {
        if let Some(extra) = &text {
            if tool_calls.is_empty() {
                t.push_str(extra);
                return;
            }
        } else if let Some(c) = call {
            tool_calls.push(c);
            return;
        }
    }
    msgs.push(Msg::Assistant {
        text: text.unwrap_or_default(),
        tool_calls: call.into_iter().collect(),
    });
}

/// Splits a `data:` URL into (media type, base64 data).
pub fn split_data_url(url: &str) -> Option<(String, String)> {
    let rest = url.strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    let media = meta.strip_suffix(";base64")?;
    Some((media.to_string(), data.to_string()))
}

#[cfg(test)]
mod namespace_tests {
    use super::*;

    #[test]
    fn namespaced_tools_are_flattened_and_history_matches() {
        let req = json!({
            "model": "m",
            "tools": [{"type": "namespace", "name": "mcp__aura", "description": "x", "tools": [
                {"type": "function", "name": "screen_text", "parameters": {"type": "object"}}]}],
            "input": [
                {"type": "function_call", "name": "screen_text", "namespace": "mcp__aura", "call_id": "c1", "arguments": "{}"},
                {"type": "function_call_output", "call_id": "c1", "output": "ok"}
            ]
        });
        let n = normalize(&req);
        assert_eq!(n.tools[0].name, "mcp__aura__screen_text");
        assert_eq!(
            n.namespaced["mcp__aura__screen_text"],
            ("mcp__aura".to_string(), "screen_text".to_string())
        );
        let Msg::Assistant { tool_calls, .. } = &n.messages[0] else {
            panic!("{:?}", n.messages)
        };
        assert_eq!(tool_calls[0].name, "mcp__aura__screen_text");
        assert_eq!(flat_name("mcp__aura__", "x"), "mcp__aura__x");
        assert!(flat_name(&"n".repeat(80), "tool").len() <= 64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_like_request_is_normalized() {
        let req = json!({
            "model": "llama-x",
            "instructions": "Você é o Aura.",
            "input": [
                {"type": "message", "role": "developer", "content": [{"type": "input_text", "text": "Modo Chat."}]},
                {"type": "message", "role": "user", "content": [
                    {"type": "input_text", "text": "o que é isso?"},
                    {"type": "input_image", "image_url": "data:image/png;base64,AAAA", "detail": "auto"}]},
                {"type": "reasoning", "summary": []},
                {"type": "function_call", "call_id": "c1", "name": "get_weather", "arguments": "{\"city\":\"SP\"}"},
                {"type": "function_call", "call_id": "c2", "name": "get_time", "arguments": "{}"},
                {"type": "function_call_output", "call_id": "c1", "output": "25C"},
                {"type": "function_call_output", "call_id": "c2", "output": [{"type": "input_text", "text": "10h"}]},
                {"type": "message", "role": "assistant", "content": [{"type": "output_text", "text": "Feito"}]}
            ],
            "tools": [
                {"type": "function", "name": "get_weather", "description": "d", "parameters": {"type": "object"}},
                {"type": "custom", "name": "apply_patch", "description": "patch"},
                {"type": "web_search"}
            ],
            "tool_choice": "auto", "parallel_tool_calls": true, "reasoning": {"effort": "high"}
        });
        let n = normalize(&req);
        assert_eq!(n.system, "Você é o Aura.\n\nModo Chat.");
        assert_eq!(n.messages.len(), 5);
        let Msg::Assistant { tool_calls, .. } = &n.messages[1] else {
            panic!()
        };
        assert_eq!(tool_calls.len(), 2);
        assert_eq!(
            n.messages[3],
            Msg::ToolResult {
                call_id: "c2".into(),
                content: "10h".into()
            }
        );
        assert!(n.custom_tools.contains("apply_patch"));
        assert_eq!(n.dropped_tools, vec!["web_search"]);
        assert_eq!(n.effort.as_deref(), Some("high"));
    }

    #[test]
    fn data_url_split() {
        assert_eq!(
            split_data_url("data:image/png;base64,QUJD"),
            Some(("image/png".into(), "QUJD".into()))
        );
        assert_eq!(split_data_url("https://x/y.png"), None);
    }
}
