//! Public MCP requests through real HostTools/Host; only OS/process/network vary.
use aura_app::{
    Host, HostConfig,
    paths::AppPaths,
    platform::Platform,
    tools::{HostTools, NoRecentMedia},
};
use aura_mcp::{ToolHandler, router, tools};
use serde_json::{Value, json};
use std::sync::{Arc, RwLock};

struct Env {
    host: Arc<Host>,
    platform: Platform,
    _dir: tempfile::TempDir,
    web: Arc<aura_web::WebService>,
    threads: Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
}
async fn env() -> Env {
    env_with_web(None).await
}
async fn env_with_web(web: Option<Arc<aura_web::WebService>>) -> Env {
    let dir = tempfile::tempdir().unwrap();
    let (platform, _) = Platform::fake();
    let mut config = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform.clone());
    config.in_memory_store = true;
    let web = web.unwrap_or_else(|| Arc::new(aura_web::WebService::live()));
    config.web = Some(web.clone());
    Env {
        host: Host::start(config).await.unwrap(),
        platform,
        _dir: dir,
        web,
        threads: Arc::default(),
    }
}
fn handler(env: &Env) -> Arc<dyn ToolHandler> {
    let store = aura_store::Store::open_in_memory().unwrap();
    let vault = Arc::new(
        aura_store::Vault::open(&store, &aura_store::StaticKeyProtector::default()).unwrap(),
    );
    let privacy = aura_app::privacy::PrivacyRepo::new(store);
    let slot = Arc::new(std::sync::OnceLock::new());
    let access: Arc<dyn aura_app::tools::ExtensionsAccess> = env.host.clone();
    slot.set(Arc::downgrade(&access)).unwrap();
    let web_slot = Arc::new(std::sync::OnceLock::new());
    let access: Arc<dyn aura_app::web::WebAccess> = env.host.clone();
    web_slot.set(Arc::downgrade(&access)).unwrap();
    Arc::new(HostTools {
        extensions: slot,
        web: web_slot,
        platform: env.platform.clone(),
        policy: Arc::new(RwLock::new(privacy.load().unwrap())),
        grants: Arc::new(RwLock::new(Default::default())),
        privacy,
        consent: Arc::new(Default::default()),
        events: tokio::sync::broadcast::channel(64).0,
        attachments: Arc::new(aura_app::attachments::AttachmentService::new(
            Arc::new(aura_ingest::NoHeavy),
            env._dir.path().join("ingest"),
        )),
        recent: Arc::new(NoRecentMedia),
        ocr_language: "pt-BR".into(),
        vault,
    })
}
// The external process is simulated by this client; both HTTP boundaries and
// transient storage are real. Echo is the model provider, not a web-service mock.
#[derive(Default)]
struct Echo {
    requests: std::sync::Mutex<std::collections::HashMap<String, Value>>,
}
impl aura_gateway::upstream::Upstream for Echo {
    fn responses(
        &self,
        body: axum::body::Bytes,
    ) -> aura_gateway::upstream::BoxFut<
        '_,
        Result<aura_gateway::upstream::StreamResponse, aura_gateway::errors::UpstreamError>,
    > {
        let request: Value = serde_json::from_slice(&body).unwrap();
        let call_id = request["input"][0]["call_id"].as_str().unwrap().to_owned();
        self.requests.lock().unwrap().insert(call_id, request);
        Box::pin(async move {
            Ok(aura_gateway::upstream::StreamResponse {
                content_type: "text/event-stream".into(),
                body: Box::pin(futures::stream::empty()),
            })
        })
    }
    fn models(
        &self,
    ) -> aura_gateway::upstream::BoxFut<'_, Result<Value, aura_gateway::errors::UpstreamError>>
    {
        Box::pin(async { Ok(json!({"data":[]})) })
    }
}
struct Source {
    web: Arc<aura_web::WebService>,
    threads: Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
}
impl aura_gateway::tool_results::ToolResultResolver for Source {
    fn resolve(
        &self,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, aura_gateway::errors::UpstreamError> {
        let threads = self.threads.lock().unwrap();
        let conversation = threads.get(thread).unwrap();
        self.web
            .resolve_tool_result(conversation, thread, tool, handle)
            .map_err(|_| {
                aura_gateway::errors::UpstreamError::new(
                    403,
                    "web_result_scope_invalid",
                    "Invalid scope",
                )
            })
    }
}
#[derive(Clone)]
struct Endpoint {
    url: String,
    gateway: Arc<aura_gateway::server::GatewayHandle>,
    threads: Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
    provider: Arc<Echo>,
}
async fn endpoint(env: &Env) -> Endpoint {
    let app = router(tools::all(), handler(env), "test-token".into());
    let routes = aura_gateway::server::Routes::new();
    let provider = Arc::new(Echo::default());
    routes.set("fixture", provider.clone());
    routes.set_tool_results(Arc::new(Source {
        web: env.web.clone(),
        threads: env.threads.clone(),
    }));
    let gateway = Arc::new(
        aura_gateway::server::start(routes, Some(app))
            .await
            .unwrap(),
    );
    Endpoint {
        url: format!("{}/mcp", gateway.base_url()),
        gateway,
        threads: env.threads.clone(),
        provider,
    }
}
async fn call(endpoint: &Endpoint, tool: &str, args: Value, conversation: Option<&str>) -> Value {
    let mut request=reqwest::Client::new().post(&endpoint.url).bearer_auth("test-token").json(&json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":tool,"arguments":args}}));
    if let Some(conversation) = conversation {
        request = request.header("X-Aura-Conversation", conversation);
    }
    let mut result: Value = request.send().await.unwrap().json().await.unwrap();
    if result["result"]["isError"] == false && payload(&result)["auraToolResult"].is_object() {
        let raw = result["result"]["content"][0]["text"].clone();
        assert!(
            !raw.as_str().unwrap().contains("https://"),
            "MCP must carry an opaque reference, not source data"
        );
        let thread = {
            let threads = endpoint.threads.lock().unwrap();
            threads
                .iter()
                .find(|(_, uuid)| conversation.is_none_or(|scope| scope == uuid.as_str()))
                .unwrap()
                .0
                .clone()
        };
        let call_id = uuid::Uuid::new_v4().to_string();
        reqwest::Client::new()
            .post(format!("{}/p/fixture/v1/responses", endpoint.gateway.base_url()))
            .bearer_auth(endpoint.gateway.token.expose()).header("thread-id", thread)
            .json(&json!({"input":[
                {"type":"function_call","call_id":call_id,"namespace":"mcp__aura","name":tool,"arguments":args.to_string()},
                {"type":"function_call_output","call_id":call_id,"output":[{"type":"input_text","text":raw}]}
            ],"stream":true})).send().await.unwrap().error_for_status().unwrap().bytes().await.unwrap();
        let delivered = endpoint
            .provider
            .requests
            .lock()
            .unwrap()
            .remove(&call_id)
            .unwrap();
        // Assertions observe the provider's actual input after gateway expansion.
        result["result"]["content"][0]["text"] = delivered["input"][1]["output"][0]["text"].clone();
    }
    result
}
fn payload(result: &Value) -> Value {
    serde_json::from_str(result["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[tokio::test]
async fn a_model_known_to_lack_tools_reports_the_web_limitation_without_blocking_chat() {
    let env = env().await;
    let provider=env.host.save_provider(serde_json::from_value(json!({"name":"Text only","preset":"custom","baseUrl":"https://provider.example/v1"})).unwrap(),Some("fixture-secret".into())).unwrap();
    env.host.save_provider_model(&provider.id,serde_json::from_value(json!({"id":"text-only","displayName":null,"contextWindow":null,"maxOutput":null,"supportsImages":false,"supportsTools":false,"supportsReasoning":false,"estimated":false,"manual":true})).unwrap()).unwrap();
    let mut events = env.host.subscribe();
    let thread = env
        .host
        .start_conversation(aura_codex::service::StartOptions {
            provider: format!("aura-{}", provider.id),
            model: Some("text-only".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    let mut warned = false;
    while let Ok(event) = events.try_recv() {
        if let aura_app::events::HostEvent::Notice { level, message } = event
            && level == "warning"
            && message == "web_tools_unsupported"
        {
            warned = true;
        }
    }
    assert!(
        warned,
        "text-only model must explain why internet research is unavailable"
    );
    assert!(
        !thread.thread_id.is_empty(),
        "ordinary chat remains available"
    );
    env.host.shutdown().await;
}

#[tokio::test]
async fn refuses_web_access_without_a_trusted_active_turn() {
    let env = env().await;
    let url = endpoint(&env).await;
    for tool in ["web_search", "web_fetch"] {
        let args = if tool == "web_search" {
            json!({"objective":"Find a manual","queries":["Aura manual"]})
        } else {
            json!({"url":"https://news.example/manual"})
        };
        let output = call(&url, tool, args, None).await;
        assert_eq!(output["result"]["isError"], true);
        assert_eq!(payload(&output)["code"], "invalid_input");
    }
    env.host.shutdown().await;
}

#[derive(Default)]
struct Network {
    requests: std::sync::Mutex<Vec<aura_web::transport::HttpRequest>>,
    pages: std::sync::Mutex<std::collections::VecDeque<aura_web::transport::HttpResponse>>,
}
impl Network {
    fn plain(&self, text: &str) {
        self.pages
            .lock()
            .unwrap()
            .push_back(aura_web::transport::HttpResponse {
                status: 200,
                headers: std::collections::BTreeMap::from([(
                    "content-type".into(),
                    "text/plain; charset=utf-8".into(),
                )]),
                body: text.as_bytes().to_vec().into(),
            });
    }
    fn json(&self, value: Value) {
        self.pages
            .lock()
            .unwrap()
            .push_back(aura_web::transport::HttpResponse {
                status: 200,
                headers: std::collections::BTreeMap::from([(
                    "content-type".into(),
                    "application/json".into(),
                )]),
                body: serde_json::to_vec(&value).unwrap().into(),
            });
    }
    fn search(&self) {
        self.json(json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"Fixture","version":"1"}}}));
        self.pages
            .lock()
            .unwrap()
            .push_back(aura_web::transport::HttpResponse {
                status: 204,
                headers: Default::default(),
                body: vec![].into(),
            });
        self.json(json!({"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"web_search","inputSchema":{"type":"object","required":["objective","search_queries"],"properties":{"objective":{"type":"string"},"search_queries":{"type":"array","items":{"type":"string"}},"session_id":{"type":"string"}}}}]}}));
        self.json(json!({"jsonrpc":"2.0","id":3,"result":{"structuredContent":{"results":[{"title":"Public report","url":"https://news.example/report","excerpts":["Production: 42 units."],"publish_date":null}]},"content":[],"isError":false}}));
    }
}
impl aura_web::transport::Transport for Network {
    fn resolve<'a>(
        &'a self,
        _host: &'a str,
    ) -> aura_mcp::BoxFut<'a, Result<Vec<std::net::IpAddr>, aura_web::WebError>> {
        Box::pin(async { Ok(vec!["93.184.216.34".parse().unwrap()]) })
    }
    fn request(
        &self,
        request: aura_web::transport::HttpRequest,
    ) -> aura_mcp::BoxFut<'_, Result<aura_web::transport::HttpResponse, aura_web::WebError>> {
        self.requests.lock().unwrap().push(request);
        let page = self.pages.lock().unwrap().pop_front();
        Box::pin(async move { page.ok_or(aura_web::WebError::Unavailable) })
    }
}
struct Clock;
impl aura_web::Clock for Clock {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH
    }
}

#[tokio::test]
async fn older_settings_enable_web_by_default() {
    let old: aura_core::settings::Settings =
        serde_json::from_value(json!({"language":"en","memories":false})).unwrap();
    assert_eq!(serde_json::to_value(old).unwrap()["webEnabled"], true);
    let env = env().await;
    assert_eq!(
        serde_json::to_value(env.host.settings()).unwrap()["webEnabled"],
        true
    );
    env.host.shutdown().await;
}

#[tokio::test]
async fn disabled_web_is_restored_after_a_host_restart_before_any_tool_can_run() {
    let dir = tempfile::tempdir().unwrap();
    let (platform, _) = Platform::fake();
    let network = Arc::new(Network::default());
    let config = || {
        let mut config = HostConfig::demo(AppPaths::new(dir.path().join("Aura")), platform.clone());
        config.web = Some(Arc::new(aura_web::WebService::new(
            network.clone(),
            Arc::new(Clock),
        )));
        config
    };
    let first = Host::start(config()).await.unwrap();
    first
        .update_settings(serde_json::from_value(json!({"webEnabled":false})).unwrap())
        .unwrap();
    first.shutdown().await;
    drop(first);
    let second_config = config();
    let web = second_config.web.as_ref().unwrap().clone();
    let env = Env {
        host: Host::start(second_config).await.unwrap(),
        platform,
        _dir: dir,
        web,
        threads: Arc::default(),
    };
    assert!(!env.host.settings().web_enabled);
    let _thread = active(&env).await;
    let url = endpoint(&env).await;
    assert_eq!(
        payload(
            &call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None
            )
            .await
        )["code"],
        "web_disabled"
    );
    assert!(network.requests.lock().unwrap().is_empty());
    env.host.shutdown().await;
}
async fn active(env: &Env) -> aura_codex::service::StartedConversation {
    active_with(env, Default::default()).await
}
async fn active_with(
    env: &Env,
    options: aura_codex::service::StartOptions,
) -> aura_codex::service::StartedConversation {
    let mut events = env.host.subscribe();
    let thread = env.host.start_conversation(options).await.unwrap();
    env.threads
        .lock()
        .unwrap()
        .insert(thread.thread_id.clone(), thread.conversation_uuid.clone());
    env.host
        .send(aura_app::host::SendRequest {
            thread_id: thread.thread_id.clone(),
            text: "/lento pesquisar informação pública".into(),
            tray: thread.thread_id.clone(),
            accepts_images: true,
            options: Default::default(),
        })
        .await
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if let Ok(aura_app::events::HostEvent::Conversation(
                aura_codex::ConversationEvent::TurnStarted { thread_id, .. },
            )) = events.recv().await
                && thread_id == thread.thread_id
            {
                break;
            }
        }
    })
    .await
    .unwrap();
    thread
}

