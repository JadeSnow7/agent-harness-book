use serde_json::json;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};
use tutorial_agent_harness::{
    AppError, Config, ConfigError, HttpResponse, Transport, TransportError, chat_once,
    parse_http_response,
};
fn config() -> Config {
    Config::from_values(&HashMap::from([
        ("OPENAI_API_KEY".into(), "sentinel-secret".into()),
        ("OPENAI_MODEL".into(), "test-model".into()),
    ]))
    .unwrap()
}
#[derive(Debug)]
struct Fake {
    result: Result<HttpResponse, TransportError>,
    captured: Arc<Mutex<Option<CapturedRequest>>>,
}
#[derive(Debug, Clone)]
struct CapturedRequest {
    endpoint: String,
    headers: BTreeMap<String, String>,
    payload: serde_json::Value,
    timeout: f64,
}
impl Transport for Fake {
    fn send(
        &self,
        endpoint: &str,
        headers: &BTreeMap<String, String>,
        request: &serde_json::Value,
        timeout: f64,
    ) -> Result<HttpResponse, TransportError> {
        *self.captured.lock().unwrap() = Some(CapturedRequest {
            endpoint: endpoint.into(),
            headers: headers.clone(),
            payload: request.clone(),
            timeout,
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
            "{value}"
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
fn fake_transport_success_returns_structured_response() {
    let (transport, captured) = fake(Ok(HttpResponse { status: 200, body: json!({"output":[{"type":"message","content":[{"type":"output_text","text":"hello back"}]}]}).to_string() }));
    let output = chat_once(&config(), "hello", &transport).unwrap();
    assert_eq!(output.text(), "hello back");
    let captured = captured.lock().unwrap().clone().unwrap();
    assert_eq!(captured.endpoint, "https://api.openai.com/v1/responses");
    assert_eq!(
        captured.headers.get("authorization"),
        Some(&"Bearer sentinel-secret".into())
    );
    assert_eq!(
        captured.headers.get("content-type"),
        Some(&"application/json".into())
    );
    assert_eq!(
        captured.payload,
        json!({"model":"test-model","input":[{"type":"message","role":"user","content":[{"type":"input_text","text":"hello"}]}]})
    );
    assert_eq!(captured.timeout, 60.0);
}
#[test]
fn complete_uses_request_model_not_config_model() {
    let (transport, captured) = fake(Ok(HttpResponse {
        status: 200,
        body: json!({"output":[{"type":"message","content":[{"type":"output_text","text":"ok"}]}]})
            .to_string(),
    }));
    let request = tutorial_agent_harness::build_model_request(&config(), "hello").unwrap();
    let request = tutorial_agent_harness::ModelRequest {
        model: "request-model".into(),
        ..request
    };
    tutorial_agent_harness::complete(&request, &config(), &transport).unwrap();
    assert_eq!(
        captured.lock().unwrap().as_ref().unwrap().payload["model"],
        "request-model"
    );
}
#[test]
fn format_response_includes_text_and_tool_candidate() {
    let response = tutorial_agent_harness::decode_response(&json!({"output":[
        {"type":"message","content":[{"type":"output_text","text":"Need a file"}]},
        {"type":"function_call","call_id":"call_42","name":"read","arguments":{"path":"hello.txt"}}
    ]}))
    .unwrap();
    assert_eq!(
        tutorial_agent_harness::format_response(&response).unwrap(),
        "Need a file\ntool_use call_42 read {\"path\":\"hello.txt\"}"
    );
}
#[test]
fn format_response_rejects_empty_response() {
    let response = tutorial_agent_harness::ModelResponse {
        id: None,
        model: None,
        message: tutorial_agent_harness::Message::try_new(
            tutorial_agent_harness::Role::Assistant,
            vec![tutorial_agent_harness::ContentBlock::Text(
                tutorial_agent_harness::TextBlock { text: " ".into() },
            )],
        )
        .unwrap(),
        status: None,
    };
    assert!(tutorial_agent_harness::format_response(&response).is_err());
}
#[test]
fn fake_transport_errors_are_safe() {
    let error = chat_once(&config(), "hello", &fake(Err(TransportError::Network)).0).unwrap_err();
    assert!(matches!(error, AppError::Transport(_)));
    assert!(!error.to_string().contains("sentinel-secret"));
}
#[test]
fn status_errors_do_not_echo_body() {
    for status in [401, 429, 500] {
        let error = parse_http_response(status, &json!({"error":"sentinel-secret"}).to_string())
            .unwrap_err();
        assert!(matches!(
            error,
            tutorial_agent_harness::ProtocolError::Api(_)
        ));
        assert!(!error.to_string().contains("sentinel-secret"));
    }
}
#[test]
fn malformed_json_errors_are_safe() {
    for body in ["not-json", "[]"] {
        let error = parse_http_response(200, body).unwrap_err();
        assert!(matches!(
            error,
            tutorial_agent_harness::ProtocolError::Decode(_)
        ));
        assert!(!error.to_string().contains("sentinel-secret"));
    }
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
