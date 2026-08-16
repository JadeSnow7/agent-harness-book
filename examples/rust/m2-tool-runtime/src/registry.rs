//! 工具注册表。

use crate::tool_types::{Tool, ToolCall, ToolError, ToolResult, ToolSpec, ToolStatus};
use std::collections::BTreeMap;

/// 按名称保存异构工具的注册表。
#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// 注册工具；同名注册会替换旧实现。
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.spec().name.clone();
        self.tools.insert(name, Box::new(tool));
    }

    /// 按稳定名称顺序返回全部规格。
    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|tool| tool.spec()).collect()
    }

    /// 校验并执行一次调用，把所有失败收敛成 ToolResult。
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

    /// 用普通 JSON 字段构造 ToolCall 并执行。
    pub fn execute_values(
        &mut self,
        call_id: impl Into<String>,
        name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> ToolResult {
        self.execute(ToolCall {
            call_id: call_id.into(),
            name: name.into(),
            arguments,
        })
    }
}

/// 要求工具参数顶层是 JSON 对象。
pub fn require_object(
    arguments: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, ToolError> {
    arguments
        .as_object()
        .ok_or_else(|| ToolError::new("arguments must be an object"))
}

/// 从参数对象读取一个非空字符串字段。
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
