//! M3 test tools: deterministic success, deterministic failure, and a tool
//! that flips a [`CancelToken`](crate::types::CancelToken) on execute.
//!
//! These implement `tool_runtime::Tool` directly, so `tool_runtime::ToolRegistry`'s
//! failure-collapsing behavior (unknown tool / bad arguments / execution
//! error all become a structured `ToolResult`) is reused as-is, the same
//! way the Python side reuses `m2-tool-runtime`'s `ToolRegistry`.

use std::collections::BTreeMap;

use agent_core::{JsonValue, ToolSpec};
use tool_runtime::Tool;

use crate::types::CancelToken;

pub struct EchoTool;

impl Tool for EchoTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            tool_id: "echo".into(),
            description: "Echo back the given value.".into(),
            input_schema: JsonValue::Object(BTreeMap::new()),
        }
    }

    fn execute(&mut self, arguments: &JsonValue) -> Result<JsonValue, String> {
        match arguments {
            JsonValue::Object(map) => map
                .get("value")
                .cloned()
                .ok_or_else(|| "echo requires 'value'".to_owned()),
            _ => Err("echo requires an object with 'value'".to_owned()),
        }
    }
}

/// Always fails; used to verify tool failure is a structured observation,
/// not an uncaught exception swallowing the whole run.
pub struct FailingTool;

impl Tool for FailingTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            tool_id: "fail".into(),
            description: "Always fails deterministically.".into(),
            input_schema: JsonValue::Object(BTreeMap::new()),
        }
    }

    fn execute(&mut self, _arguments: &JsonValue) -> Result<JsonValue, String> {
        Err("deterministic failure for testing".to_owned())
    }
}

/// Flips the given [`CancelToken`] on execute; used to test a run being
/// cancelled mid-flight.
pub struct CancellingTool {
    token: CancelToken,
}

impl CancellingTool {
    pub fn new(token: CancelToken) -> Self {
        Self { token }
    }
}

impl Tool for CancellingTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            tool_id: "cancel_trigger".into(),
            description: "Flips the cancel token on execute.".into(),
            input_schema: JsonValue::Object(BTreeMap::new()),
        }
    }

    fn execute(&mut self, _arguments: &JsonValue) -> Result<JsonValue, String> {
        self.token.cancel();
        Ok(JsonValue::String("cancel requested".into()))
    }
}
