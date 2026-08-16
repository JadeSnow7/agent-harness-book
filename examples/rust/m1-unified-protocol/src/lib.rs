//! M1 Unified Protocol：统一消息协议与 OpenAI Responses 适配器。
//!
//! 本 crate 不执行工具、不驱动 Agent 循环，只完成：
//! 统一类型定义、Provider JSON 编解码，以及一次可选的非流式调用编排。

pub mod openai_responses;
pub mod protocol;

pub use openai_responses::{
    decode_response, decode_response_json, encode_request, parse_http_response,
};
pub use protocol::{
    ContentBlock, Message, ModelRequest, ModelResponse, ProtocolError, Role, TextBlock,
    ToolDefinition, ToolResultBlock, ToolUseBlock,
};

use std::collections::HashMap;
use std::env;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT_S: f64 = 60.0;
pub const DEFAULT_PROMPT: &str = "请用一句话解释：为什么统一协议有助于后续接入工具？";

/// 一次模型调用所需的配置；API Key 只在内存中传递。
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub timeout_s: f64,
}

impl Config {
    pub fn endpoint(&self) -> String {
        format!("{}/responses", self.base_url.trim_end_matches('/'))
    }
}

/// 传输层返回的最小数据，不携带请求 Header。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: String,
}

/// 网络传输的最小接口，测试可注入 Fake Transport。
pub trait HttpTransport {
    fn post_json(
        &mut self,
        url: &str,
        headers: &HashMap<String, String>,
        payload: &serde_json::Value,
        timeout_s: f64,
    ) -> Result<HttpResponse, ProtocolError>;
}

/// 从环境变量映射读取配置，不执行网络请求，也不打印 API Key。
pub fn load_config_from_map(values: &HashMap<String, String>) -> Result<Config, ProtocolError> {
    let api_key = values
        .get("OPENAI_API_KEY")
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ProtocolError::Config(
                "missing OPENAI_API_KEY; export it before making a live request".to_owned(),
            )
        })?;
    let model = values
        .get("OPENAI_MODEL")
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ProtocolError::Config(
                "missing OPENAI_MODEL; set it to a model available to your account".to_owned(),
            )
        })?;
    let base_url = values
        .get("OPENAI_BASE_URL")
        .cloned()
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());
    let base_url = base_url.trim().to_owned();
    if base_url.is_empty() {
        return Err(ProtocolError::Config(
            "OPENAI_BASE_URL must not be empty".to_owned(),
        ));
    }
    let timeout_text = values
        .get("OPENAI_TIMEOUT_S")
        .cloned()
        .unwrap_or_else(|| DEFAULT_TIMEOUT_S.to_string());
    let timeout_s: f64 = timeout_text.parse().map_err(|_| {
        ProtocolError::Config("OPENAI_TIMEOUT_S must be a positive number".to_owned())
    })?;
    if !timeout_s.is_finite() || timeout_s <= 0.0 {
        return Err(ProtocolError::Config(
            "OPENAI_TIMEOUT_S must be a positive number".to_owned(),
        ));
    }
    Ok(Config {
        api_key,
        model,
        base_url: base_url.trim_end_matches('/').to_owned(),
        timeout_s,
    })
}

/// 从进程环境读取配置。
pub fn load_config() -> Result<Config, ProtocolError> {
    let mut values = HashMap::new();
    for key in [
        "OPENAI_API_KEY",
        "OPENAI_MODEL",
        "OPENAI_BASE_URL",
        "OPENAI_TIMEOUT_S",
    ] {
        if let Ok(value) = env::var(key) {
            values.insert(key.to_owned(), value);
        }
    }
    load_config_from_map(&values)
}

/// 生产传输实现：使用 reqwest 发送一次 HTTPS POST。
pub struct ReqwestTransport;

impl HttpTransport for ReqwestTransport {
    fn post_json(
        &mut self,
        url: &str,
        headers: &HashMap<String, String>,
        payload: &serde_json::Value,
        timeout_s: f64,
    ) -> Result<HttpResponse, ProtocolError> {
        let timeout = Duration::from_secs_f64(timeout_s);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| {
                ProtocolError::Transport(format!("request failed: {}", error_kind(&error)))
            })?;

        let mut request = client.post(url).json(payload);
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request.send().map_err(|error| {
            ProtocolError::Transport(format!("request failed: {}", error_kind(&error)))
        })?;
        let status_code = response.status().as_u16();
        let body = response.text().map_err(|error| {
            ProtocolError::Transport(format!("request failed: {}", error_kind(&error)))
        })?;
        Ok(HttpResponse { status_code, body })
    }
}

fn error_kind(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "Timeout"
    } else if error.is_connect() {
        "Connect"
    } else if error.is_request() {
        "Request"
    } else if error.is_body() {
        "Body"
    } else if error.is_decode() {
        "Decode"
    } else {
        "Error"
    }
}

/// 把简单提示词提升为统一协议请求。
pub fn build_model_request(model: &str, prompt: &str) -> Result<ModelRequest, ProtocolError> {
    ModelRequest::try_new(
        model.to_owned(),
        vec![Message::text(Role::User, prompt)?],
        None,
    )
}

/// 发送一个已经构造好的统一请求，并返回统一响应。
pub fn complete<T: HttpTransport>(
    request: &ModelRequest,
    config: &Config,
    transport: &mut T,
) -> Result<ModelResponse, ProtocolError> {
    let payload = encode_request(request)?;
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_owned(),
        format!("Bearer {}", config.api_key),
    );
    headers.insert("Content-Type".to_owned(), "application/json".to_owned());
    let response = transport.post_json(&config.endpoint(), &headers, &payload, config.timeout_s)?;
    parse_http_response(response.status_code, &response.body)
}

/// 完成一次统一协议模型调用；可注入 Fake Transport 离线测试。
pub fn chat_once<T: HttpTransport>(
    prompt: &str,
    config: &Config,
    transport: &mut T,
) -> Result<ModelResponse, ProtocolError> {
    let request = build_model_request(&config.model, prompt)?;
    complete(&request, config, transport)
}

/// 把统一响应格式化为可打印文本；工具候选单独标注。
pub fn format_response(response: &ModelResponse) -> Result<String, ProtocolError> {
    let mut lines = Vec::new();
    let text = response.text();
    if !text.is_empty() {
        lines.push(text);
    }
    for tool_use in response.tool_uses() {
        lines.push(format!(
            "[tool_use id={} name={} input={}]",
            tool_use.id, tool_use.name, tool_use.input
        ));
    }
    if lines.is_empty() {
        return Err(ProtocolError::Api(
            "decoded response contained neither text nor tool_use".to_owned(),
        ));
    }
    Ok(lines.join("\n"))
}
