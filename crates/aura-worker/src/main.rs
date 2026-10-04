//! `aura-worker`: JSON-RPC over stdio (ADR 0005). Methods:
//! * `health` → `{version, engines}`
//! * `asr.load {engine, modelDir}` / `asr.unload`
//! * `asr.transcribe {pcm (base64 i16 LE, 16 kHz mono), language?, prompt?}` → Transcript
//! * `media.keyframes`, `ingest.pdf`, `ingest.audio`, `ingest.video` →
//!   Windows build (see HANDOFF.md, 004 TK-007 and 007).

mod engine;

use aura_core::jsonrpc::{Incoming, Peer, RpcError};
use base64::Engine as _;
use serde_json::{Value, json};

fn decode_pcm(b64: &str) -> Option<Vec<f32>> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    Some(
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / i16::MAX as f32)
            .collect(),
    )
}

fn engines() -> Vec<&'static str> {
    let mut v = vec!["fake"];
    if cfg!(feature = "engines") {
        v.extend(["onnx-parakeet", "ggml-whisper"]);
    }
    v
}

/// Whether this build runs models on a GPU (the `vulkan` feature).
fn gpu() -> bool {
    cfg!(feature = "vulkan")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // `--capabilities`: one JSON line for the app's model recommendation.
    if std::env::args().any(|a| a == "--capabilities") {
        println!("{}", json!({"engines": engines(), "gpu": gpu()}));
        return;
    }
    let (_peer, mut incoming): (Peer, _) = Peer::spawn(tokio::io::stdin(), tokio::io::stdout());
    let mut current: Option<(String, Box<dyn engine::Engine>)> = None;
    while let Some(msg) = incoming.recv().await {
        let Incoming::Request {
            method,
            params,
            responder,
        } = msg
        else {
            continue;
        };
        match method.as_str() {
            "health" => responder.ok(
                json!({"version": env!("CARGO_PKG_VERSION"), "engines": engines(), "gpu": gpu()}),
            ),
            "asr.load" => {
                let engine = params["engine"].as_str().unwrap_or_default().to_string();
                let dir = std::path::PathBuf::from(params["modelDir"].as_str().unwrap_or_default());
                let key = format!("{engine}:{}", dir.display());
                if current.as_ref().is_some_and(|(k, _)| *k == key) {
                    responder.ok(json!({"loaded": key, "cached": true}));
                    continue;
                }
                current = None; // unload before loading (one model at a time)
                match engine::load(&engine, &dir) {
                    Ok(e) => {
                        current = Some((key.clone(), e));
                        responder.ok(json!({"loaded": key, "cached": false}));
                    }
                    Err(e) => responder.err(-32000, e),
                }
            }
            "asr.unload" => {
                current = None;
                responder.ok(json!({}));
            }
            "asr.transcribe" => {
                let Some((_, eng)) = current.as_mut() else {
                    responder.err(-32001, "no model loaded");
                    continue;
                };
                let Some(pcm) = params["pcm"].as_str().and_then(decode_pcm) else {
                    responder.err(RpcError::INVALID_PARAMS, "pcm must be base64 i16");
                    continue;
                };
                let lang = params["language"].as_str();
                let prompt = params["prompt"].as_str().unwrap_or_default();
                match eng.transcribe(&pcm, lang, prompt) {
                    Ok(t) => responder.ok(serde_json::to_value(t).unwrap_or(Value::Null)),
                    Err(e) => responder.err(-32002, e),
                }
            }
            #[cfg(debug_assertions)]
            "debug.exit" => std::process::exit(3),
            "media.keyframes" | "ingest.pdf" | "ingest.audio" | "ingest.video" => responder.err(
                RpcError::METHOD_NOT_FOUND,
                format!("{method} requires the Windows media build of aura-worker"),
            ),
            other => responder.err(
                RpcError::METHOD_NOT_FOUND,
                format!("unknown method {other}"),
            ),
        }
    }
}