#[tokio::test]
async fn a_single_trusted_turn_can_read_a_public_source_without_a_header() {
    let network = Arc::new(Network::default());
    network.plain("A produção foi 42 unidades.");
    let service = Arc::new(aura_web::WebService::new(network.clone(), Arc::new(Clock)));
    let env = env_with_web(Some(service)).await;
    let thread = active(&env).await;
    let url = endpoint(&env).await;
    let output = call(
        &url,
        "web_fetch",
        json!({"url":"https://news.example/report"}),
        None,
    )
    .await;
    assert_eq!(output["result"]["isError"], false);
    let page = payload(&output);
    assert_eq!(page["sourceId"], "W1");
    assert_eq!(page["text"], "A produção foi 42 unidades.");
    assert_eq!(page["externalContent"], true);
    assert_eq!(page["retrievedAt"], "1970-01-01T00:00:00Z");
    assert_eq!(network.requests.lock().unwrap().len(), 1);
    env.host.interrupt(&thread.thread_id).await.unwrap();
    env.host.shutdown().await;
}

#[tokio::test]
async fn disabling_web_cancels_reads_blocks_cache_and_resumes_without_a_new_budget() {
    let network = Arc::new(Network::default());
    network.plain("Production: 42");
    network
        .pages
        .lock()
        .unwrap()
        .push_back(aura_web::transport::HttpResponse {
            status: 200,
            headers: Default::default(),
            body: aura_web::transport::HttpBody::new(futures_pending()),
        });
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let _thread = active(&env).await;
    let url = endpoint(&env).await;
    let args = json!({"url":"https://news.example/report"});
    assert_eq!(
        payload(&call(&url, "web_fetch", args.clone(), None).await)["text"],
        "Production: 42"
    );
    let pending = {
        let url = url.clone();
        tokio::spawn(async move {
            call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/pending"}),
                None,
            )
            .await
        })
    };
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while network.requests.lock().unwrap().len() < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let mut expected = serde_json::to_value(env.host.settings()).unwrap();
    expected["webEnabled"] = json!(false);
    let saved = env
        .host
        .update_settings(serde_json::from_value(json!({"webEnabled":false})).unwrap())
        .unwrap();
    assert_eq!(
        serde_json::to_value(saved).unwrap(),
        expected,
        "web must not alter capture or command settings"
    );
    let stopped = tokio::time::timeout(std::time::Duration::from_millis(100), pending)
        .await
        .expect("disabling web cancels pending reads immediately")
        .unwrap();
    assert_eq!(payload(&stopped)["code"], "cancelled");
    for (tool, input) in [
        ("web_fetch", args.clone()),
        (
            "web_search",
            json!({"objective":"Public report","queries":["production"]}),
        ),
    ] {
        assert_eq!(
            payload(&call(&url, tool, input, None).await)["code"],
            "web_disabled"
        );
    }
    assert_eq!(network.requests.lock().unwrap().len(), 2);
    env.host
        .update_settings(serde_json::from_value(json!({"webEnabled":true})).unwrap())
        .unwrap();
    for _ in 0..4 {
        let result = call(&url, "web_fetch", args.clone(), None).await;
        assert_eq!(result["result"]["isError"], false);
        assert_eq!(payload(&result)["cached"], true);
        assert_eq!(payload(&result)["sourceId"], "W1");
    }
    assert_eq!(
        payload(&call(&url, "web_fetch", args, None).await)["code"],
        "limit_exceeded"
    );
    assert_eq!(network.requests.lock().unwrap().len(), 2);
    env.host.shutdown().await;
}

