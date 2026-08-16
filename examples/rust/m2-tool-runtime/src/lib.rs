//! M2 Tool Runtime：七个基础工具 + 注册表 + 检测反馈。

pub mod bridge;
pub mod detect;
pub mod one_step;
pub mod registry;
pub mod tool_types;
pub mod tools;
pub mod workspace;

pub use bridge::{
    result_to_message, result_to_tool_result_block, spec_to_tool_definition, tool_use_to_call,
};
pub use detect::{CheckResult, DetectionReport, Expectation, ExpectationKind, detect};
pub use one_step::{OneStepError, OneStepResult, request_with_registry_tools, run_one_tool_step};
pub use registry::ToolRegistry;
pub use tool_types::{Tool, ToolCall, ToolError, ToolResult, ToolSpec, ToolStatus};
pub use tools::build_default_registry;
pub use workspace::Workspace;

use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// 把 fixtures 复制到目标目录。
pub fn copy_fixtures(fixtures: &Path, target: &Path) -> Result<(), ToolError> {
    if target.exists() {
        fs::remove_dir_all(target).map_err(|error| ToolError::new(error.to_string()))?;
    }
    copy_dir_all(fixtures, target)
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), ToolError> {
    fs::create_dir_all(dst).map_err(|error| ToolError::new(error.to_string()))?;
    for entry in fs::read_dir(src).map_err(|error| ToolError::new(error.to_string()))? {
        let entry = entry.map_err(|error| ToolError::new(error.to_string()))?;
        let ty = entry
            .file_type()
            .map_err(|error| ToolError::new(error.to_string()))?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(|error| ToolError::new(error.to_string()))?;
        }
    }
    Ok(())
}

