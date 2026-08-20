//! Tool Runtime minimum types.

use serde_json::Value;
use std::fmt;

/// The structured outcome of one tool call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolStatus {
    Succeeded,
    Failed,
}

impl ToolStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// A model- and doc-visible tool spec.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub strict: bool,
}

/// One call the Runtime is about to execute.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
}

/// The unified shape of a tool success or failure.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolResult {
    pub call_id: String,
    pub name: String,
    pub status: ToolStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn succeeded(&self) -> bool {
        self.status == ToolStatus::Succeeded
    }

    /// Encodes the result into the text an observation carries back to the model.
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

/// Teaching-level error shared by tools and the Workspace boundary.
#[derive(Debug)]
pub enum ToolError {
    Message(String),
}

impl ToolError {
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

/// The minimum interface a registered tool implements.
pub trait Tool {
    fn spec(&self) -> ToolSpec;
    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError>;
    fn execute(&mut self, arguments: &Value) -> Result<Value, ToolError>;
}
