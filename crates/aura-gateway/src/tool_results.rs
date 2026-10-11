//! Resolve only Host-issued web results at the authenticated model boundary.
use crate::errors::UpstreamError;
use bytes::Bytes;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub trait ToolResultResolver: Send + Sync {
    fn resolve(
        &self,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, UpstreamError>;

    fn resolve_arguments(
        &self,
        _thread: &str,
        _tool: &str,
        _handle: &str,
    ) -> Result<Option<Arc<String>>, UpstreamError> {
        Ok(None)
    }

    /// Capture authority before upstream I/O, never when a late frame arrives.
    fn begin_response(&self, _thread: &str) -> Option<Arc<dyn ToolCallProtector>> {
        None
    }

    fn resolve_reasoning(
        &self,
        _thread: &str,
        _handle: &str,
    ) -> Result<Option<Arc<String>>, UpstreamError> {
        Ok(None)
    }
}

pub trait ToolCallProtector: Send + Sync {
    fn protect(&self, tool: &str, arguments: &str) -> Result<String, UpstreamError>;
    fn protects_reasoning(&self) -> bool {
        false
    }
    fn protect_reasoning(&self, _item: &Value) -> Result<Value, UpstreamError> {
        Err(UpstreamError::new(
            409,
            "web_reasoning_unavailable",
            "Reasoning unavailable",
        ))
    }
}

pub fn reasoning_envelope(item: &Value, handle: &str) -> Value {
    serde_json::json!({"type":"reasoning","id":item["id"],"summary":[],"encrypted_content":serde_json::json!({"auraPrivateReasoning":{"version":1,"handle":handle}}).to_string()})
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReasoningEnvelope {
    #[serde(rename = "auraPrivateReasoning")]
    reference: Reference,
}

const ARGUMENT_PREFIX: &str = "aura-tool-arguments:v1:";

pub fn argument_envelope(tool: &str, handle: &str) -> String {
    let reference = format!("{ARGUMENT_PREFIX}{handle}");
    if tool == "web_fetch" {
        serde_json::json!({"url":reference}).to_string()
    } else {
        serde_json::json!({"objective":reference,"queries":[reference]}).to_string()
    }
}

/// Exact schema-conforming reference; remote strings never establish authority.
pub fn argument_handle<'a>(tool: &str, args: &'a Value) -> Option<&'a str> {
    let object = args.as_object()?;
    let text = match tool {
        "web_fetch" if object.len() == 1 => object.get("url")?.as_str()?,
        "web_search" if object.len() == 2 => {
            let text = object.get("objective")?.as_str()?;
            let queries = object.get("queries")?.as_array()?;
            if queries.len() != 1 || queries[0].as_str() != Some(text) {
                return None;
            }
            text
        }
        _ => return None,
    };
    let handle = text.strip_prefix(ARGUMENT_PREFIX)?;
    valid_handle(handle).then_some(handle)
}

fn valid_handle(handle: &str) -> bool {
    handle.len() == 64
        && handle
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn envelope(handle: &str) -> String {
    serde_json::json!({"auraToolResult":{"version":1,"handle":handle}}).to_string()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    #[serde(rename = "auraToolResult")]
    reference: Reference,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    version: u8,
    handle: String,
}

pub(crate) fn contains_private_marker(body: &[u8]) -> bool {
    body.windows(b"auraToolResult".len()).any(|part| part == b"auraToolResult")
        // A JSON key may use Unicode escapes. Parse conservatively and suppress
        // diagnostics rather than letting an alternate encoding hide a handle.
        || body.windows(ARGUMENT_PREFIX.len()).any(|part| part == ARGUMENT_PREFIX.as_bytes())
        || body.windows(b"auraPrivateReasoning".len()).any(|part| part == b"auraPrivateReasoning")
        || body.windows(2).any(|part| part == b"\\u")
}

pub(crate) fn tool_name(item: &Value) -> Option<&'static str> {
    match (item["namespace"].as_str(), item["name"].as_str()) {
        (Some("mcp__aura"), Some("web_fetch")) | (None, Some("mcp__aura__web_fetch")) => {
            Some("web_fetch")
        }
        (Some("mcp__aura"), Some("web_search")) | (None, Some("mcp__aura__web_search")) => {
            Some("web_search")
        }
        _ => None,
    }
}

fn replace_text(
    text: &mut Value,
    thread: Option<&str>,
    tool: &str,
    resolver: &dyn ToolResultResolver,
) -> Result<bool, UpstreamError> {
    let Some(original) = text.as_str() else {
        return Ok(false);
    };
    let Ok(envelope) = serde_json::from_str::<Envelope>(original) else {
        return Ok(false);
    };
    let reference = envelope.reference;
    if reference.version != 1
        || reference.handle.len() != 64
        || !reference
            .handle
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Ok(false);
    }
    let thread = thread.filter(|thread| !thread.is_empty()).ok_or_else(|| {
        UpstreamError::new(
            403,
            "web_result_scope_invalid",
            "Web result requires an authenticated thread scope",
        )
    })?;
    *text = Value::String(match resolver.resolve(thread, tool, &reference.handle)? {
        Some(result) => result.as_str().to_owned(),
        None => r#"{"code":"unavailable","message":"Previous web result is no longer available. Repeat the web tool if needed."}"#.into(),
    });
    Ok(true)
}