/// 运行与 Python demo 同构的单步场景。
pub fn run_scenario(workspace_root: &Path, enable_bash: bool) -> Result<bool, ToolError> {
    let workspace = Workspace::new(workspace_root)?;
    let mut registry = build_default_registry(
        workspace.clone(),
        enable_bash,
        Some(vec![
            "echo".into(),
            "/bin/echo".into(),
            "pwd".into(),
            "/bin/pwd".into(),
        ]),
    );

    let mut steps: Vec<(Value, Vec<Expectation>)> = vec![
        (
            json!({"call_id":"c1","name":"ls","arguments":{"path":"."}}),
            vec![
                Expectation {
                    kind: ExpectationKind::ToolSucceeded,
                    text: None,
                    path: None,
                    call_id: Some("c1".into()),
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::OutputContains,
                    text: Some("hello.txt".into()),
                    path: None,
                    call_id: None,
                    error_substring: None,
                },
            ],
        ),
        (
            json!({"call_id":"c2","name":"read","arguments":{"path":"hello.txt"}}),
            vec![
                Expectation {
                    kind: ExpectationKind::ToolSucceeded,
                    text: None,
                    path: None,
                    call_id: Some("c2".into()),
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::OutputContains,
                    text: Some("hello workspace".into()),
                    path: None,
                    call_id: None,
                    error_substring: None,
                },
            ],
        ),
        (
            json!({
                "call_id":"c3",
                "name":"edit",
                "arguments":{
                    "path":"src/app.py",
                    "edits":[{"oldText":"MESSAGE = \"alpha\"","newText":"MESSAGE = \"beta\""}]
                }
            }),
            vec![
                Expectation {
                    kind: ExpectationKind::ToolSucceeded,
                    text: None,
                    path: None,
                    call_id: Some("c3".into()),
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::FileContains,
                    text: Some("MESSAGE = \"beta\"".into()),
                    path: Some("src/app.py".into()),
                    call_id: None,
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::FileNotContains,
                    text: Some("MESSAGE = \"alpha\"".into()),
                    path: Some("src/app.py".into()),
                    call_id: None,
                    error_substring: None,
                },
            ],
        ),
        (
            json!({"call_id":"c4","name":"grep","arguments":{"pattern":"beta","glob":"**/*.py"}}),
            vec![
                Expectation {
                    kind: ExpectationKind::ToolSucceeded,
                    text: None,
                    path: None,
                    call_id: Some("c4".into()),
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::OutputContains,
                    text: Some("src/app.py".into()),
                    path: None,
                    call_id: None,
                    error_substring: None,
                },
            ],
        ),
        (
            json!({"call_id":"c5","name":"find","arguments":{"pattern":"**/*.txt"}}),
            vec![
                Expectation {
                    kind: ExpectationKind::ToolSucceeded,
                    text: None,
                    path: None,
                    call_id: Some("c5".into()),
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::OutputContains,
                    text: Some("hello.txt".into()),
                    path: None,
                    call_id: None,
                    error_substring: None,
                },
            ],
        ),
        (
            json!({"call_id":"c6","name":"write","arguments":{"path":"notes/out.md","content":"# done\n"}}),
            vec![
                Expectation {
                    kind: ExpectationKind::ToolSucceeded,
                    text: None,
                    path: None,
                    call_id: Some("c6".into()),
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::FileExists,
                    text: None,
                    path: Some("notes/out.md".into()),
                    call_id: None,
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::FileContains,
                    text: Some("# done".into()),
                    path: Some("notes/out.md".into()),
                    call_id: None,
                    error_substring: None,
                },
            ],
        ),
    ];

    if enable_bash {
        steps.push((
            json!({"call_id":"c7","name":"bash","arguments":{"program":"echo","args":["ok-from-bash"]}}),
            vec![
                Expectation {
                    kind: ExpectationKind::ToolSucceeded,
                    text: None,
                    path: None,
                    call_id: Some("c7".into()),
                    error_substring: None,
                },
                Expectation {
                    kind: ExpectationKind::OutputContains,
                    text: Some("ok-from-bash".into()),
                    path: None,
                    call_id: None,
                    error_substring: None,
                },
            ],
        ));
    } else {
        steps.push((
            json!({"call_id":"c7","name":"bash","arguments":{"program":"echo","args":["should-fail"]}}),
            vec![Expectation {
                kind: ExpectationKind::ToolFailed,
                text: None,
                path: None,
                call_id: Some("c7".into()),
                error_substring: Some("disabled".into()),
            }],
        ));
    }

    steps.push((
        json!({"call_id":"c8","name":"not-a-tool","arguments":{}}),
        vec![Expectation {
            kind: ExpectationKind::ToolFailed,
            text: None,
            path: None,
            call_id: Some("c8".into()),
            error_substring: Some("unknown tool".into()),
        }],
    ));
    steps.push((
        json!({"call_id":"c9","name":"read","arguments":{"path":"../outside.txt"}}),
        vec![Expectation {
            kind: ExpectationKind::ToolFailed,
            text: None,
            path: None,
            call_id: Some("c9".into()),
            error_substring: Some("escapes workspace".into()),
        }],
    ));

    let mut all_passed = true;
    for (raw, expectations) in steps {
        let call_id = raw
            .get("call_id")
            .and_then(Value::as_str)
            .unwrap_or("call")
            .to_owned();
        let name = raw
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let arguments = raw.get("arguments").cloned().unwrap_or_else(|| json!({}));
        let result = registry.execute(ToolCall {
            call_id,
            name,
            arguments,
        });
        let report = detect(&workspace, &result, &expectations);
        println!("---");
        println!(
            "{}",
            json!({
                "call_id": result.call_id,
                "name": result.name,
                "status": result.status.as_str(),
                "error": result.error,
            })
        );
        println!("{}", report.summary());
        all_passed = all_passed && report.passed();
    }
    println!("scenario={}", if all_passed { "passed" } else { "failed" });
    Ok(all_passed)
}

pub fn default_fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python/m2-tool-runtime/fixtures")
}
