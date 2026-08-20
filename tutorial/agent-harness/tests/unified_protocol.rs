use serde_json::json;
use tutorial_agent_harness::{
    ContentBlock, Message, ModelRequest, ProtocolError, Role, TextBlock, ToolDefinition,
    ToolResultBlock, ToolUseBlock, decode_response, encode_request, openai_responses,
};
#[test]
fn read_candidate_preserves_call_id_and_arguments() {
    let body = json!({
        "output": [{
            "type": "function_call",
            "call_id": "call_42",
            "name": "read",
            "arguments": "{\"path\":\"hello.txt\"}"
        }]
    });
    let response = decode_response(&body).unwrap();
    let call = response.tool_uses()[0];
    assert_eq!(call.id, "call_42");
    assert_eq!(call.name, "read");
    assert_eq!(call.input, json!({"path":"hello.txt"}));
}
#[test]
fn request_encodes_tools_and_roles() {
    let tool = ToolDefinition::try_new(
        "read",
        "Read a file",
        json!({"type":"object","properties":{"path":{"type":"string"}}}),
        true,
    )
    .unwrap();
    let request = ModelRequest::try_new(
        "m",
        vec![Message::text(Role::User, "read").unwrap()],
        None,
        vec![tool],
    )
    .unwrap();
    let body = encode_request(&request).unwrap();
    assert_eq!(body["tools"][0]["name"], "read");
    assert_eq!(body["input"][0]["content"][0]["type"], "input_text");
}
#[test]
fn system_user_tools_and_strict_flags_keep_order() {
    let first = ToolDefinition::try_new("read", "Read", json!({"type":"object"}), true).unwrap();
    let second =
        ToolDefinition::try_new("write", "Write", json!({"type":"object"}), false).unwrap();
    let request = ModelRequest::try_new(
        "m",
        vec![Message::text(Role::User, "go").unwrap()],
        Some("policy".into()),
        vec![first, second],
    )
    .unwrap();
    let body = encode_request(&request).unwrap();
    assert_eq!(body["input"][0]["role"], "system");
    assert_eq!(body["input"][1]["role"], "user");
    assert_eq!(body["tools"][0]["name"], "read");
    assert_eq!(body["tools"][0]["strict"], true);
    assert_eq!(body["tools"][1]["name"], "write");
    assert_eq!(body["tools"][1]["strict"], false);
}
#[test]
fn invalid_request_is_rejected() {
    assert!(ModelRequest::try_new("", vec![], None, vec![]).is_err());
    let bad = ToolDefinition::try_new("read", "x", json!([]), false);
    assert!(matches!(bad, Err(ProtocolError::Encode(_))));
    assert!(Message::try_new(Role::User, vec![]).is_err());
    assert!(ToolDefinition::try_new("bad name", "x", json!({"type":"object"}), true).is_err());
    assert!(ToolDefinition::try_new("read", " ", json!({"type":"object"}), true).is_err());
    let tool = ToolDefinition::try_new("read", "x", json!({"type":"object"}), true).unwrap();
    assert!(
        ModelRequest::try_new(
            "m",
            vec![Message::text(Role::User, "x").unwrap()],
            None,
            vec![tool.clone(), tool]
        )
        .is_err()
    );
}
#[test]
fn unknown_only_output_is_not_success() {
    assert!(decode_response(&json!({"output":[{"type":"reasoning","summary":[]}]})).is_err());
}
#[test]
fn response_text_reasoning_and_unknown_items_keep_order() {
    let response = decode_response(&json!({"output":[
        {"type":"reasoning","summary":[]},
        {"type":"message","content":[
            {"type":"output_text","text":"first"},
            {"type":"refusal","refusal":"not allowed"},
            {"type":"unknown","text":"ignored"}
        ]},
        {"type":"message","content":[{"type":"output_text","text":"second"}]}
    ]}))
    .unwrap();
    assert_eq!(response.text(), "first\nnot allowed\nsecond");
}
#[test]
fn tool_result_preserves_original_call_id() {
    let message = Message::try_new(
        Role::Tool,
        vec![ContentBlock::ToolResult(ToolResultBlock {
            tool_use_id: "call_42".into(),
            content: "hello".into(),
            is_error: false,
        })],
    )
    .unwrap();
    let request = ModelRequest::try_new("m", vec![message], None, vec![]).unwrap();
    let body = encode_request(&request).unwrap();
    assert_eq!(body["input"][0]["call_id"], "call_42");
}
#[test]
fn arguments_shapes_are_checked_and_empty_values_normalize() {
    for arguments in [json!({}), json!("{}"), json!(""), serde_json::Value::Null] {
        let body = json!({"output":[{"type":"function_call","call_id":"c","name":"read","arguments":arguments}]});
        assert_eq!(
            decode_response(&body).unwrap().tool_uses()[0].input,
            json!({})
        );
    }
    for arguments in [json!([]), json!("[]"), json!("not-json")] {
        let body = json!({"output":[{"type":"function_call","call_id":"c","name":"read","arguments":arguments}]});
        assert!(decode_response(&body).is_err());
    }
}
#[test]
fn missing_call_fields_and_output_are_rejected() {
    assert!(decode_response(&json!({"output":[{"type":"function_call","name":"read"}]})).is_err());
    assert!(decode_response(&json!({"output":[{"type":"function_call","call_id":"c"}]})).is_err());
    assert!(decode_response(&json!({})).is_err());
    assert!(openai_responses::decode_response_json("[]").is_err());
}
#[test]
fn assistant_mixed_content_encodes_in_order() {
    let message = Message::try_new(
        Role::Assistant,
        vec![
            ContentBlock::Text(TextBlock {
                text: "look".into(),
            }),
            ContentBlock::ToolUse(ToolUseBlock {
                id: "c".into(),
                name: "read".into(),
                input: json!({"path":"hello.txt"}),
            }),
        ],
    )
    .unwrap();
    let body =
        encode_request(&ModelRequest::try_new("m", vec![message], None, vec![]).unwrap()).unwrap();
    assert_eq!(body["input"][0]["content"][0]["type"], "output_text");
    assert_eq!(body["input"][1]["call_id"], "c");
}
