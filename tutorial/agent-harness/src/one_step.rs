//! A fixed, two-call one-step closure for teaching purposes; this is not an Agent Loop.
//!
//! M1 drops unknown output items such as reasoning, so this module must not be
//! reused for a production-grade multi-turn Responses call; full context
//! retention or `previous_response_id` is left to M3.

use crate::Config;
use crate::bridge::{result_to_message, spec_to_tool_definition, tool_use_to_call};
use crate::model_call::{AppError, complete};
use crate::protocol::{ModelRequest, ProtocolError};
use crate::registry::ToolRegistry;
use crate::tool_types::ToolResult;
use crate::transport::Transport;
use std::fmt;

/// Raised when the fixed closure's shape does not hold.
#[derive(Debug)]
pub enum OneStepError {
    Protocol(ProtocolError),
    App(AppError),
    Shape(String),
}

impl fmt::Display for OneStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(e) => e.fmt(f),
            Self::App(e) => e.fmt(f),
            Self::Shape(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for OneStepError {}

impl From<ProtocolError> for OneStepError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

impl From<AppError> for OneStepError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}

/// Keeps the closure's key evidence for tests to assert `call_id` and ordering.
#[derive(Debug)]
pub struct OneStepResult {
    pub first_response: crate::protocol::ModelResponse,
    pub tool_result: ToolResult,
    pub followup_request: ModelRequest,
    pub final_response: crate::protocol::ModelResponse,
}

/// Generates ordered tool declarations from the registry and copies the rest of the request.
pub fn request_with_registry_tools(
    request: &ModelRequest,
    registry: &ToolRegistry,
) -> Result<ModelRequest, OneStepError> {
    let tools = registry
        .specs()
        .iter()
        .map(spec_to_tool_definition)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ModelRequest::try_new(
        request.model.clone(),
        request.messages.clone(),
        request.system.clone(),
        tools,
    )?)
}

/// Calls twice through a controlled transport, executing exactly one tool candidate in between.
///
/// A first call with zero or more than one tool candidate, or a second call
/// that requests a tool again, fails immediately; this function never retries
/// or loops, and does not define a budget or stop policy.
pub fn run_one_tool_step<T: Transport>(
    request: &ModelRequest,
    config: &Config,
    transport: &T,
    registry: &mut ToolRegistry,
) -> Result<OneStepResult, OneStepError> {
    let first_request = request_with_registry_tools(request, registry)?;
    let first = complete(&first_request, config, transport)?;
    let uses = first.tool_uses();
    if uses.len() != 1 {
        return Err(OneStepError::Shape(format!(
            "first model call must request exactly one tool; got {}",
            uses.len()
        )));
    }
    let call = tool_use_to_call(uses[0]);
    let result = registry.execute(call);

    let mut messages = first_request.messages.clone();
    messages.push(first.message.clone());
    messages.push(result_to_message(&result)?);
    let followup = ModelRequest::try_new(
        first_request.model.clone(),
        messages,
        first_request.system.clone(),
        first_request.tools.clone(),
    )?;
    let final_response = complete(&followup, config, transport)?;
    if !final_response.tool_uses().is_empty() {
        return Err(OneStepError::Shape(
            "second model call requested another tool; Agent Loop is M3".into(),
        ));
    }
    if final_response.text().is_empty() {
        return Err(OneStepError::Shape(
            "second model call returned no final text".into(),
        ));
    }
    Ok(OneStepResult {
        first_response: first,
        tool_result: result,
        followup_request: followup,
        final_response,
    })
}