#[tokio::test]
async fn ambiguous_turns_require_a_valid_header_and_keep_sources_isolated() {
    let network = Arc::new(Network::default());
    network.plain("Production: 42");
    network.plain("Production: 13");
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let first = active(&env).await;
    let second = active(&env).await;
    let url = endpoint(&env).await;
    let args = json!({"url":"https://news.example/report"});
    for header in [None, Some("unknown-conversation")] {
        let output = call(&url, "web_fetch", args.clone(), header).await;
        assert_eq!(payload(&output)["code"], "invalid_input");
        assert!(network.requests.lock().unwrap().is_empty());
    }
    let a = call(
        &url,
        "web_fetch",
        args.clone(),
        Some(&first.conversation_uuid),
    )
    .await;
    assert_eq!(a["result"]["isError"], false);
    assert_eq!(payload(&a)["sourceId"], "W1");
    assert_eq!(payload(&a)["text"], "Production: 42");
    let b = call(&url, "web_fetch", args, Some(&second.conversation_uuid)).await;
    assert_eq!(b["result"]["isError"], false);
    assert_eq!(payload(&b)["sourceId"], "W1");
    assert_eq!(payload(&b)["text"], "Production: 13");
    assert_eq!(network.requests.lock().unwrap().len(), 2);
    env.host.shutdown().await;
}

