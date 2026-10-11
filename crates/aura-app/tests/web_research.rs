//! Integrated research through Host -> pinned app-server -> gateway -> MCP.
//! Only the external model, public network/DNS and OS are controlled fixtures.
//! Run with AURA_CODEX_BIN and `--ignored`; no account or Internet is used.
use aura_app::{
    CodexRuntime, Host, HostConfig, events::HostEvent, host::SendRequest, paths::AppPaths,
    platform::Platform,
};
use aura_codex::{events::ConversationEvent, service::StartOptions};
use aura_web::{
    WebService,
    transport::{HttpRequest, HttpResponse, Transport},
};
use axum::{Router, routing::post};
use futures::future::BoxFuture;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

const FIRST: &str = "https://news.example/report";
const SECOND: &str = "https://independent.example/report";
const ANSWER: &str =
    "Produção: 42; Norte: 12; Sul: 30 [[aura-source:W1]]. Segunda produção: 30 [[aura-source:W2]].";
const BLOCKED_ANSWER: &str = "Não consegui ler a primeira página: acesso bloqueado. Segunda produção: 30 [[aura-source:W2]]. Não há leitura suficiente para comparar os totais.";
const LIMITED_ANSWER: &str = "Produção: 42 [[aura-source:W1]]. A sétima leitura foi recusada pelo limite deste Turno; não consultei outra página.";
const ARTICLE: &str = r#"<!doctype html><html><head><title>Relatório de teste</title></head><body>
<nav>MENU DESCARTÁVEL</nav><main><h1>Produção regional</h1><p>A produção foi 42 unidades.</p>
<ul><li>Norte: 12</li><li>Sul: 30</li></ul><table><tr><th>Região</th><th>Total</th></tr>
<tr><td>Norte</td><td>12</td></tr><tr><td>Sul</td><td>30</td></tr></table></main>
<script>sendCredentials()</script></body></html>"#;

#[derive(Default)]
struct PublicNetwork {
    requests: Mutex<Vec<HttpRequest>>,
    fallback: bool,
    blocked: bool,
    pending: bool,
    page_started: tokio::sync::Notify,
}
impl Transport for PublicNetwork {
    fn resolve<'a>(
        &'a self,
        _host: &'a str,
    ) -> BoxFuture<'a, Result<Vec<IpAddr>, aura_web::WebError>> {
        Box::pin(async { Ok(vec!["93.184.216.34".parse().unwrap()]) })
    }
    fn request(
        &self,
        request: HttpRequest,
    ) -> BoxFuture<'_, Result<HttpResponse, aura_web::WebError>> {
        let rpc: Value = serde_json::from_slice(&request.body).unwrap_or(Value::Null);
        let url = request.url.as_str().to_owned();
        self.requests.lock().unwrap().push(request);
        if url == FIRST && self.pending {
            self.page_started.notify_one();
            return Box::pin(async {
                Ok(HttpResponse {
                    status: 200,
                    headers: BTreeMap::from([("content-type".into(), "text/html".into())]),
                    body: aura_web::transport::HttpBody::new(futures::stream::pending()),
                })
            });
        }
        let (status, mime, bytes) = if url == FIRST && self.blocked {
            (403, "text/html", b"Forbidden".to_vec())
        } else if url == FIRST {
            (200, "text/html; charset=utf-8", ARTICLE.as_bytes().to_vec())
        } else if url == SECOND {
            (200, "text/html; charset=utf-8", b"<!doctype html><html><head><title>Relatorio independente</title></head><body><main><h1>Relatorio independente</h1><p>Segunda producao: 30 unidades.</p></main></body></html>".to_vec())
        } else if url.starts_with("https://html.duckduckgo.com/html/") {
            (200, "text/html; charset=utf-8", br#"<html><body>
<div class="result"><a class="result__a" href="https://news.example/report">Relatorio regional</a><span class="result__snippet">Resumo regional</span></div>
<div class="result"><a class="result__a" href="https://news.example/report?utm_source=fixture#top">Duplicado</a><span class="result__snippet">Resumo duplicado</span></div>
<div class="result"><a class="result__a" href="https://independent.example/report">Relatorio independente</a><span class="result__snippet">Resumo independente</span></div>
</body></html>"#.to_vec())
        } else if url == "https://search.parallel.ai/mcp" && self.fallback {
            (429, "application/json", b"{}".to_vec())
        } else if url == "https://search.parallel.ai/mcp" {
            let value = match rpc["method"].as_str() {
                Some("initialize") => {
                    json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"Research fixture","version":"1"}}})
                }
                Some("notifications/initialized") => Value::Null,
                Some("tools/list") => {
                    json!({"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"web_search","inputSchema":{"type":"object","required":["objective","search_queries"],"properties":{"objective":{"type":"string"},"search_queries":{"type":"array","items":{"type":"string"}},"session_id":{"type":"string"}}}}]}})
                }
                Some("tools/call") => {
                    json!({"jsonrpc":"2.0","id":3,"result":{"structuredContent":{"results":[
                    {"title":"Relatorio regional","url":FIRST,"excerpts":["Resumo regional"]},
                    {"title":"Duplicado","url":"https://news.example/report?utm_source=fixture#top","excerpts":["Resumo duplicado"]},
                    {"title":"Relatorio independente","url":SECOND,"excerpts":["Resumo independente"]}
                ]},"content":[],"isError":false}})
                }
                _ => return Box::pin(async { Err(aura_web::WebError::Unavailable) }),
            };
            (
                if value.is_null() { 204 } else { 200 },
                "application/json",
                serde_json::to_vec(&value).unwrap(),
            )
        } else {
            return Box::pin(async { Err(aura_web::WebError::Unavailable) });
        };
        Box::pin(async move {
            Ok(HttpResponse {
                status,
                headers: BTreeMap::from([("content-type".into(), mime.into())]),
                body: bytes.into(),
            })
        })
    }
}

