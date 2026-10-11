//! HTTP consumer seam, with real web storage and only an external model fixture.
use aura_gateway::{
    errors::UpstreamError,
    server::{GatewayHandle, Routes, start},
    tool_results::{ToolCallProtector, ToolResultResolver, argument_envelope, argument_handle},
    upstream::{BoxFut, StreamResponse, Upstream},
};
use aura_web::{WebContext, WebService};
use bytes::Bytes;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

const URL: &str = "https://independent.example/report";
struct Gate {
    reached: tokio::sync::Notify,
    release: tokio::sync::Notify,
}
struct Provider {
    stream: Vec<Bytes>,
    requests: Mutex<Vec<Value>>,
    gate: Mutex<Option<Arc<Gate>>>,
    failure: Mutex<Option<UpstreamError>>,
}
impl Upstream for Provider {
    fn responses(&self, body: Bytes) -> BoxFut<'_, Result<StreamResponse, UpstreamError>> {
        self.requests
            .lock()
            .unwrap()
            .push(serde_json::from_slice(&body).unwrap());
        let stream = self.stream.clone();
        let gate = self.gate.lock().unwrap().clone();
        let failure = self.failure.lock().unwrap().take();
        Box::pin(async move {
            if let Some(error) = failure {
                return Err(error);
            }
            if let Some(gate) = gate {
                gate.reached.notify_one();
                gate.release.notified().await;
            }
            Ok(StreamResponse {
                content_type: "text/event-stream".into(),
                body: Box::pin(futures::stream::iter(stream.into_iter().map(Ok))),
            })
        })
    }
    fn models(&self) -> BoxFut<'_, Result<Value, UpstreamError>> {
        Box::pin(async { Ok(json!({"data":[]})) })
    }
}
struct Scope {
    service: Arc<WebService>,
    context: WebContext,
    thread: String,
    active: Arc<Mutex<WebContext>>,
    private_reasoning: bool,
}
impl ToolCallProtector for Scope {
    fn protects_reasoning(&self) -> bool {
        self.private_reasoning
    }
    fn protect_reasoning(&self, item: &Value) -> Result<Value, UpstreamError> {
        self.service
            .retain_private_reasoning(&self.context, &self.thread, item.to_string())
            .map(|handle| aura_gateway::tool_results::reasoning_envelope(item, &handle))
            .map_err(|_| UpstreamError::new(409, "web_reasoning_unavailable", "Unavailable"))
    }
    fn protect(&self, tool: &str, text: &str) -> Result<String, UpstreamError> {
        self.service
            .retain_tool_arguments(&self.context, &self.thread, tool, text.into())
            .map(|handle| argument_envelope(tool, &handle))
            .map_err(|_| UpstreamError::new(409, "web_input_unavailable", "Unavailable"))
    }
}
struct Source(Arc<Scope>);
impl ToolResultResolver for Source {
    fn resolve_reasoning(
        &self,
        thread: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, UpstreamError> {
        self.0
            .service
            .resolve_private_reasoning(thread, thread, handle)
            .map_err(|_| UpstreamError::new(403, "scope", "Invalid scope"))
    }
    fn resolve(
        &self,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, UpstreamError> {
        self.0
            .service
            .resolve_tool_result(thread, thread, tool, handle)
            .map_err(|_| UpstreamError::new(403, "scope", "Invalid scope"))
    }
    fn resolve_arguments(
        &self,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, UpstreamError> {
        self.0
            .service
            .resolve_tool_arguments(thread, thread, tool, handle)
            .map_err(|_| UpstreamError::new(403, "scope", "Invalid scope"))
    }
    fn begin_response(&self, thread: &str) -> Option<Arc<dyn ToolCallProtector>> {
        (thread == self.0.thread).then(|| {
            let context = self.0.active.lock().unwrap().clone();
            Arc::new(Scope {
                service: self.0.service.clone(),
                private_reasoning: self.0.service.has_private_context(&context),
                context,
                thread: self.0.thread.clone(),
                active: self.0.active.clone(),
            }) as Arc<dyn ToolCallProtector>
        })
    }
}
fn frame(value: &Value) -> String {
    format!(
        "event: {}\r\ndata: {value}\r\n\r\n",
        value["type"].as_str().unwrap()
    )
}
fn item(tool: &str, args: &Value) -> Value {
    json!({"type":"function_call","id":"fc_web","namespace":"mcp__aura","name":tool,"call_id":"call_web","arguments":args.to_string()})
}
fn frames(item: &Value) -> String {
    frame(&json!({"type":"response.output_item.added","output_index":0,"item":item}))
        + &frame(&json!({"type":"response.output_item.done","output_index":0,"item":item}))
        + &frame(&json!({"type":"response.completed","response":{"output":[item]}}))
}
async fn fixture(stream: Vec<Bytes>) -> (GatewayHandle, Arc<Provider>, Arc<Scope>) {
    let service = Arc::new(WebService::live());
    let context = service.begin_turn("thread-A", "turn-A");
    let active = Arc::new(Mutex::new(context.clone()));
    let scope = Arc::new(Scope {
        service,
        context,
        thread: "thread-A".into(),
        active,
        private_reasoning: false,
    });
    let provider = Arc::new(Provider {
        stream,
        requests: Mutex::new(Vec::new()),
        gate: Mutex::new(None),
        failure: Mutex::new(None),
    });
    let routes = Routes::new();
    routes.set("fixture", provider.clone());
    routes.set_tool_results(Arc::new(Source(scope.clone())));
    (start(routes, None).await.unwrap(), provider, scope)
}

fn reasoning() -> Value {
    json!({"type":"reasoning","id":"rs_fixture","summary":[{"type":"summary_text","text":"PRIVATE_REASONING_PAGE https://independent.example/report"}],"encrypted_content":"PRIVATE_OPAQUE_BLOB","content":[{"type":"reasoning_text","text":"PRIVATE_REASONING_DETAIL"}]})
}

#[tokio::test]
async fn private_reasoning_and_opaque_fields_are_original_at_the_provider_only() {
    let original = reasoning();
    let stream = frame(
        &json!({"type":"response.output_item.added","output_index":0,"item":original}),
    ) + &frame(
        &json!({"type":"response.reasoning_summary_part.added","item_id":"rs_fixture","output_index":0,"part":{"type":"summary_text","text":URL}}),
    ) + &frame(
        &json!({"type":"response.reasoning_summary_text.delta","item_id":"rs_fixture","output_index":0,"delta":"PRIVATE_REASONING_PAGE"}),
    ) + &frame(
        &json!({"type":"response.output_item.done","output_index":0,"item":original}),
    ) + &frame(&json!({"type":"response.completed","response":{"output":[original]}}));
    let (gateway, provider, scope) =
        fixture(stream.bytes().map(|byte| Bytes::from(vec![byte])).collect()).await;
    scope
        .service
        .retain_tool_result(&scope.context, "thread-A", "web_fetch", "{}".into())
        .unwrap();
    let response = post(&gateway, "thread-A", &json!({"input":[]}))
        .await
        .text()
        .await
        .unwrap();
    for literal in [
        URL,
        "PRIVATE_REASONING_PAGE",
        "PRIVATE_OPAQUE_BLOB",
        "PRIVATE_REASONING_DETAIL",
    ] {
        assert!(!response.contains(literal));
    }
    let observed = events(&response);
    assert_eq!(
        observed.len(),
        3,
        "No private reasoning deltas reach the sidecar/UI"
    );
    assert_eq!(observed[1]["item"]["summary"], json!([]));
    assert_eq!(observed[1]["item"], observed[2]["response"]["output"][0]);
    post(
        &gateway,
        "thread-A",
        &json!({"input":[observed[1]["item"]]}),
    )
    .await
    .bytes()
    .await
    .unwrap();
    assert_eq!(provider.requests.lock().unwrap()[1]["input"][0], original);
}

#[tokio::test]
async fn ordinary_reasoning_without_private_web_context_keeps_its_original_fields() {
    let original = json!({"type":"reasoning","id":"rs_public","summary":[{"type":"summary_text","text":"PUBLIC_REASONING"}],"encrypted_content":"PUBLIC_OPAQUE"});
    let stream =
        frame(&json!({"type":"response.output_item.done","output_index":0,"item":original}));
    let (gateway, _, _) = fixture(vec![Bytes::from(stream)]).await;
    let response = post(&gateway, "thread-A", &json!({"input":[]}))
        .await
        .text()
        .await
        .unwrap();
    assert_eq!(events(&response)[0]["item"], original);
}

#[tokio::test]
async fn late_private_reasoning_cannot_use_the_authority_of_a_new_turn() {
    let original = reasoning();
    let (gateway, provider, scope) = fixture(vec![Bytes::from(frames(&original))]).await;
    scope
        .service
        .retain_tool_result(&scope.context, "thread-A", "web_fetch", "{}".into())
        .unwrap();
    let gate = Arc::new(Gate {
        reached: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    *provider.gate.lock().unwrap() = Some(gate.clone());
    let endpoint = format!("{}/p/fixture/v1/responses", gateway.base_url());
    let token = gateway.token.expose().clone();
    let pending = tokio::spawn(async move {
        reqwest::Client::new()
            .post(endpoint)
            .bearer_auth(token)
            .header("thread-id", "thread-A")
            .json(&json!({"input":[]}))
            .send()
            .await
            .unwrap()
            .bytes()
            .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), gate.reached.notified())
        .await
        .unwrap();
    *scope.active.lock().unwrap() = scope.service.begin_turn("thread-A", "turn-B");
    gate.release.notify_one();
    assert!(
        pending.await.unwrap().is_err(),
        "Late private reasoning fails without a raw fallback or new-turn authority"
    );
    assert!(
        !scope
            .service
            .has_private_context(&scope.active.lock().unwrap())
    );
}

#[tokio::test]
async fn reasoning_admission_respects_the_shared_limit_without_a_raw_fallback() {
    let original = reasoning();
    let (gateway, _, scope) = fixture(vec![Bytes::from(frames(&original))]).await;
    for _ in 0..32 {
        scope
            .service
            .retain_tool_result(&scope.context, "thread-A", "web_fetch", "{}".into())
            .unwrap();
    }
    assert!(
        post(&gateway, "thread-A", &json!({"input":[]}))
            .await
            .bytes()
            .await
            .is_err(),
        "Full shared storage rejects reasoning instead of exposing summary or ciphertext"
    );
}

#[tokio::test]
#[ignore = "requires a new isolated AURA_GATEWAY_DUMP_DIR; run in a separate process"]
async fn private_web_diagnostics_are_suppressed_with_a_public_positive_control() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("AURA_GATEWAY_DUMP_DIR").expect("Isolated diagnostic directory required"),
    );
    assert!(
        directory.is_dir() && std::fs::read_dir(&directory).unwrap().next().is_none(),
        "Diagnostic fixture must be a new empty directory"
    );
    let (gateway, provider, scope) = fixture(vec![]).await;
    assert_eq!(
        post(
            &gateway,
            "public-thread",
            &json!({"input":"PUBLIC_DIAGNOSTIC_CONTROL"})
        )
        .await
        .status(),
        200
    );
    let positive: Vec<_> = std::fs::read_dir(&directory)
        .unwrap()
        .map(|file| file.unwrap().path())
        .collect();
    assert_eq!(
        positive.len(),
        1,
        "Diagnostics are really enabled for the ordinary request"
    );
    assert!(
        std::fs::read_to_string(&positive[0])
            .unwrap()
            .contains("PUBLIC_DIAGNOSTIC_CONTROL")
    );
    let args = scope
        .service
        .retain_tool_arguments(
            &scope.context,
            "thread-A",
            "web_fetch",
            json!({"url":URL}).to_string(),
        )
        .unwrap();
    let result = scope
        .service
        .retain_tool_result(
            &scope.context,
            "thread-A",
            "web_fetch",
            "PRIVATE_PAGE_039".into(),
        )
        .unwrap();
    let original = reasoning();
    let reasoning = scope
        .service
        .retain_private_reasoning(&scope.context, "thread-A", original.to_string())
        .unwrap();
    let mut call = item("web_fetch", &json!({}));
    call["arguments"] = json!(argument_envelope("web_fetch", &args));
    let request = json!({"input":[call,{"type":"function_call_output","call_id":"call_web","output":aura_gateway::tool_results::envelope(&result)},aura_gateway::tool_results::reasoning_envelope(&original,&reasoning)]});
    assert_eq!(post(&gateway, "thread-A", &request).await.status(), 200);
    assert_eq!(provider.requests.lock().unwrap()[1]["input"][2], original);
    *provider.failure.lock().unwrap() = Some(UpstreamError::new(
        502,
        "PRIVATE_PROVIDER_CODE",
        format!("{URL} PRIVATE_PAGE_039 PRIVATE_OPAQUE_BLOB {args} {result} {reasoning}"),
    ));
    let error = post(&gateway, "thread-A", &request).await;
    assert_eq!(error.status(), 502);
    let error = error.text().await.unwrap();
    assert!(
        error.contains("web_provider_error"),
        "Untrusted provider code is replaced before diagnostics or response"
    );
    for literal in [
        URL,
        "PRIVATE_PROVIDER_CODE",
        "PRIVATE_PAGE_039",
        "PRIVATE_OPAQUE_BLOB",
        args.as_str(),
        result.as_str(),
        reasoning.as_str(),
    ] {
        assert!(
            !error.contains(literal),
            "Private provider error does not expose payload or nonce"
        );
    }
    assert_eq!(post(&gateway, "thread-B", &request).await.status(), 403);
    assert_eq!(
        provider.requests.lock().unwrap().len(),
        3,
        "Cross-thread error never reaches the provider"
    );
    assert_eq!(
        std::fs::read_dir(&directory).unwrap().count(),
        1,
        "Private payloads and errors never produce dumps, before or after expansion"
    );
    let public = std::fs::read_to_string(&positive[0]).unwrap();
    for literal in [
        URL,
        "PRIVATE_PAGE_039",
        "PRIVATE_OPAQUE_BLOB",
        args.as_str(),
        result.as_str(),
        reasoning.as_str(),
    ] {
        assert!(
            !public.contains(literal),
            "Public diagnostic contains none of the private fixture values"
        );
    }
}

#[tokio::test]
async fn expired_reasoning_references_keep_messages_and_never_send_bogus_ciphertext() {
    let (gateway, provider, scope) = fixture(vec![]).await;
    let original = reasoning();
    let handle = scope
        .service
        .retain_private_reasoning(&scope.context, "thread-A", original.to_string())
        .unwrap();
    let safe = aura_gateway::tool_results::reasoning_envelope(&original, &handle);
    scope.service.begin_turn("thread-A", "turn-B");
    let message = json!({"role":"user","content":"Continue from the public answer"});
    post(&gateway, "thread-A", &json!({"input":[safe,message]}))
        .await
        .bytes()
        .await
        .unwrap();
    assert_eq!(
        provider.requests.lock().unwrap()[0]["input"],
        json!([message])
    );
}

#[tokio::test]
async fn reasoning_references_require_their_issuing_thread_kind_and_item_identity() {
    let (gateway, provider, scope) = fixture(vec![]).await;
    scope.service.begin_turn("thread-B", "turn-B");
    let original = reasoning();
    let handle = scope
        .service
        .retain_private_reasoning(&scope.context, "thread-A", original.to_string())
        .unwrap();
    let safe = aura_gateway::tool_results::reasoning_envelope(&original, &handle);
    assert_eq!(
        post(&gateway, "thread-B", &json!({"input":[safe]}))
            .await
            .status(),
        403
    );
    let mut changed = safe.clone();
    changed["id"] = json!("another-item");
    assert_eq!(
        post(&gateway, "thread-A", &json!({"input":[changed]}))
            .await
            .status(),
        403
    );
    let result = scope
        .service
        .retain_tool_result(&scope.context, "thread-A", "web_fetch", "{}".into())
        .unwrap();
    let wrong_kind = aura_gateway::tool_results::reasoning_envelope(&original, &result);
    assert_eq!(
        post(&gateway, "thread-A", &json!({"input":[wrong_kind]}))
            .await
            .status(),
        403
    );
    assert!(provider.requests.lock().unwrap().is_empty());
}
async fn post(gateway: &GatewayHandle, thread: &str, body: &Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{}/p/fixture/v1/responses", gateway.base_url()))
        .bearer_auth(gateway.token.expose())
        .header("thread-id", thread)
        .json(body)
        .send()
        .await
        .unwrap()
}
fn events(stream: &str) -> Vec<Value> {
    stream
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .map(|data| serde_json::from_str(data).unwrap())
        .collect()
}

#[tokio::test]
async fn web_arguments_are_private_on_the_stream_and_original_at_the_provider() {
    let args = json!({"url":URL,"maxChars":1234});
    let original = item("web_fetch", &args);
    let (gateway, provider, scope) = fixture(vec![Bytes::from(frames(&original))]).await;
    let response = post(&gateway, "thread-A", &json!({"input":[]}))
        .await
        .text()
        .await
        .unwrap();
    assert!(!response.contains(URL));
    let events = events(&response);
    assert_eq!(events[0]["item"]["arguments"], "");
    let safe = &events[1]["item"];
    let safe_args: Value = serde_json::from_str(safe["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(safe_args.as_object().unwrap().len(), 1);
    let handle = argument_handle("web_fetch", &safe_args).expect("Executable reference");
    assert_eq!(
        scope
            .service
            .resolve_tool_arguments("thread-A", "thread-A", "web_fetch", handle)
            .unwrap()
            .unwrap()
            .as_str(),
        args.to_string()
    );
    assert_eq!(
        events[2]["response"]["output"][0]["arguments"],
        safe["arguments"]
    );
    post(&gateway, "thread-A", &json!({"input":[safe]}))
        .await
        .bytes()
        .await
        .unwrap();
    assert_eq!(
        provider.requests.lock().unwrap()[1]["input"][0]["arguments"],
        args.to_string()
    );
}

#[tokio::test]
async fn duplicate_call_ids_are_rejected_before_a_second_private_call_is_issued() {
    let first = item("web_fetch", &json!({"url":URL}));
    let mut second = first.clone();
    second["id"] = json!("fc_other");
    let stream = frame(&json!({"type":"response.output_item.added","output_index":0,"item":first}))
        + &frame(&json!({"type":"response.output_item.added","output_index":1,"item":second}));
    let (gateway, _, _) = fixture(vec![Bytes::from(stream)]).await;
    assert!(
        post(&gateway, "thread-A", &json!({"input":[]}))
            .await
            .bytes()
            .await
            .is_err(),
        "An ambiguous call_id cannot acquire two executable references"
    );
}

#[tokio::test]
async fn byte_fragmented_unicode_search_and_public_output_keep_their_original_text() {
    let args = json!({"objective":"Comparar produção em São Paulo","queries":["produção relatório"],"maxResults":3});
    let original = item("web_search", &args);
    let message = json!({"type":"message","id":"msg","role":"assistant","content":[{"type":"output_text","text":"Resposta pública: ação 🦋"}]});
    let stream = frames(&original)
        + &frame(&json!({"type":"response.output_item.done","output_index":1,"item":message}));
    let (gateway, provider, _) =
        fixture(stream.bytes().map(|byte| Bytes::from(vec![byte])).collect()).await;
    let body = post(&gateway, "thread-A", &json!({"input":[]}))
        .await
        .text()
        .await
        .unwrap();
    assert!(!body.contains("Comparar produção"));
    assert!(!body.contains("produção relatório"));
    assert!(body.contains("Resposta pública: ação 🦋"));
    let events = events(&body);
    let safe = &events[1]["item"];
    let safe_args: Value = serde_json::from_str(safe["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(safe_args.as_object().unwrap().len(), 2);
    assert_eq!(safe_args["queries"].as_array().unwrap().len(), 1);
    assert_eq!(safe_args["objective"], safe_args["queries"][0]);
    assert!(argument_handle("web_search", &safe_args).is_some());
    post(&gateway, "thread-A", &json!({"input":[safe]}))
        .await
        .bytes()
        .await
        .unwrap();
    assert_eq!(
        provider.requests.lock().unwrap()[1]["input"][0]["arguments"],
        args.to_string()
    );
}

#[tokio::test]
async fn interleaved_web_and_other_tools_do_not_share_argument_transformation() {
    let web = item("web_fetch", &json!({"url":URL}));
    let mut public = item("web_fetch", &json!({"url":"PUBLIC_TOOL_ARGUMENT"}));
    public["id"] = json!("other");
    public["call_id"] = json!("call_other");
    public["namespace"] = json!("mcp__other");
    let stream = frame(&json!({"type":"response.output_item.added","output_index":0,"item":web}))
        + &frame(&json!({"type":"response.output_item.added","output_index":1,"item":public}))
        + &frame(
            &json!({"type":"response.function_call_arguments.delta","item_id":"fc_web","output_index":0,"delta":URL}),
        )
        + &frame(
            &json!({"type":"response.function_call_arguments.delta","item_id":"other","output_index":1,"delta":"PUBLIC_TOOL_ARGUMENT"}),
        )
        + &frame(
            &json!({"type":"response.function_call_arguments.done","item_id":"fc_web","output_index":0,"arguments":web["arguments"]}),
        )
        + &frame(&json!({"type":"response.output_item.done","output_index":0,"item":web}));
    let (gateway, _, _) = fixture(vec![Bytes::from(stream)]).await;
    let body = post(&gateway, "thread-A", &json!({"input":[]}))
        .await
        .text()
        .await
        .unwrap();
    assert!(!body.contains(URL));
    let events = events(&body);
    assert_eq!(events[1]["item"], public);
    assert_eq!(events[2]["delta"], "");
    assert_eq!(events[3]["delta"], "PUBLIC_TOOL_ARGUMENT");
    assert_eq!(events[4]["arguments"], events[5]["item"]["arguments"]);
}

#[tokio::test]
async fn references_are_not_authorized_by_user_text_or_another_namespace_or_thread() {
    let (gateway, provider, scope) = fixture(vec![]).await;
    scope.service.begin_turn("thread-B", "turn-B");
    let handle = scope
        .service
        .retain_tool_arguments(
            &scope.context,
            "thread-A",
            "web_fetch",
            json!({"url":URL}).to_string(),
        )
        .unwrap();
    let reference = argument_envelope("web_fetch", &handle);
    let mut other = item("web_fetch", &json!({}));
    other["namespace"] = json!("mcp__other");
    other["arguments"] = json!(reference);
    let user = json!({"type":"message","role":"user","content":reference});
    let body = json!({"input":[other,user]});
    assert_eq!(post(&gateway, "thread-A", &body).await.status(), 200);
    assert_eq!(provider.requests.lock().unwrap()[0], body);
    let mut trusted = item("web_fetch", &json!({}));
    trusted["arguments"] = json!(reference);
    assert_eq!(
        post(&gateway, "thread-B", &json!({"input":[trusted]}))
            .await
            .status(),
        403
    );
    assert_eq!(provider.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn a_changed_completed_snapshot_fails_instead_of_persisting_new_raw_arguments() {
    let original = item("web_fetch", &json!({"url":URL}));
    let mut changed = original.clone();
    changed["arguments"] = json!(json!({"url":"https://other.example/private"}).to_string());
    let stream =
        frame(&json!({"type":"response.output_item.done","output_index":0,"item":original}))
            + &frame(&json!({"type":"response.completed","response":{"output":[changed]}}));
    let (gateway, _, _) = fixture(vec![Bytes::from(stream)]).await;
    assert!(
        post(&gateway, "thread-A", &json!({"input":[]}))
            .await
            .bytes()
            .await
            .is_err()
    );
}

#[tokio::test]
async fn late_provider_response_cannot_borrow_the_authority_of_a_new_turn() {
    let original = item("web_fetch", &json!({"url":URL}));
    let (gateway, provider, scope) = fixture(vec![Bytes::from(frames(&original))]).await;
    let gate = Arc::new(Gate {
        reached: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    *provider.gate.lock().unwrap() = Some(gate.clone());
    let endpoint = format!("{}/p/fixture/v1/responses", gateway.base_url());
    let token = gateway.token.expose().clone();
    let pending = tokio::spawn(async move {
        reqwest::Client::new()
            .post(endpoint)
            .bearer_auth(token)
            .header("thread-id", "thread-A")
            .json(&json!({"input":[]}))
            .send()
            .await
            .unwrap()
            .bytes()
            .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), gate.reached.notified())
        .await
        .unwrap();
    *scope.active.lock().unwrap() = scope.service.begin_turn("thread-A", "turn-B");
    gate.release.notify_one();
    assert!(
        pending.await.unwrap().is_err(),
        "Late input belongs to the cancelled turn, even while a new turn is active"
    );
}

#[tokio::test]
async fn expired_arguments_in_history_allow_continuation_without_revealing_old_input() {
    let (gateway, provider, scope) = fixture(vec![]).await;
    let handle = scope
        .service
        .retain_tool_arguments(
            &scope.context,
            "thread-A",
            "web_fetch",
            json!({"url":URL}).to_string(),
        )
        .unwrap();
    let mut call = item("web_fetch", &json!({}));
    call["arguments"] = json!(argument_envelope("web_fetch", &handle));
    *scope.active.lock().unwrap() = scope.service.begin_turn("thread-A", "turn-B");
    assert_eq!(
        post(&gateway, "thread-A", &json!({"input":[call]}))
            .await
            .status(),
        200
    );
    let text = provider.requests.lock().unwrap()[0]["input"][0]["arguments"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        serde_json::from_str::<Value>(&text).unwrap(),
        json!({"url":"aura:previous-web-input-unavailable"})
    );
    assert!(!text.contains(URL));
    assert!(!text.contains(&handle));
}

#[tokio::test]
async fn exceeded_private_memory_returns_no_raw_argument_fallback() {
    let original = item("web_fetch", &json!({"url":URL}));
    let (gateway, _, scope) = fixture(vec![Bytes::from(frames(&original))]).await;
    for _ in 0..32 {
        scope
            .service
            .retain_tool_result(&scope.context, "thread-A", "web_fetch", "{}".into())
            .unwrap();
    }
    assert!(
        post(&gateway, "thread-A", &json!({"input":[]}))
            .await
            .bytes()
            .await
            .is_err()
    );
}

#[tokio::test]
async fn sequential_completed_calls_with_a_reused_call_id_restore_each_original_input() {
    let (gateway, provider, scope) = fixture(vec![]).await;
    let first = json!({"url":URL});
    let second = json!({"url":"https://other.example/report"});
    let mut input = Vec::new();
    for args in [&first, &second] {
        let handle = scope
            .service
            .retain_tool_arguments(&scope.context, "thread-A", "web_fetch", args.to_string())
            .unwrap();
        let mut call = item("web_fetch", &json!({}));
        call["arguments"] = json!(argument_envelope("web_fetch", &handle));
        input.push(call);
        input.push(json!({"type":"function_call_output","call_id":"call_web","output":"{}"}));
    }
    assert_eq!(
        post(&gateway, "thread-A", &json!({"input":input}))
            .await
            .status(),
        200
    );
    let requests = provider.requests.lock().unwrap();
    let first_text = first.to_string();
    let second_text = second.to_string();
    assert!(
        requests[0]["input"][0]["arguments"].as_str() == Some(first_text.as_str()),
        "First completed call restores original input"
    );
    assert!(
        requests[0]["input"][2]["arguments"].as_str() == Some(second_text.as_str()),
        "Second completed call restores original input"
    );
}
