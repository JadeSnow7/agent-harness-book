use reqwest::StatusCode;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::time::Duration;

const DEFAULT_INPUT: &str = "请用一句话解释：为什么单次模型调用还不是 Agent？";

/// 一次模型调用所需的配置；API Key 只在内存中传递。
#[derive(Debug)]
struct Config {
    api_key: String,
    model: String,
    base_url: String,
    timeout_s: f64,
}

impl Config {
    /// 拼出 Responses API 端点，不让调用流程重复处理斜杠。
    fn endpoint(&self) -> String {
        format!("{}/responses", self.base_url.trim_end_matches('/'))
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("m0-model-call: {error}");
        std::process::exit(1);
    }
}

/// 从进程环境读取配置，不执行网络请求，也不打印 API Key。
fn load_config() -> Result<Config, AppError> {
    let mut values = HashMap::new();
    for key in ["OPENAI_API_KEY", "OPENAI_MODEL"] {
        if let Ok(value) = env::var(key) {
            values.insert(key.to_owned(), value);
        }
    }
    load_config_from_map(&values)
}

/// 从环境变量映射读取配置；测试可以传入固定 map，不依赖真实进程环境。
fn load_config_from_map(values: &HashMap<String, String>) -> Result<Config, AppError> {
    let api_key = values
        .get("OPENAI_API_KEY")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Config(
                "missing OPENAI_API_KEY; export it before making a live request".to_owned(),
            )
        })?;
    let model = values
        .get("OPENAI_MODEL")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Config(
                "missing OPENAI_MODEL; set it to a model available to your account".to_owned(),
            )
        })?;

    Ok(Config {
        api_key: api_key.clone(),
        model: model.clone(),
        base_url: "https://api.openai.com/v1".to_owned(),
        timeout_s: 60.0,
    })
}

fn run() -> Result<(), AppError> {
    // 凭据只从进程环境读取，既不进入源码，也不出现在错误信息里。
    let config = load_config()?;

    // 复用 Client 可以共享连接池；超时则为教学程序提供明确的停止边界。
    let client = Client::builder()
        .timeout(Duration::from_secs_f64(config.timeout_s))
        .build()
        .map_err(AppError::Request)?;
    let response = client
        .post(config.endpoint())
        .bearer_auth(config.api_key)
        .json(&json!({
            "model": config.model,
            "input": DEFAULT_INPUT,
        }))
        .send()
        .map_err(AppError::Request)?;

    let status = response.status();
    let body = response.text().map_err(AppError::Request)?;
    println!("{}", parse_api_response(status, &body)?);
    Ok(())
}

fn parse_api_response(status: StatusCode, body: &str) -> Result<String, AppError> {
    // 不回显错误响应正文，避免服务端或代理意外反射敏感请求内容。
    if !status.is_success() {
        return Err(AppError::ApiStatus(status.as_u16()));
    }

    let response: Value = serde_json::from_str(body).map_err(|_| AppError::InvalidJson)?;
    extract_output_text(&response)
}

fn extract_output_text(response: &Value) -> Result<String, AppError> {
    // Responses API 的 output 可能先包含 reasoning 等项目，不能固定读取 output[0]。
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

#[derive(Debug)]
enum AppError {
    Config(String),
    Request(reqwest::Error),
    ApiStatus(u16),
    InvalidJson,
    MissingOutputText,
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => formatter.write_str(message),
            Self::Request(error) => write!(formatter, "request failed: {error}"),
            Self::ApiStatus(status) => write!(formatter, "API request failed with HTTP {status}"),
            Self::InvalidJson => formatter.write_str("API returned invalid JSON"),
            Self::MissingOutputText => formatter.write_str("API response contained no output_text"),
        }
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_missing_api_key_is_a_config_error() {
        // 没有 API Key 时返回安全错误，错误中不包含密钥。
        let mut values = HashMap::new();
        values.insert("OPENAI_MODEL".to_owned(), "gpt-test".to_owned());
        let error = load_config_from_map(&values).unwrap_err();
        assert!(error.to_string().contains("missing OPENAI_API_KEY"));
    }

    #[test]
    fn config_missing_model_is_a_config_error() {
        // 缺少 OPENAI_MODEL 时必须失败，而不是回落写死的模型名。
        let mut values = HashMap::new();
        values.insert("OPENAI_API_KEY".to_owned(), "secret-value".to_owned());
        let error = load_config_from_map(&values).unwrap_err();
        assert!(error.to_string().contains("missing OPENAI_MODEL"));
    }

    #[test]
    fn config_loads_without_printing_secret() {
        let mut values = HashMap::new();
        values.insert("OPENAI_API_KEY".to_owned(), "secret-value".to_owned());
        values.insert("OPENAI_MODEL".to_owned(), "gpt-5.6-luna".to_owned());
        let config = load_config_from_map(&values).unwrap();
        assert_eq!(config.api_key, "secret-value");
        assert_eq!(config.model, "gpt-5.6-luna");
        assert_eq!(config.endpoint(), "https://api.openai.com/v1/responses");
    }

    #[test]
    fn extracts_output_text_from_a_normal_response() {
        let body = json!({
            "output": [{
                "type": "message",
                "content": [{"type": "output_text", "text": "hello"}]
            }]
        });

        assert_eq!(extract_output_text(&body).unwrap(), "hello");
    }

    #[test]
    fn skips_reasoning_items_before_the_message() {
        let body = json!({
            "output": [
                {"type": "reasoning", "summary": []},
                {
                    "type": "message",
                    "content": [{"type": "output_text", "text": "answer"}]
                }
            ]
        });

        assert_eq!(extract_output_text(&body).unwrap(), "answer");
    }

    #[test]
    fn joins_multiple_output_text_items_in_order() {
        let body = json!({
            "output": [{
                "type": "message",
                "content": [
                    {"type": "output_text", "text": "first"},
                    {"type": "refusal", "refusal": "ignored"},
                    {"type": "output_text", "text": "second"}
                ]
            }]
        });

        assert_eq!(extract_output_text(&body).unwrap(), "first\nsecond");
    }

    #[test]
    fn reports_a_missing_text_output() {
        let error = extract_output_text(&json!({"output": []})).unwrap_err();
        assert!(matches!(error, AppError::MissingOutputText));
    }

    #[test]
    fn reports_invalid_json_without_echoing_the_body() {
        let body = "not-json-with-secret-value";
        let error = parse_api_response(StatusCode::OK, body).unwrap_err();
        assert!(matches!(error, AppError::InvalidJson));
        assert!(!error.to_string().contains("secret-value"));
    }

    #[test]
    fn reports_http_status_without_echoing_the_body() {
        let body = r#"{"error":{"message":"secret-value"}}"#;
        let error = parse_api_response(StatusCode::UNAUTHORIZED, body).unwrap_err();
        assert!(matches!(error, AppError::ApiStatus(401)));
        assert!(!error.to_string().contains("secret-value"));
    }
}
