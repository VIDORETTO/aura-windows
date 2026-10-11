use aura_web::{
    Clock, SearchRequest, WebError, WebService,
    transport::{HttpRequest, HttpResponse, Transport},
};
use futures::StreamExt;
use futures::future::BoxFuture;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH
    }
}

struct AdjustableClock(Mutex<SystemTime>);
impl Clock for AdjustableClock {
    fn now(&self) -> SystemTime {
        *self.0.lock().unwrap()
    }
}

#[tokio::test]
async fn registry_metadata_and_the_latest_search_share_the_eight_mebibyte_budget() {
    let network = Arc::new(Network::default());
    for index in 1..=4 {
        network.parallel(json!([{"title":"x".repeat(2_000_000),"url":format!("https://news.example/report-{index}"),"excerpts":["A public fact."]}]));
    }
    network.parallel(json!([{"title":"Small report","url":"https://news.example/small","excerpts":["Small public fact."]}]));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let mut query = request();
    query.refresh = true;
    for index in 1..=3 {
        let context = service.begin_turn("conversation", &format!("turn-{index}"));
        assert_eq!(
            service
                .search(&context, query.clone())
                .await
                .unwrap()
                .results[0]
                .source_id,
            format!("W{index}")
        );
    }
    let fourth = service.begin_turn("conversation", "turn-4");
    assert!(
        matches!(
            service.search(&fourth, query.clone()).await,
            Err(WebError::LimitExceeded)
        ),
        "four retained titles plus the newest result exceed 8 MiB"
    );
    assert!(
        service.source(&fourth, "W4").is_none(),
        "failed admission must not create verified provenance"
    );
    let next = service.begin_turn("conversation", "turn-5");
    assert_eq!(
        service.search(&next, query).await.unwrap().results[0].source_id,
        "W4",
        "failed admission must not consume a source identifier"
    );
}

