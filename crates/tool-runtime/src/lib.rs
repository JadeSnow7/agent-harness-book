//! Deterministic, side-effect-free tool execution.

use agent_core::{
    AgentError, JsonValue, ToolCall, ToolExecutor, ToolResult, ToolResultStatus, ToolSpec,
};
use std::collections::BTreeMap;

pub trait Tool: Send {
    fn spec(&self) -> ToolSpec;

    fn validate_arguments(&self, arguments: &JsonValue) -> Result<(), String> {
        let _ = arguments;
        Ok(())
    }

    fn execute(&mut self, arguments: &JsonValue) -> Result<JsonValue, String>;
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools
            .insert(tool.spec().tool_id.clone(), Box::new(tool));
    }

    pub fn lookup(&self, tool_id: &str) -> Option<&dyn Tool> {
        self.tools.get(tool_id).map(Box::as_ref)
    }

    pub fn contains(&self, tool_id: &str) -> bool {
        self.tools.contains_key(tool_id)
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|tool| tool.spec()).collect()
    }
}

impl ToolExecutor for ToolRegistry {
    fn execute(&mut self, call: ToolCall) -> Result<ToolResult, AgentError> {
        let Some(tool) = self.tools.get_mut(&call.tool_id) else {
            return Ok(ToolResult {
                call_id: call.call_id,
                status: ToolResultStatus::Failed,
                output: None,
                error: Some(format!("unknown tool: {}", call.tool_id)),
            });
        };
        if let Err(error) = tool.validate_arguments(&call.arguments) {
            return Ok(ToolResult {
                call_id: call.call_id,
                status: ToolResultStatus::Failed,
                output: None,
                error: Some(error),
            });
        }
        match tool.execute(&call.arguments) {
            Ok(output) => Ok(ToolResult {
                call_id: call.call_id,
                status: ToolResultStatus::Succeeded,
                output: Some(output),
                error: None,
            }),
            Err(error) => Ok(ToolResult {
                call_id: call.call_id,
                status: ToolResultStatus::Failed,
                output: None,
                error: Some(error),
            }),
        }
    }
}

#[derive(Default)]
pub struct EchoTool;

impl Tool for EchoTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            tool_id: "echo".into(),
            description: "Returns the supplied value without side effects".into(),
            input_schema: JsonValue::object([
                ("type".into(), JsonValue::String("object".into())),
                (
                    "required".into(),
                    JsonValue::array([JsonValue::String("value".into())]),
                ),
            ]),
        }
    }

    fn validate_arguments(&self, arguments: &JsonValue) -> Result<(), String> {
        let JsonValue::Object(values) = arguments else {
            return Err("arguments_must_be_object".into());
        };
        let Some(JsonValue::Number(_)) = values.get("value") else {
            return Err("argument 'value' must be an integer".into());
        };
        Ok(())
    }

    fn execute(&mut self, arguments: &JsonValue) -> Result<JsonValue, String> {
        Ok(arguments.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn args(value: i64) -> JsonValue {
        let mut values = BTreeMap::new();
        values.insert("value".into(), JsonValue::from(value));
        JsonValue::Object(values)
    }

    #[test]
    fn echo_succeeds_and_invalid_args_are_data() {
        let mut registry = ToolRegistry::default();
        registry.register(EchoTool);
        let ok = registry
            .execute(ToolCall {
                call_id: "c1".into(),
                tool_id: "echo".into(),
                arguments: args(7),
            })
            .expect("result");
        assert_eq!(ok.status, ToolResultStatus::Succeeded);
        assert_eq!(ok.output, Some(args(7)));
        let bad = registry
            .execute(ToolCall {
                call_id: "c2".into(),
                tool_id: "echo".into(),
                arguments: JsonValue::Null,
            })
            .expect("structured failure");
        assert_eq!(bad.status, ToolResultStatus::Failed);
    }

    #[test]
    fn unknown_tool_is_structured_failure() {
        let mut registry = ToolRegistry::default();
        let result = registry
            .execute(ToolCall {
                call_id: "c1".into(),
                tool_id: "missing".into(),
                arguments: JsonValue::Null,
            })
            .expect("structured failure");
        assert_eq!(result.status, ToolResultStatus::Failed);
    }

    #[test]
    fn registration_and_lookup_are_available() {
        let mut registry = ToolRegistry::default();
        assert!(registry.is_empty());
        registry.register(EchoTool);
        assert_eq!(registry.len(), 1);
        assert_eq!(
            registry.lookup("echo").map(|tool| tool.spec().tool_id),
            Some("echo".into())
        );
    }

    #[test]
    fn tool_failures_remain_structured_results() {
        struct FailingTool;

        impl Tool for FailingTool {
            fn spec(&self) -> ToolSpec {
                ToolSpec {
                    tool_id: "failing".into(),
                    description: "fixture failure".into(),
                    input_schema: JsonValue::Null,
                }
            }

            fn execute(&mut self, _: &JsonValue) -> Result<JsonValue, String> {
                Err("fixture_failure".into())
            }
        }

        let mut registry = ToolRegistry::default();
        registry.register(FailingTool);
        let result = registry
            .execute(ToolCall {
                call_id: "failure".into(),
                tool_id: "failing".into(),
                arguments: JsonValue::Null,
            })
            .expect("tool failure is data");
        assert_eq!(result.status, ToolResultStatus::Failed);
        assert_eq!(result.error.as_deref(), Some("fixture_failure"));
    }
}
