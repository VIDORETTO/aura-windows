use aura_web::{
    Clock, FetchRequest, WebError, WebService,
    transport::{HttpRequest, HttpResponse, Transport},
};
use futures::future::BoxFuture;
use std::{
    collections::{BTreeMap, VecDeque},
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
struct Network {
    responses: Mutex<VecDeque<Result<HttpResponse, WebError>>>,
    requests: Mutex<Vec<HttpRequest>>,
    resolutions: Mutex<Vec<String>>,
    addresses: Mutex<Option<Vec<IpAddr>>>,
    rebind: std::sync::atomic::AtomicBool,
}
impl Network {
    fn page(&self, content_type: &str, body: impl Into<Vec<u8>>) {
        self.responses.lock().unwrap().push_back(Ok(HttpResponse {
            status: 200,
            headers: BTreeMap::from([("content-type".into(), content_type.into())]),
            body: body.into().into(),
        }));
    }
}
impl Transport for Network {
    fn resolve<'a>(&'a self, host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        self.resolutions.lock().unwrap().push(host.into());
        let addresses = self
            .addresses
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| vec!["93.184.216.34".parse().unwrap()]);
        if self.rebind.load(std::sync::atomic::Ordering::Relaxed) {
            *self.addresses.lock().unwrap() = Some(vec!["127.0.0.1".parse().unwrap()]);
        }
        Box::pin(async move { Ok(addresses) })
    }
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        self.requests.lock().unwrap().push(request);
        let response = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Err(WebError::Unavailable));
        Box::pin(async move { response })
    }
}
struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH
    }
}
fn request() -> FetchRequest {
    FetchRequest {
        url: "https://news.example/report".into(),
        start_char: 0,
        max_chars: 20_000,
        document_version: None,
        refresh: false,
    }
}

#[tokio::test]
async fn reads_article_facts_and_structure_without_navigation_or_active_html() {
    let network = Arc::new(Network::default());
    network.page(
        "text/html; charset=utf-8",
        include_bytes!("fixtures/report.html").to_vec(),
    );
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service
        .fetch(&context, request())
        .await
        .expect("a public article should be readable");
    assert_eq!(output.title, "Relatório de teste");
    assert_eq!(output.source_id, "W1");
    assert_eq!(output.final_url, "https://news.example/report");
    assert_eq!(output.requested_url, output.final_url);
    // Structured Markdown escapes punctuation; the literal fact is preserved.
    assert!(output.text.contains("A produção foi 42 unidades\\."));
    assert!(output.text.contains("Norte: 12"));
    assert!(output.text.contains("Sul: 30"));
    assert!(output.text.contains("| Norte | 12 |"));
    assert!(output.text.contains("| Sul | 30 |"));
    assert!(output.text.contains("# Relatório de teste"));
    for discarded in [
        "MENU DESCARTÁVEL",
        "STYLE DESCARTÁVEL",
        "SCRIPT DESCARTÁVEL",
        "RODAPÉ DESCARTÁVEL",
        "<article>",
    ] {
        assert!(!output.text.contains(discarded));
    }
    assert_eq!(output.retrieved_at, "1970-01-01T00:00:00Z");
    assert!(output.published_at.is_none());
    assert!(output.external_content);
    assert!(!output.cached);
    let sent = network.requests.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].method, "GET");
    assert_eq!(
        sent[0].addresses,
        vec!["93.184.216.34".parse::<IpAddr>().unwrap()]
    );
}

