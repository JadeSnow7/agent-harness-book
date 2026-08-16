//! 参数化进程工具：不经过 shell，程序名按精确 allowlist 匹配。

use crate::registry::{require_object, require_string};
use crate::tool_types::{Tool, ToolError, ToolSpec};
use crate::workspace::Workspace;
use serde_json::{Value, json};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_S: f64 = 10.0;
const MAX_OUTPUT_CHARS: usize = 20_000;

/// 默认关闭的参数化进程工具。
pub struct BashTool {
    workspace: Workspace,
    enabled: bool,
    allowlist: Vec<String>,
    default_timeout_s: f64,
}

impl BashTool {
    /// 绑定 Workspace、启用开关和精确程序 allowlist。
    pub fn new(workspace: Workspace, enabled: bool, allowlist: Option<Vec<String>>) -> Self {
        Self {
            workspace,
            enabled,
            allowlist: allowlist.unwrap_or_else(|| {
                vec![
                    "echo".into(),
                    "/bin/echo".into(),
                    "pwd".into(),
                    "/bin/pwd".into(),
                ]
            }),
            default_timeout_s: DEFAULT_TIMEOUT_S,
        }
    }
}

impl Tool for BashTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "bash".into(),
            description: "Run one allowlisted program in the workspace without a shell".into(),
            input_schema: json!({
                "type": "object",
                "required": ["program", "args"],
                "properties": {
                    "program": {"type": "string"},
                    "args": {"type": "array", "items": {"type": "string"}},
                    "timeout_s": {"type": "number"}
                },
                "additionalProperties": false
            }),
            strict: false,
        }
    }

    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError> {
        let object = require_object(arguments)?;
        let _ = require_string(object, "program")?;
        let args = object
            .get("args")
            .and_then(Value::as_array)
            .ok_or_else(|| ToolError::new("args must be an array of strings"))?;
        if args.iter().any(|item| !item.is_string()) {
            return Err(ToolError::new("args must be an array of strings"));
        }
        if let Some(timeout) = object.get("timeout_s") {
            let value = timeout
                .as_f64()
                .ok_or_else(|| ToolError::new("timeout_s must be a positive number"))?;
            if value <= 0.0 {
                return Err(ToolError::new("timeout_s must be a positive number"));
            }
        }
        Ok(())
    }

    fn execute(&mut self, arguments: &Value) -> Result<Value, ToolError> {
        if !self.enabled {
            return Err(ToolError::new(
                "bash is disabled; enable explicitly for trusted demos only",
            ));
        }
        let object = require_object(arguments)?;
        let program = require_string(object, "program")?;
        if !self.allowlist.iter().any(|allowed| allowed == program) {
            return Err(ToolError::new("program rejected by exact allowlist"));
        }
        let args = object
            .get("args")
            .and_then(Value::as_array)
            .ok_or_else(|| ToolError::new("args must be an array of strings"))?
            .iter()
            .map(|value| value.as_str().unwrap_or_default())
            .collect::<Vec<_>>();
        let timeout_s = object
            .get("timeout_s")
            .and_then(Value::as_f64)
            .unwrap_or(self.default_timeout_s);

        let mut child = Command::new(program)
            .args(&args)
            .current_dir(self.workspace.root())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| ToolError::new(error.to_string()))?;
        let deadline = Instant::now() + Duration::from_secs_f64(timeout_s);
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ToolError::new(format!(
                        "program timed out after {timeout_s}s"
                    )));
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(error) => return Err(ToolError::new(error.to_string())),
            }
        }
        let output = child
            .wait_with_output()
            .map_err(|error| ToolError::new(error.to_string()))?;
        let (stdout, stdout_cut) =
            truncate_chars(String::from_utf8_lossy(&output.stdout).into_owned());
        let (stderr, stderr_cut) =
            truncate_chars(String::from_utf8_lossy(&output.stderr).into_owned());
        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            return Err(ToolError::new(format!(
                "exit {code}: {}",
                if detail.is_empty() {
                    "no output"
                } else {
                    detail
                }
            )));
        }
        Ok(json!({
            "program": program,
            "args": args,
            "exit_code": output.status.code().unwrap_or(0),
            "stdout": stdout,
            "stderr": stderr,
            "truncated": stdout_cut || stderr_cut,
        }))
    }
}

fn truncate_chars(text: String) -> (String, bool) {
    if text.chars().count() <= MAX_OUTPUT_CHARS {
        return (text, false);
    }
    (text.chars().take(MAX_OUTPUT_CHARS).collect(), true)
}
