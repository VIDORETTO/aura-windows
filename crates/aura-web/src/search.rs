use crate::{
    SearchRequest, WebError,
    transport::{HttpRequest, Transport},
};
use futures::StreamExt;
use serde_json::{Value, json};
use std::{collections::BTreeMap, net::IpAddr};
use url::Url;

pub(crate) struct Row {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published_at: Option<String>,
}

pub(crate) fn contains_userinfo(raw: &str) -> bool {
    let authority = raw
        .strip_prefix("//")
        .or_else(|| raw.split_once("://").map(|(_, rest)| rest));
    authority.is_some_and(|rest| {
        rest.split(['/', '?', '#'])
            .next()
            .is_some_and(|authority| authority.contains('@'))
    })
}

pub(crate) fn normalize_url(raw: &str) -> Result<String, WebError> {
    if raw.chars().count() > 2048
        || raw.trim() != raw
        || raw.chars().any(char::is_control)
        || raw.contains('\\')
        || contains_userinfo(raw)
    {
        return Err(WebError::UnsafeUrl);
    }
    let mut url = Url::parse(raw).map_err(|_| WebError::UnsafeUrl)?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || !matches!(
            (url.scheme(), url.port_or_known_default()),
            ("http", Some(80)) | ("https", Some(443))
        )
    {
        return Err(WebError::UnsafeUrl);
    }
    let host = url
        .host_str()
        .ok_or(WebError::UnsafeUrl)?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host == "localhost"
        || [".localhost", ".local", ".internal"]
            .iter()
            .any(|end| host.ends_with(end))
    {
        return Err(WebError::UnsafeUrl);
    }
    if let Ok(ip) = host.trim_matches(['[', ']']).parse::<IpAddr>()
        && !public_ip(ip)
    {
        return Err(WebError::UnsafeUrl);
    }
    url.set_fragment(None);
    let pairs: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| !key.to_ascii_lowercase().starts_with("utm_"))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    url.set_query(None);
    if !pairs.is_empty() {
        url.query_pairs_mut().extend_pairs(pairs);
    }
    let normalized: String = url.into();
    if normalized.chars().count() > 2048 {
        return Err(WebError::UnsafeUrl);
    }
    Ok(normalized)
}

pub(crate) fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_broadcast()
                && !ip.is_documentation()
                && a != 0
                && a < 224
                && !(a == 100 && (64..=127).contains(&b))
                && !(a == 192 && b == 0 && c == 0)
                && !(a == 192 && b == 88 && c == 99)
                && !(a == 198 && (b == 18 || b == 19))
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ip.to_ipv4_mapped() {
                return public_ip(v4.into());
            }
            let segments = ip.segments();
            (segments[0] & 0xe000) == 0x2000
                && segments[0] != 0x2002
                && !(segments[0] == 0x2001 && segments[1] <= 0x01ff)
                && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
                && !(segments[0] == 0x3fff && segments[1] < 0x1000)
        }
    }
}

pub(crate) struct BufferedResponse {
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

pub(crate) async fn http(
    network: &dyn Transport,
    url: Url,
    method: &str,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
) -> Result<BufferedResponse, WebError> {
    let rpc_id = serde_json::from_slice::<Value>(&body)
        .ok()
        .and_then(|value| value["id"].as_u64());
    let response = send(network, url, method, headers, body).await?;
    match response.status {
        200..=299 => {}
        401 | 403 => return Err(WebError::Blocked),
        429 => return Err(WebError::RateLimited),
        _ => return Err(WebError::Unavailable),
    }
    let mut stream = response.body.into_stream();
    let mut body = Vec::new();
    let mut sse = SseCursor::default();
    let is_sse = response
        .headers
        .get("content-type")
        .is_some_and(|ct| ct.starts_with("text/event-stream"));
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if body.len().saturating_add(chunk.len()) > 2 * 1024 * 1024 {
            return Err(WebError::TooLarge);
        }
        body.extend_from_slice(&chunk);
        if is_sse && rpc_id.is_some_and(|id| sse.message(&body, id).is_some()) {
            break;
        }
    }
    Ok(BufferedResponse {
        headers: response.headers,
        body,
    })
}

