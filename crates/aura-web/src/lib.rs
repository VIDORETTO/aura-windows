//! Keyless public-web access. Only the Host supplies conversation/turn context.
mod cache;
mod delivery;
mod fetch;
mod search;
pub mod transport;
pub use fetch::{FetchOutput, FetchRequest};

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::SystemTime,
};
use transport::Transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum WebError {
    #[error("Invalid web request or unavailable turn context")]
    InvalidInput,
    #[error("Web access is disabled")]
    WebDisabled,
    #[error("The destination is not a public web address")]
    UnsafeUrl,
    #[error("The search service rate limited the request")]
    RateLimited,
    #[error("The web service is unavailable")]
    Unavailable,
    #[error("The web request timed out")]
    Timeout,
    #[error("The site blocked automated access")]
    Blocked,
    #[error("This content type is not supported")]
    UnsupportedContent,
    #[error("The response exceeds the web size limit")]
    TooLarge,
    #[error("The web request limit was reached")]
    LimitExceeded,
    #[error("The web request was cancelled")]
    Cancelled,
    #[error("No readable content was found")]
    ExtractFailed,
}

pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchRequest {
    pub objective: String,
    pub queries: Vec<String>,
    #[serde(default = "default_results")]
    pub max_results: usize,
    #[serde(default)]
    pub refresh: bool,
}
fn default_results() -> usize {
    5
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSource {
    pub source_id: String,
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published_at: Option<String>,
    pub retrieved_at: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOutput {
    pub provider: String,
    pub query_context: SearchRequest,
    pub results: Vec<WebSource>,
    pub searched_at: String,
    pub cached: bool,
    pub degraded: bool,
    pub warnings: Vec<String>,
}

/// A capability created by a trusted Host turn, never decoded from tool input.
#[derive(Clone, PartialEq, Eq)]
pub struct WebContext {
    conversation: String,
    turn: String,
    session: String,
    admitted: bool,
    capability: uuid::Uuid,
}

struct Conversation {
    session: String,
    turn: String,
    sources: HashMap<String, WebSource>,
    in_flight: usize,
    searches: usize,
    fetches: usize,
    documents: HashMap<String, fetch::Document>,
    search_cache: HashMap<String, cache::SearchEntry>,
    deliveries: HashMap<String, delivery::Entry>,
    capability: uuid::Uuid,
    cancellation: tokio_util::sync::CancellationToken,
    next_source: usize,
}
struct SearchSlot<'a> {
    all: &'a Mutex<HashMap<String, Conversation>>,
    conversation: String,
    session: String,
}
impl Drop for SearchSlot<'_> {
    fn drop(&mut self) {
        if let Some(state) = self
            .all
            .lock()
            .unwrap()
            .get_mut(&self.conversation)
            .filter(|state| state.session == self.session)
        {
            state.in_flight = state.in_flight.saturating_sub(1);
        }
    }
}
pub struct WebService {
    network: Arc<dyn Transport>,
    clock: Arc<dyn Clock>,
    conversations: Mutex<HashMap<String, Conversation>>,
    cache_sequence: AtomicU64,
    enabled: AtomicBool,
}
impl WebService {
    pub fn live() -> Self {
        Self::new(Arc::new(transport::ReqwestTransport), Arc::new(SystemClock))
    }
    pub fn new(network: Arc<dyn Transport>, clock: Arc<dyn Clock>) -> Self {
        Self {
            network,
            clock,
            conversations: Mutex::new(HashMap::new()),
            cache_sequence: AtomicU64::new(0),
            enabled: AtomicBool::new(true),
        }
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }
    /// Cancel every outstanding capability on a preference change. Re-enabling
    /// preserves usage, source IDs, cache and the external session. Only the
    /// trusted Host can acquire the replacement capability with begin_turn.
    pub fn set_enabled(&self, enabled: bool) {
        let mut all = self.conversations.lock().unwrap();
        if self.enabled.swap(enabled, Ordering::AcqRel) == enabled {
            return;
        }
        for state in all.values_mut() {
            state.cancellation.cancel();
            state.deliveries.clear();
            state.capability = uuid::Uuid::new_v4();
            if enabled {
                state.cancellation = tokio_util::sync::CancellationToken::new();
            }
        }
    }
    pub fn begin_turn(&self, conversation: &str, turn: &str) -> WebContext {
        let mut all = self.conversations.lock().unwrap();
        let previous = all.get(conversation);
        let existing_bytes = previous.map_or(0, cache::retained_bytes);
        let proposed = if let Some(state) = previous {
            existing_bytes - state.turn.capacity() + turn.len()
                - if state.turn != turn {
                    cache::delivery_bytes(state)
                } else {
                    0
                }
        } else {
            1024 + std::mem::size_of::<Conversation>() + 36 + conversation.len() + turn.len()
        };
        if proposed > 8 * 1024 * 1024
            || all.values().map(cache::retained_bytes).sum::<usize>() - existing_bytes + proposed
                > 32 * 1024 * 1024
        {
            let session = previous
                .map(|state| state.session.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            if let Some(state) = all.get_mut(conversation) {
                state.cancellation.cancel();
                state.deliveries.clear();
            }
            return WebContext {
                conversation: conversation.into(),
                turn: turn.into(),
                session,
                capability: uuid::Uuid::new_v4(),
                admitted: false,
            };
        }
        let state = all
            .entry(conversation.into())
            .or_insert_with(|| Conversation {
                session: uuid::Uuid::new_v4().to_string(),
                turn: turn.into(),
                sources: HashMap::new(),
                in_flight: 0,
                searches: 0,
                fetches: 0,
                documents: HashMap::new(),
                search_cache: HashMap::new(),
                deliveries: HashMap::new(),
                capability: uuid::Uuid::new_v4(),
                cancellation: tokio_util::sync::CancellationToken::new(),
                next_source: 1,
            });
        if state.turn != turn {
            state.cancellation.cancel();
            state.deliveries.clear();
            state.cancellation = tokio_util::sync::CancellationToken::new();
            state.capability = uuid::Uuid::new_v4();
            state.turn = turn.into();
            state.searches = 0;
            state.fetches = 0;
        }
        let context = WebContext {
            conversation: conversation.into(),
            turn: turn.into(),
            session: state.session.clone(),
            admitted: true,
            capability: state.capability,
        };
        cache::enforce_limits(&mut all);
        context
    }
    pub fn cancel_turn(&self, context: &WebContext) {
        if let Some(state) = self
            .conversations
            .lock()
            .unwrap()
            .get_mut(&context.conversation)
            .filter(|s| s.capability == context.capability)
        {
            state.cancellation.cancel();
            state.deliveries.clear();
        }
    }
    /// Trusted historical provenance, never accepted through tool arguments.
    /// Reserve every previously issued number, including uncited identifiers.
    pub fn restore_sources(
        &self,
        context: &WebContext,
        last_id: usize,
        sources: Vec<WebSource>,
    ) -> Result<(), WebError> {
        if !context.admitted {
            return Err(WebError::LimitExceeded);
        }
        let mut all = self.conversations.lock().unwrap();
        let Some(state) = all
            .get_mut(&context.conversation)
            .filter(|state| state.capability == context.capability)
        else {
            return Err(WebError::Cancelled);
        };
        if let Some(next) = last_id.checked_add(1) {
            state.next_source = state.next_source.max(next);
        }
        let mut changes = Vec::new();
        for source in sources {
            let Some(number) = source
                .source_id
                .strip_prefix('W')
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|number| *number > 0)
            else {
                continue;
            };
            let Some(next) = number.checked_add(1) else {
                continue;
            };
            let Ok(url) = search::normalize_url(&source.url) else {
                continue;
            };
            if state
                .sources
                .values()
                .chain(changes.iter().map(|(_, source)| source))
                .any(|known| known.source_id == source.source_id && known.url != source.url)
            {
                continue;
            }
            state.next_source = state.next_source.max(next);
            if !state.sources.contains_key(&url) && !changes.iter().any(|(key, _)| key == &url) {
                changes.push((url, source));
            }
        }
        cache::admit_sources(&all, &context.conversation, &changes, 0)?;
        let state = all.get_mut(&context.conversation).unwrap();
        for (url, source) in changes {
            state.sources.insert(url, source);
        }
        cache::enforce_limits(&mut all);
        Ok(())
    }
    pub fn cancel_conversation(&self, conversation: &str) {
        if let Some(state) = self.conversations.lock().unwrap().remove(conversation) {
            state.cancellation.cancel();
        }
    }
    /// Provenance from this trusted capability, never from a model-supplied ID.
    pub fn source(&self, context: &WebContext, source_id: &str) -> Option<WebSource> {
        let all = self.conversations.lock().unwrap();
        let state = all.get(&context.conversation)?;
        if !self.is_enabled()
            || state.turn != context.turn
            || state.capability != context.capability
            || state.cancellation.is_cancelled()
        {
            return None;
        }
        state
            .sources
            .values()
            .find(|source| source.source_id == source_id)
            .cloned()
    }
    /// Metadata already consulted in this conversation remains usable for
    /// citations after web is disabled. This never reads document/search cache
    /// or authorizes a tool result. Closed/replaced conversations cannot match.
    pub fn citation_source(&self, context: &WebContext, source_id: &str) -> Option<WebSource> {
        let all = self.conversations.lock().unwrap();
        let state = all.get(&context.conversation)?;
        if state.session != context.session || state.turn != context.turn {
            return None;
        }
        state
            .sources
            .values()
            .find(|source| source.source_id == source_id)
            .cloned()
    }
    pub async fn search(
        &self,
        context: &WebContext,
        request: SearchRequest,
    ) -> Result<SearchOutput, WebError> {
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.perform_search(context, request),
        )
        .await
        .unwrap_or(Err(WebError::Timeout))
    }

    async fn perform_search(
        &self,
        context: &WebContext,
        request: SearchRequest,
    ) -> Result<SearchOutput, WebError> {
        if request.objective.trim().is_empty()
            || request.objective.chars().count() > 2000
            || !(1..=3).contains(&request.queries.len())
            || request
                .queries
                .iter()
                .any(|q| q.trim().is_empty() || q.chars().count() > 200)
            || !(1..=10).contains(&request.max_results)
            || context.conversation.is_empty()
            || context.turn.is_empty()
        {
            return Err(WebError::InvalidInput);
        }
        let cache_key =
            serde_json::to_string(&(&request.objective, &request.queries, request.max_results))
                .map_err(|_| WebError::InvalidInput)?;
        let (session, cancellation, _slot) = {
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
            if state.in_flight >= 2 || state.searches >= 3 {
                return Err(WebError::LimitExceeded);
            }
            state.searches += 1;
            let now = self.clock.now();
            state.search_cache.retain(|_, entry| entry.fresh(now));
            if !request.refresh
                && let Some(entry) = state.search_cache.get_mut(&cache_key)
            {
                entry.last_used = self.cache_sequence.fetch_add(1, Ordering::Relaxed);
                let mut output = (*entry.output).clone();
                output.query_context = request;
                output.cached = true;
                return Ok(output);
            }
            state.in_flight += 1;
            (
                state.session.clone(),
                state.cancellation.clone(),
                SearchSlot {
                    all: &self.conversations,
                    conversation: context.conversation.clone(),
                    session: context.session.clone(),
                },
            )
        };
        let search = async {
            // Reserve part of the shared deadline for the free alternative.
            let primary = tokio::time::timeout(
                std::time::Duration::from_secs(15),
                search::parallel(self.network.as_ref(), &session, &request),
            )
            .await
            .unwrap_or(Err(WebError::Timeout));
            match primary {
                Ok(rows) => Ok((rows, "parallel", false)),
                Err(
                    WebError::Unavailable
                    | WebError::Timeout
                    | WebError::RateLimited
                    | WebError::Blocked,
                ) => {
                    if cancellation.is_cancelled() {
                        return Err(WebError::Cancelled);
                    }
                    let rows = search::duckduckgo(self.network.as_ref(), &request)
                        .await
                        .map_err(|e| match e {
                            WebError::Unavailable | WebError::Timeout | WebError::RateLimited => {
                                WebError::Unavailable
                            }
                            other => other,
                        })?;
                    Ok((rows, "duckduckgo", true))
                }
                Err(other) => Err(other),
            }
        };
        let (rows, provider, degraded) = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(WebError::Cancelled),
            result = search => result?,
        };
        let searched_at = chrono::DateTime::<chrono::Utc>::from(self.clock.now())
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
        let mut results = Vec::new();
        let mut changes = Vec::new();
        let mut next_source = state.next_source;
        for row in rows {
            let Ok(url) = search::normalize_url(&row.url) else {
                continue;
            };
            if results.iter().any(|r: &WebSource| r.url == url) {
                continue;
            }
            let id = if let Some(known) = state.sources.get(&url) {
                known.source_id.clone()
            } else {
                let id = format!("W{next_source}");
                next_source = next_source.checked_add(1).ok_or(WebError::LimitExceeded)?;
                id
            };
            let source = WebSource {
                source_id: id,
                title: row.title,
                url: url.clone(),
                snippet: row.snippet.chars().take(1000).collect(),
                published_at: row.published_at,
                retrieved_at: searched_at.clone(),
                kind: "searchSnippet".into(),
            };
            if !state
                .sources
                .get(&url)
                .is_some_and(|known| known.kind == "pageContent")
            {
                changes.push((url, source.clone()));
            }
            results.push(source);
            if results.len() == request.max_results {
                break;
            }
        }
        let output = SearchOutput {
            provider: provider.into(),
            query_context: request,
            results,
            searched_at,
            cached: false,
            degraded,
            warnings: if degraded {
                vec!["primary_unavailable".into()]
            } else {
                vec![]
            },
        };
        let entry = cache::SearchEntry {
            output: Arc::new(output.clone()),
            created_at: self.clock.now(),
            last_used: self.cache_sequence.fetch_add(1, Ordering::Relaxed),
        };
        cache::admit_sources(
            &all,
            &context.conversation,
            &changes,
            cache_key.capacity() + entry.bytes(),
        )?;
        let state = all.get_mut(&context.conversation).unwrap();
        state.next_source = next_source;
        for (url, source) in changes {
            state.sources.insert(url, source);
        }
        state.search_cache.insert(cache_key, entry);
        cache::enforce_limits(&mut all);
        Ok(output)
    }
}
