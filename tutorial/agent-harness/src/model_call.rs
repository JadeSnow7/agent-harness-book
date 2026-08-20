use crate::{
    Config, ConfigError, openai_responses,
    protocol::{Message, ModelRequest, ModelResponse, ProtocolError, Role},
    transport::{Transport, TransportError},
};
use std::{collections::BTreeMap, fmt};
pub const DEFAULT_INPUT: &str = "请用一句话解释：为什么统一协议有助于后续接入工具？";
/// Builds the smallest unified request for the CLI example.
pub fn build_model_request(config: &Config, input: &str) -> Result<ModelRequest, ProtocolError> {
    ModelRequest::try_new(
        config.model.clone(),
        vec![Message::text(Role::User, input)?],
        None,
        Vec::new(),
    )
}
/// Encodes a unified request, sends it through low-level transport, and decodes the response.
pub fn complete<T: Transport>(
    request: &ModelRequest,
    config: &Config,
    transport: &T,
) -> Result<ModelResponse, AppError> {
    let payload = openai_responses::encode_request(request).map_err(AppError::Protocol)?;
    let mut headers = BTreeMap::new();
    headers.insert("authorization".into(), format!("Bearer {}", config.api_key));
    headers.insert("content-type".into(), "application/json".into());
    let response = transport
        .send(&config.endpoint(), &headers, &payload, config.timeout_s)
        .map_err(AppError::Transport)?;
    openai_responses::parse_http_response(response.status, &response.body)
        .map_err(AppError::Protocol)
}
/// Performs one request using the unified request builder.
pub fn chat_once<T: Transport>(
    config: &Config,
    input: &str,
    transport: &T,
) -> Result<ModelResponse, AppError> {
    let request = build_model_request(config, input).map_err(AppError::Protocol)?;
    complete(&request, config, transport)
}
/// Formats displayable text and tool candidates, rejecting an empty response.
pub fn format_response(response: &ModelResponse) -> Result<String, ProtocolError> {
    let text = response.text();
    let mut lines = Vec::new();
    if !text.is_empty() {
        lines.push(text);
    }
    for tool in response.tool_uses() {
        lines.push(format!("tool_use {} {} {}", tool.id, tool.name, tool.input));
    }
    if lines.is_empty() {
        return Err(ProtocolError::Decode(
            "response contained no displayable content".into(),
        ));
    }
    Ok(lines.join("\n"))
}
/// Application-facing error categories that never include secrets or response bodies.
#[derive(Debug)]
pub enum AppError {
    Config(ConfigError),
    Transport(TransportError),
    Protocol(ProtocolError),
}
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(e) => e.fmt(f),
            Self::Transport(e) => e.fmt(f),
            Self::Protocol(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for AppError {}