#[tokio::test]
async fn historical_provenance_cannot_exceed_the_registry_memory_budget() {
    let service = WebService::new(Arc::new(Network::default()), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let sources = (1..=5)
        .map(|number| aura_web::WebSource {
            source_id: format!("W{number}"),
            title: "x".repeat(2_000_000),
            url: format!("https://news.example/report-{number}"),
            snippet: String::new(),
            published_at: None,
            retrieved_at: "1970-01-01T00:00:00Z".into(),
            kind: "pageContent".into(),
        })
        .collect();
    let _ = service.restore_sources(&context, 5, sources);
    assert!(
        service.source(&context, "W5").is_none(),
        "ten million title bytes cannot be retained in an 8 MiB registry"
    );
}

#[tokio::test]
async fn registry_metadata_is_also_bounded_across_conversations_at_thirty_two_mebibytes() {
    let network = Arc::new(Network::default());
    for index in 1..=16 {
        network.parallel(json!([{"title":"x".repeat(2_000_000),"url":format!("https://news.example/report-{index}"),"excerpts":["Public fact."]}]));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let mut query = request();
    query.refresh = true;
    for conversation in 1..=5 {
        let context = service.begin_turn(&format!("conversation-{conversation}"), "turn");
        for _ in 0..3 {
            service.search(&context, query.clone()).await.unwrap();
        }
    }
    let context = service.begin_turn("conversation-6", "turn");
    assert!(
        matches!(
            service.search(&context, query).await,
            Err(WebError::LimitExceeded)
        ),
        "thirty-two million retained title bytes plus the newest result exceed 32 MiB"
    );
    assert!(service.source(&context, "W1").is_none());
    let long_turn = "t".repeat(2_000_000);
    service.begin_turn("cold-1", &long_turn);
    let rejected = service.begin_turn("cold-2", &long_turn);
    let before = network.requests.lock().unwrap().len();
    assert!(
        matches!(
            service.search(&rejected, request()).await,
            Err(WebError::LimitExceeded)
        ),
        "even empty conversation metadata must fit the aggregate budget before network"
    );
    assert_eq!(network.requests.lock().unwrap().len(), before);
    service.cancel_conversation("cold-1");
    let restored = service.begin_turn("cold-2", &long_turn);
    assert!(
        service.restore_sources(&restored, 0, vec![]).is_ok(),
        "closing releases metadata capacity"
    );
}

#[tokio::test]
async fn closing_a_conversation_releases_slots_without_letting_old_completions_change_the_new_lifetime()
 {
    let network = Arc::new(Network::default());
    for _ in 0..3 {
        network
            .responses
            .lock()
            .unwrap()
            .push_back(Ok(HttpResponse {
                status: 200,
                headers: BTreeMap::from([("content-type".into(), "application/json".into())]),
                body: aura_web::transport::HttpBody::new(futures::stream::pending()),
            }));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let old_context = service.begin_turn("conversation", "old-turn");
    let mut old = Box::pin(service.search(&old_context, request()));
    assert!(futures::poll!(old.as_mut()).is_pending());
    service.cancel_conversation("conversation");
    let context = service.begin_turn("conversation", "new-turn");
    let mut first = Box::pin(service.search(&context, request()));
    let mut second = Box::pin(service.search(&context, request()));
    assert!(futures::poll!(first.as_mut()).is_pending());
    assert!(
        futures::poll!(second.as_mut()).is_pending(),
        "closed lifetime must not occupy a slot in the reopened conversation"
    );
    assert!(matches!(old.await, Err(WebError::Cancelled)));
    assert!(
        matches!(
            service.search(&context, request()).await,
            Err(WebError::LimitExceeded)
        ),
        "an old completion cannot release a slot occupied by the new lifetime"
    );
    assert_eq!(network.requests.lock().unwrap().len(), 3);
}

#[tokio::test]
async fn repeats_search_from_cache_for_fifteen_minutes_but_refresh_and_expiry_use_network() {
    let network = Arc::new(Network::default());
    for title in ["Manual público", "Manual atualizado", "Manual recente"] {
        network.parallel(json!([{"title":title,"url":"https://news.example/manual","excerpts":["Manual de uso."]}]));
    }
    let clock = Arc::new(AdjustableClock(Mutex::new(UNIX_EPOCH)));
    let service = WebService::new(network.clone(), clock.clone());
    let context = service.begin_turn("conversation", "turn-1");
    let original = service.search(&context, request()).await.unwrap();
    assert!(!original.cached);
    *clock.0.lock().unwrap() = UNIX_EPOCH + std::time::Duration::from_secs(14 * 60);
    let cached = service.search(&context, request()).await.unwrap();
    assert!(cached.cached);
    assert_eq!(cached.results, original.results);
    assert_eq!(cached.searched_at, "1970-01-01T00:00:00Z");
    assert_eq!(network.requests.lock().unwrap().len(), 4);
    let mut refresh = request();
    refresh.refresh = true;
    let updated = service.search(&context, refresh).await.unwrap();
    assert!(!updated.cached);
    assert_eq!(updated.results[0].title, "Manual atualizado");
    assert_eq!(updated.results[0].source_id, "W1");
    *clock.0.lock().unwrap() = UNIX_EPOCH + std::time::Duration::from_secs(30 * 60);
    let next = service.begin_turn("conversation", "turn-2");
    let expired = service.search(&next, request()).await.unwrap();
    assert!(!expired.cached);
    assert_eq!(expired.results[0].title, "Manual recente");
    assert_eq!(network.requests.lock().unwrap().len(), 12);
}

#[tokio::test]
async fn search_cache_is_isolated_by_conversation_and_counts_cached_calls_toward_the_budget() {
    let network = Arc::new(Network::default());
    network.parallel(json!([{"title":"First report","url":"https://news.example/manual","excerpts":["Production: 42"]}]));
    network.parallel(json!([{"title":"Second report","url":"https://news.example/manual","excerpts":["Production: 13"]}]));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let first = service.begin_turn("conversation-A", "turn-1");
    assert_eq!(
        service.search(&first, request()).await.unwrap().results[0].snippet,
        "Production: 42"
    );
    for _ in 0..2 {
        let cached = service.search(&first, request()).await.unwrap();
        assert!(cached.cached);
        assert_eq!(cached.results[0].source_id, "W1");
    }
    assert!(matches!(
        service.search(&first, request()).await,
        Err(WebError::LimitExceeded)
    ));
    let second = service.begin_turn("conversation-B", "turn-1");
    let fresh = service.search(&second, request()).await.unwrap();
    assert!(!fresh.cached);
    assert_eq!(fresh.results[0].snippet, "Production: 13");
    assert_eq!(fresh.results[0].source_id, "W1");
    assert_eq!(network.requests.lock().unwrap().len(), 8);
    service.set_enabled(false);
    assert!(matches!(
        service.search(&second, request()).await,
        Err(WebError::WebDisabled)
    ));
    service.set_enabled(true);
    assert!(matches!(
        service.search(&second, request()).await,
        Err(WebError::Cancelled)
    ));
    let resumed = service.begin_turn("conversation-B", "turn-1");
    assert!(service.search(&resumed, request()).await.unwrap().cached);
    assert_eq!(network.requests.lock().unwrap().len(), 8);
}

#[tokio::test]
async fn search_and_document_cache_share_the_thirty_two_entry_lru() {
    let network = Arc::new(Network::default());
    network.parallel(json!([{"title":"Manual público","url":"https://news.example/manual","excerpts":["Manual de uso."]}]));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let first = service.begin_turn("conversation", "first-turn");
    service.search(&first, request()).await.unwrap();
    for index in 0..31 {
        network
            .responses
            .lock()
            .unwrap()
            .push_back(Ok(HttpResponse {
                status: 200,
                headers: BTreeMap::from([("content-type".into(), "text/plain".into())]),
                body: b"Production: 42".to_vec().into(),
            }));
        let context = service.begin_turn("conversation", &format!("turn-{index}"));
        let input =
            serde_json::from_value(json!({"url":format!("https://news.example/doc-{index}")}))
                .unwrap();
        service.fetch(&context, input).await.unwrap();
    }
    network.parallel(
        json!([{"title":"Other manual","url":"https://news.example/other","excerpts":[]}]),
    );
    let next = service.begin_turn("conversation", "next-turn");
    let mut other = request();
    other.queries = vec!["other manual".into()];
    service.search(&next, other).await.unwrap();
    // One search + 31 documents filled the shared budget. Adding another
    // search evicts the oldest search even though its own cache has only two.
    network.parallel(
        json!([{"title":"Updated manual","url":"https://news.example/manual","excerpts":[]}]),
    );
    let reloaded = service.search(&next, request()).await.unwrap();
    assert!(!reloaded.cached);
    assert_eq!(reloaded.results[0].title, "Updated manual");
    assert_eq!(network.requests.lock().unwrap().len(), 43);
}

#[derive(Default)]
struct Network {
    responses: Mutex<VecDeque<Result<HttpResponse, WebError>>>,
    requests: Mutex<Vec<HttpRequest>>,
    resolutions: Mutex<Vec<String>>,
}
impl Network {
    fn json(&self, value: Value) {
        self.responses.lock().unwrap().push_back(Ok(HttpResponse {
            status: 200,
            headers: BTreeMap::from([("content-type".into(), "application/json".into())]),
            body: serde_json::to_vec(&value).unwrap().into(),
        }));
    }
    fn parallel(&self, results: Value) {
        self.json(json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{"tools":{}},"serverInfo":{"name":"Test search","version":"1"}}}));
        self.responses.lock().unwrap().push_back(Ok(HttpResponse {
            status: 204,
            headers: BTreeMap::new(),
            body: vec![].into(),
        }));
        self.json(serde_json::from_str(include_str!("fixtures/parallel-tools.json")).unwrap());
        self.json(json!({"jsonrpc":"2.0","id":3,"result":{"structuredContent":{"results":results},"content":[],"isError":false}}));
    }
}
impl Transport for Network {
    fn resolve<'a>(&'a self, host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        self.resolutions.lock().unwrap().push(host.into());
        Box::pin(async { Ok(vec!["93.184.216.34".parse().unwrap()]) })
    }
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        self.requests.lock().unwrap().push(request);
        let response = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected external HTTP request");
        Box::pin(async move { response })
    }
}
fn request() -> SearchRequest {
    SearchRequest {
        objective: "Encontrar manual do Aura".into(),
        queries: vec!["Aura manual".into()],
        max_results: 5,
        refresh: false,
    }
}

