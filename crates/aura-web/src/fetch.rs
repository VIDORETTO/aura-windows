use crate::{
    SearchSlot, WebContext, WebError, WebService, WebSource, search, transport::Transport,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, atomic::Ordering},
    time::{Duration, SystemTime},
};
use url::Url;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FetchRequest {
    pub url: String,
    #[serde(default)]
    pub start_char: usize,
    #[serde(default = "default_chars")]
    pub max_chars: usize,
    #[serde(default)]
    pub document_version: Option<String>,
    #[serde(default)]
    pub refresh: bool,
}
fn default_chars() -> usize {
    20_000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchOutput {
    pub source_id: String,
    pub requested_url: String,
    pub final_url: String,
    pub title: String,
    pub content_type: String,
    pub extractor: String,
    pub text: String,
    pub retrieved_at: String,
    pub published_at: Option<String>,
    pub start_char: usize,
    pub end_char: usize,
    pub next_start_char: Option<usize>,
    pub document_version: String,
    pub truncated: bool,
    pub document_truncated: bool,
    pub cached: bool,
    pub warnings: Vec<String>,
    pub external_content: bool,
}

#[derive(Clone)]
pub(crate) struct Page {
    pub final_url: String,
    pub title: String,
    pub content_type: String,
    pub extractor: String,
    pub text: String,
    pub published_at: Option<String>,
}

#[derive(Clone)]
pub(crate) struct Document {
    page: Arc<Page>,
    source_id: String,
    retrieved_at: String,
    version: String,
    created_at: SystemTime,
    document_truncated: bool,
    pub(crate) last_used: u64,
}

impl WebService {
    pub async fn fetch(
        &self,
        context: &WebContext,
        request: FetchRequest,
    ) -> Result<FetchOutput, WebError> {
        tokio::time::timeout(
            Duration::from_secs(30),
            self.perform_fetch(context, request),
        )
        .await
        .unwrap_or(Err(WebError::Timeout))
    }
    async fn perform_fetch(
        &self,
        context: &WebContext,
        request: FetchRequest,
    ) -> Result<FetchOutput, WebError> {
        if context.conversation.is_empty()
            || context.turn.is_empty()
            || request.start_char > 100_000
            || !(1..=20_000).contains(&request.max_chars)
            || (request.start_char > 0 && request.document_version.is_none())
            || (request.refresh && (request.start_char > 0 || request.document_version.is_some()))
        {
            return Err(WebError::InvalidInput);
        }
        search::normalize_url(&request.url)?;
        let now = self.clock.now();
        let (cancellation, cached, _slot) = {
            let mut all = self.conversations.lock().unwrap();
            if !self.is_enabled() {
                return Err(WebError::WebDisabled);
            }
            if !context.admitted {
                return Err(WebError::LimitExceeded);
            }
            let state = all
                .get_mut(&context.conversation)
                .ok_or(WebError::InvalidInput)?;
            if state.turn != context.turn
                || state.capability != context.capability
                || state.cancellation.is_cancelled()
            {
                return Err(WebError::Cancelled);
            }
            if state.in_flight >= 2 || state.fetches >= 6 {
                return Err(WebError::LimitExceeded);
            }
            state.in_flight += 1;
            state.fetches += 1;
            state.documents.retain(|_, doc| {
                now.duration_since(doc.created_at)
                    .is_ok_and(|age| age < Duration::from_secs(900))
            });
            let cached = state
                .documents
                .get_mut(&request.url)
                .filter(|_| !request.refresh)
                .map(|doc| {
                    doc.last_used = self.cache_sequence.fetch_add(1, Ordering::Relaxed);
                    doc.clone()
                });
            (
                state.cancellation.clone(),
                cached,
                SearchSlot {
                    all: &self.conversations,
                    conversation: context.conversation.clone(),
                    session: context.session.clone(),
                },
            )
        };
        if let Some(version) = &request.document_version {
            let doc = cached
                .filter(|doc| &doc.version == version)
                .ok_or(WebError::InvalidInput)?;
            return doc.slice(request, true);
        }
        if let Some(doc) = cached {
            return doc.slice(request, true);
        }
        let mut page = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(WebError::Cancelled),
            result = read(self.network.as_ref(), &request) => result?,
        };
        let document_truncated = page.text.chars().count() > 100_000;
        if document_truncated {
            page.text = page.text.chars().take(100_000).collect();
        }
        let now = self.clock.now();
        let retrieved_at = chrono::DateTime::<chrono::Utc>::from(now)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let mut all = self.conversations.lock().unwrap();
        if !self.is_enabled() {
            return Err(WebError::Cancelled);
        }
        let state = all
            .get_mut(&context.conversation)
            .filter(|s| {
                s.turn == context.turn
                    && s.capability == context.capability
                    && !s.cancellation.is_cancelled()
            })
            .ok_or(WebError::Cancelled)?;
        let key = search::normalize_url(&page.final_url)?;
        let existing_id = state
            .sources
            .get(&key)
            .map(|source| source.source_id.clone());
        let next_source = if existing_id.is_some() {
            state.next_source
        } else {
            state
                .next_source
                .checked_add(1)
                .ok_or(WebError::LimitExceeded)?
        };
        let id = existing_id.unwrap_or_else(|| format!("W{}", state.next_source));
        let changes = vec![(
            key,
            WebSource {
                source_id: id.clone(),
                title: page.title.clone(),
                url: page.final_url.clone(),
                snippet: page.text.chars().take(1000).collect(),
                published_at: page.published_at.clone(),
                retrieved_at: retrieved_at.clone(),
                kind: "pageContent".into(),
            },
        )];
        let doc = Document {
            page: Arc::new(page),
            source_id: id,
            retrieved_at,
            version: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            document_truncated,
            last_used: self.cache_sequence.fetch_add(1, Ordering::Relaxed),
        };
        let latest_bytes = document_bytes(&HashMap::from([(request.url.clone(), doc.clone())]));
        crate::cache::admit_sources(&all, &context.conversation, &changes, latest_bytes)?;
        let state = all.get_mut(&context.conversation).unwrap();
        state.next_source = next_source;
        for (key, source) in changes {
            state.sources.insert(key, source);
        }
        state.documents.insert(request.url.clone(), doc.clone());
        crate::cache::enforce_limits(&mut all);
        doc.slice(request, false)
    }
}

