//! Tool registry: lookup, argument validation, execution, and failure collapse.

use crate::tool_types::{Tool, ToolCall, ToolError, ToolResult, ToolSpec, ToolStatus};
use std::collections::BTreeMap;

/// Holds heterogeneous tools by name.
#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Registers a tool; a second registration under the same name replaces the first.
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.spec().name.clone();
        self.tools.insert(name, Box::new(tool));
    }

    /// Returns all specs in stable name order.
    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|tool| tool.spec()).collect()
    }

    /// Validates and executes one call, collapsing every failure into a `ToolResult`.
    pub fn execute(&mut self, call: ToolCall) -> ToolResult {
        let Some(tool) = self.tools.get_mut(&call.name) else {
            return ToolResult {
                call_id: call.call_id,
                name: call.name.clone(),
                status: ToolStatus::Failed,
                output: None,
                error: Some(format!("unknown tool: {}", call.name)),
            };
        };

        if let Err(error) = tool.validate_arguments(&call.arguments) {
            return ToolResult {
                call_id: call.call_id,
                name: call.name,
                status: ToolStatus::Failed,
                output: None,
                error: Some(format!("invalid arguments: {error}")),
            };
        }

        match tool.execute(&call.arguments) {
            Ok(output) => ToolResult {
                call_id: call.call_id,
                name: call.name,
                status: ToolStatus::Succeeded,
                output: Some(output),
                error: None,
            },
            Err(error) => ToolResult {
                call_id: call.call_id,
                name: call.name,
                status: ToolStatus::Failed,
                output: None,
                error: Some(error.to_string()),
            },
        }
    }
}

/// Requires the top-level tool argument value to be a JSON object.
pub fn require_object(
    arguments: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, ToolError> {
    arguments
        .as_object()
        .ok_or_else(|| ToolError::new("arguments must be an object"))
}

/// Reads one non-empty string field from an argument object.
pub fn require_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<&'a str, ToolError> {
    object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ToolError::new(format!("{key} must be a non-empty string")))
}
