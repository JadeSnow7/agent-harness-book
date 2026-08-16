//! M2 Runtime 与 M1 统一协议之间的显式桥接。

use crate::{ToolCall, ToolError, ToolResult, ToolSpec, ToolStatus};
use m1_unified_protocol::{
    ContentBlock, Message, Role, ToolDefinition, ToolResultBlock, ToolUseBlock,
};

/// 把 Runtime 规格转换成 provider-neutral 工具声明。
pub fn spec_to_tool_definition(spec: &ToolSpec) -> Result<ToolDefinition, ToolError> {
    ToolDefinition::try_new(
        spec.name.clone(),
        spec.description.clone(),
        spec.input_schema.clone(),
        spec.strict,
    )
    .map_err(|error| ToolError::new(error.to_string()))
}

/// 保留 M1 `call_id`，将模型候选动作转换为 Runtime 调用。
pub fn tool_use_to_call(block: &ToolUseBlock) -> ToolCall {
    ToolCall {
        call_id: block.id.clone(),
        name: block.name.clone(),
        arguments: block.input.clone(),
    }
}

/// 把结构化结果编码为下一次模型请求中的工具观察。
pub fn result_to_tool_result_block(result: &ToolResult) -> ToolResultBlock {
    ToolResultBlock {
        tool_use_id: result.call_id.clone(),
        content: result.as_text(),
        is_error: result.status == ToolStatus::Failed,
    }
}

/// 构造 role=tool 的统一消息。
pub fn result_to_message(result: &ToolResult) -> Result<Message, ToolError> {
    Message::try_new(
        Role::Tool,
        vec![ContentBlock::ToolResult(result_to_tool_result_block(
            result,
        ))],
    )
    .map_err(|error| ToolError::new(error.to_string()))
}