#[tokio::test]
async fn interrupt_cancels_pending_web_before_waiting_for_process_completion() {
    let network = Arc::new(Network::default());
    network
        .pages
        .lock()
        .unwrap()
        .push_back(aura_web::transport::HttpResponse {
            status: 200,
            headers: Default::default(),
            body: aura_web::transport::HttpBody::new(futures_pending()),
        });
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let thread = active(&env).await;
    let url = endpoint(&env).await;
    let pending = {
        let url = url.clone();
        tokio::spawn(async move {
            call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None,
            )
            .await
        })
    };
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while network.requests.lock().unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    env.host.interrupt(&thread.thread_id).await.unwrap();
    let output = tokio::time::timeout(std::time::Duration::from_millis(100), pending)
        .await
        .expect("web cancellation must not wait for process completion")
        .unwrap();
    assert_eq!(payload(&output)["code"], "cancelled");
    assert_eq!(
        payload(
            &call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None
            )
            .await
        )["code"],
        "invalid_input"
    );
    env.host.shutdown().await;
}
fn futures_pending() -> impl futures::Stream<Item = Result<Vec<u8>, aura_web::WebError>> + Send {
    futures::stream::pending()
}

#[tokio::test]
async fn closing_an_ephemeral_conversation_cancels_its_pending_web_read() {
    let network = Arc::new(Network::default());
    network
        .pages
        .lock()
        .unwrap()
        .push_back(aura_web::transport::HttpResponse {
            status: 200,
            headers: Default::default(),
            body: aura_web::transport::HttpBody::new(futures_pending()),
        });
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let thread = active_with(
        &env,
        aura_codex::service::StartOptions {
            ephemeral: true,
            ..Default::default()
        },
    )
    .await;
    let url = endpoint(&env).await;
    let pending = {
        let url = url.clone();
        tokio::spawn(async move {
            call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None,
            )
            .await
        })
    };
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while network.requests.lock().unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    env.host.close_ephemeral(&thread.thread_id).await.unwrap();
    let output = tokio::time::timeout(std::time::Duration::from_millis(100), pending)
        .await
        .expect("closing must cancel web immediately")
        .unwrap();
    assert_eq!(payload(&output)["code"], "cancelled");
    assert_eq!(
        payload(
            &call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None
            )
            .await
        )["code"],
        "invalid_input"
    );
    env.host.shutdown().await;
}

