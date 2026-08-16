//! M1：Provider 无关的统一协议类型。
//!
//! 本模块只定义消息、内容块、模型请求/响应和可安全展示的错误类型。
//! 它不发送网络请求，也不执行工具。

use std::fmt;

/// 统一协议层可安全展示给调用者的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    Config(String),
    Transport(String),
    Api(String),
    Decode(String),
    Encode(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(detail) => formatter.write_str(detail),
            Self::Transport(detail) => formatter.write_str(detail),
            Self::Api(detail) => formatter.write_str(detail),
            Self::Decode(detail) => formatter.write_str(detail),
            Self::Encode(detail) => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for ProtocolError {}

/// 消息角色；Tool 表示工具观察，不是模型自身输出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// 普通文本内容块。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBlock {
    pub text: String,
}

/// 模型提出的工具调用候选；本章只解析，不执行。
#[derive(Clone, Debug, PartialEq)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// 工具执行后的观察；供后续轮次重新进入模型输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}

/// 统一内容块。
#[derive(Clone, Debug, PartialEq)]
pub enum ContentBlock {
    Text(TextBlock),
    ToolUse(ToolUseBlock),
    ToolResult(ToolResultBlock),
}

/// 告诉模型“有哪些函数可用”的 Provider 无关定义。
#[derive(Clone, Debug, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub strict: bool,
}

impl ToolDefinition {
    /// 构造并校验一个函数工具定义；Runtime 仍需执行前再次校验参数。
    pub fn try_new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        strict: bool,
    ) -> Result<Self, ProtocolError> {
        let name = name.into();
        if name.is_empty()
            || name.len() > 64
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(ProtocolError::Encode(
                "tool name must be 1-64 letters, digits, underscores, or hyphens".to_owned(),
            ));
        }
        let description = description.into();
        if description.trim().is_empty() {
            return Err(ProtocolError::Encode(
                "tool description must not be empty".to_owned(),
            ));
        }
        if !input_schema.is_object()
            || input_schema.get("type").and_then(|v| v.as_str()) != Some("object")
        {
            return Err(ProtocolError::Encode(
                "tool input_schema.type must be 'object'".to_owned(),
            ));
        }
        Ok(Self {
            name,
            description,
            input_schema,
            strict,
        })
    }
}

/// 一条带角色的统一消息。
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

impl Message {
    pub fn try_new(role: Role, content: Vec<ContentBlock>) -> Result<Self, ProtocolError> {
        if content.is_empty() {
            return Err(ProtocolError::Encode(
                "message content must not be empty".to_owned(),
            ));
        }
        Ok(Self { role, content })
    }

    pub fn text(role: Role, text: impl Into<String>) -> Result<Self, ProtocolError> {
        Self::try_new(
            role,
            vec![ContentBlock::Text(TextBlock { text: text.into() })],
        )
    }
}

/// 一次模型调用的统一请求。
#[derive(Clone, Debug, PartialEq)]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub tools: Vec<ToolDefinition>,
}

impl ModelRequest {
    pub fn try_new(
        model: impl Into<String>,
        messages: Vec<Message>,
        system: Option<String>,
    ) -> Result<Self, ProtocolError> {
        let model = model.into();
        if model.trim().is_empty() {
            return Err(ProtocolError::Encode("model must not be empty".to_owned()));
        }
        if messages.is_empty() {
            return Err(ProtocolError::Encode(
                "messages must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            model,
            messages,
            system,
            tools: Vec::new(),
        })
    }

    /// 构造带工具定义的请求，并拒绝同名工具造成的歧义。
    pub fn try_new_with_tools(
        model: impl Into<String>,
        messages: Vec<Message>,
        system: Option<String>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Self, ProtocolError> {
        let mut request = Self::try_new(model, messages, system)?;
        let mut names = std::collections::BTreeSet::new();
        for tool in &tools {
            if !names.insert(tool.name.as_str()) {
                return Err(ProtocolError::Encode(
                    "tool names must be unique within a request".to_owned(),
                ));
            }
        }
        request.tools = tools;
        Ok(request)
    }
}

/// 一次模型调用的统一响应。
#[derive(Clone, Debug, PartialEq)]
pub struct ModelResponse {
    pub id: Option<String>,
    pub model: Option<String>,
    pub message: Message,
    pub status: Option<String>,
}

impl ModelResponse {
    /// 按顺序合并所有非空文本块；没有文本时返回空字符串。
    pub fn text(&self) -> String {
        self.message
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text(text) if !text.text.trim().is_empty() => {
                    Some(text.text.as_str())
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 返回响应中的全部工具调用候选。
    pub fn tool_uses(&self) -> Vec<&ToolUseBlock> {
        self.message
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolUse(tool_use) => Some(tool_use),
                _ => None,
            })
            .collect()
    }
}