fn tool(body: &Value, wanted: &str) -> Option<(String, Option<String>)> {
    for item in body["tools"].as_array().into_iter().flatten() {
        if item["type"] == "namespace" && item["name"] == "mcp__aura" {
            if item["tools"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|inner| inner["name"] == wanted)
            {
                return Some((wanted.into(), Some("mcp__aura".into())));
            }
        } else if item["name"] == format!("mcp__aura__{wanted}") {
            return Some((item["name"].as_str().unwrap().into(), None));
        }
    }
    None
}
fn event(kind: &str, data: Value) -> String {
    format!("event: {kind}\ndata: {data}\n\n")
}
fn reply(body: &Value, stage: usize, blocked: bool, limited: bool) -> String {
    let wanted = match stage {
        0 => Some("web_search"),
        1 | 2 => Some("web_fetch"),
        3..=7 if limited => Some("web_fetch"),
        _ => None,
    };
    let args = match stage {
        0 => {
            json!({"objective":"Comparar producao regional com uma fonte independente","queries":["producao regional"]})
        }
        1 => json!({"url":FIRST}),
        _ if limited => json!({"url":FIRST}),
        _ => json!({"url":SECOND}),
    };
    let item = if let Some((name, namespace)) = wanted.and_then(|wanted| tool(body, wanted)) {
        let mut item = json!({"type":"function_call","id":format!("fc_research_{stage}"),"call_id":format!("call_research_{stage}"),"name":name,"arguments":args.to_string()});
        if let Some(namespace) = namespace {
            item["namespace"] = json!(namespace);
        }
        item
    } else {
        json!({"type":"message","id":"msg_research","role":"assistant","status":"completed","content":[{"type":"output_text","text":if blocked { BLOCKED_ANSWER } else if limited { LIMITED_ANSWER } else { ANSWER },"annotations":[]}]})
    };
    event(
        "response.created",
        json!({"type":"response.created","response":{"id":format!("resp_research_{stage}"),"status":"in_progress","output":[]}}),
    ) + &event(
        "response.output_item.added",
        json!({"type":"response.output_item.added","output_index":0,"item":item}),
    ) + &event(
        "response.output_item.done",
        json!({"type":"response.output_item.done","output_index":0,"item":item}),
    ) + &event(
        "response.completed",
        json!({"type":"response.completed","response":{"id":format!("resp_research_{stage}"),"status":"completed","output":[item],"usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    )
}
fn outputs(body: &Value) -> Vec<Value> {
    body["input"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|item| item["type"] == "function_call_output")
        .flat_map(|item| item["output"].as_array().into_iter().flatten())
        .filter_map(|part| part["text"].as_str())
        .filter_map(|text| serde_json::from_str(text).ok())
        .collect()
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn duplicate_search_html_facts_two_domains_and_citations_cross_the_complete_host_chain() {
    research(false, false, false, false).await;
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn rate_limited_primary_uses_free_fallback_and_keeps_the_same_read_and_citation_chain() {
    research(true, false, false, false).await;
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn a_blocked_page_reaches_the_provider_as_an_error_and_only_the_read_source_is_cited() {
    research(false, true, false, false).await;
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn interrupting_a_pending_read_ends_the_real_turn_without_read_provenance_or_further_pages() {
    research(false, false, true, false).await;
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn the_seventh_read_is_refused_in_the_host_chain_even_when_previous_reads_use_cache() {
    research(false, false, false, true).await;
}

async fn research(fallback: bool, blocked: bool, pending: bool, limited: bool) {
    let binary = std::env::var_os("AURA_CODEX_BIN").expect("pinned app-server required");
    let directory = tempfile::Builder::new()
        .prefix("aura-research-")
        .tempdir_in(
            std::env::var_os("AURA_E2E_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir),
        )
        .unwrap();
    let model_requests = Arc::new(Mutex::new(Vec::<Value>::new()));
    let seen = model_requests.clone();
    let provider = Router::new().route(
        "/v1/responses",
        post(move |axum::Json(body): axum::Json<Value>| {
            let seen = seen.clone();
            async move {
                let stage = {
                    let mut seen = seen.lock().unwrap();
                    let stage = seen.len();
                    seen.push(body.clone());
                    stage
                };
                (
                    [("content-type", "text/event-stream")],
                    reply(&body, stage, blocked, limited),
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}/v1", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, provider).await.unwrap();
    });
    let network = Arc::new(PublicNetwork {
        fallback,
        blocked,
        pending,
        ..Default::default()
    });
    let (platform, _) = Platform::fake();
    let mut config = HostConfig::demo(AppPaths::new(directory.path().join("Aura")), platform);
    config.codex = CodexRuntime::Binary {
        program: Some(binary.into()),
        on_spawn: None,
    };
    config.web = Some(Arc::new(WebService::new(
        network.clone(),
        Arc::new(aura_web::SystemClock),
    )));
    let host = Host::start(config).await.unwrap();
    let provider = host.save_provider(serde_json::from_value(json!({"name":"Research fixture","preset":"custom","wire":"responses","baseUrl":base_url,"auth":"none"})).unwrap(), None).unwrap();
    let mut events = host.subscribe();
    let conversation = host
        .start_conversation(StartOptions {
            provider: format!("aura-{}", provider.id),
            model: Some("fixture-model".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    host.send(SendRequest {
        thread_id: conversation.thread_id.clone(),
        text: "Pesquise, leia as duas fontes e compare os fatos regionais 42/12/30.".into(),
        tray: conversation.thread_id.clone(),
        accepts_images: false,
        options: Default::default(),
    })
    .await
    .unwrap();
    if pending {
        tokio::time::timeout(Duration::from_secs(10), network.page_started.notified())
            .await
            .expect("the external page request must actually start before interruption");
        host.interrupt(&conversation.thread_id).await.unwrap();
    }
    let mut sources = Vec::new();
    let mut answer = String::new();
    let mut terminal_status = None;
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match events.recv().await.unwrap() {
                HostEvent::WebSource {
                    thread_id, source, ..
                } => {
                    assert_eq!(thread_id, conversation.thread_id);
                    if source.kind == "pageContent" {
                        sources.push(source);
                    }
                }
                HostEvent::Conversation(ConversationEvent::MessageCompleted { text, .. }) => {
                    answer = text
                }
                HostEvent::Conversation(ConversationEvent::TurnCompleted {
                    error, status, ..
                }) => {
                    assert!(error.is_none());
                    terminal_status = Some(status);
                    break;
                }
                _ => {}
            }
        }
    })
    .await
    .expect("complete public research turn");
    let transcript = host
        .open_conversation(&conversation.thread_id)
        .await
        .unwrap();
    host.shutdown().await;
    server.abort();
    if pending {
        assert_eq!(
            terminal_status,
            Some(aura_codex::events::TurnStatus::Interrupted)
        );
        assert!(
            sources.is_empty(),
            "a cancelled stream must not advertise a read page"
        );
        assert!(
            answer.is_empty(),
            "the interrupted turn must not deliver the scripted final answer"
        );
        assert!(transcript.iter().all(|message| message.sources.is_empty()));
        assert!(
            network
                .requests
                .lock()
                .unwrap()
                .iter()
                .all(|request| request.url.as_str() != SECOND)
        );
        return;
    }
    if limited {
        assert_eq!(answer.trim(), LIMITED_ANSWER);
        assert_eq!(
            sources.len(),
            6,
            "six successful tool reads before the public limit error"
        );
        assert!(
            sources
                .iter()
                .all(|source| source.source_id == "W1" && source.url == FIRST)
        );
        let final_message = transcript
            .iter()
            .find(|message| message.role == "assistant")
            .unwrap();
        assert_eq!(final_message.sources.len(), 1);
        assert_eq!(final_message.sources[0].source_id, "W1");
        let requests = model_requests.lock().unwrap();
        assert_eq!(requests.len(), 9);
        assert!(
            outputs(&requests[8])
                .iter()
                .any(|output| output["code"] == "limit_exceeded")
        );
        assert_eq!(
            network
                .requests
                .lock()
                .unwrap()
                .iter()
                .filter(|request| request.url.as_str() == FIRST)
                .count(),
            1,
            "cached reads still consume the tool budget without downloading again"
        );
        return;
    }
    let expected_answer = if blocked { BLOCKED_ANSWER } else { ANSWER };
    assert_eq!(answer.trim(), expected_answer);
    assert_eq!(sources.len(), if blocked { 1 } else { 2 });
    if !blocked {
        assert_eq!((&*sources[0].source_id, &*sources[0].url), ("W1", FIRST));
    }
    assert_eq!(
        (
            &*sources.last().unwrap().source_id,
            &*sources.last().unwrap().url
        ),
        ("W2", SECOND)
    );
    let final_message = transcript
        .iter()
        .find(|message| message.role == "assistant")
        .unwrap();
    assert_eq!(final_message.text.trim(), expected_answer);
    assert_eq!(final_message.sources.len(), if blocked { 1 } else { 2 });
    if blocked {
        assert_eq!(final_message.sources[0].source_id, "W2");
    }
    assert!(
        final_message
            .sources
            .iter()
            .all(|source| source.snippet.is_empty())
    );
    let requests = model_requests.lock().unwrap();
    assert_eq!(
        requests.len(),
        4,
        "two pages must reach the provider before its final response"
    );
    let search = outputs(&requests[1]);
    assert_eq!(
        search[0]["provider"],
        if fallback { "duckduckgo" } else { "parallel" }
    );
    assert_eq!(search[0]["degraded"], fallback);
    assert_eq!(
        search[0]["results"].as_array().unwrap().len(),
        2,
        "duplicate URL must not become another source"
    );
    assert_eq!(search[0]["results"][0]["sourceId"], "W1");
    assert_eq!(search[0]["results"][1]["sourceId"], "W2");
    let pages = outputs(&requests[3]);
    if blocked {
        let failed = outputs(&requests[2]);
        assert!(
            failed.iter().any(|output| output["code"] == "blocked"),
            "public error must reach the provider"
        );
        assert!(
            pages.iter().all(|page| page["sourceId"] != "W1"),
            "blocked page must not become read provenance"
        );
    } else {
        let regional = pages.iter().find(|page| page["sourceId"] == "W1").unwrap();
        let text = regional["text"].as_str().unwrap();
        for fact in ["A produção foi 42 unidades", "Norte: 12", "Sul: 30"] {
            assert!(text.contains(fact), "missing literal {fact}");
        }
        assert!(!text.contains("MENU DESCARTÁVEL") && !text.contains("sendCredentials"));
        assert_eq!(regional["finalUrl"], FIRST);
        assert_eq!(regional["externalContent"], true);
    }
    let independent = pages.iter().find(|page| page["sourceId"] == "W2").unwrap();
    assert!(
        independent["text"]
            .as_str()
            .unwrap()
            .contains("Segunda producao: 30 unidades")
    );
    assert_eq!(independent["finalUrl"], SECOND);
    for request in network.requests.lock().unwrap().iter() {
        assert!(
            !request.headers.contains_key("authorization")
                && !request.headers.contains_key("cookie")
        );
        assert!(!String::from_utf8_lossy(&request.body).contains(&conversation.thread_id));
        assert!(!String::from_utf8_lossy(&request.body).contains(&conversation.conversation_uuid));
    }
}