pub(crate) async fn send(
    network: &dyn Transport,
    url: Url,
    method: &str,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
) -> Result<crate::transport::HttpResponse, WebError> {
    normalize_url(url.as_str())?;
    let addresses = match url.host().ok_or(WebError::UnsafeUrl)? {
        url::Host::Ipv4(ip) => vec![IpAddr::V4(ip)],
        url::Host::Ipv6(ip) => vec![IpAddr::V6(ip)],
        url::Host::Domain(host) => network.resolve(host).await?,
    };
    if addresses.is_empty() || addresses.iter().any(|ip| !public_ip(*ip)) {
        return Err(WebError::UnsafeUrl);
    }
    network
        .request(HttpRequest {
            url,
            method: method.into(),
            headers,
            body,
            addresses,
        })
        .await
}

/// Scan newly received bytes once; heartbeats and unrelated notifications must
/// not cause quadratic rescanning or wait for EOF after the requested reply.
#[derive(Default)]
struct SseCursor {
    event_start: usize,
    scanned: usize,
}
impl SseCursor {
    fn message(&mut self, body: &[u8], id: u64) -> Option<Value> {
        let mut at = self.scanned;
        while at < body.len() {
            let length = if body.get(at..at + 4) == Some(b"\r\n\r\n") {
                4
            } else if matches!(body.get(at..at + 3), Some(b"\r\n\n" | b"\n\r\n")) {
                3
            } else if matches!(body.get(at..at + 2), Some(b"\n\n" | b"\r\r" | b"\n\r")) {
                2
            } else {
                at += 1;
                continue;
            };
            let event = std::str::from_utf8(&body[self.event_start..at]).ok();
            at += length;
            self.event_start = at;
            self.scanned = at;
            if let Some(event) = event {
                let normalized = event.replace("\r\n", "\n").replace('\r', "\n");
                let data = normalized
                    .lines()
                    .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
                    .collect::<Vec<_>>()
                    .join("\n");
                if let Ok(value) = serde_json::from_str::<Value>(&data)
                    && value.get("id").and_then(Value::as_u64) == Some(id)
                {
                    return Some(value);
                }
            }
        }
        self.scanned = body.len().saturating_sub(3).max(self.event_start);
        None
    }
}

fn rpc_json(response: &BufferedResponse, id: u64) -> Result<Value, WebError> {
    let parsed: Value = if response
        .headers
        .get("content-type")
        .is_some_and(|ct| ct.starts_with("text/event-stream"))
    {
        SseCursor::default()
            .message(&response.body, id)
            .ok_or(WebError::Unavailable)?
    } else {
        serde_json::from_slice(&response.body).map_err(|_| WebError::Unavailable)?
    };
    if parsed.get("id").and_then(Value::as_u64) != Some(id)
        || parsed.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || parsed.get("error").is_some()
    {
        return Err(WebError::Unavailable);
    }
    parsed.get("result").cloned().ok_or(WebError::Unavailable)
}

pub(crate) async fn parallel(
    network: &dyn Transport,
    session: &str,
    request: &SearchRequest,
) -> Result<Vec<Row>, WebError> {
    let url = Url::parse("https://search.parallel.ai/mcp").unwrap();
    let mut headers = BTreeMap::from([
        ("content-type".into(), "application/json".into()),
        (
            "accept".into(),
            "application/json, text/event-stream".into(),
        ),
    ]);
    let init = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"Aura","version":env!("CARGO_PKG_VERSION")}}});
    let response = http(
        network,
        url.clone(),
        "POST",
        headers.clone(),
        serde_json::to_vec(&init).unwrap(),
    )
    .await?;
    let initialized = rpc_json(&response, 1)?;
    let version = initialized["protocolVersion"]
        .as_str()
        .filter(|s| ["2025-03-26", "2025-06-18", "2025-11-25"].contains(s))
        .ok_or(WebError::Unavailable)?;
    headers.insert("mcp-protocol-version".into(), version.into());
    if let Some(session_header) = response.headers.get("mcp-session-id") {
        if session_header.len() > 200 || !session_header.bytes().all(|b| (33..=126).contains(&b)) {
            return Err(WebError::Unavailable);
        }
        headers.insert("mcp-session-id".into(), session_header.clone());
    }
    http(
        network,
        url.clone(),
        "POST",
        headers.clone(),
        br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#.to_vec(),
    )
    .await?;
    let response = http(
        network,
        url.clone(),
        "POST",
        headers.clone(),
        br#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#.to_vec(),
    )
    .await?;
    let list = rpc_json(&response, 2)?;
    let schema = list["tools"]
        .as_array()
        .and_then(|tools| tools.iter().find(|tool| tool["name"] == "web_search"))
        .map(|tool| &tool["inputSchema"])
        .ok_or(WebError::Unavailable)?;
    if schema["properties"]["objective"]["type"] != "string"
        || schema["properties"]["search_queries"]["type"] != "array"
        || schema["properties"]["session_id"]["type"] != "string"
        || schema["required"].as_array().is_none_or(|required| {
            required.iter().any(|p| {
                !["objective", "search_queries", "session_id"]
                    .iter()
                    .any(|known| p == known)
            })
        })
    {
        return Err(WebError::Unavailable);
    }
    let call = json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"web_search","arguments":{"objective":request.objective,"search_queries":request.queries,"session_id":session}}});
    let response = http(
        network,
        url,
        "POST",
        headers,
        serde_json::to_vec(&call).unwrap(),
    )
    .await?;
    let result = rpc_json(&response, 3)?;
    if result["isError"] == true {
        return Err(WebError::Unavailable);
    }
    let payload = match result
        .get("structuredContent")
        .filter(|p| p.get("results").is_some())
    {
        Some(p) => p.clone(),
        None => result["content"]
            .as_array()
            .and_then(|items| {
                items
                    .iter()
                    .filter(|item| item["type"] == "text")
                    .find_map(|item| serde_json::from_str::<Value>(item["text"].as_str()?).ok())
            })
            .ok_or(WebError::Unavailable)?,
    };
    let rows = payload["results"].as_array().ok_or(WebError::Unavailable)?;
    Ok(rows
        .iter()
        .filter_map(|row| {
            let title = row["title"].as_str()?.to_owned();
            let url = row["url"].as_str()?.to_owned();
            let snippet = row["excerpts"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            let published_at = row["publish_date"]
                .as_str()
                .filter(|s| {
                    chrono::DateTime::parse_from_rfc3339(s).is_ok()
                        || chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok()
                })
                .map(str::to_owned);
            Some(Row {
                title,
                url,
                snippet,
                published_at,
            })
        })
        .collect())
}

