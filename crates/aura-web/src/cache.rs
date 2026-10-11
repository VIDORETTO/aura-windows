//! Shared in-memory budget for search results and extracted documents.
use crate::{Conversation, SearchOutput, WebError, WebSource, fetch};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

pub(crate) struct SearchEntry {
    pub output: Arc<SearchOutput>,
    pub created_at: SystemTime,
    pub last_used: u64,
}
impl SearchEntry {
    pub fn fresh(&self, now: SystemTime) -> bool {
        now.duration_since(self.created_at)
            .is_ok_and(|age| age < Duration::from_secs(900))
    }
    pub(crate) fn bytes(&self) -> usize {
        let output = &self.output;
        std::mem::size_of::<Self>()
            + std::mem::size_of::<SearchOutput>()
            + 256
            + output.provider.capacity()
            + output.searched_at.capacity()
            + output.query_context.objective.capacity()
            + output.query_context.queries.capacity() * std::mem::size_of::<String>()
            + output
                .query_context
                .queries
                .iter()
                .map(String::capacity)
                .sum::<usize>()
            + output.warnings.capacity() * std::mem::size_of::<String>()
            + output.warnings.iter().map(String::capacity).sum::<usize>()
            + output.results.capacity() * std::mem::size_of::<crate::WebSource>()
            + output
                .results
                .iter()
                .map(|s| {
                    s.source_id.capacity()
                        + s.title.capacity()
                        + s.url.capacity()
                        + s.snippet.capacity()
                        + s.retrieved_at.capacity()
                        + s.kind.capacity()
                        + s.published_at.as_ref().map_or(0, String::capacity)
                })
                .sum::<usize>()
    }
}
fn bytes(state: &Conversation) -> usize {
    retained_bytes(state)
        + fetch::document_bytes(&state.documents)
        + state
            .search_cache
            .iter()
            .map(|(key, entry)| key.capacity() + entry.bytes())
            .sum::<usize>()
}
pub(crate) fn delivery_bytes(state: &Conversation) -> usize {
    state
        .deliveries
        .iter()
        .map(|(key, entry)| key.capacity() + entry.bytes())
        .sum()
}
pub(crate) fn retained_bytes(state: &Conversation) -> usize {
    registry_bytes(state) + delivery_bytes(state)
}
fn source_bytes(key: &String, source: &WebSource) -> usize {
    // Includes duplicate URL storage and conservative hash-bucket overhead.
    key.capacity()
        + std::mem::size_of::<WebSource>()
        + 128
        + source.source_id.capacity()
        + source.title.capacity()
        + source.url.capacity()
        + source.snippet.capacity()
        + source.retrieved_at.capacity()
        + source.kind.capacity()
        + source.published_at.as_ref().map_or(0, String::capacity)
}
pub(crate) fn registry_bytes(state: &Conversation) -> usize {
    1024 + std::mem::size_of::<Conversation>()
        + state.session.capacity()
        + state.turn.capacity()
        + state
            .sources
            .iter()
            .map(|(key, source)| source_bytes(key, source))
            .sum::<usize>()
}
/// Preserve stable provenance; evict expendable cached bodies/results instead.
/// If metadata plus the newest entry cannot fit, reject before mutating IDs,
/// provenance or cache. Never silently evict an ID and bind it to another URL.
pub(crate) fn admit_sources(
    all: &HashMap<String, Conversation>,
    conversation: &str,
    changes: &[(String, WebSource)],
    latest_bytes: usize,
) -> Result<(), WebError> {
    let state = all.get(conversation).ok_or(WebError::Cancelled)?;
    let current = retained_bytes(state);
    let proposed = changes.iter().fold(current, |bytes, (key, source)| {
        bytes
            - state
                .sources
                .get_key_value(key)
                .map_or(0, |(key, old)| source_bytes(key, old))
            + source_bytes(key, source)
    });
    let aggregate = all.values().map(retained_bytes).sum::<usize>() - current + proposed;
    if proposed.saturating_add(latest_bytes) > 8 * 1024 * 1024
        || aggregate.saturating_add(latest_bytes) > 32 * 1024 * 1024
    {
        return Err(WebError::LimitExceeded);
    }
    Ok(())
}
// The same monotonically increasing access sequence is shared by both caches.
fn oldest(state: &Conversation) -> Option<(bool, String, u64)> {
    state
        .documents
        .iter()
        .map(|(key, doc)| (false, key.clone(), doc.last_used))
        .chain(
            state
                .search_cache
                .iter()
                .map(|(key, entry)| (true, key.clone(), entry.last_used)),
        )
        .min_by_key(|(_, _, used)| *used)
}
fn evict(state: &mut Conversation, search: bool, key: &str) {
    if search {
        state.search_cache.remove(key);
    } else {
        state.documents.remove(key);
    }
}
pub(crate) fn enforce_limits(all: &mut HashMap<String, Conversation>) {
    for state in all.values_mut() {
        while state.documents.len() + state.search_cache.len() + state.deliveries.len() > 32
            || bytes(state) > 8 * 1024 * 1024
        {
            let Some((search, key, _)) = oldest(state) else {
                break;
            };
            evict(state, search, &key);
        }
    }
    while all.values().map(bytes).sum::<usize>() > 32 * 1024 * 1024 {
        let candidate = all
            .iter()
            .filter_map(|(conversation, state)| {
                oldest(state).map(|(search, key, used)| (conversation.clone(), search, key, used))
            })
            .min_by_key(|(_, _, _, used)| *used);
        let Some((conversation, search, key, _)) = candidate else {
            break;
        };
        evict(all.get_mut(&conversation).unwrap(), search, &key);
    }
}
