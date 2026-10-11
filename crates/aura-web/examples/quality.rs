//! Bounded live evaluation; writes metadata only, never complete page bodies.
use aura_web::{
    FetchRequest, SearchRequest, SystemClock, WebError, WebService,
    transport::{HttpRequest, HttpResponse, ReqwestTransport, Transport},
};
use futures::future::BoxFuture;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    net::IpAddr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const QUERIES: [(&str, &str); 12] = [
    ("Rust ownership official book", "rust-lang.org"),
    ("Tauri v2 invoke commands", "v2.tauri.app"),
    ("React useEffect documentation", "react.dev"),
    ("Microsoft WebView2 introduction", "learn.microsoft.com"),
    ("Python pathlib documentation", "docs.python.org"),
    ("SQLite foreign keys documentation", "sqlite.org"),
    ("IBGE Censo 2022 resultados", "ibge.gov.br"),
    ("INPE programa queimadas dados", "inpe.br"),
    ("Museu do Amanhã horário visita", "museudoamanha.org.br"),
    ("W3C WCAG 2.2 recommendation", "w3.org"),
    (
        "OpenAI web search tools documentation",
        "developers.openai.com",
    ),
    ("SearXNG search API JSON", "docs.searxng.org"),
];

struct Budget {
    http: AtomicUsize,
    deadline: Instant,
    transport: ReqwestTransport,
}
impl Transport for Budget {
    fn resolve<'a>(&'a self, host: &'a str) -> BoxFuture<'a, Result<Vec<IpAddr>, WebError>> {
        Box::pin(async move {
            if Instant::now() >= self.deadline {
                return Err(WebError::LimitExceeded);
            }
            self.transport.resolve(host).await
        })
    }
    fn request(&self, request: HttpRequest) -> BoxFuture<'_, Result<HttpResponse, WebError>> {
        Box::pin(async move {
            if Instant::now() >= self.deadline
                || self
                    .http
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                        (count < 60).then_some(count + 1)
                    })
                    .is_err()
            {
                return Err(WebError::LimitExceeded);
            }
            self.transport.request(request).await
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    query: usize,
    url: String,
    needle: String,
    inspection: String,
}

fn save(path: &PathBuf, report: &Value) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path, serde_json::to_vec_pretty(report)?)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: quality <report.json> <targets.json>".into());
    }
    let report_path = PathBuf::from(&args[0]);
    let targets_path = PathBuf::from(&args[1]);
    if report_path.exists() || targets_path.exists() {
        return Err(
            "use new report/targets paths; target inspection must follow this search run".into(),
        );
    }
    let started = Instant::now();
    let deadline = started + Duration::from_secs(30 * 60);
    let network = Arc::new(Budget {
        http: AtomicUsize::new(0),
        deadline,
        transport: ReqwestTransport,
    });
    let service = WebService::new(network.clone(), Arc::new(SystemClock));
    let conversation = uuid::Uuid::new_v4().to_string();
    let mut report = json!({"startedAt":chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::now()).to_rfc3339(),
        "phase":"search", "searches":[], "reads":[], "http":0, "elapsedSeconds":0});
    save(&report_path, &report)?;
    for (index, (query, domain)) in QUERIES.iter().enumerate() {
        if Instant::now() >= deadline || network.http.load(Ordering::Relaxed) >= 60 {
            break;
        }
        let context = service.begin_turn(&conversation, &format!("search-{}", index + 1));
        let call_started = Instant::now();
        let result = service
            .search(
                &context,
                SearchRequest {
                    objective: format!("Encontrar a fonte oficial para {query}"),
                    queries: vec![(*query).into()],
                    max_results: 5,
                    refresh: true,
                },
            )
            .await;
        let row = match result {
            Ok(output) => json!({"case":index+1,"query":query,"domain":domain,"output":output}),
            Err(error) => json!({"case":index+1,"query":query,"domain":domain,"error":error}),
        };
        report["searches"].as_array_mut().unwrap().push(row);
        let last = report["searches"]
            .as_array_mut()
            .unwrap()
            .last_mut()
            .unwrap();
        last["elapsedMs"] = json!(call_started.elapsed().as_millis());
        report["http"] = json!(network.http.load(Ordering::Relaxed));
        report["elapsedSeconds"] = json!(started.elapsed().as_secs());
        save(&report_path, &report)?;
        eprintln!("search {}/12 complete; HTTP {}", index + 1, report["http"]);
    }
    if report["searches"].as_array().unwrap().len() != 12
        || network.http.load(Ordering::Relaxed) >= 60
    {
        report["phase"] = json!("budget_exhausted");
        save(&report_path, &report)?;
        return Err("evaluation budget exhausted before all search/reading cases".into());
    }
    report["phase"] = json!("awaiting_inspected_targets");
    save(&report_path, &report)?;
    eprintln!(
        "Search complete. Waiting for ten independently inspected targets at the specified path."
    );
    while !targets_path.exists() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    if !targets_path.exists() {
        report["phase"] = json!("inspection_deadline");
        save(&report_path, &report)?;
        return Err("30 minute evaluation deadline reached while awaiting inspection".into());
    }
    let targets: Vec<Target> = serde_json::from_slice(&std::fs::read(&targets_path)?)?;
    if targets.len() != 10 {
        return Err("exactly ten inspected static HTML targets are required".into());
    }
    for target in &targets {
        if !(1..=12).contains(&target.query)
            || target.needle.trim().is_empty()
            || target.inspection.trim().is_empty()
            || !report["searches"][target.query - 1]["output"]["results"]
                .as_array()
                .is_some_and(|sources| sources.iter().any(|source| source["url"] == target.url))
        {
            return Err("target must be an inspected URL returned by its original query, with a literal oracle".into());
        }
    }
    report["phase"] = json!("reading");
    for (index, target) in targets.iter().enumerate() {
        if Instant::now() >= deadline || network.http.load(Ordering::Relaxed) >= 60 {
            break;
        }
        let context = service.begin_turn(&conversation, &format!("read-{}", index + 1));
        let call_started = Instant::now();
        let result = service
            .fetch(
                &context,
                FetchRequest {
                    url: target.url.clone(),
                    start_char: 0,
                    max_chars: 20_000,
                    document_version: None,
                    refresh: true,
                },
            )
            .await;
        let mut row = json!({"query":target.query,"url":target.url,"needle":target.needle,"inspection":target.inspection});
        match result {
            Ok(output) => {
                row["matched"] = json!(output.text.contains(&target.needle));
                row["finalUrl"] = json!(output.final_url);
                row["title"] = json!(output.title);
                row["extractor"] = json!(output.extractor);
                row["chars"] = json!(output.end_char - output.start_char);
                row["truncated"] = json!(output.truncated);
                row["warnings"] = json!(output.warnings);
            }
            Err(error) => {
                row["error"] = json!(error);
                row["matched"] = json!(false);
            }
        }
        row["elapsedMs"] = json!(call_started.elapsed().as_millis());
        report["reads"].as_array_mut().unwrap().push(row);
        report["http"] = json!(network.http.load(Ordering::Relaxed));
        report["elapsedSeconds"] = json!(started.elapsed().as_secs());
        save(&report_path, &report)?;
        eprintln!("read {}/10 complete; HTTP {}", index + 1, report["http"]);
    }
    report["phase"] = json!(if report["reads"].as_array().unwrap().len() == 10 {
        "complete"
    } else {
        "budget_exhausted"
    });
    save(&report_path, &report)?;
    service.cancel_conversation(&conversation);
    Ok(())
}