pub(crate) async fn duckduckgo(
    network: &dyn Transport,
    request: &SearchRequest,
) -> Result<Vec<Row>, WebError> {
    let mut pages = Vec::new();
    for query in &request.queries {
        let mut url = Url::parse("https://html.duckduckgo.com/html/").unwrap();
        url.query_pairs_mut().append_pair("q", query);
        let response = http(
            network,
            url,
            "GET",
            BTreeMap::from([("accept".into(), "text/html".into())]),
            vec![],
        )
        .await?;
        let rows = tokio::task::spawn_blocking(move || parse_ddg(&response.body))
            .await
            .map_err(|_| WebError::Unavailable)??;
        pages.push(rows);
    }
    // Interleave ranks across queries instead of appending a whole query first.
    let mut interleaved = Vec::new();
    let longest = pages.iter().map(Vec::len).max().unwrap_or(0);
    let mut pages: Vec<_> = pages.into_iter().map(Vec::into_iter).collect();
    for _ in 0..longest {
        for page in &mut pages {
            if let Some(row) = page.next() {
                interleaved.push(row);
            }
        }
    }
    Ok(interleaved)
}

fn parse_ddg(body: &[u8]) -> Result<Vec<Row>, WebError> {
    let html = std::str::from_utf8(body).map_err(|_| WebError::Unavailable)?;
    let doc = dom_query::Document::from(html);
    if doc
        .select("#challenge-form, #anomaly-form, .anomaly-modal")
        .iter()
        .next()
        .is_some()
    {
        return Err(WebError::Blocked);
    }
    let mut rows = Vec::new();
    let results = doc.select(".result");
    if results.iter().next().is_none() && doc.select(".no-results").iter().next().is_none() {
        return Err(WebError::Unavailable);
    }
    for result in &results {
        let link = result.select(".result__a");
        let Some(href) = link.attr("href") else {
            continue;
        };
        let Ok(mut url) = Url::parse("https://html.duckduckgo.com")
            .unwrap()
            .join(&href)
        else {
            continue;
        };
        if url
            .host_str()
            .is_some_and(|host| host == "duckduckgo.com" || host.ends_with(".duckduckgo.com"))
            && url.path() == "/l/"
        {
            let Some(destination) = url
                .query_pairs()
                .find(|(key, _)| key == "uddg")
                .map(|(_, value)| value.into_owned())
            else {
                continue;
            };
            let Ok(destination) = Url::parse(&destination) else {
                continue;
            };
            url = destination;
        }
        rows.push(Row {
            title: link.text().trim().into(),
            url: url.into(),
            snippet: result.select(".result__snippet").text().trim().into(),
            published_at: None,
        });
    }
    Ok(rows)
}