#[tokio::test]
async fn shutting_down_cancels_pending_web_even_without_a_completed_turn() {
    let network = Arc::new(Network::default());
    network
        .pages
        .lock()
        .unwrap()
        .push_back(aura_web::transport::HttpResponse {
            status: 200,
            headers: Default::default(),
            body: aura_web::transport::HttpBody::new(futures_pending()),
        });
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let _thread = active(&env).await;
    let url = endpoint(&env).await;
    let pending = {
        let url = url.clone();
        tokio::spawn(async move {
            call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None,
            )
            .await
        })
    };
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while network.requests.lock().unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    env.host.shutdown().await;
    let output = tokio::time::timeout(std::time::Duration::from_millis(100), pending)
        .await
        .expect("shutdown must cancel web even without turn/completed")
        .unwrap();
    assert_eq!(payload(&output)["code"], "cancelled");
    assert_eq!(
        payload(
            &call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None
            )
            .await
        )["code"],
        "invalid_input"
    );
}

#[tokio::test]
async fn deleting_a_conversation_clears_its_document_cache_after_the_turn_finished() {
    let network = Arc::new(Network::default());
    network.plain("Old fact: 42");
    network.plain("New fact: 13");
    let service = Arc::new(aura_web::WebService::new(network.clone(), Arc::new(Clock)));
    let env = env_with_web(Some(service.clone())).await;
    let thread = active(&env).await;
    let url = endpoint(&env).await;
    let page = call(
        &url,
        "web_fetch",
        json!({"url":"https://news.example/report"}),
        None,
    )
    .await;
    assert_eq!(payload(&page)["text"], "Old fact: 42");
    env.host.interrupt(&thread.thread_id).await.unwrap();
    env.host
        .delete_conversation(&thread.thread_id)
        .await
        .unwrap();
    let context = service.begin_turn(&thread.conversation_uuid, "new-trusted-turn");
    let page = service
        .fetch(
            &context,
            serde_json::from_value(json!({"url":"https://news.example/report"})).unwrap(),
        )
        .await
        .unwrap();
    assert!(!page.cached);
    assert_eq!(page.text, "New fact: 13");
    assert_eq!(network.requests.lock().unwrap().len(), 2);
    env.host.shutdown().await;
}