pub(crate) fn document_bytes(documents: &HashMap<String, Document>) -> usize {
    documents
        .iter()
        .map(|(key, doc)| {
            key.capacity()
                + doc.source_id.capacity()
                + doc.version.capacity()
                + doc.retrieved_at.capacity()
                + doc.page.final_url.capacity()
                + doc.page.title.capacity()
                + doc.page.content_type.capacity()
                + doc.page.extractor.capacity()
                + doc.page.text.capacity()
                + doc.page.published_at.as_ref().map_or(0, String::capacity)
                + std::mem::size_of::<Document>()
                + std::mem::size_of::<Page>()
                + 256
        })
        .sum()
}
impl Document {
    fn slice(&self, request: FetchRequest, cached: bool) -> Result<FetchOutput, WebError> {
        let length = self.page.text.chars().count();
        if request.start_char > length {
            return Err(WebError::InvalidInput);
        }
        let end_char = request
            .start_char
            .saturating_add(request.max_chars)
            .min(length);
        let next_start_char = (end_char < length).then_some(end_char);
        Ok(FetchOutput {
            source_id: self.source_id.clone(),
            requested_url: request.url,
            final_url: self.page.final_url.clone(),
            title: self.page.title.clone(),
            content_type: self.page.content_type.clone(),
            extractor: self.page.extractor.clone(),
            text: self
                .page
                .text
                .chars()
                .skip(request.start_char)
                .take(end_char - request.start_char)
                .collect(),
            retrieved_at: self.retrieved_at.clone(),
            published_at: self.page.published_at.clone(),
            start_char: request.start_char,
            end_char,
            next_start_char,
            document_version: self.version.clone(),
            truncated: next_start_char.is_some(),
            document_truncated: self.document_truncated,
            cached,
            warnings: if self.document_truncated {
                vec!["document_truncated".into()]
            } else {
                vec![]
            },
            external_content: true,
        })
    }
}

