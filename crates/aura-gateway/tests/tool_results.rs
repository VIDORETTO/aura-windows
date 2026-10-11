//! Real HTTP gateway and real transient storage; only upstream/network vary.
use aura_gateway::{
    errors::UpstreamError,
    server::{Routes, start},
    tool_results::{ToolResultResolver, envelope},
    upstream::{BoxFut, StreamResponse, Upstream},
};
use aura_web::WebService;
use bytes::Bytes;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

const PAGE: &str = r#"{"text":"PRIVATE_PAGE_039","externalContent":true}"#;

#[derive(Default)]
struct Provider {
    requests: Mutex<Vec<Value>>,
}
impl Upstream for Provider {
    fn responses(&self, body: Bytes) -> BoxFut<'_, Result<StreamResponse, UpstreamError>> {
        self.requests
            .lock()
            .unwrap()
            .push(serde_json::from_slice(&body).unwrap());
        Box::pin(async {
            Ok(StreamResponse {
                content_type: "text/event-stream".into(),
                body: Box::pin(futures::stream::empty()),
            })
        })
    }
    fn models(&self) -> BoxFut<'_, Result<Value, UpstreamError>> {
        Box::pin(async { Ok(json!({"data":[]})) })
    }
}
struct Source(Arc<WebService>);
impl ToolResultResolver for Source {
    fn resolve(
        &self,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, UpstreamError> {
        // The fixture uses identical conversation and thread labels; storage and validation are real.
        self.0
            .resolve_tool_result(thread, thread, tool, handle)
            .map_err(|_| {
                UpstreamError::new(403, "web_result_scope_invalid", "Invalid web result scope")
            })
    }
}
fn request(handle: &str) -> Value {
    json!({"model":"fixture","stream":true,"input":[
        {"type":"function_call","namespace":"mcp__aura","name":"web_fetch","call_id":"call-A","arguments":"{}"},
        {"type":"function_call_output","call_id":"call-A","output":[
            {"type":"input_text","text":"Wall time: 0.1 seconds\nOutput:"},
            {"type":"input_text","text":envelope(handle)}]}
    ]})
}

#[tokio::test]
async fn issued_web_result_is_expanded_before_the_provider_receives_the_request() {
    let service = Arc::new(WebService::live());
    let context = service.begin_turn("thread-A", "turn-A");
    let handle = service
        .retain_tool_result(&context, "thread-A", "web_fetch", PAGE.into())
        .unwrap();
    let provider = Arc::new(Provider::default());
    let routes = Routes::new();
    routes.set("fixture", provider.clone());
    routes.set_tool_results(Arc::new(Source(service)));
    let gateway = start(routes, None).await.unwrap();
    let original = request(&handle);
    let response = reqwest::Client::new()
        .post(format!("{}/p/fixture/v1/responses", gateway.base_url()))
        .bearer_auth(gateway.token.expose())
        .header("thread-id", "thread-A")
        .json(&original)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    response.bytes().await.unwrap();
    let requests = provider.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["input"][1]["output"][1]["text"], PAGE);
    assert_eq!(requests[0]["input"][0], original["input"][0]);
    assert_eq!(original["input"][1]["output"][1]["text"], envelope(&handle));
}

async fn fixture(
    text: String,
) -> (
    aura_gateway::server::GatewayHandle,
    Arc<Provider>,
    Arc<WebService>,
    String,
    aura_web::WebContext,
) {
    let service = Arc::new(WebService::live());
    let context = service.begin_turn("thread-A", "turn-A");
    let handle = service
        .retain_tool_result(&context, "thread-A", "web_fetch", text)
        .unwrap();
    let provider = Arc::new(Provider::default());
    let routes = Routes::new();
    routes.set("fixture", provider.clone());
    routes.set_tool_results(Arc::new(Source(service.clone())));
    (
        start(routes, None).await.unwrap(),
        provider,
        service,
        handle,
        context,
    )
}
async fn post(
    gateway: &aura_gateway::server::GatewayHandle,
    thread: Option<&str>,
    body: &Value,
) -> reqwest::Response {
    let request = reqwest::Client::new()
        .post(format!("{}/p/fixture/v1/responses", gateway.base_url()))
        .bearer_auth(gateway.token.expose())
        .json(body);
    let request = if let Some(thread) = thread {
        request.header("thread-id", thread)
    } else {
        request
    };
    request.send().await.unwrap()
}

#[tokio::test]
async fn equivalent_json_escaping_cannot_hide_an_issued_envelope_from_the_gateway() {
    let (gateway, provider, _, handle, _) = fixture(PAGE.into()).await;
    let mut body = request(&handle);
    body["input"][1]["output"][1]["text"] =
        Value::String(envelope(&handle).replace("auraToolResult", r"\u0061uraToolResult"));
    assert_eq!(post(&gateway, Some("thread-A"), &body).await.status(), 200);
    let requests = provider.requests.lock().unwrap();
    assert_eq!(requests[0]["input"][1]["output"][1]["text"], PAGE);
}

