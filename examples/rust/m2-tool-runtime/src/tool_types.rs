//! Tool Runtime 最小类型。

use serde_json::Value;
use std::fmt;

/// 一次工具调用的结构化状态。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolStatus {
    Succeeded,
    Failed,
}

impl ToolStatus {
    /// 返回稳定、可序列化的状态字符串。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// 模型和文档可见的工具规格。
#[derive(Clone, Debug, PartialEq)]
pub struct ToolSpec {
    /// 注册和调用使用的唯一名字。
    pub name: String,
    /// 给模型阅读的能力说明。
    pub description: String,
    /// 描述参数对象的 JSON Schema。
    pub input_schema: Value,
    /// 是否请求 Provider 使用严格 JSON Schema；Runtime 仍会再次校验。
    pub strict: bool,
}

/// Runtime 即将执行的一次调用。
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    /// Provider 分配并需原样回传的调用标识。
    pub call_id: String,
    /// Registry 中的工具名。
    pub name: String,
    /// 已解码但仍需 Runtime 校验的参数。
    pub arguments: Value,
}

/// 工具成功或失败的统一数据形状。
#[derive(Clone, Debug, PartialEq)]
pub struct ToolResult {
    /// 与 ToolCall 相同的调用标识。
    pub call_id: String,
    /// 实际查找的工具名。
    pub name: String,
    /// 成功或失败状态。
    pub status: ToolStatus,
    /// 成功时的可序列化输出。
    pub output: Option<Value>,
    /// 失败时的安全错误信息。
    pub error: Option<String>,
}

impl ToolResult {
    /// 是否成功。
    pub fn succeeded(&self) -> bool {
        self.status == ToolStatus::Succeeded
    }

    /// 把结果编码成工具观察文本。
    pub fn as_text(&self) -> String {
        if self.succeeded() {
            match &self.output {
                None => String::new(),
                Some(Value::String(text)) => text.clone(),
                Some(value) => value.to_string(),
            }
        } else {
            self.error.clone().unwrap_or_else(|| "tool failed".into())
        }
    }
}

/// 工具内部和 Workspace 的教学级错误。
#[derive(Debug)]
pub enum ToolError {
    Message(String),
}

impl ToolError {
    /// 从可展示文本构造错误。
    pub fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ToolError {}

/// 七工具共同实现的最小接口。
pub trait Tool {
    /// 返回模型可见规格。
    fn spec(&self) -> ToolSpec;
    /// 在产生副作用前校验参数。
    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError>;
    /// 执行工具；错误由 Registry 收敛为 ToolResult。
    fn execute(&mut self, arguments: &Value) -> Result<Value, ToolError>;
}