#[tokio::test]
async fn searches_without_a_key_and_returns_a_real_source() {
    let network = Arc::new(Network::default());
    network.parallel(json!([{"title":"Manual público","url":"https://news.example/manual","excerpts":["Manual de uso."],"publish_date":null}]));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("local-conversation-id", "turn-1");
    let output = service
        .search(&context, request())
        .await
        .expect("keyless search should succeed");
    assert_eq!(output.provider, "parallel");
    assert!(!output.degraded);
    assert!(!output.cached);
    assert_eq!(
        serde_json::to_value(output.results).unwrap(),
        json!([{
            "sourceId":"W1", "title":"Manual público", "url":"https://news.example/manual", "snippet":"Manual de uso.",
            "publishedAt":null, "retrievedAt":"1970-01-01T00:00:00Z", "kind":"searchSnippet"
        }])
    );
    let sent = network.requests.lock().unwrap();
    assert_eq!(sent.len(), 4);
    for http in sent.iter() {
        assert_eq!(http.url.as_str(), "https://search.parallel.ai/mcp");
        assert!(!http.headers.keys().any(|h| {
            ["authorization", "cookie", "referer"].contains(&h.to_ascii_lowercase().as_str())
        }));
        assert!(!String::from_utf8_lossy(&http.body).contains("local-conversation-id"));
    }
    let call: Value = serde_json::from_slice(&sent[3].body).unwrap();
    assert_eq!(call["params"]["name"], "web_search");
    let args = &call["params"]["arguments"];
    assert_eq!(args["objective"], "Encontrar manual do Aura");
    assert_eq!(args["search_queries"], json!(["Aura manual"]));
    assert_eq!(args.as_object().unwrap().len(), 3);
    assert!(uuid::Uuid::parse_str(args["session_id"].as_str().unwrap()).is_ok());
}

