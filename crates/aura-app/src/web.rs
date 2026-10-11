//! Host-owned web authority. Tool arguments never establish a conversation/turn.
use crate::Host;
use aura_mcp::{BoxFut, CallContext, ToolOutput};
use aura_web::{WebContext, WebError, WebService};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock, Weak},
};

pub trait WebAccess: Send + Sync {
    fn call_web<'a>(
        &'a self,
        tool: &'a str,
        args: Value,
        context: CallContext,
    ) -> BoxFut<'a, ToolOutput>;
}
pub type WebSlot = Arc<OnceLock<Weak<dyn WebAccess>>>;

#[derive(Clone)]
pub(crate) struct ActiveTurn {
    pub uuid: String,
    pub turn_id: String,
    pub context: WebContext,
    restoration_error: Option<WebError>,
}
#[derive(Default)]
struct TurnRegistry {
    active: HashMap<String, ActiveTurn>,
    conversations: HashMap<String, String>,
    shutdown: bool,
}
pub(crate) struct WebTurns {
    pub service: Arc<WebService>,
    registry: Mutex<TurnRegistry>,
}
impl WebTurns {
    pub fn new(service: Arc<WebService>) -> Self {
        Self {
            service,
            registry: Mutex::new(TurnRegistry::default()),
        }
    }
    pub fn started(&self, thread_id: &str, uuid: &str, turn_id: &str) {
        if thread_id.is_empty() || uuid.is_empty() || turn_id.is_empty() {
            return;
        }
        let mut registry = self.registry.lock().unwrap();
        if registry.shutdown {
            return;
        }
        let context = self.service.begin_turn(uuid, turn_id);
        registry.conversations.insert(thread_id.into(), uuid.into());
        registry.active.insert(
            thread_id.into(),
            ActiveTurn {
                uuid: uuid.into(),
                turn_id: turn_id.into(),
                context,
                restoration_error: None,
            },
        );
    }
    pub fn set_enabled(&self, enabled: bool) {
        let mut registry = self.registry.lock().unwrap();
        if self.service.is_enabled() == enabled {
            return;
        }
        self.service.set_enabled(enabled);
        if enabled {
            for turn in registry.active.values_mut() {
                turn.context = self.service.begin_turn(&turn.uuid, &turn.turn_id);
            }
        }
    }
    pub fn restore(
        &self,
        thread_id: &str,
        last_id: usize,
        sources: Vec<aura_web::WebSource>,
    ) -> bool {
        let mut registry = self.registry.lock().unwrap();
        if let Some(turn) = registry.active.get_mut(thread_id) {
            match self
                .service
                .restore_sources(&turn.context, last_id, sources)
            {
                Ok(()) => return true,
                Err(error) => turn.restoration_error = Some(error),
            }
        }
        false
    }
    pub fn restore_failed(&self, thread_id: &str) {
        if let Some(turn) = self.registry.lock().unwrap().active.get_mut(thread_id) {
            turn.restoration_error = Some(WebError::Unavailable);
        }
    }
    pub fn finished(&self, thread_id: &str, turn_id: &str) {
        let mut registry = self.registry.lock().unwrap();
        let active = &mut registry.active;
        if active
            .get(thread_id)
            .is_some_and(|turn| turn.turn_id == turn_id)
            && let Some(turn) = active.remove(thread_id)
        {
            self.service.cancel_turn(&turn.context);
        }
    }
    pub fn interrupt(&self, thread_id: &str) {
        if let Some(turn) = self.registry.lock().unwrap().active.remove(thread_id) {
            self.service.cancel_turn(&turn.context);
        }
    }
    pub fn close(&self, thread_id: &str, uuid: &str) {
        let mut registry = self.registry.lock().unwrap();
        registry.active.remove(thread_id);
        registry.conversations.remove(thread_id);
        self.service.cancel_conversation(uuid);
    }
    pub fn cancel_active(&self) {
        let mut registry = self.registry.lock().unwrap();
        for (_, turn) in registry.active.drain() {
            self.service.cancel_turn(&turn.context);
        }
    }
    pub fn shutdown(&self) {
        let mut registry = self.registry.lock().unwrap();
        registry.shutdown = true;
        registry.active.clear();
        for (_, uuid) in registry.conversations.drain() {
            self.service.cancel_conversation(&uuid);
        }
    }
    fn resolve(&self, context: &CallContext) -> Result<(String, ActiveTurn), WebError> {
        let registry = self.registry.lock().unwrap();
        if !self.service.is_enabled() {
            return Err(WebError::WebDisabled);
        }
        let active = &registry.active;
        let turn = if context.conversation.is_empty() {
            if active.len() != 1 {
                return Err(WebError::InvalidInput);
            }
            active.iter().next()
        } else {
            active
                .iter()
                .find(|(_, turn)| turn.uuid == context.conversation)
        };
        let (thread_id, turn) = turn.ok_or(WebError::InvalidInput)?;
        if let Some(error) = turn.restoration_error {
            return Err(error);
        }
        Ok((thread_id.clone(), turn.clone()))
    }
    pub fn cited_sources(
        &self,
        thread_id: &str,
        text: &str,
    ) -> Option<(String, Vec<aura_web::WebSource>)> {
        let registry = self.registry.lock().unwrap();
        let turn = registry.active.get(thread_id)?;
        let mut sources = Vec::new();
        for segment in text.split("[[aura-source:").skip(1) {
            let Some((id, _)) = segment.split_once("]]") else {
                continue;
            };
            if !sources
                .iter()
                .any(|source: &aura_web::WebSource| source.source_id == id)
                && let Some(source) = self.service.citation_source(&turn.context, id)
            {
                sources.push(source);
            }
        }
        Some((turn.turn_id.clone(), sources))
    }
    fn publish_sources<'a>(
        &self,
        context: &WebContext,
        source_ids: impl IntoIterator<Item = &'a str>,
        mut emit: impl FnMut(aura_web::WebSource) -> Result<(), WebError>,
    ) -> Result<(), WebError> {
        let registry = self.registry.lock().unwrap();
        if !self.service.is_enabled()
            || !registry
                .active
                .values()
                .any(|turn| &turn.context == context)
        {
            return Err(WebError::Cancelled);
        }
        for id in source_ids {
            if let Some(source) = self.service.source(context, id) {
                emit(source)?;
            }
        }
        Ok(())
    }
}
impl aura_gateway::tool_results::ToolResultResolver for WebTurns {
    fn begin_response(
        &self,
        thread: &str,
    ) -> Option<Arc<dyn aura_gateway::tool_results::ToolCallProtector>> {
        let registry = self.registry.lock().unwrap();
        let turn = registry.active.get(thread)?;
        Some(Arc::new(PrivateCallScope {
            service: self.service.clone(),
            context: turn.context.clone(),
            thread: thread.into(),
            private_reasoning: self.service.has_private_context(&turn.context),
        }))
    }

