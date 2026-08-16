use m1_unified_protocol::openai_responses::tool_result_message;
use m1_unified_protocol::{
    Config, ContentBlock, HttpResponse, HttpTransport, Message, ModelRequest, ModelResponse,
    ProtocolError, Role, TextBlock, ToolDefinition, ToolUseBlock, build_model_request, chat_once,
    complete, decode_response, encode_request, format_response, load_config_from_map,
    parse_http_response,
};
use serde_json::{Value, json};
use std::collections::HashMap;

struct FakeTransport {
    response: HttpResponse,
    last_url: Option<String>,
    last_headers: Option<HashMap<String, String>>,
    last_payload: Option<Value>,
}

impl HttpTransport for FakeTransport {
    fn post_json(
        &mut self,
        url: &str,
        headers: &HashMap<String, String>,
        payload: &Value,
        _timeout_s: f64,
    ) -> Result<HttpResponse, ProtocolError> {
        self.last_url = Some(url.to_owned());
        self.last_headers = Some(headers.clone());
        self.last_payload = Some(payload.clone());
        Ok(self.response.clone())
    }
}

fn message_output(text: &str) -> Value {
    json!({
        "type": "message",
        "role": "assistant",
        "content": [{"type": "output_text", "text": text}],
    })
}

#[test]
fn empty_message_is_rejected() {
    let error = Message::try_new(Role::User, vec![]).unwrap_err();
    assert!(matches!(error, ProtocolError::Encode(_)));
    assert!(error.to_string().contains("content must not be empty"));
}

#[test]
fn response_text_and_tool_uses() {
    let response = ModelResponse {
        id: Some("resp_1".into()),
        model: Some("gpt-test".into()),
        message: Message::try_new(
            Role::Assistant,
            vec![
                ContentBlock::Text(TextBlock {
                    text: "first".into(),
                }),
                ContentBlock::ToolUse(ToolUseBlock {
                    id: "call_1".into(),
                    name: "echo".into(),
                    input: json!({"text": "hi"}),
                }),
                ContentBlock::Text(TextBlock {
                    text: "second".into(),
                }),
            ],
        )
        .unwrap(),
        status: None,
    };
    assert_eq!(response.text(), "first\nsecond");
    assert_eq!(response.tool_uses().len(), 1);
    assert_eq!(response.tool_uses()[0].name, "echo");
}

#[test]
fn encodes_system_and_user_text() {
    let request = ModelRequest::try_new(
        "gpt-test",
        vec![Message::text(Role::User, "Hello").unwrap()],
        Some("be brief".into()),
    )
    .unwrap();
    let payload = encode_request(&request).unwrap();
    assert_eq!(payload["model"], "gpt-test");
    assert_eq!(
        payload["input"],
        json!([
            {
                "type": "message",
                "role": "system",
                "content": [{"type": "input_text", "text": "be brief"}],
            },
            {
                "type": "message",
                "role": "user",
                "content": [{"type": "input_text", "text": "Hello"}],
            }
        ])
    );
}

#[test]
fn rejects_invalid_and_duplicate_tool_definitions() {
    let error = ToolDefinition::try_new("not a tool", "invalid", json!({"type":"object"}), true)
        .unwrap_err();
    assert!(error.to_string().contains("tool name"));

    let tool =
        ToolDefinition::try_new("echo", "Echo text", json!({"type":"object"}), true).unwrap();
    let error = ModelRequest::try_new_with_tools(
        "gpt-test",
        vec![Message::text(Role::User, "hello").unwrap()],
        None,
        vec![tool.clone(), tool],
    )
    .unwrap_err();
    assert!(error.to_string().contains("unique"));
}