#[tokio::test]
async fn a_completed_turn_cannot_reuse_web_authority_or_reset_the_budget_with_arguments() {
    let network = Arc::new(Network::default());
    network.plain("Production: 42");
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let thread = active(&env).await;
    let url = endpoint(&env).await;
    for argument in ["turnId", "conversationId", "sessionId"] {
        let mut args = json!({"url":"https://news.example/report"});
        args[argument] = json!("invented-authority");
        assert_eq!(
            payload(&call(&url, "web_fetch", args, None).await)["code"],
            "invalid_input"
        );
    }
    assert!(network.requests.lock().unwrap().is_empty());
    for _ in 0..6 {
        let result = call(
            &url,
            "web_fetch",
            json!({"url":"https://news.example/report"}),
            None,
        )
        .await;
        assert_eq!(result["result"]["isError"], false);
    }
    assert_eq!(
        payload(
            &call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                None
            )
            .await
        )["code"],
        "limit_exceeded"
    );
    let mut events = env.host.subscribe();
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if let aura_app::events::HostEvent::Conversation(
                aura_codex::ConversationEvent::TurnCompleted { thread_id, .. },
            ) = events.recv().await.unwrap()
                && thread_id == thread.thread_id
            {
                break;
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(
        payload(
            &call(
                &url,
                "web_fetch",
                json!({"url":"https://news.example/report"}),
                Some(&thread.conversation_uuid)
            )
            .await
        )["code"],
        "invalid_input"
    );
    let mut events = env.host.subscribe();
    env.host
        .send(aura_app::host::SendRequest {
            thread_id: thread.thread_id.clone(),
            text: "/lento pesquise de novo".into(),
            tray: thread.thread_id.clone(),
            accepts_images: false,
            options: Default::default(),
        })
        .await
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if let aura_app::events::HostEvent::Conversation(
                aura_codex::ConversationEvent::TurnStarted { thread_id, .. },
            ) = events.recv().await.unwrap()
                && thread_id == thread.thread_id
            {
                break;
            }
        }
    })
    .await
    .unwrap();
    let result = call(
        &url,
        "web_fetch",
        json!({"url":"https://news.example/report"}),
        None,
    )
    .await;
    assert_eq!(result["result"]["isError"], false);
    assert_eq!(payload(&result)["cached"], true);
    assert_eq!(network.requests.lock().unwrap().len(), 1);
    env.host.shutdown().await;
}

