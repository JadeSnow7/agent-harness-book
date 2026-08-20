//! The explicit bridge between Runtime types and the M1 protocol layer.
//!
//! This module is the only place allowed to know both sides; `protocol` must
//! never learn about Runtime types.

use crate::openai_responses;
use crate::protocol::{Message, ProtocolError, ToolDefinition, ToolResultBlock, ToolUseBlock};
use crate::tool_types::{ToolCall, ToolResult, ToolSpec, ToolStatus};

/// Converts a Runtime spec into the provider-neutral tool declaration the model sees.
pub fn spec_to_tool_definition(spec: &ToolSpec) -> Result<ToolDefinition, ProtocolError> {
    ToolDefinition::try_new(
        spec.name.clone(),
        spec.description.clone(),
        spec.input_schema.clone(),
        spec.strict,
    )
}

/// Converts a model-proposed candidate into a Runtime call, preserving `call_id`.
pub fn tool_use_to_call(block: &ToolUseBlock) -> ToolCall {
    ToolCall {
        call_id: block.id.clone(),
        name: block.name.clone(),
        arguments: block.input.clone(),
    }
}

/// Encodes a structured result as the tool observation block for the next request.
pub fn result_to_tool_result_block(result: &ToolResult) -> ToolResultBlock {
    ToolResultBlock {
        tool_use_id: result.call_id.clone(),
        content: result.as_text(),
        is_error: result.status == ToolStatus::Failed,
    }
}

/// Builds the role=tool message carrying one tool observation.
pub fn result_to_message(result: &ToolResult) -> Result<Message, ProtocolError> {
    openai_responses::tool_result_message(
        result.call_id.clone(),
        result.as_text(),
        result.status == ToolStatus::Failed,
    )
}