#[test]
fn encodes_function_tool_definitions_in_order() {
    let request = ModelRequest::try_new_with_tools(
        "gpt-test",
        vec![Message::text(Role::User, "Use a tool").unwrap()],
        None,
        vec![
            ToolDefinition::try_new(
                "echo",
                "Echo text",
                json!({
                    "type":"object",
                    "properties":{"text":{"type":"string"}},
                    "required":["text"],
                    "additionalProperties":false
                }),
                true,
            )
            .unwrap(),
            ToolDefinition::try_new(
                "pwd",
                "Return workspace path",
                json!({"type":"object","properties":{},"additionalProperties":false}),
                false,
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let payload = encode_request(&request).unwrap();
    assert_eq!(payload["tools"][0]["name"], "echo");
    assert_eq!(payload["tools"][1]["name"], "pwd");
    assert_eq!(payload["tools"][0]["type"], "function");
    assert_eq!(payload["tools"][0]["strict"], true);
    assert_eq!(payload["tools"][1]["strict"], false);
}

#[test]
fn encodes_tool_result_as_function_call_output() {
    let request = ModelRequest::try_new(
        "gpt-test",
        vec![tool_result_message("call_1", r#"{"ok":true}"#).unwrap()],
        None,
    )
    .unwrap();
    let payload = encode_request(&request).unwrap();
    assert_eq!(
        payload["input"],
        json!([{
            "type": "function_call_output",
            "call_id": "call_1",
            "output": r#"{"ok":true}"#,
        }])
    );
}

#[test]
fn encodes_assistant_tool_use() {
    let request = ModelRequest::try_new(
        "gpt-test",
        vec![
            Message::try_new(
                Role::Assistant,
                vec![
                    ContentBlock::Text(TextBlock {
                        text: "calling".into(),
                    }),
                    ContentBlock::ToolUse(ToolUseBlock {
                        id: "call_9".into(),
                        name: "echo".into(),
                        input: json!({"text": "ping"}),
                    }),
                ],
            )
            .unwrap(),
        ],
        None,
    )
    .unwrap();
    let payload = encode_request(&request).unwrap();
    assert_eq!(payload["input"][0]["role"], "assistant");
    assert_eq!(payload["input"][1]["type"], "function_call");
    assert_eq!(payload["input"][1]["call_id"], "call_9");
    let arguments: Value =
        serde_json::from_str(payload["input"][1]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments, json!({"text": "ping"}));
}

#[test]
fn skips_reasoning_and_joins_text() {
    let body = json!({
        "id": "resp_abc",
        "model": "gpt-test",
        "status": "completed",
        "output": [
            {"type": "reasoning", "summary": []},
            message_output("first"),
            {
                "type": "message",
                "content": [
                    {"type": "output_text", "text": "second"},
                    {"type": "refusal", "refusal": "ignored-as-text-if-present"},
                ],
            }
        ]
    });
    let response = decode_response(&body).unwrap();
    assert_eq!(response.id.as_deref(), Some("resp_abc"));
    assert_eq!(response.model.as_deref(), Some("gpt-test"));
    assert_eq!(response.status.as_deref(), Some("completed"));
    assert_eq!(response.text(), "first\nsecond\nignored-as-text-if-present");
}

#[test]
fn decodes_function_call_arguments_string() {
    let body = json!({
        "output": [{
            "type": "function_call",
            "call_id": "call_1",
            "name": "echo",
            "arguments": r#"{"text":"hi"}"#,
        }]
    });
    let response = decode_response(&body).unwrap();
    let tool_use = response.tool_uses()[0];
    assert_eq!(tool_use.id, "call_1");
    assert_eq!(tool_use.name, "echo");
    assert_eq!(tool_use.input, json!({"text": "hi"}));
}

#[test]
fn decodes_function_call_arguments_object() {
    let body = json!({
        "output": [{
            "type": "function_call",
            "call_id": "call_2",
            "name": "echo",
            "arguments": {"text": "object"},
        }]
    });
    let response = decode_response(&body).unwrap();
    assert_eq!(response.tool_uses()[0].input, json!({"text": "object"}));
}

#[test]
fn missing_output_is_decode_error() {
    let error = decode_response(&json!({})).unwrap_err();
    assert!(error.to_string().contains("output list"));
}

#[test]
fn empty_output_is_decode_error() {
    let error = decode_response(&json!({"output": [{"type": "reasoning"}]})).unwrap_err();
    assert!(error.to_string().contains("no decodable content"));
}

#[test]
fn invalid_arguments_json_is_decode_error() {
    let body = json!({
        "output": [{
            "type": "function_call",
            "call_id": "call_1",
            "name": "echo",
            "arguments": "not-json",
        }]
    });
    let error = decode_response(&body).unwrap_err();
    assert!(error.to_string().contains("not valid JSON"));
}

#[test]
fn function_call_requires_call_id_name_and_object_arguments() {
    let cases = [
        (
            json!({"type":"function_call","name":"echo","arguments":"{}"}),
            "call_id",
        ),
        (
            json!({"type":"function_call","call_id":"call_1","arguments":"{}"}),
            "name",
        ),
        (
            json!({"type":"function_call","call_id":"call_1","name":"echo","arguments":"[]"}),
            "must be an object",
        ),
    ];
    for (item, expected) in cases {
        let error = decode_response(&json!({"output":[item]})).unwrap_err();
        assert!(error.to_string().contains(expected));
    }
}

#[test]
fn unknown_output_is_skipped_but_cannot_be_the_only_output() {
    let response = decode_response(&json!({
        "output":[{"type":"future_item"}, message_output("known")]
    }))
    .unwrap();
    assert_eq!(response.text(), "known");
    let error = decode_response(&json!({"output":[{"type":"future_item"}]})).unwrap_err();
    assert!(error.to_string().contains("no decodable content"));
}

#[test]
fn http_error_does_not_echo_body() {
    let error = parse_http_response(401, r#"{"error":{"message":"secret-response"}}"#).unwrap_err();
    assert_eq!(error.to_string(), "API request failed with HTTP 401");
    assert!(!error.to_string().contains("secret-response"));
}

#[test]
fn invalid_json_is_decode_error() {
    let error = parse_http_response(200, "not-json").unwrap_err();
    assert!(error.to_string().contains("invalid JSON"));
}

#[test]
fn missing_api_key_and_model() {
    let mut values = HashMap::new();
    values.insert("OPENAI_MODEL".into(), "gpt-test".into());
    let error = load_config_from_map(&values).unwrap_err();
    assert!(error.to_string().contains("missing OPENAI_API_KEY"));

    let mut values = HashMap::new();
    values.insert("OPENAI_API_KEY".into(), "secret-value".into());
    let error = load_config_from_map(&values).unwrap_err();
    assert!(error.to_string().contains("missing OPENAI_MODEL"));
}

#[test]
fn timeout_must_be_positive_and_finite() {
    for bad in ["inf", "nan", "0", "-1", "abc"] {
        let mut values = HashMap::new();
        values.insert("OPENAI_API_KEY".into(), "secret-value".into());
        values.insert("OPENAI_MODEL".into(), "gpt-test".into());
        values.insert("OPENAI_TIMEOUT_S".into(), bad.into());
        let error = load_config_from_map(&values).unwrap_err();
        assert!(
            error.to_string().contains("positive number"),
            "expected rejection for {bad:?}, got {error}"
        );
    }
}

#[test]
fn chat_once_round_trip_with_fake_transport() {
    let config = Config {
        api_key: "secret-value".into(),
        model: "gpt-test".into(),
        base_url: "https://example.test/v1".into(),
        timeout_s: 5.0,
    };
    let mut transport = FakeTransport {
        response: HttpResponse {
            status_code: 200,
            body: json!({
                "id": "resp_1",
                "model": "gpt-test",
                "output": [message_output("hello protocol")],
            })
            .to_string(),
        },
        last_url: None,
        last_headers: None,
        last_payload: None,
    };

    let response = chat_once("Hello", &config, &mut transport).unwrap();
    assert_eq!(response.text(), "hello protocol");
    assert_eq!(
        transport.last_url.as_deref(),
        Some("https://example.test/v1/responses")
    );
    let headers = transport.last_headers.unwrap();
    assert_eq!(
        headers.get("Authorization").map(String::as_str),
        Some("Bearer secret-value")
    );
    assert_eq!(
        headers.get("Content-Type").map(String::as_str),
        Some("application/json")
    );
    let expected = encode_request(&build_model_request("gpt-test", "Hello").unwrap()).unwrap();
    assert_eq!(transport.last_payload.unwrap(), expected);
}

#[test]
fn complete_sends_the_exact_unified_request() {
    let config = Config {
        api_key: "secret-value".into(),
        model: "config-model".into(),
        base_url: "https://example.test/v1".into(),
        timeout_s: 5.0,
    };
    let request = ModelRequest::try_new_with_tools(
        "request-model",
        vec![Message::text(Role::User, "Hello").unwrap()],
        None,
        vec![ToolDefinition::try_new("echo", "Echo text", json!({"type":"object"}), true).unwrap()],
    )
    .unwrap();
    let mut transport = FakeTransport {
        response: HttpResponse {
            status_code: 200,
            body: json!({"output": [message_output("done")]}).to_string(),
        },
        last_url: None,
        last_headers: None,
        last_payload: None,
    };
    let response = complete(&request, &config, &mut transport).unwrap();
    assert_eq!(response.text(), "done");
    assert_eq!(
        transport.last_payload.unwrap(),
        encode_request(&request).unwrap()
    );
}

#[test]
fn format_response_includes_tool_use() {
    let response = ModelResponse {
        id: None,
        model: None,
        message: Message::try_new(
            Role::Assistant,
            vec![
                ContentBlock::Text(TextBlock {
                    text: "done".into(),
                }),
                ContentBlock::ToolUse(ToolUseBlock {
                    id: "c1".into(),
                    name: "echo".into(),
                    input: json!({"text": "x"}),
                }),
            ],
        )
        .unwrap(),
        status: None,
    };
    let rendered = format_response(&response).unwrap();
    assert!(rendered.contains("done"));
    assert!(rendered.contains("tool_use"));
    assert!(rendered.contains("echo"));
}
