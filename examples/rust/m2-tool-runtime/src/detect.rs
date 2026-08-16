//! 单步 postcondition checker；不等同于 M7 任务级 Validation/Evidence。

use crate::tool_types::ToolResult;
use crate::workspace::Workspace;
use std::fmt;

/// 单步可检查的后置条件种类。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpectationKind {
    ToolSucceeded,
    ToolFailed,
    OutputContains,
    FileContains,
    FileNotContains,
    FileExists,
}

/// 一条局部后置条件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expectation {
    pub kind: ExpectationKind,
    pub text: Option<String>,
    pub path: Option<String>,
    pub call_id: Option<String>,
    pub error_substring: Option<String>,
}

/// 单项检查结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckResult {
    pub expectation: Expectation,
    pub passed: bool,
    pub detail: String,
}

/// 一次工具调用后的全部检查结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionReport {
    pub checks: Vec<CheckResult>,
}

impl DetectionReport {
    /// 是否所有局部检查都通过。
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }

    /// 生成面向读者的多行摘要。
    pub fn summary(&self) -> String {
        let status = if self.passed() { "passed" } else { "failed" };
        let mut lines = vec![format!("detection={status} checks={}", self.checks.len())];
        for check in &self.checks {
            let mark = if check.passed { "ok" } else { "FAIL" };
            lines.push(format!(
                "  [{mark}] {}: {}",
                kind_name(&check.expectation.kind),
                check.detail
            ));
        }
        lines.join("\n")
    }
}

impl fmt::Display for DetectionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary())
    }
}

/// 对一次结果和当前 Workspace 运行局部后置条件。
pub fn detect(
    workspace: &Workspace,
    result: &ToolResult,
    expectations: &[Expectation],
) -> DetectionReport {
    let checks = expectations
        .iter()
        .map(|expectation| evaluate(workspace, result, expectation))
        .collect();
    DetectionReport { checks }
}

fn evaluate(workspace: &Workspace, result: &ToolResult, expectation: &Expectation) -> CheckResult {
    match expectation.kind {
        ExpectationKind::ToolSucceeded => {
            if expectation
                .call_id
                .as_ref()
                .is_some_and(|call_id| call_id != &result.call_id)
            {
                return CheckResult {
                    expectation: expectation.clone(),
                    passed: false,
                    detail: "call_id mismatch".into(),
                };
            }
            let ok = result.succeeded();
            CheckResult {
                expectation: expectation.clone(),
                passed: ok,
                detail: if ok {
                    "tool succeeded".into()
                } else {
                    format!("tool failed: {:?}", result.error)
                },
            }
        }
        ExpectationKind::ToolFailed => {
            if expectation
                .call_id
                .as_ref()
                .is_some_and(|call_id| call_id != &result.call_id)
            {
                return CheckResult {
                    expectation: expectation.clone(),
                    passed: false,
                    detail: "call_id mismatch".into(),
                };
            }
            if result.succeeded() {
                return CheckResult {
                    expectation: expectation.clone(),
                    passed: false,
                    detail: "expected failure, got success".into(),
                };
            }
            if let Some(sub) = &expectation.error_substring {
                let haystack = result.error.clone().unwrap_or_default();
                let ok = haystack.contains(sub);
                return CheckResult {
                    expectation: expectation.clone(),
                    passed: ok,
                    detail: if ok {
                        "error matched".into()
                    } else {
                        format!("error did not contain {sub:?}")
                    },
                };
            }
            CheckResult {
                expectation: expectation.clone(),
                passed: true,
                detail: format!("failed as expected: {:?}", result.error),
            }
        }
        ExpectationKind::OutputContains => {
            let text = expectation.text.clone().unwrap_or_default();
            let blob = output_blob(result);
            let ok = blob.contains(&text);
            CheckResult {
                expectation: expectation.clone(),
                passed: ok,
                detail: if ok {
                    "output contains text".into()
                } else {
                    format!("output missing {text:?}")
                },
            }
        }
        ExpectationKind::FileExists => {
            let path = expectation.path.clone().unwrap_or_default();
            match workspace.resolve(Some(&path), false) {
                Ok(target) => {
                    let ok = target.exists();
                    CheckResult {
                        expectation: expectation.clone(),
                        passed: ok,
                        detail: if ok {
                            "file exists".into()
                        } else {
                            format!("missing file {path:?}")
                        },
                    }
                }
                Err(error) => CheckResult {
                    expectation: expectation.clone(),
                    passed: false,
                    detail: error.to_string(),
                },
            }
        }
        ExpectationKind::FileContains | ExpectationKind::FileNotContains => file_text_check(
            workspace,
            expectation,
            expectation.kind == ExpectationKind::FileContains,
        ),
    }
}

fn file_text_check(
    workspace: &Workspace,
    expectation: &Expectation,
    should_contain: bool,
) -> CheckResult {
    let path = expectation.path.clone().unwrap_or_default();
    let text = expectation.text.clone().unwrap_or_default();
    match workspace.resolve(Some(&path), true) {
        Ok(target) => match std::fs::read_to_string(target) {
            Ok(content) => {
                let contains = content.contains(&text);
                let ok = if should_contain { contains } else { !contains };
                let detail = if should_contain {
                    if ok {
                        "file contains text".into()
                    } else {
                        format!("file missing {text:?}")
                    }
                } else if ok {
                    "file does not contain text".into()
                } else {
                    format!("file still contains {text:?}")
                };
                CheckResult {
                    expectation: expectation.clone(),
                    passed: ok,
                    detail,
                }
            }
            Err(error) => CheckResult {
                expectation: expectation.clone(),
                passed: false,
                detail: error.to_string(),
            },
        },
        Err(error) => CheckResult {
            expectation: expectation.clone(),
            passed: false,
            detail: error.to_string(),
        },
    }
}

fn output_blob(result: &ToolResult) -> String {
    if !result.succeeded() {
        return result.error.clone().unwrap_or_default();
    }
    match &result.output {
        None => String::new(),
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(value) => value.to_string(),
    }
}

fn kind_name(kind: &ExpectationKind) -> &'static str {
    match kind {
        ExpectationKind::ToolSucceeded => "tool_succeeded",
        ExpectationKind::ToolFailed => "tool_failed",
        ExpectationKind::OutputContains => "output_contains",
        ExpectationKind::FileContains => "file_contains",
        ExpectationKind::FileNotContains => "file_not_contains",
        ExpectationKind::FileExists => "file_exists",
    }
}
