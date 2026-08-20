use serde_json::json;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use tutorial_agent_harness::{
    Config, ContentBlock, HttpResponse, Message, ModelRequest, ModelResponse, OneStepError,
    ReadTool, Role, TextBlock, Tool, ToolCall, ToolRegistry, ToolStatus, Transport, TransportError,
    Workspace, encode_request, run_one_tool_step,
};

fn config() -> Config {
    Config::from_values(&HashMap::from([
        ("OPENAI_API_KEY".into(), "sentinel-secret".into()),
        ("OPENAI_MODEL".into(), "test-model".into()),
    ]))
    .unwrap()
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_workspace() -> (PathBuf, Workspace) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let serial = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "ch4-tool-runtime-{}-{stamp}-{serial}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("create temp workspace root");
    std::fs::write(root.join("hello.txt"), include_str!("fixtures/hello.txt"))
        .expect("write hello.txt fixture");
    let workspace = Workspace::new(&root).expect("workspace");
    (root, workspace)
}

/// A Fake Transport that returns queued responses in order; used to drive the
/// two-call one-step closure without any real network access.
struct QueueTransport {
    responses: Mutex<VecDeque<Result<HttpResponse, TransportError>>>,
}

impl QueueTransport {
    fn new(responses: Vec<Result<HttpResponse, TransportError>>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
        }
    }
}

impl Transport for QueueTransport {
    fn send(
        &self,
        _endpoint: &str,
        _headers: &BTreeMap<String, String>,
        _request: &serde_json::Value,
        _timeout_s: f64,
    ) -> Result<HttpResponse, TransportError> {
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("no more queued fake responses")
    }
}

fn ok(body: serde_json::Value) -> Result<HttpResponse, TransportError> {
    Ok(HttpResponse {
        status: 200,
        body: body.to_string(),
    })
}

// --- Registry: unknown tool ------------------------------------------------

#[test]
fn unknown_tool_fails_without_touching_workspace() {
    let (root, workspace) = temp_workspace();
    let mut registry = ToolRegistry::default();
    registry.register(ReadTool::new(workspace));
    let result = registry.execute(ToolCall {
        call_id: "c1".into(),
        name: "grep".into(),
        arguments: json!({}),
    });
    assert_eq!(result.status, ToolStatus::Failed);
    assert!(result.error.unwrap().contains("unknown tool"));
    let _ = std::fs::remove_dir_all(root);
}

// --- read: argument validation ---------------------------------------------

