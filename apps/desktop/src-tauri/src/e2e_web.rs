//! Native QA composition: only the remote DNS/HTTP boundary is synthetic.
//! Compiled exclusively with `e2e`, enabled explicitly by AURA_E2E_WEB=1.
use aura_app::{CodexRuntime, HostConfig};
use aura_web::{
    WebError, WebService,
    transport::{HttpBody, HttpRequest, HttpResponse, Transport},
};
use futures::future::BoxFuture;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::Write,
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub fn configure(cfg: &mut HostConfig) -> Result<(), Box<dyn std::error::Error>> {
    let program = PathBuf::from(std::env::var("AURA_CODEX_BIN")?);
    if !program.is_file() {
        return Err("Native web E2E requires the explicit cached app-server binary".into());
    }
    let on_spawn = match &cfg.codex {
        CodexRuntime::Binary { on_spawn, .. } => on_spawn.clone(),
        CodexRuntime::Fake => None,
    };
    cfg.codex = CodexRuntime::Binary {
        program: Some(program),
        on_spawn,
    };
    std::fs::create_dir_all(&cfg.paths.root)?;
    let requests = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(cfg.paths.root.join("e2e-web-requests.jsonl"))?;
    cfg.web = Some(Arc::new(WebService::new(
        Arc::new(PublicNetwork {
            requests: Mutex::new(requests),
        }),
        Arc::new(aura_web::SystemClock),
    )));
    Ok(())
}

struct PublicNetwork {
    requests: Mutex<std::fs::File>,
}
const FIRST: &str = "https://news.example/report";
const SECOND: &str = "https://independent.example/report";
const ARTICLE: &str = r#"<!doctype html><html><head><title>Regional report</title></head><body>
<nav>DISCARD THIS MENU</nav><main><h1>Regional production</h1><p>Production: 42 units.</p>
<ul><li>North: 12</li><li>South: 30</li></ul>
<table><tr><th>Region</th><th>Total</th></tr><tr><td>North</td><td>12</td></tr>
<tr><td>South</td><td>30</td></tr></table></main><script>sendCredentials()</script></body></html>"#;

impl Transport for PublicNetwork {
    fn resolve<'a>(&'a self, _host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        Box::pin(async { Ok(vec![IpAddr::from([93, 184, 216, 34])]) })
    }
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        Box::pin(async move {
            let url = request.url.as_str();
            // Only these literal fixture addresses may enter the trace. No
            // request headers, body, query context or provider input is logged.
            if !matches!(
                url,
                FIRST
                    | SECOND
                    | "https://news.example/pending"
                    | "https://news.example/blocked"
                    | "https://search.parallel.ai/mcp"
            ) {
                return Err(WebError::Unavailable);
            }
            writeln!(
                self.requests.lock().map_err(|_| WebError::Unavailable)?,
                "{}",
                json!({"url":url})
            )
            .map_err(|_| WebError::Unavailable)?;
            let (status, mime, body) = match url {
                "https://news.example/pending" => {
                    (200, "text/html", HttpBody::new(futures::stream::pending()))
                }
                "https://news.example/blocked" => (403, "text/html", b"Forbidden".to_vec().into()),
                FIRST => (200, "text/html; charset=utf-8", ARTICLE.as_bytes().to_vec().into()),
                SECOND => (200, "text/html; charset=utf-8", b"<!doctype html><html><head><title>Independent report</title></head><body><main><h1>Independent report</h1><p>Second production: 30 units.</p></main></body></html>".to_vec().into()),
                _ => {
                    let rpc: Value = serde_json::from_slice(&request.body).map_err(|_| WebError::InvalidInput)?;
                    let result = match rpc["method"].as_str() {
                        Some("initialize") => json!({"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"Native web QA","version":"1"}}),
                        Some("notifications/initialized") => return Ok(HttpResponse { status:204, headers:BTreeMap::new(), body:Vec::new().into() }),
                        Some("tools/list") => json!({"tools":[{"name":"web_search","description":"Synthetic public search","inputSchema":{"type":"object"}}]}),
                        Some("tools/call") => json!({"structuredContent":{"results":[
                            {"title":"Regional report","url":FIRST,"excerpts":["Regional summary"]},
                            {"title":"Duplicate report","url":"https://news.example/report?utm_source=fixture#top","excerpts":["Duplicate summary"]},
                            {"title":"Independent report","url":SECOND,"excerpts":["Independent summary"]}
                        ]},"content":[],"isError":false}),
                        _ => return Err(WebError::Unavailable),
                    };
                    (200, "application/json", serde_json::to_vec(&json!({"jsonrpc":"2.0","id":rpc["id"],"result":result})).map_err(|_| WebError::Unavailable)?.into())
                }
            };
            Ok(HttpResponse {
                status,
                headers: BTreeMap::from([("content-type".into(), mime.into())]),
                body,
            })
        })
    }
}