#[tokio::test]
async fn a_rate_limited_primary_uses_one_free_alternative() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Ok(HttpResponse {
            status: 429,
            headers: BTreeMap::new(),
            body: b"private upstream error details".to_vec().into(),
        }));
    network.responses.lock().unwrap().push_back(Ok(HttpResponse { status:200, headers:BTreeMap::from([("content-type".into(), "text/html; charset=utf-8".into())]), body:br#"<html><div class="result"><a class="result__a" href="https://news.example/manual">Manual alternativo</a><a class="result__snippet">Manual de uso.</a></div></html>"#.to_vec().into() }));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service
        .search(&context, request())
        .await
        .expect("the free alternative should work");
    assert_eq!(output.provider, "duckduckgo");
    assert!(output.degraded);
    assert_eq!(output.results[0].title, "Manual alternativo");
    assert_eq!(output.results[0].url, "https://news.example/manual");
    assert_eq!(output.warnings, ["primary_unavailable"]);
    let sent = network.requests.lock().unwrap();
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[1].url.host_str(), Some("html.duckduckgo.com"));
    assert_eq!(sent[1].url.query(), Some("q=Aura+manual"));
    assert!(!sent[1].headers.contains_key("authorization"));
}

#[tokio::test]
async fn omits_a_source_url_over_the_contract_limit() {
    let network = Arc::new(Network::default());
    network.parallel(json!([
        {"title":"Oversized", "url":format!("https://news.example/{}", "a".repeat(2048)), "excerpts":[]},
        {"title":"Manual", "url":"https://news.example/manual", "excerpts":["Manual de uso."]}
    ]));
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.search(&context, request()).await.unwrap();
    assert_eq!(output.results.len(), 1);
    assert_eq!(output.results[0].url, "https://news.example/manual");
    assert_eq!(output.results[0].source_id, "W1");
}

#[tokio::test]
async fn rejects_out_of_contract_input_before_dns_or_http() {
    let mut too_long = request();
    too_long.objective = "x".repeat(2001);
    let mut too_many = request();
    too_many.queries = vec!["q".into(); 4];
    let mut too_many_results = request();
    too_many_results.max_results = 11;
    let mut empty = request();
    empty.objective = " \n ".into();
    for invalid in [too_long, too_many, too_many_results, empty] {
        let network = Arc::new(Network::default());
        network.parallel(json!([]));
        let service = WebService::new(network.clone(), Arc::new(FixedClock));
        let context = service.begin_turn("conversation", "turn");
        assert!(matches!(
            service.search(&context, invalid).await,
            Err(WebError::InvalidInput)
        ));
        assert!(network.requests.lock().unwrap().is_empty());
        assert!(network.resolutions.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn normalizes_sources_without_losing_query_semantics_or_inventing_dates() {
    let network = Arc::new(Network::default());
    network.parallel(json!([
        {"title":"Primeiro","url":"https://NEWS.example:443/manual#top","excerpts":["Primeiro trecho"]},
        {"title":"Duplicado","url":"https://news.example/manual?utm_source=x","excerpts":["Outro trecho"]},
        {"title":"Script","url":"javascript:alert(1)","excerpts":[]},
        {"title":"Consulta 1","url":"https://news.example/outro?q=1","excerpts":[]},
        {"title":"Consulta 2","url":"https://news.example/outro?q=2","excerpts":[],"publish_date":"recent"}
    ]));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.search(&context, request()).await.unwrap();
    assert_eq!(
        output
            .results
            .iter()
            .map(|s| (&*s.source_id, &*s.url))
            .collect::<Vec<_>>(),
        [
            ("W1", "https://news.example/manual"),
            ("W2", "https://news.example/outro?q=1"),
            ("W3", "https://news.example/outro?q=2")
        ]
    );
    assert_eq!(output.results[0].title, "Primeiro");
    assert_eq!(output.results[0].snippet, "Primeiro trecho");
    assert!(output.results.iter().all(|s| s.published_at.is_none()));
}

struct GatedNetwork {
    arrived: tokio::sync::Semaphore,
    release: tokio::sync::Semaphore,
}
impl Transport for GatedNetwork {
    fn resolve<'a>(&'a self, _host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        Box::pin(async { Ok(vec!["93.184.216.34".parse().unwrap()]) })
    }
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        Box::pin(async move {
            let call: Value = serde_json::from_slice(&request.body).unwrap();
            let body = match call["method"].as_str().unwrap() {
                "initialize" => {
                    self.arrived.add_permits(1);
                    self.release.acquire().await.unwrap().forget();
                    json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26"}})
                }
                "notifications/initialized" => {
                    return Ok(HttpResponse {
                        status: 204,
                        headers: BTreeMap::new(),
                        body: vec![].into(),
                    });
                }
                "tools/list" => {
                    json!({"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"web_search","inputSchema":{"required":["objective","search_queries"],"properties":{"objective":{"type":"string"},"search_queries":{"type":"array"},"session_id":{"type":"string"}}}}]}})
                }
                "tools/call" => {
                    json!({"jsonrpc":"2.0","id":3,"result":{"structuredContent":{"results":[]},"isError":false}})
                }
                unexpected => panic!("unexpected external method {unexpected}"),
            };
            Ok(HttpResponse {
                status: 200,
                headers: BTreeMap::new(),
                body: serde_json::to_vec(&body).unwrap().into(),
            })
        })
    }
}

#[tokio::test]
async fn admits_two_concurrent_searches_and_refuses_a_third_without_network() {
    let network = Arc::new(GatedNetwork {
        arrived: tokio::sync::Semaphore::new(0),
        release: tokio::sync::Semaphore::new(0),
    });
    let service = Arc::new(WebService::new(network.clone(), Arc::new(FixedClock)));
    let context = service.begin_turn("conversation", "turn");
    let mut jobs = Vec::new();
    for _ in 0..2 {
        let (service, context) = (service.clone(), context.clone());
        jobs.push(tokio::spawn(async move {
            service.search(&context, request()).await
        }));
    }
    network.arrived.acquire_many(2).await.unwrap().forget();
    let third = tokio::time::timeout(
        std::time::Duration::from_millis(50),
        service.search(&context, request()),
    )
    .await;
    network.release.add_permits(2);
    for job in jobs {
        assert!(job.await.unwrap().is_ok());
    }
    assert!(matches!(third, Ok(Err(WebError::LimitExceeded))));
    assert_eq!(network.arrived.available_permits(), 0);
}

#[tokio::test]
async fn search_budget_is_per_turn_and_session_does_not_rotate_at_the_limit() {
    let network = Arc::new(Network::default());
    for _ in 0..5 {
        network.parallel(json!([]));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    for index in 0..3 {
        let mut input = request();
        input.queries = vec![format!("Aura manual {index}")];
        assert!(service.search(&context, input).await.is_ok());
    }
    let same_turn = service.begin_turn("conversation", "turn");
    let count = network.requests.lock().unwrap().len();
    assert!(matches!(
        service.search(&same_turn, request()).await,
        Err(WebError::LimitExceeded)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), count);
    let next_turn = service.begin_turn("conversation", "next-turn");
    assert!(service.search(&next_turn, request()).await.is_ok());
    let other = service.begin_turn("other-conversation", "turn");
    assert!(service.search(&other, request()).await.is_ok());
    let sent = network.requests.lock().unwrap();
    let sessions: Vec<String> = sent
        .iter()
        .filter_map(|http| {
            let call: Value = serde_json::from_slice(&http.body).ok()?;
            (call["method"] == "tools/call").then(|| {
                call["params"]["arguments"]["session_id"]
                    .as_str()
                    .unwrap()
                    .into()
            })
        })
        .collect();
    assert_eq!(sessions.len(), 5);
    assert!(sessions[..4].iter().all(|s| s == &sessions[0]));
    assert_ne!(sessions[4], sessions[0]);
}

struct SlowNetwork;
impl Transport for SlowNetwork {
    fn resolve<'a>(&'a self, _host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        Box::pin(async { Ok(vec!["93.184.216.34".parse().unwrap()]) })
    }
    fn request(&self, _request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        Box::pin(async {
            tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            Err(WebError::Unavailable)
        })
    }
}

#[tokio::test(start_paused = true)]
async fn primary_and_alternative_share_one_thirty_second_deadline() {
    let service = WebService::new(Arc::new(SlowNetwork), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let start = tokio::time::Instant::now();
    assert!(matches!(
        service.search(&context, request()).await,
        Err(WebError::Timeout)
    ));
    assert_eq!(start.elapsed(), std::time::Duration::from_secs(30));
}

#[tokio::test]
async fn ending_the_turn_cancels_pending_io_without_trying_the_alternative() {
    let network = Arc::new(GatedNetwork {
        arrived: tokio::sync::Semaphore::new(0),
        release: tokio::sync::Semaphore::new(0),
    });
    let service = Arc::new(WebService::new(network.clone(), Arc::new(FixedClock)));
    let context = service.begin_turn("conversation", "turn");
    let task_service = service.clone();
    let mut pending = tokio::spawn(async move { task_service.search(&context, request()).await });
    network.arrived.acquire().await.unwrap().forget();
    service.begin_turn("conversation", "next-turn");
    let cancelled = tokio::time::timeout(std::time::Duration::from_millis(50), &mut pending).await;
    pending.abort();
    assert!(matches!(cancelled, Ok(Ok(Err(WebError::Cancelled)))));
    assert_eq!(network.arrived.available_permits(), 0);
}

#[tokio::test]
#[ignore = "One public query against the anonymous service; never part of offline CI"]
async fn live_anonymous_search() {
    let service = WebService::live();
    let context = service.begin_turn("live-contract-check", "turn");
    let output = service
        .search(
            &context,
            SearchRequest {
                objective: "Find the official Rust language documentation".into(),
                queries: vec!["official Rust language documentation".into()],
                max_results: 5,
                refresh: false,
            },
        )
        .await
        .unwrap();
    println!(
        "keyless provider={}, degraded={}, sources={}",
        output.provider,
        output.degraded,
        output.results.len()
    );
    assert!(
        output
            .results
            .iter()
            .any(|source| source.url.starts_with("https://doc.rust-lang.org/"))
    );
}

struct CancelOnFailure {
    cancel: Mutex<Option<(std::sync::Weak<WebService>, aura_web::WebContext)>>,
    alternatives: std::sync::atomic::AtomicUsize,
}
impl Transport for CancelOnFailure {
    fn resolve<'a>(&'a self, _host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        Box::pin(async { Ok(vec!["93.184.216.34".parse().unwrap()]) })
    }
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        Box::pin(async move {
            if request.url.host_str() == Some("search.parallel.ai") {
                let (service, context) = self.cancel.lock().unwrap().take().unwrap();
                service.upgrade().unwrap().cancel_turn(&context);
                Err(WebError::Unavailable)
            } else {
                self.alternatives
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(HttpResponse {
                    status: 200,
                    headers: BTreeMap::new(),
                    body: br#"<div class="no-results">No results</div>"#.to_vec().into(),
                })
            }
        })
    }
}

#[tokio::test]
async fn cancellation_at_primary_failure_never_sends_the_query_to_an_alternative() {
    let network = Arc::new(CancelOnFailure {
        cancel: Mutex::new(None),
        alternatives: std::sync::atomic::AtomicUsize::new(0),
    });
    let service = Arc::new(WebService::new(network.clone(), Arc::new(FixedClock)));
    let context = service.begin_turn("conversation", "turn");
    *network.cancel.lock().unwrap() = Some((Arc::downgrade(&service), context.clone()));
    assert!(matches!(
        service.search(&context, request()).await,
        Err(WebError::Cancelled)
    ));
    assert_eq!(
        network
            .alternatives
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

#[tokio::test(start_paused = true)]
async fn consumes_the_matching_sse_message_without_waiting_for_the_server_to_close() {
    let network = Arc::new(Network::default());
    network.parallel(json!([]));
    {
        let mut responses = network.responses.lock().unwrap();
        let response = responses.back_mut().unwrap().as_mut().unwrap();
        response
            .headers
            .insert("content-type".into(), "text/event-stream".into());
        let message = format!("data: {}\n\n", json!({"jsonrpc":"2.0","id":3,"result":{"isError":false,"content":[{"type":"text","text":"{\"results\":[{\"title\":\"Manual público\",\"url\":\"https://news.example/manual\",\"excerpts\":[\"Manual de uso.\"]}]}"}]}})).into_bytes();
        response.body = aura_web::transport::HttpBody::new(
            futures::stream::once(async move { Ok(message) }).chain(futures::stream::pending()),
        );
        responses.push_back(Ok(HttpResponse {
            status: 200,
            headers: BTreeMap::new(),
            body: br#"<div class="no-results">No results</div>"#.to_vec().into(),
        }));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let start = tokio::time::Instant::now();
    let output = service.search(&context, request()).await.unwrap();
    assert_eq!(output.provider, "parallel");
    assert_eq!(output.results[0].title, "Manual público");
    assert_eq!(output.results[0].snippet, "Manual de uso.");
    assert_eq!(start.elapsed(), std::time::Duration::ZERO);
    assert_eq!(network.requests.lock().unwrap().len(), 4);
}

#[tokio::test]
async fn reports_unavailability_without_leaking_raw_upstream_errors() {
    let network = Arc::new(Network::default());
    for _ in 0..2 {
        network
            .responses
            .lock()
            .unwrap()
            .push_back(Ok(HttpResponse {
                status: 503,
                headers: BTreeMap::new(),
                body: b"cookie=private-value; full conversation contents"
                    .to_vec()
                    .into(),
            }));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let error = service.search(&context, request()).await.unwrap_err();
    assert_eq!(error, WebError::Unavailable);
    assert_eq!(error.to_string(), "The web service is unavailable");
    assert_eq!(serde_json::to_value(error).unwrap(), "unavailable");
    assert_eq!(network.requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_captcha_is_a_block_instead_of_an_empty_success() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Err(WebError::Timeout));
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Ok(HttpResponse {
            status: 202,
            headers: BTreeMap::new(),
            body: br#"<form id="challenge-form"><p>Confirm you are human</p></form>"#
                .to_vec()
                .into(),
        }));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.search(&context, request()).await,
        Err(WebError::Blocked)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn a_successful_empty_search_does_not_try_the_alternative() {
    let network = Arc::new(Network::default());
    network.parallel(json!([]));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.search(&context, request()).await.unwrap();
    assert!(output.results.is_empty());
    assert!(!output.degraded);
    assert_eq!(output.provider, "parallel");
    assert_eq!(network.requests.lock().unwrap().len(), 4);
}

#[tokio::test]
async fn the_alternative_interleaves_three_queries_and_unwraps_result_links() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Err(WebError::Unavailable));
    for (first, second, href) in [
        (
            "A1",
            "A2",
            "//duckduckgo.com/l/?uddg=https%3A%2F%2Fnews.example%2Fa1%3Futm_source%3Dx",
        ),
        ("B1", "B2", "https://news.example/b1"),
        ("C1", "C2", "https://news.example/c1"),
    ] {
        let html = format!(
            r#"<div class="result"><a class="result__a" href="{href}">{first}</a><span class="result__snippet">x &amp; y.</span></div><div class="result"><a class="result__a" href="https://news.example/{second}">{second}</a></div>"#
        );
        network
            .responses
            .lock()
            .unwrap()
            .push_back(Ok(HttpResponse {
                status: 200,
                headers: BTreeMap::new(),
                body: html.into_bytes().into(),
            }));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let mut input = request();
    input.queries = vec!["query A".into(), "query B".into(), "query C".into()];
    let output = service.search(&context, input).await.unwrap();
    assert_eq!(
        output
            .results
            .iter()
            .map(|source| source.title.as_str())
            .collect::<Vec<_>>(),
        ["A1", "B1", "C1", "A2", "B2"]
    );
    assert_eq!(output.results[0].url, "https://news.example/a1");
    assert_eq!(output.results[0].snippet, "x & y.");
    assert_eq!(network.requests.lock().unwrap().len(), 4);
}

#[tokio::test]
async fn stops_an_oversized_http_stream_before_parsing_or_fallback() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Ok(HttpResponse {
            status: 200,
            headers: BTreeMap::new(),
            body: vec![b'x'; 2 * 1024 * 1024 + 1].into(),
        }));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.search(&context, request()).await,
        Err(WebError::TooLarge)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn two_timeout_failures_return_one_unavailable_result_with_two_attempts() {
    let network = Arc::new(Network::default());
    for _ in 0..2 {
        network
            .responses
            .lock()
            .unwrap()
            .push_back(Err(WebError::Timeout));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.search(&context, request()).await,
        Err(WebError::Unavailable)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn sse_handles_fragmented_utf8_heartbeats_and_unrelated_messages() {
    let network = Arc::new(Network::default());
    network.parallel(json!([]));
    {
        let mut responses = network.responses.lock().unwrap();
        let response = responses.back_mut().unwrap().as_mut().unwrap();
        response
            .headers
            .insert("content-type".into(), "text/event-stream".into());
        let text = format!(
            ": keepalive\r\n\r\ndata: {{\"jsonrpc\":\"2.0\",\"id\":99,\"result\":{{}}}}\r\n\r\ndata: {}\r\n\r\n",
            json!({"jsonrpc":"2.0","id":3,"result":{"isError":false,"structuredContent":{"results":[{"title":"Título público","url":"https://news.example/manual","excerpts":["Informação íntegra."]}]}}})
        );
        let chunks: Vec<_> = text
            .into_bytes()
            .chunks(1)
            .map(|chunk| Ok(chunk.to_vec()))
            .collect();
        response.body = aura_web::transport::HttpBody::new(
            futures::stream::iter(chunks).chain(futures::stream::pending()),
        );
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.search(&context, request()).await.unwrap();
    assert_eq!(output.provider, "parallel");
    assert_eq!(output.results[0].title, "Título público");
    assert_eq!(output.results[0].snippet, "Informação íntegra.");
    assert_eq!(network.requests.lock().unwrap().len(), 4);
}