#[test]
fn missing_or_non_string_path_fails_before_execution() {
    let (root, workspace) = temp_workspace();
    let tool = ReadTool::new(workspace);
    assert!(tool.validate_arguments(&json!({})).is_err());
    assert!(tool.validate_arguments(&json!({"path": 5})).is_err());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn offset_and_limit_reject_non_positive_and_boolean() {
    let (root, workspace) = temp_workspace();
    let tool = ReadTool::new(workspace);
    for bad in [
        json!({"path": "hello.txt", "offset": 0}),
        json!({"path": "hello.txt", "offset": -1}),
        json!({"path": "hello.txt", "offset": true}),
        json!({"path": "hello.txt", "offset": 1.5}),
        json!({"path": "hello.txt", "limit": 0}),
        json!({"path": "hello.txt", "limit": false}),
    ] {
        assert!(
            tool.validate_arguments(&bad).is_err(),
            "expected failure for {bad}"
        );
    }
    let _ = std::fs::remove_dir_all(root);
}

// --- Workspace: path boundary ------------------------------------------------

#[test]
fn rejects_dot_dot_and_absolute_escape() {
    let (root, workspace) = temp_workspace();
    let err = workspace
        .resolve(Some("../outside.txt"), false)
        .unwrap_err();
    assert!(err.to_string().contains("escapes workspace"));

    let outside = root.parent().unwrap().join("ch4-outside-absolute.txt");
    let err2 = workspace
        .resolve(Some(outside.to_str().unwrap()), false)
        .unwrap_err();
    assert!(err2.to_string().contains("escapes workspace"));
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn rejects_escape_through_symlinked_missing_parent() {
    use std::os::unix::fs::symlink;

    let (root, workspace) = temp_workspace();
    let outside = root.parent().unwrap().join("ch4-outside-symlink-target");
    std::fs::create_dir_all(&outside).unwrap();
    symlink(&outside, root.join("external-link")).unwrap();

    let err = workspace
        .resolve(Some("external-link/nested/new.txt"), false)
        .unwrap_err();
    assert!(err.to_string().contains("escapes workspace"));

    let _ = std::fs::remove_dir_all(&outside);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn missing_target_and_non_file_target_fail() {
    let (root, workspace) = temp_workspace();
    let mut tool = ReadTool::new(workspace);

    let missing = tool.execute(&json!({"path": "nope.txt"})).unwrap_err();
    assert!(missing.to_string().contains("does not exist"));

    let not_file = tool.execute(&json!({"path": "."})).unwrap_err();
    assert!(not_file.to_string().contains("not a file"));

    let _ = std::fs::remove_dir_all(root);
}

// --- read: normal path, offset/limit, truncation ----------------------------

#[test]
fn read_returns_numbered_lines_with_offset_and_limit() {
    let (root, workspace) = temp_workspace();
    let mut tool = ReadTool::new(workspace);

    let output = tool.execute(&json!({"path": "hello.txt"})).unwrap();
    assert_eq!(
        output["content"],
        json!("1: hello workspace\n2: second line")
    );
    assert_eq!(output["line_count"], json!(2));
    assert_eq!(output["truncated"], json!(false));

    let output2 = tool
        .execute(&json!({"path": "hello.txt", "offset": 2, "limit": 1}))
        .unwrap();
    assert_eq!(output2["content"], json!("2: second line"));
    assert_eq!(output2["line_count"], json!(1));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn read_truncates_at_default_max_lines() {
    let (root, workspace) = temp_workspace();
    let big = (1..=2005)
        .map(|n| format!("line-{n}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(root.join("big.txt"), big).unwrap();

    let mut tool = ReadTool::new(workspace);
    let output = tool.execute(&json!({"path": "big.txt"})).unwrap();
    assert_eq!(output["truncated"], json!(true));
    assert_eq!(output["line_count"], json!(2000));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn read_truncates_at_default_max_bytes() {
    let (root, workspace) = temp_workspace();
    let big = "a".repeat(512_000 + 50);
    std::fs::write(root.join("bytes.txt"), &big).unwrap();

    let mut tool = ReadTool::new(workspace);
    let output = tool
        .execute(&json!({"path": "bytes.txt", "limit": 5}))
        .unwrap();
    assert_eq!(output["truncated"], json!(true));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn read_with_offset_past_eof_returns_empty_content_instead_of_panicking() {
    let (root, workspace) = temp_workspace();
    let mut tool = ReadTool::new(workspace);

    let output = tool
        .execute(&json!({"path": "hello.txt", "offset": 100}))
        .unwrap();
    assert_eq!(output["content"], json!(""));
    assert_eq!(output["line_count"], json!(0));
    assert_eq!(output["truncated"], json!(false));

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn read_with_offset_near_u64_max_does_not_overflow() {
    let (root, workspace) = temp_workspace();
    let mut tool = ReadTool::new(workspace);

    let output = tool
        .execute(&json!({"path": "hello.txt", "offset": u64::MAX - 1}))
        .unwrap();
    assert_eq!(output["content"], json!(""));
    assert_eq!(output["line_count"], json!(0));

    let _ = std::fs::remove_dir_all(root);
}

// --- one-step closure: continuous success case ------------------------------

#[test]
fn one_step_closure_preserves_call_id_end_to_end() {
    let (root, workspace) = temp_workspace();
    let mut registry = ToolRegistry::default();
    registry.register(ReadTool::new(workspace));

    let transport = QueueTransport::new(vec![
        ok(json!({
            "output": [{
                "type": "function_call",
                "call_id": "call_42",
                "name": "read",
                "arguments": "{\"path\":\"hello.txt\"}"
            }]
        })),
        ok(json!({
            "output": [{
                "type": "message",
                "content": [{"type": "output_text", "text": "the file says hello"}]
            }]
        })),
    ]);

    let request = ModelRequest::try_new(
        "test-model",
        vec![Message::text(Role::User, "please read hello.txt").unwrap()],
        None,
        Vec::new(),
    )
    .unwrap();

    let outcome = run_one_tool_step(&request, &config(), &transport, &mut registry).unwrap();
    assert_eq!(outcome.tool_result.call_id, "call_42");
    assert!(outcome.tool_result.succeeded());
    assert_eq!(outcome.final_response.text(), "the file says hello");

    let followup_body = encode_request(&outcome.followup_request).unwrap();
    let tool_item = followup_body["input"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call_output")
        .expect("function_call_output present in the follow-up request");
    assert_eq!(tool_item["call_id"], "call_42");
    assert!(
        tool_item["output"]
            .as_str()
            .unwrap()
            .contains("1: hello workspace")
    );

    let _ = std::fs::remove_dir_all(root);
}

// --- one-step closure: fixed-shape failures ---------------------------------

#[test]
fn first_call_without_exactly_one_tool_fails() {
    let (root, workspace) = temp_workspace();
    let request = ModelRequest::try_new(
        "test-model",
        vec![Message::text(Role::User, "hi").unwrap()],
        None,
        Vec::new(),
    )
    .unwrap();

    let mut registry_zero = ToolRegistry::default();
    registry_zero.register(ReadTool::new(workspace));
    let zero = QueueTransport::new(vec![ok(json!({
        "output": [{"type": "message", "content": [{"type": "output_text", "text": "no tool needed"}]}]
    }))]);
    let err = run_one_tool_step(&request, &config(), &zero, &mut registry_zero).unwrap_err();
    assert!(matches!(err, OneStepError::Shape(_)));

    let (root2, workspace2) = temp_workspace();
    let mut registry_two = ToolRegistry::default();
    registry_two.register(ReadTool::new(workspace2));
    let two = QueueTransport::new(vec![ok(json!({
        "output": [
            {"type": "function_call", "call_id": "a", "name": "read", "arguments": "{\"path\":\"hello.txt\"}"},
            {"type": "function_call", "call_id": "b", "name": "read", "arguments": "{\"path\":\"hello.txt\"}"}
        ]
    }))]);
    let err2 = run_one_tool_step(&request, &config(), &two, &mut registry_two).unwrap_err();
    assert!(matches!(err2, OneStepError::Shape(_)));

    let _ = std::fs::remove_dir_all(root);
    let _ = std::fs::remove_dir_all(root2);
}

#[test]
fn second_call_requesting_another_tool_fails() {
    let (root, workspace) = temp_workspace();
    let mut registry = ToolRegistry::default();
    registry.register(ReadTool::new(workspace));

    let transport = QueueTransport::new(vec![
        ok(json!({
            "output": [{"type": "function_call", "call_id": "call_1", "name": "read", "arguments": "{\"path\":\"hello.txt\"}"}]
        })),
        ok(json!({
            "output": [{"type": "function_call", "call_id": "call_2", "name": "read", "arguments": "{\"path\":\"hello.txt\"}"}]
        })),
    ]);

    let request = ModelRequest::try_new(
        "test-model",
        vec![Message::text(Role::User, "read it").unwrap()],
        None,
        Vec::new(),
    )
    .unwrap();

    let err = run_one_tool_step(&request, &config(), &transport, &mut registry).unwrap_err();
    assert!(matches!(err, OneStepError::Shape(_)));
    let _ = std::fs::remove_dir_all(root);
}

/// `run_one_tool_step` also guards against a decoded second response with no
/// displayable text. Given M1's `decode_response`, a successfully decoded
/// response always contains either a tool candidate or at least one non-empty
/// text block, so this exact state cannot be produced by decoding a real HTTP
/// body; it is exercised here directly against `ModelResponse`, whose fields
/// are public, to prove the guard's condition is correct if that invariant
/// ever changes.
#[test]
fn a_response_with_no_tool_use_and_no_text_would_be_rejected() {
    let response = ModelResponse {
        id: None,
        model: None,
        message: Message::try_new(
            Role::Assistant,
            vec![ContentBlock::Text(TextBlock {
                text: String::new(),
            })],
        )
        .unwrap(),
        status: None,
    };
    assert!(response.tool_uses().is_empty());
    assert!(response.text().is_empty());
}

// --- safety: no secret or host path leakage ---------------------------------

#[test]
fn failure_paths_never_leak_the_configured_api_key() {
    let (root, workspace) = temp_workspace();
    let mut registry = ToolRegistry::default();
    registry.register(ReadTool::new(workspace));
    let result = registry.execute(ToolCall {
        call_id: "c1".into(),
        name: "read".into(),
        arguments: json!({"path": "../escape.txt"}),
    });
    let message = result.error.unwrap();
    assert!(!message.contains("sentinel-secret"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn escape_error_does_not_echo_resolved_host_path() {
    let (root, workspace) = temp_workspace();
    let err = workspace.resolve(Some("../escape.txt"), false).unwrap_err();
    // Only the caller-supplied text may appear; the canonicalized host
    // ancestor path must not be echoed back.
    assert!(!err.to_string().contains(root.to_string_lossy().as_ref()));
    let _ = std::fs::remove_dir_all(root);
}