#[tokio::test]
async fn continues_a_plain_document_from_the_returned_version_without_network() {
    let network = Arc::new(Network::default());
    network.page("text/plain; charset=utf-8", b"ABCDE".to_vec());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let mut first_request = request();
    first_request.max_chars = 3;
    let first = service.fetch(&context, first_request).await.unwrap();
    assert_eq!(first.text, "ABC");
    assert_eq!(
        (first.start_char, first.end_char, first.next_start_char),
        (0, 3, Some(3))
    );
    assert!(first.truncated);
    assert!(!first.document_truncated);
    assert!(!first.cached);
    let mut continuation = request();
    continuation.start_char = 3;
    continuation.document_version = Some(first.document_version.clone());
    let second = service.fetch(&context, continuation).await.unwrap();
    assert_eq!(second.text, "DE");
    assert_eq!(
        (second.start_char, second.end_char, second.next_start_char),
        (3, 5, None)
    );
    assert!(!second.truncated);
    assert!(second.cached);
    assert_eq!(second.document_version, first.document_version);
    assert_eq!(second.source_id, first.source_id);
    assert_eq!(network.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn rejects_forbidden_url_authorities_before_dns_or_http() {
    for url in [
        "https://127.0.0.1/",
        "http://10.0.0.1/",
        "http://169.254.169.254/",
        "https://[::1]/",
        "https://[::ffff:127.0.0.1]/",
        "https://name:secret@news.example/",
        "https://@news.example/",
        "https://news.example:8443/",
        "file:///C:/secret",
        "https://localhost./",
        "https://printer.local/",
    ] {
        let network = Arc::new(Network::default());
        let service = WebService::new(network.clone(), Arc::new(FixedClock));
        let context = service.begin_turn("conversation", "turn");
        let mut input = request();
        input.url = url.into();
        assert!(
            matches!(
                service.fetch(&context, input).await,
                Err(WebError::UnsafeUrl)
            ),
            "{url}"
        );
        assert!(network.resolutions.lock().unwrap().is_empty());
        assert!(network.requests.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn rejects_reserved_transition_and_benchmark_destinations_before_connection() {
    for url in [
        "http://192.88.99.1/",
        "http://192.0.0.8/",
        "http://198.18.0.1/",
        "http://100.64.0.1/",
        "https://[2002:7f00:1::]/",
        "https://[2001:db8::1]/",
        "https://[3fff::1]/",
    ] {
        let network = Arc::new(Network::default());
        let service = WebService::new(network.clone(), Arc::new(FixedClock));
        let context = service.begin_turn("conversation", "turn");
        let mut input = request();
        input.url = url.into();
        assert!(
            matches!(
                service.fetch(&context, input).await,
                Err(WebError::UnsafeUrl)
            ),
            "{url}"
        );
        assert!(network.requests.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn rejects_mixed_public_and_private_dns_without_connecting() {
    let network = Arc::new(Network::default());
    *network.addresses.lock().unwrap() = Some(vec![
        "93.184.216.34".parse().unwrap(),
        "10.0.0.1".parse().unwrap(),
    ]);
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::UnsafeUrl)
    ));
    assert_eq!(network.resolutions.lock().unwrap().len(), 1);
    assert!(network.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn follows_a_public_redirect_and_preserves_requested_and_final_urls() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Ok(HttpResponse {
            status: 302,
            headers: BTreeMap::from([("location".into(), "https://docs.example/final".into())]),
            body: vec![].into(),
        }));
    network.page("text/plain", b"ABCDE".to_vec());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.fetch(&context, request()).await.unwrap();
    assert_eq!(output.requested_url, "https://news.example/report");
    assert_eq!(output.final_url, "https://docs.example/final");
    assert_eq!(output.text, "ABCDE");
    assert_eq!(
        network.resolutions.lock().unwrap().as_slice(),
        ["news.example", "docs.example"]
    );
    assert_eq!(network.requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn refuses_a_redirect_to_a_private_destination_without_resolving_it() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Ok(HttpResponse {
            status: 302,
            headers: BTreeMap::from([(
                "location".into(),
                "http://169.254.169.254/latest/meta-data/".into(),
            )]),
            body: vec![].into(),
        }));
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::UnsafeUrl)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 1);
    assert_eq!(
        network.resolutions.lock().unwrap().as_slice(),
        ["news.example"]
    );
}

#[tokio::test]
async fn refuses_pdf_instead_of_claiming_it_was_read_as_html() {
    let network = Arc::new(Network::default());
    network.page(
        "application/pdf",
        b"%PDF-1.7 synthetic ASCII PDF document".to_vec(),
    );
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::UnsupportedContent)
    ));
}

#[tokio::test]
async fn reports_a_captcha_page_as_blocked() {
    let network = Arc::new(Network::default());
    network.page("text/html",b"<html><title>Robot verification</title><body><form id=challenge-form>Verify you are human</form></body></html>".to_vec());
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::Blocked)
    ));
}

#[tokio::test]
async fn reports_a_login_only_page_as_unsupported() {
    let network = Arc::new(Network::default());
    network.page("text/html",b"<html><title>Login</title><body><h1>Sign in</h1><form><input type=password>Log in to continue</form></body></html>".to_vec());
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::UnsupportedContent)
    ));
}