    fn resolve_reasoning(
        &self,
        thread: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, aura_gateway::errors::UpstreamError> {
        let registry = self.registry.lock().unwrap();
        let uuid = registry.conversations.get(thread).ok_or_else(|| {
            aura_gateway::errors::UpstreamError::new(
                403,
                "web_result_scope_invalid",
                "Unknown reasoning scope",
            )
        })?;
        self.service
            .resolve_private_reasoning(uuid, thread, handle)
            .map_err(|_| {
                aura_gateway::errors::UpstreamError::new(
                    403,
                    "web_result_scope_invalid",
                    "Invalid reasoning scope",
                )
            })
    }

    fn resolve_arguments(
        &self,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, aura_gateway::errors::UpstreamError> {
        let registry = self.registry.lock().unwrap();
        let uuid = registry.conversations.get(thread).ok_or_else(|| {
            aura_gateway::errors::UpstreamError::new(
                403,
                "web_result_scope_invalid",
                "Unknown web input scope",
            )
        })?;
        self.service
            .resolve_tool_arguments(uuid, thread, tool, handle)
            .map_err(|_| {
                aura_gateway::errors::UpstreamError::new(
                    403,
                    "web_result_scope_invalid",
                    "Invalid web input scope",
                )
            })
    }

    fn resolve(
        &self,
        thread: &str,
        tool: &str,
        handle: &str,
    ) -> Result<Option<Arc<String>>, aura_gateway::errors::UpstreamError> {
        let registry = self.registry.lock().unwrap();
        let uuid = registry.conversations.get(thread).ok_or_else(|| {
            aura_gateway::errors::UpstreamError::new(
                403,
                "web_result_scope_invalid",
                "Unknown web result scope",
            )
        })?;
        self.service
            .resolve_tool_result(uuid, thread, tool, handle)
            .map_err(|_| {
                aura_gateway::errors::UpstreamError::new(
                    403,
                    "web_result_scope_invalid",
                    "Invalid web result scope",
                )
            })
    }
}
struct PrivateCallScope {
    service: Arc<WebService>,
    context: WebContext,
    thread: String,
    private_reasoning: bool,
}
impl aura_gateway::tool_results::ToolCallProtector for PrivateCallScope {
    fn protects_reasoning(&self) -> bool {
        self.private_reasoning
    }
    fn protect_reasoning(
        &self,
        item: &Value,
    ) -> Result<Value, aura_gateway::errors::UpstreamError> {
        let handle = self
            .service
            .retain_private_reasoning(&self.context, &self.thread, item.to_string())
            .map_err(|_| {
                aura_gateway::errors::UpstreamError::new(
                    409,
                    "web_reasoning_unavailable",
                    "Reasoning unavailable",
                )
            })?;
        Ok(aura_gateway::tool_results::reasoning_envelope(
            item, &handle,
        ))
    }
    fn protect(
        &self,
        tool: &str,
        arguments: &str,
    ) -> Result<String, aura_gateway::errors::UpstreamError> {
        let handle = self
            .service
            .retain_tool_arguments(&self.context, &self.thread, tool, arguments.into())
            .map_err(|_| {
                aura_gateway::errors::UpstreamError::new(
                    409,
                    "web_input_unavailable",
                    "Web tool input is no longer available",
                )
            })?;
        Ok(aura_gateway::tool_results::argument_envelope(tool, &handle))
    }
}
fn error(error: WebError) -> ToolOutput {
    let code = serde_json::to_value(error).unwrap();
    ToolOutput::error(code.as_str().unwrap(), &error.to_string())
}
impl WebAccess for Host {
    fn call_web<'a>(
        &'a self,
        tool: &'a str,
        args: Value,
        context: CallContext,
    ) -> BoxFut<'a, ToolOutput> {
        Box::pin(async move {
            let (thread_id, turn) = match self.web.resolve(&context) {
                Ok(turn) => turn,
                Err(err) => return error(err),
            };
            let context = &turn.context;
            let args =
                if let Some(handle) = aura_gateway::tool_results::argument_handle(tool, &args) {
                    match self
                        .web
                        .service
                        .resolve_tool_arguments(&turn.uuid, &thread_id, tool, handle)
                    {
                        Ok(Some(original)) => match serde_json::from_str::<Value>(&original) {
                            Ok(args) => args,
                            Err(_) => return error(WebError::InvalidInput),
                        },
                        Ok(None) => return error(WebError::Unavailable),
                        Err(err) => return error(err),
                    }
                } else {
                    args
                };
            let emit = |source| {
                if self.observe_web_source_id(&thread_id, &source).is_err() {
                    return Err(WebError::Unavailable);
                }
                self.push_event(crate::events::HostEvent::WebSource {
                    thread_id: thread_id.clone(),
                    turn_id: turn.turn_id.clone(),
                    source,
                });
                Ok(())
            };
            match tool {
                aura_mcp::tools::WEB_FETCH => {
                    let input = match serde_json::from_value::<aura_web::FetchRequest>(args) {
                        Ok(input) => input,
                        Err(_) => return error(WebError::InvalidInput),
                    };
                    let result = self.web.service.fetch(context, input).await;
                    if let Ok(page) = &result
                        && let Err(err) =
                            self.web
                                .publish_sources(context, [page.source_id.as_str()], emit)
                    {
                        return error(err);
                    }
                    output(&self.web.service, context, &thread_id, tool, result)
                }
                aura_mcp::tools::WEB_SEARCH => {
                    let input = match serde_json::from_value::<aura_web::SearchRequest>(args) {
                        Ok(input) => input,
                        Err(_) => return error(WebError::InvalidInput),
                    };
                    let result = self.web.service.search(context, input).await;
                    if let Ok(search) = &result
                        && let Err(err) = self.web.publish_sources(
                            context,
                            search
                                .results
                                .iter()
                                .map(|source| source.source_id.as_str()),
                            emit,
                        )
                    {
                        return error(err);
                    }
                    output(&self.web.service, context, &thread_id, tool, result)
                }
                _ => error(WebError::InvalidInput),
            }
        })
    }
}
fn output<T: serde::Serialize>(
    service: &WebService,
    context: &WebContext,
    thread: &str,
    tool: &str,
    result: Result<T, WebError>,
) -> ToolOutput {
    match result {
        Ok(output) => match service.retain_tool_result(
            context,
            thread,
            tool,
            serde_json::to_string(&output).unwrap(),
        ) {
            Ok(handle) => ToolOutput::text(aura_gateway::tool_results::envelope(&handle)),
            Err(err) => error(err),
        },
        Err(err) => error(err),
    }
}
