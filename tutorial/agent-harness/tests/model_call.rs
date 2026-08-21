use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};
use tutorial_agent_harness::{
    AppError, Config, ConfigError, HttpResponse, Transport, TransportError, build_request,
    chat_once, parse_http_response,
};

/// 构造一份测试专用配置；API Key 使用哨兵值，方便断言错误信息不泄漏它。
fn config() -> Config {
    Config::from_values(&HashMap::from([
        ("OPENAI_API_KEY".into(), "sentinel-secret".into()),
        ("OPENAI_MODEL".into(), "test-model".into()),
    ]))
    .unwrap()
}

/// 记录最后一次被发送的请求，供测试断言 endpoint/headers/payload/timeout。
#[derive(Debug, Clone)]
struct CapturedRequest {
    endpoint: String,
    headers: BTreeMap<String, String>,
    payload: serde_json::Value,
    timeout: f64,
}

/// 可注入的假 Transport：返回预置结果并记录收到的请求，不发起真实网络请求。
#[derive(Debug)]
struct Fake {
    result: Result<HttpResponse, TransportError>,
    captured: Arc<Mutex<Option<CapturedRequest>>>,
}

impl Transport for Fake {
    fn send(
        &self,
        endpoint: &str,
        headers: &BTreeMap<String, String>,
        request: &serde_json::Value,
        timeout_s: f64,
    ) -> Result<HttpResponse, TransportError> {
        *self.captured.lock().unwrap() = Some(CapturedRequest {
            endpoint: endpoint.into(),
            headers: headers.clone(),
            payload: request.clone(),
            timeout: timeout_s,
        });
        self.result.clone()
    }
}

fn fake(
    result: Result<HttpResponse, TransportError>,
) -> (Fake, Arc<Mutex<Option<CapturedRequest>>>) {
    let captured = Arc::new(Mutex::new(None));
    (
        Fake {
            result,
            captured: captured.clone(),
        },
        captured,
    )
}

#[test]
fn config_validates_required_values() {
    let missing = HashMap::from([("OPENAI_MODEL".into(), "m".into())]);
    assert_eq!(
        Config::from_values(&missing),
        Err(ConfigError::Missing("OPENAI_API_KEY"))
    );

    let missing_model = HashMap::from([("OPENAI_API_KEY".into(), "k".into())]);
    assert_eq!(
        Config::from_values(&missing_model),
        Err(ConfigError::Missing("OPENAI_MODEL"))
    );

    let empty_base = HashMap::from([
        ("OPENAI_API_KEY".into(), "k".into()),
        ("OPENAI_MODEL".into(), "m".into()),
        ("OPENAI_BASE_URL".into(), " ".into()),
    ]);
    assert_eq!(
        Config::from_values(&empty_base),
        Err(ConfigError::InvalidBaseUrl)
    );

    for value in ["no", "NaN", "inf", "0", "-1", "1e308"] {
        let values = HashMap::from([
            ("OPENAI_API_KEY".into(), "k".into()),
            ("OPENAI_MODEL".into(), "m".into()),
            ("OPENAI_TIMEOUT_S".into(), value.into()),
        ]);
        assert_eq!(
            Config::from_values(&values),
            Err(ConfigError::InvalidTimeout),
            "timeout value {value} should be rejected"
        );
    }
}

#[test]
fn endpoint_trims_slashes() {
    let values = HashMap::from([
        ("OPENAI_API_KEY".into(), "k".into()),
        ("OPENAI_MODEL".into(), "m".into()),
        (
            "OPENAI_BASE_URL".into(),
            "https://example.test/v1///".into(),
        ),
    ]);
    assert_eq!(
        Config::from_values(&values).unwrap().endpoint(),
        "https://example.test/v1/responses"
    );
}

#[test]
fn build_request_is_provider_shaped() {
    let request = build_request(&config(), "Hello");
    assert_eq!(request, json!({"model": "test-model", "input": "Hello"}));
}

#[test]
fn build_request_does_not_touch_filesystem_for_a_reading_prompt() {
    // M0 只把 prompt 当纯文本转发；即便文字提到 hello.txt，也不应该触发任何文件访问。
    let prompt = "请读取工作区 hello.txt 的第一行并告诉我内容；如果无法访问文件，不要猜测。";
    let request = build_request(&config(), prompt);
    assert_eq!(request, json!({"model": "test-model", "input": prompt}));
}

#[test]
fn chat_once_sends_expected_request_and_extracts_text() {
    let (transport, captured) = fake(Ok(HttpResponse {
        status: 200,
        body: json!({"output": [{
            "type": "message",
            "content": [{"type": "output_text", "text": "hello back"}]
        }]})
        .to_string(),
    }));

    let text = chat_once(&config(), "Hello", &transport).unwrap();
    assert_eq!(text, "hello back");

    let captured = captured.lock().unwrap().clone().unwrap();
    assert_eq!(captured.endpoint, "https://api.openai.com/v1/responses");
    assert_eq!(
        captured.headers.get("authorization"),
        Some(&"Bearer sentinel-secret".to_owned())
    );
    assert_eq!(
        captured.headers.get("content-type"),
        Some(&"application/json".to_owned())
    );
    assert_eq!(
        captured.payload,
        json!({"model": "test-model", "input": "Hello"})
    );
    assert_eq!(captured.timeout, 60.0);
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
    })
    .to_string();

    assert_eq!(parse_http_response(200, &body).unwrap(), "answer");
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
    })
    .to_string();

    assert_eq!(parse_http_response(200, &body).unwrap(), "first\nsecond");
}

#[test]
fn chat_once_errors_never_contain_the_api_key() {
    let (transport, _) = fake(Err(TransportError::Network));
    let error = chat_once(&config(), "hello", &transport).unwrap_err();
    assert!(matches!(error, AppError::Transport(_)));
    assert!(!error.to_string().contains("sentinel-secret"));
}

#[test]
fn status_errors_do_not_echo_body() {
    for status in [401, 429, 500] {
        let body = json!({"error": "sentinel-secret"}).to_string();
        let error = parse_http_response(status, &body).unwrap_err();
        assert!(matches!(error, AppError::ApiStatus(s) if s == status));
        assert!(!error.to_string().contains("sentinel-secret"));
    }
}

#[test]
fn reports_invalid_json_without_echoing_the_body() {
    let body = "not-json-with-sentinel-secret";
    let error = parse_http_response(200, body).unwrap_err();
    assert!(matches!(error, AppError::InvalidJson));
    assert!(!error.to_string().contains("sentinel-secret"));
}

#[test]
fn missing_output_text_is_a_missing_output_text_error() {
    let body = json!({"output": []}).to_string();
    let error = parse_http_response(200, &body).unwrap_err();
    assert!(matches!(error, AppError::MissingOutputText));
}

#[test]
fn non_object_json_is_reported_as_missing_output_text() {
    // 已知边界，不是 bug：body 是合法 JSON 但不是对象（例如裸数组）时，`Value::get("output")`
    // 在非对象值上直接返回 None，因此归类为 MissingOutputText 而不是单独的“非对象”错误。
    // 这与 examples/rust/m0-model-call 的既有行为一致，本任务不引入新的错误细分。
    let error = parse_http_response(200, "[]").unwrap_err();
    assert!(matches!(error, AppError::MissingOutputText));
}

#[test]
fn cli_fails_without_configuration() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_tutorial-agent-harness"))
        .env_clear()
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing OPENAI_API_KEY"));
}