#[tokio::test]
async fn reports_a_javascript_only_shell_as_unsupported() {
    let network = Arc::new(Network::default());
    network.page("text/html",b"<html><body><div id=root></div><script>loadArticle()</script><noscript>Enable JavaScript to continue</noscript></body></html>".to_vec());
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::UnsupportedContent)
    ));
}

#[tokio::test]
async fn refuses_an_empty_document_without_inventing_content() {
    let network = Arc::new(Network::default());
    network.page(
        "text/html",
        b"<html><title>Empty</title><body></body></html>".to_vec(),
    );
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::ExtractFailed)
    ));
}

#[tokio::test]
async fn pagination_counts_unicode_characters_without_splitting_an_emoji() {
    let network = Arc::new(Network::default());
    network.page("text/plain", "A😀BCé".as_bytes().to_vec());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let mut input = request();
    input.max_chars = 3;
    let first = service.fetch(&context, input).await.unwrap();
    assert_eq!(first.text, "A😀B");
    assert_eq!(first.next_start_char, Some(3));
    let mut input = request();
    input.start_char = 3;
    input.document_version = Some(first.document_version);
    let second = service.fetch(&context, input).await.unwrap();
    assert_eq!(second.text, "Cé");
    assert_eq!(second.end_char, 5);
    assert_eq!(second.next_start_char, None);
    assert_eq!(network.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn a_version_from_another_document_is_rejected_without_new_network() {
    let network = Arc::new(Network::default());
    network.page("text/plain", b"ABCDE".to_vec());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    service.fetch(&context, request()).await.unwrap();
    let mut input = request();
    input.start_char = 3;
    input.document_version = Some("different-version".into());
    assert!(matches!(
        service.fetch(&context, input).await,
        Err(WebError::InvalidInput)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
#[ignore = "One public static page; never part of offline CI"]
async fn live_public_html() {
    let service = WebService::live();
    let context = service.begin_turn("public-smoke", "turn");
    let mut input = request();
    input.url = "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html".into();
    let output = service
        .fetch(&context, input)
        .await
        .expect("public Rust documentation");
    assert!(output.title.contains("Ownership"));
    assert!(output.text.contains("ownership"));
    assert!(output.text.contains("String"));
    assert!(!output.text.contains("<script"));
    assert!(output.external_content);
    println!(
        "public HTML: extractor={}, chars={}, truncated={}",
        output.extractor,
        output.text.chars().count(),
        output.truncated
    );
}

#[tokio::test]
async fn evicted_pagination_versions_require_a_restart_without_network() {
    let network = Arc::new(Network::default());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let mut first_version = String::new();
    for i in 0..33 {
        network.page("text/plain", b"ABCDE".to_vec());
        let context = service.begin_turn("conversation", &format!("turn-{i}"));
        let mut input = request();
        input.url = format!("https://news.example/doc-{i}");
        input.max_chars = 3;
        let output = service.fetch(&context, input).await.unwrap();
        if i == 0 {
            first_version = output.document_version;
        }
    }
    let context = service.begin_turn("conversation", "turn-34");
    let mut input = request();
    input.url = "https://news.example/doc-0".into();
    input.start_char = 3;
    input.document_version = Some(first_version);
    assert!(matches!(
        service.fetch(&context, input).await,
        Err(WebError::InvalidInput)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 33);
}

#[tokio::test]
async fn rejects_a_decoded_stream_over_two_mebibytes_before_extraction() {
    let network = Arc::new(Network::default());
    network.page("text/plain", vec![b'A'; 2 * 1024 * 1024 + 1]);
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::TooLarge)
    ));
}

#[tokio::test(start_paused = true)]
async fn a_page_that_never_finishes_hits_the_shared_thirty_second_deadline() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Ok(HttpResponse {
            status: 200,
            headers: BTreeMap::new(),
            body: aura_web::transport::HttpBody::new(futures::stream::pending()),
        }));
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let started = tokio::time::Instant::now();
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::Timeout)
    ));
    assert_eq!(started.elapsed(), std::time::Duration::from_secs(30));
}

