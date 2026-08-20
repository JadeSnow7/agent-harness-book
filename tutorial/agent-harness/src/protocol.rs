use serde_json::Value;
use std::{collections::BTreeSet, fmt};

/// Safe, provider-neutral failures at encoding, decoding, or API boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    Encode(String),
    Decode(String),
    Api(String),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(s) | Self::Decode(s) | Self::Api(s) => f.write_str(s),
        }
    }
}
impl std::error::Error for ProtocolError {}

#[derive(Clone, Debug, PartialEq, Eq)]
/// The four roles shared by model messages and tool observations.
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}
impl Role {
    /// Returns the provider-neutral role spelling used at adapter boundaries.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// Plain model text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextBlock {
    pub text: String,
}
/// A model-proposed tool action; this chapter never executes it.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolUseBlock {
    pub id: String,
    pub name: String,
    pub input: Value,
}
/// A tool observation associated with one original tool-use id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolResultBlock {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}
/// The smallest content vocabulary shared by requests and responses.
#[derive(Clone, Debug, PartialEq)]
pub enum ContentBlock {
    Text(TextBlock),
    ToolUse(ToolUseBlock),
    ToolResult(ToolResultBlock),
}

/// A role plus one or more ordered content blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}
impl Message {
    /// Creates a message and rejects an empty content list.
    pub fn try_new(role: Role, content: Vec<ContentBlock>) -> Result<Self, ProtocolError> {
        if content.is_empty() {
            return Err(ProtocolError::Encode(
                "message content must not be empty".into(),
            ));
        }
        Ok(Self { role, content })
    }
    /// Convenience constructor for one text block.
    pub fn text(role: Role, text: impl Into<String>) -> Result<Self, ProtocolError> {
        Self::try_new(
            role,
            vec![ContentBlock::Text(TextBlock { text: text.into() })],
        )
    }
}

/// A provider-neutral function declaration.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub strict: bool,
}
impl ToolDefinition {
    /// Creates and validates a tool name, description, and object schema.
    pub fn try_new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        strict: bool,
    ) -> Result<Self, ProtocolError> {
        let name = name.into();
        if name.is_empty()
            || name.len() > 64
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(ProtocolError::Encode(
                "tool name must be 1-64 letters, digits, underscores, or hyphens".into(),
            ));
        }
        let description = description.into();
        if description.trim().is_empty() {
            return Err(ProtocolError::Encode(
                "tool description must not be empty".into(),
            ));
        }
        if input_schema.get("type").and_then(Value::as_str) != Some("object") {
            return Err(ProtocolError::Encode(
                "tool input_schema.type must be 'object'".into(),
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

/// A complete provider-neutral model request.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub tools: Vec<ToolDefinition>,
}
impl ModelRequest {
    /// Creates a request and rejects empty or duplicate definitions.
    pub fn try_new(
        model: impl Into<String>,
        messages: Vec<Message>,
        system: Option<String>,
        tools: Vec<ToolDefinition>,
    ) -> Result<Self, ProtocolError> {
        let model = model.into();
        if model.trim().is_empty() {
            return Err(ProtocolError::Encode("model must not be empty".into()));
        }
        if messages.is_empty() {
            return Err(ProtocolError::Encode("messages must not be empty".into()));
        }
        let mut names = BTreeSet::new();
        for tool in &tools {
            if !names.insert(&tool.name) {
                return Err(ProtocolError::Encode(
                    "tool names must be unique within a request".into(),
                ));
            }
        }
        Ok(Self {
            model,
            messages,
            system,
            tools,
        })
    }
}

/// A decoded model response containing text and/or tool candidates.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelResponse {
    pub id: Option<String>,
    pub model: Option<String>,
    pub message: Message,
    pub status: Option<String>,
}
impl ModelResponse {
    /// Joins non-empty text blocks in their original order.
    pub fn text(&self) -> String {
        self.message
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text(t) if !t.text.trim().is_empty() => Some(t.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    /// Returns all tool candidates without executing them.
    pub fn tool_uses(&self) -> Vec<&ToolUseBlock> {
        self.message
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::ToolUse(t) => Some(t),
                _ => None,
            })
            .collect()
    }
}
