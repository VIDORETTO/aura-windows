//! Transient tool results share the web service's memory and lifecycle.
use crate::{WebContext, WebError, WebService, cache};
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

pub(crate) struct Entry {
    thread: String,
    tool: String,
    text: Arc<String>,
    capability: uuid::Uuid,
    created_at: SystemTime,
    kind: Kind,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Result,
    Arguments,
    Reasoning,
}
impl Entry {
    pub(crate) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + 256
            + self.thread.capacity()
            + self.tool.capacity()
            + self.text.capacity()
    }
    fn fresh(&self, now: SystemTime) -> bool {
        now.duration_since(self.created_at)
            .is_ok_and(|age| age < Duration::from_secs(900))
    }
}

impl WebService {
    /// Capture whether the active model request contains private web deliveries.
    pub fn has_private_context(&self, context: &WebContext) -> bool {
        let all = self.conversations.lock().unwrap();
        let now = self.clock.now();
        self.is_enabled()
            && all.get(&context.conversation).is_some_and(|state| {
                state.capability == context.capability
                    && !state.cancellation.is_cancelled()
                    && state.deliveries.values().any(|entry| entry.fresh(now))
            })
    }

    pub fn retain_private_reasoning(
        &self,
        context: &WebContext,
        thread: &str,
        text: String,
    ) -> Result<String, WebError> {
        self.retain_delivery(context, thread, "reasoning", text, Kind::Reasoning)
    }

    pub fn resolve_private_reasoning(
        &self,
        conversation: &str,
        thread: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, WebError> {
        self.resolve_delivery(conversation, thread, "reasoning", handle, Kind::Reasoning)
    }
    /// Called by the trusted Host, never with a context decoded from model input.
    pub fn retain_tool_result(
        &self,
        context: &WebContext,
        thread: &str,
        tool: &str,
        text: String,
    ) -> Result<String, WebError> {
        self.retain_delivery(context, thread, tool, text, Kind::Result)
    }

    /// Provider-issued arguments use the same bounded, cancellable storage.
    pub fn retain_tool_arguments(
        &self,
        context: &WebContext,
        thread: &str,
        tool: &str,
        text: String,
    ) -> Result<String, WebError> {
        self.retain_delivery(context, thread, tool, text, Kind::Arguments)
    }

    fn retain_delivery(
        &self,
        context: &WebContext,
        thread: &str,
        tool: &str,
        text: String,
        kind: Kind,
    ) -> Result<String, WebError> {
        if thread.is_empty()
            || !(matches!(tool, "web_search" | "web_fetch")
                || (tool == "reasoning" && kind == Kind::Reasoning))
        {
            return Err(WebError::InvalidInput);
        }
        if !context.admitted {
            return Err(WebError::LimitExceeded);
        }
        let mut all = self.conversations.lock().unwrap();
        if !self.is_enabled() {
            return Err(WebError::WebDisabled);
        }
        let now = self.clock.now();
        for state in all.values_mut() {
            state.deliveries.retain(|_, entry| entry.fresh(now));
        }
        let state = all.get(&context.conversation).ok_or(WebError::Cancelled)?;
        if state.capability != context.capability
            || state.session != context.session
            || state.turn != context.turn
            || state.cancellation.is_cancelled()
        {
            return Err(WebError::Cancelled);
        }
        let entry = Entry {
            thread: thread.into(),
            tool: tool.into(),
            text: Arc::new(text),
            capability: context.capability,
            created_at: now,
            kind,
        };
        cache::admit_sources(&all, &context.conversation, &[], 64 + entry.bytes())?;
        if state.deliveries.len() >= 32 {
            return Err(WebError::LimitExceeded);
        }
        let mut random = [0u8; 32];
        getrandom::fill(&mut random).map_err(|_| WebError::Unavailable)?;
        let handle: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        if all
            .values()
            .any(|state| state.deliveries.contains_key(&handle))
        {
            return Err(WebError::Unavailable);
        }
        all.get_mut(&context.conversation)
            .unwrap()
            .deliveries
            .insert(handle.clone(), entry);
        cache::enforce_limits(&mut all);
        Ok(handle)
    }

    /// Scope is supplied by the Host after authenticating the gateway request.
    pub fn resolve_tool_result(
        &self,
        conversation: &str,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, WebError> {
        self.resolve_delivery(conversation, thread, tool, handle, Kind::Result)
    }

    pub fn resolve_tool_arguments(
        &self,
        conversation: &str,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, WebError> {
        self.resolve_delivery(conversation, thread, tool, handle, Kind::Arguments)
    }

    fn resolve_delivery(
        &self,
        conversation: &str,
        thread: &str,
        tool: &str,
        handle: &str,
        kind: Kind,
    ) -> Result<Option<Arc<String>>, WebError> {
        let mut all = self.conversations.lock().unwrap();
        if !self.is_enabled() {
            return Ok(None);
        }
        let now = self.clock.now();
        for state in all.values_mut() {
            state.deliveries.retain(|_, entry| entry.fresh(now));
        }
        for (owner, state) in all.iter() {
            if let Some(entry) = state.deliveries.get(handle) {
                if owner != conversation
                    || entry.thread != thread
                    || entry.tool != tool
                    || entry.kind != kind
                {
                    return Err(WebError::InvalidInput);
                }
                if state.capability != entry.capability || state.cancellation.is_cancelled() {
                    return Ok(None);
                }
                return Ok(Some(entry.text.clone()));
            }
        }
        Ok(None)
    }
}