pub(crate) async fn read(
    network: &dyn Transport,
    request: &FetchRequest,
) -> Result<Page, WebError> {
    search::normalize_url(&request.url)?;
    let mut url = Url::parse(&request.url).map_err(|_| WebError::UnsafeUrl)?;
    url.set_fragment(None);
    for redirects in 0..=3 {
        let response = search::send(
            network,
            url.clone(),
            "GET",
            BTreeMap::from([("accept".into(), "text/html, text/plain".into())]),
            vec![],
        )
        .await?;
        match response.status {
            301 | 302 | 303 | 307 | 308 => {
                if redirects == 3 {
                    return Err(WebError::LimitExceeded);
                }
                let location = response
                    .headers
                    .get("location")
                    .ok_or(WebError::Unavailable)?;
                if location.chars().any(char::is_control)
                    || location.contains('\\')
                    || search::contains_userinfo(location)
                {
                    return Err(WebError::UnsafeUrl);
                }
                url = url.join(location).map_err(|_| WebError::UnsafeUrl)?;
                search::normalize_url(url.as_str())?;
                url.set_fragment(None);
                continue;
            }
            200..=299 => {}
            401 | 403 => return Err(WebError::Blocked),
            429 => return Err(WebError::RateLimited),
            _ => return Err(WebError::Unavailable),
        }
        let content_type = response
            .headers
            .get("content-type")
            .cloned()
            .unwrap_or_default();
        let final_url = url.to_string();
        let mut body = Vec::new();
        let mut stream = response.body.into_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if body.len().saturating_add(chunk.len()) > 2 * 1024 * 1024 {
                return Err(WebError::TooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        return tokio::task::spawn_blocking(move || extract(&final_url, &content_type, body))
            .await
            .map_err(|_| WebError::ExtractFailed)?;
    }
    Err(WebError::LimitExceeded)
}

fn extract(url: &str, content_type: &str, bytes: Vec<u8>) -> Result<Page, WebError> {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if !matches!(
        mime.as_str(),
        "" | "text/plain" | "text/html" | "application/xhtml+xml"
    ) || bytes.starts_with(b"%PDF-")
    {
        return Err(WebError::UnsupportedContent);
    }
    let declared = content_type.split(';').skip(1).find_map(|part| {
        let (name, value) = part.split_once('=')?;
        name.trim()
            .eq_ignore_ascii_case("charset")
            .then(|| value.trim().trim_matches(['\'', '"']))
    });
    let (encoding, skip) = if let Some(bom) = encoding_rs::Encoding::for_bom(&bytes) {
        bom
    } else {
        (
            match declared {
                Some(label) => encoding_rs::Encoding::for_label(label.as_bytes())
                    .ok_or(WebError::UnsupportedContent)?,
                None => encoding_rs::UTF_8,
            },
            0,
        )
    };
    let (decoded, errors) = encoding.decode_without_bom_handling(&bytes[skip..]);
    if errors
        || decoded
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t' | '\x0c'))
    {
        return Err(WebError::UnsupportedContent);
    }
    let html = decoded.into_owned();
    let sniff_html = html.trim_start().starts_with('<');
    if mime == "text/plain" || mime.is_empty() && !sniff_html {
        if html.trim().is_empty() {
            return Err(WebError::ExtractFailed);
        }
        return Ok(Page {
            final_url: url.into(),
            title: Url::parse(url)
                .ok()
                .and_then(|url| url.host_str().map(str::to_owned))
                .unwrap_or_default(),
            content_type: content_type.into(),
            extractor: "plain_text".into(),
            text: html,
            published_at: None,
        });
    }
    let doc = dom_query::Document::from(html.as_str());
    if doc.select("*").length() > 20_000 {
        return Err(WebError::TooLarge);
    }
    if doc.select("#challenge-form,#anomaly-form,.anomaly-modal,#cf-challenge-running,.g-recaptcha,[id^=cf-chl]").length() > 0 { return Err(WebError::Blocked); }
    if doc.select("input[type=password]").length() > 0
        && doc
            .select("article,main,[role=main]")
            .text()
            .trim()
            .is_empty()
    {
        return Err(WebError::UnsupportedContent);
    }
    let title = doc.select("title").text().trim().to_owned();
    let has_scripts = doc.select("script").length() > 0;
    doc.select(
        "script,style,nav,footer,header,aside,iframe,object,embed,svg,form,noscript,[hidden]",
    )
    .remove();
    let cfg = dom_smoothie::Config {
        max_elements_to_parse: 20_000,
        text_mode: dom_smoothie::TextMode::Markdown,
        ..Default::default()
    };
    let article = dom_smoothie::Readability::new(doc.html().to_string(), Some(url), Some(cfg))
        .ok()
        .and_then(|mut reader| reader.parse().ok());
    let (text, extractor, published_at) =
        if let Some(article) = article.filter(|article| article.length >= 500) {
            (
                article.text_content.to_string(),
                "readability",
                article
                    .published_time
                    .filter(|value| chrono::DateTime::parse_from_rfc3339(value).is_ok()),
            )
        } else {
            let main = doc.select("main,[role=main]");
            let selected = if main.length() > 0 {
                main.first()
            } else if doc.select("article").length() > 0 {
                doc.select("article").first()
            } else {
                doc.select("body").first()
            };
            (
                selected
                    .nodes()
                    .iter()
                    .map(|node| node.md(None).to_string())
                    .collect::<Vec<_>>()
                    .join("\n\n"),
                "dom_fallback",
                None,
            )
        };
    if text.trim().is_empty() {
        return Err(if has_scripts {
            WebError::UnsupportedContent
        } else {
            WebError::ExtractFailed
        });
    }
    Ok(Page {
        final_url: url.into(),
        title,
        content_type: content_type.into(),
        extractor: extractor.into(),
        text,
        published_at,
    })
}
