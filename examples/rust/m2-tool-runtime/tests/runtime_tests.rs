use m1_unified_protocol::{
    Config, HttpResponse, HttpTransport, Message, ModelRequest, ProtocolError, Role,
};
use m2_tool_runtime::{
    Expectation, ExpectationKind, ToolCall, ToolRegistry, ToolStatus, Workspace,
    build_default_registry, copy_fixtures, default_fixtures_dir, detect, run_one_tool_step,
    run_scenario, tools::BashTool,
};
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_workspace() -> (std::path::PathBuf, Workspace) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let serial = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("m2-test-{}-{stamp}-{serial}", std::process::id()));
    copy_fixtures(&default_fixtures_dir(), &root).expect("copy fixtures");
    let workspace = Workspace::new(&root).expect("workspace");
    (root, workspace)
}

#[test]
fn rejects_path_escape() {
    let (root, workspace) = temp_workspace();
    let err = workspace
        .resolve(Some("../outside.txt"), false)
        .unwrap_err();
    assert!(err.to_string().contains("escapes workspace"));
    let mut registry = build_default_registry(workspace, false, None);
    let result = registry.execute(ToolCall {
        call_id: "c1".into(),
        name: "read".into(),
        arguments: json!({"path": "../secret.txt"}),
    });
    assert_eq!(result.status, ToolStatus::Failed);
    assert!(result.error.unwrap().contains("escapes workspace"));
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn rejects_absolute_escape_and_symlink_parent() {
    use std::os::unix::fs::symlink;

    let (root, workspace) = temp_workspace();
    let outside = root.parent().unwrap().join("m2-outside");
    std::fs::create_dir_all(&outside).unwrap();
    assert!(
        workspace
            .resolve(Some(&outside.to_string_lossy()), false)
            .is_err()
    );
    symlink(&outside, root.join("external-link")).unwrap();
    assert!(
        workspace
            .resolve(Some("external-link/new.txt"), false)
            .is_err()
    );
    let _ = std::fs::remove_dir_all(root);
    let _ = std::fs::remove_dir_all(outside);
}

#[test]
fn read_write_edit_round_trip() {
    let (root, workspace) = temp_workspace();
    let mut registry = build_default_registry(workspace.clone(), false, None);

    let written = registry.execute(ToolCall {
        call_id: "w".into(),
        name: "write".into(),
        arguments: json!({"path":"notes/a.txt","content":"payload"}),
    });
    assert!(written.succeeded());

    let read = registry.execute(ToolCall {
        call_id: "r".into(),
        name: "read".into(),
        arguments: json!({"path":"notes/a.txt","offset":1,"limit":1}),
    });
    assert!(read.succeeded());
    assert!(read.as_text().contains("payload"));

    let edited = registry.execute(ToolCall {
        call_id: "e".into(),
        name: "edit".into(),
        arguments: json!({
            "path":"src/app.py",
            "edits":[{"oldText":"MESSAGE = \"alpha\"","newText":"MESSAGE = \"beta\""}]
        }),
    });
    assert!(edited.succeeded());
    let text = std::fs::read_to_string(root.join("src/app.py")).unwrap();
    assert!(text.contains("MESSAGE = \"beta\""));
    assert!(!text.contains("MESSAGE = \"alpha\""));

    std::fs::write(root.join("overlap.txt"), "abc").unwrap();
    let overlap = registry.execute(ToolCall {
        call_id: "overlap".into(),
        name: "edit".into(),
        arguments: json!({
            "path":"overlap.txt",
            "edits":[
                {"oldText":"abc","newText":"x"},
                {"oldText":"bc","newText":"y"}
            ]
        }),
    });
    assert_eq!(overlap.status, ToolStatus::Failed);
    assert_eq!(
        std::fs::read_to_string(root.join("overlap.txt")).unwrap(),
        "abc"
    );
    assert!(std::fs::read_dir(&root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    }));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn search_and_list_tools() {
    let (root, workspace) = temp_workspace();
    let mut registry = build_default_registry(workspace, false, None);
    let ls = registry.execute(ToolCall {
        call_id: "l".into(),
        name: "ls".into(),
        arguments: json!({"path":"."}),
    });
    assert!(ls.succeeded());
    assert!(ls.as_text().contains("hello.txt"));

    let find = registry.execute(ToolCall {
        call_id: "f".into(),
        name: "find".into(),
        arguments: json!({"pattern":"**/*.py"}),
    });
    assert!(find.succeeded());
    assert!(find.as_text().contains("app.py"));

    std::fs::write(root.join("top.py"), "TOP").unwrap();
    std::fs::write(root.join("src/a1.txt"), "a").unwrap();
    let parity_find = registry.execute(ToolCall {
        call_id: "fp".into(),
        name: "find".into(),
        arguments: json!({"pattern":"**/*.py"}),
    });
    assert_eq!(
        parity_find.output.unwrap()["matches"],
        json!(["src/app.py", "src/util.py", "top.py"])
    );
    let question = registry.execute(ToolCall {
        call_id: "fq".into(),
        name: "find".into(),
        arguments: json!({"pattern":"src/a?.txt"}),
    });
    assert_eq!(question.output.unwrap()["matches"], json!(["src/a1.txt"]));
    let limited = registry.execute(ToolCall {
        call_id: "fl".into(),
        name: "find".into(),
        arguments: json!({"pattern":"**/*","limit":1}),
    });
    assert_eq!(limited.output.as_ref().unwrap()["count"], 1);
    assert_eq!(limited.output.unwrap()["truncated"], true);
    let unsupported = registry.execute(ToolCall {
        call_id: "fu".into(),
        name: "find".into(),
        arguments: json!({"pattern":"src/[ab].txt"}),
    });
    assert!(unsupported.error.unwrap().contains("not supported"));

    let grep = registry.execute(ToolCall {
        call_id: "g".into(),
        name: "grep".into(),
        arguments: json!({"pattern":"MESSAGE","glob":"**/*.py"}),
    });
    assert!(grep.succeeded());
    assert!(grep.as_text().contains("app.py"));
    std::fs::write(root.join("search.txt"), "before\nAlpha.\nafter\n").unwrap();
    let regex = registry.execute(ToolCall {
        call_id: "gr".into(),
        name: "grep".into(),
        arguments: json!({"pattern":"alpha\\.$","ignoreCase":true,"context":1}),
    });
    assert!(regex.succeeded());
    assert_eq!(regex.output.as_ref().unwrap()["count"], 1);
    assert!(regex.as_text().contains("before"));
    let literal = registry.execute(ToolCall {
        call_id: "gl".into(),
        name: "grep".into(),
        arguments: json!({"pattern":"Alpha.","literal":true}),
    });
    assert_eq!(literal.output.unwrap()["count"], 1);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn bash_disabled_allowlist_and_unknown_tool() {
    let (root, workspace) = temp_workspace();
    let mut registry = build_default_registry(workspace.clone(), false, None);
    let disabled = registry.execute(ToolCall {
        call_id: "b0".into(),
        name: "bash".into(),
        arguments: json!({"program":"echo","args":["hi"]}),
    });
    assert_eq!(disabled.status, ToolStatus::Failed);
    assert!(disabled.error.unwrap().contains("disabled"));

    let unknown = registry.execute(ToolCall {
        call_id: "u".into(),
        name: "missing".into(),
        arguments: json!({}),
    });
    assert!(unknown.error.unwrap().contains("unknown tool"));

    let invalid = registry.execute(ToolCall {
        call_id: "invalid".into(),
        name: "read".into(),
        arguments: json!({}),
    });
    assert!(invalid.error.unwrap().contains("invalid arguments"));
    let missing = registry.execute(ToolCall {
        call_id: "missing-file".into(),
        name: "read".into(),
        arguments: json!({"path":"missing.txt"}),
    });
    assert!(missing.error.unwrap().contains("does not exist"));

    let mut enabled = ToolRegistry::default();
    enabled.register(BashTool::new(
        workspace,
        true,
        Some(vec![
            "echo".into(),
            "/bin/echo".into(),
            "/usr/bin/false".into(),
            "/bin/sleep".into(),
        ]),
    ));
    let ok = enabled.execute(ToolCall {
        call_id: "b1".into(),
        name: "bash".into(),
        arguments: json!({"program":"echo","args":["hi; pwd"]}),
    });
    assert!(ok.succeeded());
    assert!(ok.as_text().contains("hi; pwd"));

    let bad = enabled.execute(ToolCall {
        call_id: "b2".into(),
        name: "bash".into(),
        arguments: json!({"program":"rm","args":["-rf", "/"]}),
    });
    assert!(bad.error.unwrap().contains("allowlist"));

    let nonzero = enabled.execute(ToolCall {
        call_id: "b3".into(),
        name: "bash".into(),
        arguments: json!({"program":"/usr/bin/false","args":[]}),
    });
    assert!(nonzero.error.unwrap().contains("exit"));

    let timeout = enabled.execute(ToolCall {
        call_id: "b4".into(),
        name: "bash".into(),
        arguments: json!({"program":"/bin/sleep","args":["1"],"timeout_s":0.01}),
    });
    assert!(timeout.error.unwrap().contains("timed out"));

    let huge = "x".repeat(20_100);
    let truncated = enabled.execute(ToolCall {
        call_id: "b5".into(),
        name: "bash".into(),
        arguments: json!({"program":"echo","args":[huge]}),
    });
    assert!(truncated.succeeded());
    assert_eq!(truncated.output.unwrap()["truncated"], true);
    let _ = std::fs::remove_dir_all(root);
}

#[derive(Default)]
struct ScriptedTransport {
    bodies: VecDeque<serde_json::Value>,
    payloads: Vec<serde_json::Value>,
}

impl HttpTransport for ScriptedTransport {
    fn post_json(
        &mut self,
        _url: &str,
        _headers: &HashMap<String, String>,
        payload: &serde_json::Value,
        _timeout_s: f64,
    ) -> Result<HttpResponse, ProtocolError> {
        self.payloads.push(payload.clone());
        Ok(HttpResponse {
            status_code: 200,
            body: self.bodies.pop_front().unwrap().to_string(),
        })
    }
}

fn function_response(calls: &[(&str, &str, serde_json::Value)]) -> serde_json::Value {
    json!({
        "id":"resp_1",
        "model":"gpt-test",
        "status":"completed",
        "output": calls.iter().map(|(call_id, name, arguments)| json!({
            "type":"function_call",
            "call_id": call_id,
            "name": name,
            "arguments": arguments.to_string()
        })).collect::<Vec<_>>()
    })
}

fn text_response() -> serde_json::Value {
    json!({
        "id":"resp_2",
        "model":"gpt-test",
        "status":"completed",
        "output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"done"}]}]
    })
}

#[test]
fn fixed_two_call_closure_preserves_call_id() {
    let (root, workspace) = temp_workspace();
    let mut registry = build_default_registry(workspace, false, None);
    let request = ModelRequest::try_new(
        "gpt-test",
        vec![Message::text(Role::User, "read hello.txt").unwrap()],
        None,
    )
    .unwrap();
    let config = Config {
        api_key: "test-key".into(),
        model: "gpt-test".into(),
        base_url: "https://example.test/v1".into(),
        timeout_s: 1.0,
    };
    let mut transport = ScriptedTransport {
        bodies: VecDeque::from(vec![
            function_response(&[("call_42", "read", json!({"path":"hello.txt"}))]),
            text_response(),
        ]),
        payloads: Vec::new(),
    };
    let result = run_one_tool_step(&request, &config, &mut transport, &mut registry).unwrap();
    assert_eq!(result.tool_result.call_id, "call_42");
    assert_eq!(result.final_response.text(), "done");
    assert_eq!(transport.payloads.len(), 2);
    assert_eq!(transport.payloads[0]["tools"].as_array().unwrap().len(), 7);
    assert!(
        transport.payloads[1]["input"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["type"] == "function_call_output" && item["call_id"] == "call_42" })
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn fixed_closure_rejects_zero_multiple_and_second_tool_call() {
    let (root, workspace) = temp_workspace();
    let request = ModelRequest::try_new(
        "gpt-test",
        vec![Message::text(Role::User, "test").unwrap()],
        None,
    )
    .unwrap();
    let config = Config {
        api_key: "test-key".into(),
        model: "gpt-test".into(),
        base_url: "https://example.test/v1".into(),
        timeout_s: 1.0,
    };
    let cases = vec![
        vec![text_response()],
        vec![function_response(&[
            ("a", "read", json!({"path":"hello.txt"})),
            ("b", "ls", json!({"path":"."})),
        ])],
        vec![
            function_response(&[("a", "read", json!({"path":"hello.txt"}))]),
            function_response(&[("b", "ls", json!({"path":"."}))]),
        ],
    ];
    for bodies in cases {
        let mut registry = build_default_registry(workspace.clone(), false, None);
        let mut transport = ScriptedTransport {
            bodies: VecDeque::from(bodies),
            payloads: Vec::new(),
        };
        assert!(run_one_tool_step(&request, &config, &mut transport, &mut registry).is_err());
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn detection_and_scenario() {
    let (root, workspace) = temp_workspace();
    let mut registry = build_default_registry(workspace.clone(), false, None);
    let result = registry.execute(ToolCall {
        call_id: "e".into(),
        name: "edit".into(),
        arguments: json!({
            "path":"src/app.py",
            "edits":[{"oldText":"MESSAGE = \"alpha\"","newText":"MESSAGE = \"beta\""}]
        }),
    });
    let report = detect(
        &workspace,
        &result,
        &[
            Expectation {
                kind: ExpectationKind::ToolSucceeded,
                text: None,
                path: None,
                call_id: None,
                error_substring: None,
            },
            Expectation {
                kind: ExpectationKind::FileContains,
                text: Some("beta".into()),
                path: Some("src/app.py".into()),
                call_id: None,
                error_substring: None,
            },
        ],
    );
    assert!(report.passed());

    let scenario_root = root.join("scenario");
    copy_fixtures(&default_fixtures_dir(), &scenario_root).unwrap();
    assert!(run_scenario(&scenario_root, false).unwrap());
    let _ = std::fs::remove_dir_all(root);
}