#[tokio::test]
async fn document_truncation_stops_at_one_hundred_thousand_unicode_characters() {
    let network = Arc::new(Network::default());
    network.page("text/plain", "😀".repeat(100_001).into_bytes());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let mut output = service.fetch(&context, request()).await.unwrap();
    assert!(output.document_truncated);
    assert_eq!(output.text.chars().count(), 20_000);
    assert_eq!(output.warnings, ["document_truncated"]);
    for start in [20_000, 40_000, 60_000, 80_000] {
        let mut input = request();
        input.start_char = start;
        input.document_version = Some(output.document_version.clone());
        output = service.fetch(&context, input).await.unwrap();
        assert_eq!(output.text, "😀".repeat(20_000));
    }
    assert_eq!(output.end_char, 100_000);
    assert_eq!(output.next_start_char, None);
    assert!(!output.truncated);
    assert!(output.document_truncated);
    assert_eq!(network.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn cancellation_stops_a_pending_read_and_does_not_register_a_successful_source() {
    let network = Arc::new(Network::default());
    network
        .responses
        .lock()
        .unwrap()
        .push_back(Ok(HttpResponse {
            status: 200,
            headers: BTreeMap::new(),
            body: aura_web::transport::HttpBody::new(futures::stream::pending()),
        }));
    let service = Arc::new(WebService::new(network.clone(), Arc::new(FixedClock)));
    let context = service.begin_turn("conversation", "turn");
    let worker = {
        let service = service.clone();
        let context = context.clone();
        tokio::spawn(async move { service.fetch(&context, request()).await })
    };
    while network.requests.lock().unwrap().is_empty() {
        tokio::task::yield_now().await;
    }
    service.cancel_turn(&context);
    assert!(matches!(worker.await.unwrap(), Err(WebError::Cancelled)));
    network.page("text/plain", b"ABCDE".to_vec());
    let next = service.begin_turn("conversation", "next-turn");
    let output = service.fetch(&next, request()).await.unwrap();
    assert_eq!(output.source_id, "W1");
    assert!(!output.cached);
    assert_eq!(network.requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn fetch_budget_refuses_the_seventh_read_even_from_cache() {
    let network = Arc::new(Network::default());
    network.page("text/plain", b"ABCDE".to_vec());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    for _ in 0..6 {
        service.fetch(&context, request()).await.unwrap();
    }
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::LimitExceeded)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 1);
    let next = service.begin_turn("conversation", "next-turn");
    assert!(service.fetch(&next, request()).await.unwrap().cached);
}

#[tokio::test]
async fn pagination_expires_after_fifteen_minutes_without_mixing_a_new_document() {
    struct MovingClock(std::sync::atomic::AtomicU64);
    impl Clock for MovingClock {
        fn now(&self) -> SystemTime {
            UNIX_EPOCH
                + std::time::Duration::from_secs(self.0.load(std::sync::atomic::Ordering::Relaxed))
        }
    }
    let clock = Arc::new(MovingClock(std::sync::atomic::AtomicU64::new(0)));
    let network = Arc::new(Network::default());
    network.page("text/plain", b"ABCDE".to_vec());
    let service = WebService::new(network.clone(), clock.clone());
    let context = service.begin_turn("conversation", "turn");
    let first = service.fetch(&context, request()).await.unwrap();
    clock.0.store(901, std::sync::atomic::Ordering::Relaxed);
    let mut input = request();
    input.start_char = 3;
    input.document_version = Some(first.document_version);
    assert!(matches!(
        service.fetch(&context, input).await,
        Err(WebError::InvalidInput)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn document_cache_evicts_by_bytes_before_reaching_thirty_two_entries() {
    let network = Arc::new(Network::default());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let mut version = String::new();
    for i in 0..23 {
        network.page("text/plain", "😀".repeat(100_000).into_bytes());
        let context = service.begin_turn("conversation", &format!("turn-{i}"));
        let mut input = request();
        input.url = format!("https://news.example/large-{i}");
        let output = service.fetch(&context, input).await.unwrap();
        if i == 0 {
            version = output.document_version;
        }
    }
    let context = service.begin_turn("conversation", "turn-24");
    let mut input = request();
    input.url = "https://news.example/large-0".into();
    input.start_char = 20_000;
    input.document_version = Some(version);
    assert!(matches!(
        service.fetch(&context, input).await,
        Err(WebError::InvalidInput)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 23);
}

#[tokio::test]
async fn aggregate_document_cache_evicts_across_conversations_at_thirty_two_mebibytes() {
    let network = Arc::new(Network::default());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let mut first_version = String::new();
    // 18*400000 bytes <8MiB per conversation; 5*18*400000 >32MiB globally.
    for conversation in 0..5 {
        for document in 0..18 {
            network.page("text/plain", "😀".repeat(100_000).into_bytes());
            let context = service.begin_turn(
                &format!("conversation-{conversation}"),
                &format!("turn-{document}"),
            );
            let mut input = request();
            input.url = format!("https://news.example/doc-{document}");
            let output = service.fetch(&context, input).await.unwrap();
            if conversation == 0 && document == 0 {
                first_version = output.document_version;
            }
        }
    }
    let context = service.begin_turn("conversation-0", "next-turn");
    let mut input = request();
    input.url = "https://news.example/doc-0".into();
    input.start_char = 20_000;
    input.document_version = Some(first_version);
    assert!(matches!(
        service.fetch(&context, input).await,
        Err(WebError::InvalidInput)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 90);
}

#[tokio::test]
async fn redirects_are_limited_to_three_and_do_not_request_a_fourth_destination() {
    let network = Arc::new(Network::default());
    for i in 0..4 {
        network
            .responses
            .lock()
            .unwrap()
            .push_back(Ok(HttpResponse {
                status: 302,
                headers: BTreeMap::from([("location".into(), format!("/hop-{i}"))]),
                body: vec![].into(),
            }));
    }
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::LimitExceeded)
    ));
    assert_eq!(network.requests.lock().unwrap().len(), 4);
}

#[tokio::test]
async fn refuses_an_html_document_over_the_element_work_limit() {
    let network = Arc::new(Network::default());
    network.page(
        "text/html",
        format!("<html><body>{}</body></html>", "<p>Text</p>".repeat(20_001)).into_bytes(),
    );
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::TooLarge)
    ));
}

#[tokio::test]
async fn honors_the_declared_html_charset_without_corrupting_the_facts() {
    let network = Arc::new(Network::default());
    network.page("text/html; charset=iso-8859-1",b"<html><title>Relat\xf3rio</title><body><article><h1>Produ\xe7\xe3o</h1><p>A produ\xe7\xe3o foi 42 unidades.</p></article></body></html>".to_vec());
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.fetch(&context, request()).await.unwrap();
    assert_eq!(output.title, "Relatório");
    assert!(output.text.contains("Produção"));
    assert!(output.text.contains("A produção foi 42 unidades\\."));
}

#[tokio::test]
async fn reads_a_nested_main_article_once_instead_of_duplicating_its_facts() {
    let network = Arc::new(Network::default());
    network.page("text/html",b"<html><title>Report</title><body><main><article><p>Production: 42 units.</p></article></main></body></html>".to_vec());
    let service = WebService::new(network, Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.fetch(&context, request()).await.unwrap();
    assert_eq!(output.text.matches("Production: 42 units").count(), 1);
}

#[tokio::test]
async fn an_empty_turn_context_is_invalid_before_network() {
    let network = Arc::new(Network::default());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("", "");
    assert!(matches!(
        service.fetch(&context, request()).await,
        Err(WebError::InvalidInput)
    ));
    assert!(network.resolutions.lock().unwrap().is_empty());
    assert!(network.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn pins_the_approved_addresses_without_resolving_again_after_a_dns_rebind() {
    let network = Arc::new(Network::default());
    network
        .rebind
        .store(true, std::sync::atomic::Ordering::Relaxed);
    network.page("text/plain", b"ABCDE".to_vec());
    let service = WebService::new(network.clone(), Arc::new(FixedClock));
    let context = service.begin_turn("conversation", "turn");
    let output = service.fetch(&context, request()).await.unwrap();
    assert_eq!(output.text, "ABCDE");
    assert_eq!(network.resolutions.lock().unwrap().len(), 1);
    assert_eq!(
        network.requests.lock().unwrap()[0].addresses,
        vec!["93.184.216.34".parse::<IpAddr>().unwrap()]
    );
    assert_eq!(
        *network.addresses.lock().unwrap(),
        Some(vec!["127.0.0.1".parse().unwrap()])
    );
}
