//! Conversation continuity across Turns, restarts and mode changes.
//! Host -> pinned app-server -> gateway -> scripted Responses provider.
//! Run with AURA_CODEX_BIN and `--ignored`; no account or Internet is used.
//! The oracle is what the model actually receives on the follow-up request.
use aura_app::{
    CodexRuntime, Host, HostConfig, events::HostEvent, host::SendRequest, paths::AppPaths,
    platform::Platform,
};
use aura_codex::{events::ConversationEvent, modes::ConversationMode, service::StartOptions};
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
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

const PAGE: &str = "https://news.example/report";
const ARTICLE: &str = r#"<!doctype html><html><head><title>Relatório de teste</title></head><body>
<main><h1>Produção regional</h1><p>A produção foi 42 unidades.</p>
<ul><li>Norte: 12</li><li>Sul: 30</li></ul></main></body></html>"#;

struct Network;
impl Transport for Network {
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
        let (status, mime, bytes) = if url == PAGE {
            (200, "text/html; charset=utf-8", ARTICLE.as_bytes().to_vec())
        } else if url == "https://search.parallel.ai/mcp" {
            let value = match rpc["method"].as_str() {
                Some("initialize") => {
                    json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"fixture","version":"1"}}})
                }
                Some("notifications/initialized") => Value::Null,
                Some("tools/list") => {
                    json!({"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"web_search","inputSchema":{"type":"object","required":["objective","search_queries"],"properties":{"objective":{"type":"string"},"search_queries":{"type":"array","items":{"type":"string"}},"session_id":{"type":"string"}}}}]}})
                }
                Some("tools/call") => {
                    json!({"jsonrpc":"2.0","id":3,"result":{"structuredContent":{"results":[
                        {"title":"Relatorio regional","url":PAGE,"excerpts":["Resumo regional"]}
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

/// What the scripted model does at a given request index.
#[derive(Clone)]
enum Step {
    Say(&'static str),
    Call(&'static str, Value),
}

fn event(kind: &str, data: Value) -> String {
    format!("event: {kind}\ndata: {data}\n\n")
}

fn aura_tool(body: &Value, wanted: &str) -> (String, Option<String>) {
    for item in body["tools"].as_array().into_iter().flatten() {
        if item["type"] == "namespace" && item["name"] == "mcp__aura" {
            return (wanted.into(), Some("mcp__aura".into()));
        } else if item["name"] == format!("mcp__aura__{wanted}") {
            return (item["name"].as_str().unwrap().into(), None);
        }
    }
    panic!("aura tool {wanted} not offered to the model");
}

fn reply(body: &Value, index: usize, step: &Step) -> String {
    let item = match step {
        Step::Say(text) => {
            json!({"type":"message","id":format!("msg_{index}"),"role":"assistant","status":"completed","content":[{"type":"output_text","text":text,"annotations":[]}]})
        }
        Step::Call(tool, args) => {
            let (name, namespace) = aura_tool(body, tool);
            let mut item = json!({"type":"function_call","id":format!("fc_{index}"),"call_id":format!("call_{index}"),"name":name,"arguments":args.to_string()});
            if let Some(namespace) = namespace {
                item["namespace"] = json!(namespace);
            }
            item
        }
    };
    let id = format!("resp_{index}");
    event(
        "response.created",
        json!({"type":"response.created","response":{"id":id,"status":"in_progress","output":[]}}),
    ) + &event(
        "response.output_item.added",
        json!({"type":"response.output_item.added","output_index":0,"item":item}),
    ) + &event(
        "response.output_item.done",
        json!({"type":"response.output_item.done","output_index":0,"item":item}),
    ) + &event(
        "response.completed",
        json!({"type":"response.completed","response":{"id":id,"status":"completed","output":[item],"usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    )
}

struct Provider {
    base_url: String,
    requests: Arc<Mutex<Vec<Value>>>,
    server: tokio::task::JoinHandle<()>,
}

async fn provider(script: Vec<Step>) -> Provider {
    let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
    let seen = requests.clone();
    let script = Arc::new(script);
    let router = Router::new().route(
        "/v1/responses",
        post(move |axum::Json(body): axum::Json<Value>| {
            let seen = seen.clone();
            let script = script.clone();
            async move {
                let index = {
                    let mut seen = seen.lock().unwrap();
                    seen.push(body.clone());
                    seen.len() - 1
                };
                let step = script
                    .get(index)
                    .cloned()
                    .unwrap_or(Step::Say("(fim do roteiro)"));
                (
                    [("content-type", "text/event-stream")],
                    reply(&body, index, &step),
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}/v1", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    Provider {
        base_url,
        requests,
        server,
    }
}

fn workdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("aura-continuity-")
        .tempdir_in(
            std::env::var_os("AURA_E2E_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir),
        )
        .unwrap()
}

async fn host(dir: &Path, web: bool) -> Arc<Host> {
    let binary = std::env::var_os("AURA_CODEX_BIN").expect("pinned app-server required");
    let (platform, _) = Platform::fake();
    let mut config = HostConfig::demo(AppPaths::new(dir.join("Aura")), platform);
    config.codex = CodexRuntime::Binary {
        program: Some(binary.into()),
        on_spawn: None,
    };
    if web {
        config.web = Some(Arc::new(WebService::new(
            Arc::new(Network),
            Arc::new(aura_web::SystemClock),
        )));
    }
    Host::start(config).await.unwrap()
}

async fn start(host: &Host, base_url: &str) -> aura_codex::service::StartedConversation {
    let provider = host.save_provider(serde_json::from_value(json!({"name":"Continuity fixture","preset":"custom","wire":"responses","baseUrl":base_url,"auth":"none"})).unwrap(), None).unwrap();
    host.start_conversation(StartOptions {
        provider: format!("aura-{}", provider.id),
        model: Some("fixture-model".into()),
        ..Default::default()
    })
    .await
    .unwrap()
}

/// Sends one user message and waits for that Turn to finish; returns the answer.
async fn turn(host: &Host, thread_id: &str, text: &str) -> String {
    let mut events = host.subscribe();
    host.send(SendRequest {
        thread_id: thread_id.into(),
        text: text.into(),
        tray: thread_id.into(),
        accepts_images: false,
        options: Default::default(),
    })
    .await
    .unwrap();
    let mut answer = String::new();
    tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            match events.recv().await.unwrap() {
                HostEvent::Conversation(ConversationEvent::MessageCompleted { text, .. }) => {
                    answer = text
                }
                HostEvent::Conversation(ConversationEvent::TurnCompleted { error, .. }) => {
                    assert!(error.is_none(), "turn failed: {error:?}");
                    break;
                }
                _ => {}
            }
        }
    })
    .await
    .expect("turn completes");
    answer
}

/// Text the model receives as conversation input (everything except tool specs).
fn model_input(request: &Value) -> String {
    request["input"].to_string()
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn a_follow_up_turn_still_sees_the_page_read_in_the_previous_turn() {
    let provider = provider(vec![
        Step::Call(
            "web_search",
            json!({"objective":"Verificar produção regional","queries":["producao regional"]}),
        ),
        Step::Call("web_fetch", json!({"url":PAGE})),
        Step::Say("Sim, é real: a produção foi 42 unidades [[aura-source:W1]]."),
        Step::Say("No Sul foram 30 unidades, segundo a página lida antes."),
    ])
    .await;
    let dir = workdir();
    let host = host(dir.path(), true).await;
    let conversation = start(&host, &provider.base_url).await;
    turn(&host, &conversation.thread_id, "Isso é real? Pesquise.").await;
    turn(&host, &conversation.thread_id, "E quanto foi no Sul?").await;
    host.shutdown().await;
    provider.server.abort();

    let requests = provider.requests.lock().unwrap();
    assert_eq!(
        requests.len(),
        4,
        "two tool calls + answer, then one answer"
    );
    let same_turn = model_input(&requests[2]);
    assert!(
        same_turn.contains("A produção foi 42 unidades"),
        "sanity: within the research Turn the page reaches the model"
    );
    let follow_up = model_input(&requests[3]);
    let history: Vec<String> = requests[3]["input"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| {
            matches!(
                item["type"].as_str(),
                Some("function_call" | "function_call_output")
            )
        })
        .map(|item| item.to_string())
        .collect();
    assert!(
        !follow_up.contains("no longer available"),
        "the follow-up Turn received an 'unavailable' placeholder instead of \
         the research the assistant itself did one message earlier:\n{}",
        history.join("\n")
    );
    assert!(
        follow_up.contains("Sul: 30"),
        "the page read in Turn 1 must still be visible to the model in Turn 2"
    );
    assert!(
        follow_up.contains(PAGE),
        "the model must still know which URL it read (tool arguments)"
    );
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn plain_facts_survive_follow_ups_and_an_app_restart() {
    let provider = provider(vec![
        Step::Say("Anotado: ZEBRA-7341."),
        Step::Say("Seu código é ZEBRA-7341."),
        Step::Say("Continua ZEBRA-7341."),
    ])
    .await;
    let dir = workdir();
    let first = host(dir.path(), false).await;
    let conversation = start(&first, &provider.base_url).await;
    turn(
        &first,
        &conversation.thread_id,
        "Meu código é ZEBRA-7341. Guarde.",
    )
    .await;
    turn(&first, &conversation.thread_id, "Qual é o meu código?").await;
    first.shutdown().await;
    drop(first);

    let second = host(dir.path(), false).await;
    let transcript = second
        .open_conversation(&conversation.thread_id)
        .await
        .unwrap();
    assert_eq!(
        transcript.len(),
        4,
        "history shows both exchanges after restart"
    );
    turn(
        &second,
        &conversation.thread_id,
        "E agora, depois de reiniciar?",
    )
    .await;
    second.shutdown().await;
    provider.server.abort();

    let requests = provider.requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    for (index, request) in requests.iter().enumerate().skip(1) {
        let input = model_input(request);
        assert!(
            input.contains("Meu código é ZEBRA-7341"),
            "request {index} lost the user fact"
        );
        assert!(
            input.contains("Anotado: ZEBRA-7341."),
            "request {index} lost the earlier answer"
        );
    }
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn reopening_after_restart_does_not_announce_a_mode_change_that_never_happened() {
    let provider = provider(vec![Step::Say("Oi."), Step::Say("Oi de novo.")]).await;
    let dir = workdir();
    let first = host(dir.path(), false).await;
    let conversation = start(&first, &provider.base_url).await;
    turn(&first, &conversation.thread_id, "Olá").await;
    first.shutdown().await;
    drop(first);

    let second = host(dir.path(), false).await;
    second
        .open_conversation(&conversation.thread_id)
        .await
        .unwrap();
    turn(&second, &conversation.thread_id, "Continuando").await;
    second.shutdown().await;
    provider.server.abort();

    let requests = provider.requests.lock().unwrap();
    let last = requests.last().unwrap()["input"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|item| item["role"] == "user")
        .unwrap()
        .to_string();
    assert!(
        !last.contains("<aura-mode>"),
        "the user never changed the mode (still Chat), yet the model is told \
         'Mudança de modo pedida pelo usuário': {last}"
    );
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn switching_chat_to_task_mid_conversation_reaches_the_model() {
    let provider = provider(vec![Step::Say("Só leitura."), Step::Say("Agora posso.")]).await;
    let dir = workdir();
    let host = host(dir.path(), false).await;
    let conversation = start(&host, &provider.base_url).await;
    turn(&host, &conversation.thread_id, "Crie uma pasta.").await;
    host.set_mode(
        &conversation.thread_id,
        ConversationMode::Task {
            granted: vec![],
            network: false,
        },
    )
    .await
    .unwrap();
    turn(
        &host,
        &conversation.thread_id,
        "Você já está no modo tarefa, pode fazer.",
    )
    .await;
    host.shutdown().await;
    provider.server.abort();

    let requests = provider.requests.lock().unwrap();
    let follow_up = model_input(&requests[1]);
    assert!(
        follow_up.contains("Modo Tarefa"),
        "the Task mode instruction must reach the model after the switch"
    );
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn a_mode_switch_made_just_before_a_restart_still_reaches_the_model() {
    let provider = provider(vec![Step::Say("Só leitura."), Step::Say("Agora posso.")]).await;
    let dir = workdir();
    let first = host(dir.path(), false).await;
    let conversation = start(&first, &provider.base_url).await;
    turn(&first, &conversation.thread_id, "Olá").await;
    first
        .set_mode(
            &conversation.thread_id,
            ConversationMode::Task {
                granted: vec![],
                network: false,
            },
        )
        .await
        .unwrap();
    first.shutdown().await;
    drop(first);

    let second = host(dir.path(), false).await;
    turn(&second, &conversation.thread_id, "Crie a pasta.").await;
    second.shutdown().await;
    provider.server.abort();

    let requests = provider.requests.lock().unwrap();
    assert!(
        model_input(&requests[1]).contains("Modo Tarefa"),
        "the switch to Task was never told to the model"
    );
}

#[tokio::test]
#[ignore = "requires the pinned app-server in AURA_CODEX_BIN; no account/Internet"]
async fn research_from_before_an_app_restart_is_still_visible_to_the_model() {
    let provider = provider(vec![
        Step::Call(
            "web_search",
            json!({"objective":"Verificar produção regional","queries":["producao regional"]}),
        ),
        Step::Call("web_fetch", json!({"url":PAGE})),
        Step::Say("A produção foi 42 unidades [[aura-source:W1]]."),
        Step::Say("No Norte foram 12."),
    ])
    .await;
    let dir = workdir();
    let first = host(dir.path(), true).await;
    let conversation = start(&first, &provider.base_url).await;
    turn(
        &first,
        &conversation.thread_id,
        "Pesquise a produção regional.",
    )
    .await;
    first.shutdown().await;
    drop(first);

    let second = host(dir.path(), true).await;
    turn(&second, &conversation.thread_id, "E no Norte?").await;
    second.shutdown().await;
    provider.server.abort();

    let requests = provider.requests.lock().unwrap();
    let follow_up = model_input(&requests[3]);
    assert!(
        follow_up.contains("Norte: 12"),
        "page content lost after restart"
    );
    assert!(follow_up.contains(PAGE), "read URL lost after restart");
}