#[tokio::test]
async fn search_and_read_preserve_one_source_and_send_only_public_queries() {
    let network = Arc::new(Network::default());
    network.search();
    network.plain("Production: 42 units.");
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let thread = active(&env).await;
    let url = endpoint(&env).await;
    let search = call(
        &url,
        "web_search",
        json!({"objective":"Find public production report","queries":["production report"]}),
        None,
    )
    .await;
    assert_eq!(search["result"]["isError"], false);
    let sources = payload(&search);
    assert_eq!(sources["provider"], "parallel");
    assert_eq!(sources["results"][0]["sourceId"], "W1");
    assert_eq!(sources["results"][0]["kind"], "searchSnippet");
    let page = call(
        &url,
        "web_fetch",
        json!({"url":"https://news.example/report"}),
        None,
    )
    .await;
    assert_eq!(page["result"]["isError"], false);
    assert_eq!(payload(&page)["sourceId"], "W1");
    assert_eq!(payload(&page)["text"], "Production: 42 units.");
    {
        let sent = network.requests.lock().unwrap();
        assert_eq!(sent.len(), 5);
        let wire: Value = serde_json::from_slice(&sent[3].body).unwrap();
        assert_eq!(
            wire["params"]["arguments"]["objective"],
            "Find public production report"
        );
        assert_eq!(
            wire["params"]["arguments"]["search_queries"],
            json!(["production report"])
        );
        assert_eq!(wire["params"]["arguments"].as_object().unwrap().len(), 3);
        assert!(
            uuid::Uuid::parse_str(wire["params"]["arguments"]["session_id"].as_str().unwrap())
                .is_ok()
        );
        for request in sent.iter() {
            assert!(!String::from_utf8_lossy(&request.body).contains(&thread.conversation_uuid));
            assert!(!request.headers.contains_key("authorization"));
            assert!(!request.headers.contains_key("cookie"));
        }
    }
    env.host.shutdown().await;
}

#[tokio::test]
async fn public_source_events_keep_turn_identity_and_read_provenance_after_repeated_search() {
    let network = Arc::new(Network::default());
    network.search();
    network.plain("Verified production: 42 units.");
    let env = env_with_web(Some(Arc::new(aura_web::WebService::new(
        network.clone(),
        Arc::new(Clock),
    ))))
    .await;
    let thread = active(&env).await;
    let mut events = env.host.subscribe();
    let url = endpoint(&env).await;
    let search_args =
        json!({"objective":"Find public production report","queries":["production report"]});
    let search = call(&url, "web_search", search_args.clone(), None).await;
    assert_eq!(search["result"]["isError"], false);
    let mut source_events = Vec::new();
    while let Ok(event) = events.try_recv() {
        let event = serde_json::to_value(event).unwrap();
        if event["channel"] == "webSource" {
            source_events.push(event["event"].clone());
        }
    }
    assert_eq!(
        source_events.len(),
        1,
        "each successful source must reach the UI"
    );
    assert_eq!(source_events[0]["threadId"], thread.thread_id);
    assert!(
        source_events[0]["turnId"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert_eq!(source_events[0]["source"]["sourceId"], "W1");
    assert_eq!(source_events[0]["source"]["kind"], "searchSnippet");
    assert_eq!(
        source_events[0]["source"]["url"],
        "https://news.example/report"
    );
    call(
        &url,
        "web_fetch",
        json!({"url":"https://news.example/report"}),
        None,
    )
    .await;
    let repeated = call(&url, "web_search", search_args, None).await;
    assert_eq!(payload(&repeated)["cached"], true);
    network.search();
    call(&url, "web_search", json!({"objective":"Find public production report","queries":["production report"],"refresh":true}), None).await;
    while let Ok(event) = events.try_recv() {
        let event = serde_json::to_value(event).unwrap();
        if event["channel"] == "webSource" {
            source_events.push(event["event"].clone());
        }
    }
    assert_eq!(source_events.len(), 4);
    for event in &source_events[1..] {
        assert_eq!(event["threadId"], thread.thread_id);
        assert_eq!(event["turnId"], source_events[0]["turnId"]);
        assert_eq!(
            event["source"]["kind"], "pageContent",
            "later search must not downgrade a source already read"
        );
        assert_eq!(event["source"]["snippet"], "Verified production: 42 units.");
    }
    assert_eq!(network.requests.lock().unwrap().len(), 9);
    env.host.shutdown().await;
}