fn replace_output(
    output: &mut Value,
    thread: Option<&str>,
    tool: &str,
    resolver: &dyn ToolResultResolver,
) -> Result<bool, UpstreamError> {
    if output.is_string() {
        return replace_text(output, thread, tool, resolver);
    }
    let mut changed = false;
    for block in output.as_array_mut().into_iter().flatten() {
        if matches!(block["type"].as_str(), Some("input_text" | "text"))
            && block.as_object().is_some_and(|object| object.len() == 2)
            && let Some(text) = block.get_mut("text")
        {
            changed |= replace_text(text, thread, tool, resolver)?;
        }
    }
    Ok(changed)
}

pub(crate) fn prepare(
    body: Bytes,
    thread: Option<&str>,
    resolver: &dyn ToolResultResolver,
) -> Result<(Bytes, bool), UpstreamError> {
    if !contains_private_marker(&body) {
        return Ok((body, false));
    }
    let mut request: Value = serde_json::from_slice(&body)
        .map_err(|_| UpstreamError::new(400, "invalid_request", "Invalid model request"))?;
    let mut changed = restore_reasoning(&mut request, thread, resolver)?;
    let mut pending: HashMap<String, Option<&'static str>> = HashMap::new();
    let mut eligible = HashSet::new();
    let mut open: HashMap<&str, Option<usize>> = HashMap::new();
    for (index, item) in request["input"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let Some(id) = item["call_id"].as_str() else {
            continue;
        };
        match item["type"].as_str() {
            Some("function_call") => {
                open.entry(id)
                    .and_modify(|value| *value = None)
                    .or_insert(Some(index));
            }
            Some("function_call_output") => {
                if let Some(Some(index)) = open.remove(id) {
                    eligible.insert(index);
                }
            }
            _ => {}
        }
    }
    eligible.extend(open.values().flatten().copied());
    for (index, item) in request["input"]
        .as_array_mut()
        .into_iter()
        .flatten()
        .enumerate()
    {
        let Some(id) = item["call_id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .map(str::to_owned)
        else {
            continue;
        };
        match item["type"].as_str() {
            Some("function_call") => {
                let tool = tool_name(item);
                if eligible.contains(&index)
                    && let Some(tool) = tool
                {
                    changed |= replace_arguments(item, thread, tool, resolver)?;
                }
                // Duplicate pending IDs are ambiguous, never a source of authority.
                pending
                    .entry(id)
                    .and_modify(|value| *value = None)
                    .or_insert(tool);
            }
            Some("function_call_output") => {
                if let Some(Some(tool)) = pending.remove(&id)
                    && let Some(output) = item.get_mut("output")
                {
                    changed |= replace_output(output, thread, tool, resolver)?;
                }
            }
            _ => {}
        }
    }
    let body = if changed {
        Bytes::from(serde_json::to_vec(&request).expect("JSON value"))
    } else {
        body
    };
    Ok((body, true))
}

fn restore_reasoning(
    request: &mut Value,
    thread: Option<&str>,
    resolver: &dyn ToolResultResolver,
) -> Result<bool, UpstreamError> {
    let Some(input) = request["input"].as_array_mut() else {
        return Ok(false);
    };
    let mut output = Vec::with_capacity(input.len());
    let mut changed = false;
    for mut item in std::mem::take(input) {
        let envelope = (item["type"] == "reasoning")
            .then(|| {
                item["encrypted_content"]
                    .as_str()
                    .and_then(|text| serde_json::from_str::<ReasoningEnvelope>(text).ok())
            })
            .flatten();
        if let Some(envelope) = envelope
            && envelope.reference.version == 1
            && valid_handle(&envelope.reference.handle)
        {
            let thread = thread.filter(|thread| !thread.is_empty()).ok_or_else(|| {
                UpstreamError::new(403, "web_result_scope_invalid", "Missing reasoning scope")
            })?;
            if item["summary"]
                .as_array()
                .is_none_or(|summary| !summary.is_empty())
            {
                return Err(UpstreamError::new(
                    403,
                    "web_result_scope_invalid",
                    "Invalid reasoning reference",
                ));
            }
            changed = true;
            let Some(original) = resolver.resolve_reasoning(thread, &envelope.reference.handle)?
            else {
                continue;
            };
            let original: Value = serde_json::from_str(&original).map_err(|_| {
                UpstreamError::new(502, "web_reasoning_unavailable", "Reasoning unavailable")
            })?;
            if item["id"] != original["id"] || original["type"] != "reasoning" {
                return Err(UpstreamError::new(
                    403,
                    "web_result_scope_invalid",
                    "Invalid reasoning scope",
                ));
            }
            item = original;
        }
        output.push(item);
    }
    *input = output;
    Ok(changed)
}

fn replace_arguments(
    item: &mut Value,
    thread: Option<&str>,
    tool: &str,
    resolver: &dyn ToolResultResolver,
) -> Result<bool, UpstreamError> {
    let Some(text) = item["arguments"].as_str() else {
        return Ok(false);
    };
    let Ok(args) = serde_json::from_str::<Value>(text) else {
        return Ok(false);
    };
    let Some(handle) = argument_handle(tool, &args) else {
        return Ok(false);
    };
    let thread = thread.filter(|value| !value.is_empty()).ok_or_else(|| {
        UpstreamError::new(403, "web_result_scope_invalid", "Missing web input scope")
    })?;
    item["arguments"] = Value::String(match resolver.resolve_arguments(thread, tool, handle)? {
        Some(original) => original.as_str().to_owned(),
        None => {
            if tool == "web_fetch" {
                serde_json::json!({"url":"aura:previous-web-input-unavailable"}).to_string()
            } else {
                serde_json::json!({"objective":"Previous web input unavailable","queries":["Previous web input unavailable"]}).to_string()
            }
        }
    });
    Ok(true)
}