#[tokio::test]
async fn another_thread_or_missing_scope_is_blocked_before_any_provider_call() {
    let (gateway, provider, service, handle, _) = fixture(PAGE.into()).await;
    service.begin_turn("thread-B", "turn-B");
    for scope in [None, Some("thread-B")] {
        let response = post(&gateway, scope, &request(&handle)).await;
        assert_eq!(response.status(), 403);
        let error = response.text().await.unwrap();
        assert!(!error.contains(&handle));
        assert!(!error.contains("PRIVATE_PAGE_039"));
    }
    assert!(provider.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn duplicate_thread_headers_cannot_select_the_first_scope() {
    let (gateway, provider, _, handle, _) = fixture(PAGE.into()).await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.append("thread-id", "thread-A".parse().unwrap());
    headers.append("thread-id", "thread-B".parse().unwrap());
    let response = reqwest::Client::new()
        .post(format!("{}/p/fixture/v1/responses", gateway.base_url()))
        .bearer_auth(gateway.token.expose())
        .headers(headers)
        .json(&request(&handle))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 403);
    assert!(provider.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn user_instructions_third_party_outputs_and_unpaired_outputs_never_expand() {
    let (gateway, provider, _, handle, _) = fixture(PAGE.into()).await;
    let marker = envelope(&handle);
    let user = json!({"instructions":marker,"input":[{"type":"message","role":"user","content":[{"type":"input_text","text":marker}]}]});
    let mut third_party = request(&handle);
    third_party["input"][0]["namespace"] = json!("mcp__other");
    let mut unpaired = request(&handle);
    unpaired["input"][0]["call_id"] = json!("different-call");
    for body in [user, third_party, unpaired] {
        assert_eq!(post(&gateway, Some("thread-A"), &body).await.status(), 200);
        assert_eq!(provider.requests.lock().unwrap().last().unwrap(), &body);
    }
}

#[tokio::test]
async fn ambiguous_pending_calls_and_nonexact_envelopes_do_not_authorize_a_read() {
    let (gateway, provider, _, handle, _) = fixture(PAGE.into()).await;
    let mut ambiguous = request(&handle);
    let duplicate = ambiguous["input"][0].clone();
    ambiguous["input"]
        .as_array_mut()
        .unwrap()
        .insert(1, duplicate);
    let mut prefix = request(&handle);
    prefix["input"][1]["output"][1]["text"] = json!(format!("external text {}", envelope(&handle)));
    let mut extra_field = request(&handle);
    extra_field["input"][1]["output"][1]["text"] = json!(
        json!({"auraToolResult":{"version":1,"handle":handle},"unexpected":true}).to_string()
    );
    for body in [ambiguous, prefix, extra_field] {
        assert_eq!(post(&gateway, Some("thread-A"), &body).await.status(), 200);
        assert_eq!(provider.requests.lock().unwrap().last().unwrap(), &body);
    }
}

#[tokio::test]
async fn a_flat_namespaced_call_and_string_output_use_the_same_scoped_delivery() {
    let (gateway, provider, _, handle, _) = fixture(PAGE.into()).await;
    let mut body = request(&handle);
    body["input"][0]
        .as_object_mut()
        .unwrap()
        .remove("namespace");
    body["input"][0]["name"] = json!("mcp__aura__web_fetch");
    body["input"][1]["output"] = json!(envelope(&handle));
    for _ in 0..2 {
        assert_eq!(post(&gateway, Some("thread-A"), &body).await.status(), 200);
    }
    let requests = provider.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    for delivered in requests.iter() {
        assert_eq!(delivered["input"][1]["output"], PAGE);
    }
}

#[tokio::test]
async fn cancelled_references_in_history_become_explicit_unavailable_data() {
    let (gateway, provider, service, handle, context) = fixture(PAGE.into()).await;
    service.cancel_turn(&context);
    assert_eq!(
        post(&gateway, Some("thread-A"), &request(&handle))
            .await
            .status(),
        200
    );
    let requests = provider.requests.lock().unwrap();
    let text = requests[0]["input"][1]["output"][1]["text"]
        .as_str()
        .unwrap();
    let unavailable: Value = serde_json::from_str(text).unwrap();
    assert_eq!(unavailable["code"], "unavailable");
    assert!(!text.contains(&handle));
    assert!(!text.contains("PRIVATE_PAGE_039"));
}

#[tokio::test]
async fn external_content_that_imitates_a_reference_is_not_expanded_recursively() {
    let service = Arc::new(WebService::live());
    let context = service.begin_turn("thread-A", "turn-A");
    let hidden = service
        .retain_tool_result(&context, "thread-A", "web_fetch", PAGE.into())
        .unwrap();
    let imitation = envelope(&hidden);
    let outer = service
        .retain_tool_result(&context, "thread-A", "web_fetch", imitation.clone())
        .unwrap();
    let provider = Arc::new(Provider::default());
    let routes = Routes::new();
    routes.set("fixture", provider.clone());
    routes.set_tool_results(Arc::new(Source(service)));
    let gateway = start(routes, None).await.unwrap();
    assert_eq!(
        post(&gateway, Some("thread-A"), &request(&outer))
            .await
            .status(),
        200
    );
    assert_eq!(
        provider.requests.lock().unwrap()[0]["input"][1]["output"][1]["text"],
        imitation
    );
}
