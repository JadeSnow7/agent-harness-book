use crate::{
    Config, ConfigError,
    transport::{Transport, TransportError},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fmt};

/// 本章默认使用的示例 prompt；只是教学案例，不是调用方必须使用的固定输入。
pub const DEFAULT_INPUT: &str = "请用一句话解释：为什么单次模型调用还不是 Agent？";

/// 构造当前 Provider 的最小请求 JSON；纯函数，不接触网络，因此可以单独测试请求形状。
pub fn build_request(config: &Config, input: &str) -> Value {
    json!({ "model": config.model, "input": input })
}

/// 从 Responses API 的 output 中提取所有非空 output_text 并按顺序拼接。
/// output 缺失、不是数组，或数组中没有任何非空 output_text，统一归类为 MissingOutputText，
/// 这与 examples/rust/m0-model-call 的既有行为保持一致，不引入新的错误细分。
fn extract_output_text(response: &Value) -> Result<String, AppError> {
    let text = response
        .get("output")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("content").and_then(Value::as_array))
        .flatten()
        .filter(|content| content.get("type").and_then(Value::as_str) == Some("output_text"))
        .filter_map(|content| content.get("text").and_then(Value::as_str))
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if text.is_empty() {
        Err(AppError::MissingOutputText)
    } else {
        Ok(text)
    }
}

/// 检查 HTTP 状态、解析 JSON，并提取最终文本。
///
/// 失败时错误只保留状态码或错误类别，不回显响应正文——Provider 或中间代理返回的正文可能
/// 意外反射请求中的敏感内容，把它塞进错误信息会把这个风险转嫁给日志和终端历史。
pub fn parse_http_response(status: u16, body: &str) -> Result<String, AppError> {
    if !(200..300).contains(&status) {
        return Err(AppError::ApiStatus(status));
    }
    let response: Value = serde_json::from_str(body).map_err(|_| AppError::InvalidJson)?;
    extract_output_text(&response)
}

/// 完成一次模型调用：构造请求、通过 Transport 发送、解析响应。
///
/// `Transport` 是泛型参数而不是 trait object：生产 `main.rs` 和测试都在编译期就知道具体
/// 类型（`ReqwestTransport` 或测试里的 Fake），单态化可以避免为唯一已知实现支付动态分发
/// 和堆分配的代价；本章也没有"运行时按需切换多个 Transport 实现"的需求。
pub fn chat_once<T: Transport>(
    config: &Config,
    input: &str,
    transport: &T,
) -> Result<String, AppError> {
    let request = build_request(config, input);
    let mut headers = BTreeMap::new();
    headers.insert("authorization".into(), format!("Bearer {}", config.api_key));
    headers.insert("content-type".into(), "application/json".into());
    let response = transport
        .send(&config.endpoint(), &headers, &request, config.timeout_s)
        .map_err(AppError::Transport)?;
    parse_http_response(response.status, &response.body)
}

/// 应用层错误分类；`Display` 绝不包含 API Key、Authorization Header 或 Provider 响应正文。
#[derive(Debug)]
pub enum AppError {
    Config(ConfigError),
    Transport(TransportError),
    ApiStatus(u16),
    InvalidJson,
    MissingOutputText,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(e) => e.fmt(f),
            Self::Transport(e) => e.fmt(f),
            Self::ApiStatus(status) => write!(f, "API request failed with HTTP {status}"),
            Self::InvalidJson => f.write_str("API returned invalid JSON"),
            Self::MissingOutputText => f.write_str("API response contained no output_text"),
        }
    }
}

impl std::error::Error for AppError {}
