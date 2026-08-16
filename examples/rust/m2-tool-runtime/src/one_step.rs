//! Fake Transport 教学用的固定一步闭环；它不是 Agent Loop。
//!
//! M1 不保留 reasoning 等未知输出项，因此这里不用于生产级真实多轮调用。

use crate::bridge::{result_to_message, spec_to_tool_definition, tool_use_to_call};
use crate::{ToolError, ToolRegistry, ToolResult};
use m1_unified_protocol::{Config, HttpTransport, ModelRequest, ModelResponse, complete};
use std::fmt;

/// 固定闭环形状不满足约束时的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneStepError(pub String);

impl fmt::Display for OneStepError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for OneStepError {}

impl From<ToolError> for OneStepError {
    fn from(error: ToolError) -> Self {
        Self(error.to_string())
    }
}

/// 保留闭环关键证据，供测试检查 call_id 与第二次请求。
#[derive(Debug)]
pub struct OneStepResult {
    pub first_response: ModelResponse,
    pub tool_result: ToolResult,
    pub followup_request: ModelRequest,
    pub final_response: ModelResponse,
}

/// 从 Registry 生成有序模型工具声明，并复制请求其余字段。
pub fn request_with_registry_tools(
    request: &ModelRequest,
    registry: &ToolRegistry,
) -> Result<ModelRequest, OneStepError> {
    let tools = registry
        .specs()
        .iter()
        .map(spec_to_tool_definition)
        .collect::<Result<Vec<_>, _>>()?;
    ModelRequest::try_new_with_tools(
        request.model.clone(),
        request.messages.clone(),
        request.system.clone(),
        tools,
    )
    .map_err(|error| OneStepError(error.to_string()))
}

/// 第一次恰好一个工具候选，执行后第二次必须给出最终文本。
pub fn run_one_tool_step<T: HttpTransport>(
    request: &ModelRequest,
    config: &Config,
    transport: &mut T,
    registry: &mut ToolRegistry,
) -> Result<OneStepResult, OneStepError> {
    let first_request = request_with_registry_tools(request, registry)?;
    let first = complete(&first_request, config, transport)
        .map_err(|error| OneStepError(error.to_string()))?;
    let uses = first.tool_uses();
    if uses.len() != 1 {
        return Err(OneStepError(format!(
            "first model call must request exactly one tool; got {}",
            uses.len()
        )));
    }
    let call = tool_use_to_call(uses[0]);
    let result = registry.execute(call);
    let mut messages = first_request.messages.clone();
    messages.push(first.message.clone());
    messages.push(result_to_message(&result)?);
    let followup = ModelRequest::try_new_with_tools(
        first_request.model.clone(),
        messages,
        first_request.system.clone(),
        first_request.tools.clone(),
    )
    .map_err(|error| OneStepError(error.to_string()))?;
    let final_response =
        complete(&followup, config, transport).map_err(|error| OneStepError(error.to_string()))?;
    if !final_response.tool_uses().is_empty() {
        return Err(OneStepError(
            "second model call requested another tool; Agent Loop is M3".into(),
        ));
    }
    if final_response.text().is_empty() {
        return Err(OneStepError(
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
