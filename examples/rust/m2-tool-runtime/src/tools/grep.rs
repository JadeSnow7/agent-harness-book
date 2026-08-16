use crate::registry::{require_object, require_string};
use crate::tool_types::{Tool, ToolError, ToolSpec};
use crate::workspace::Workspace;
use regex::RegexBuilder;
use serde_json::{Value, json};
use std::path::PathBuf;

const DEFAULT_LIMIT: usize = 100;
const MAX_FILE_BYTES: u64 = 1_000_000;

/// 在 Workspace 文本文件中执行正则或字面量搜索。
pub struct GrepTool {
    workspace: Workspace,
}

impl GrepTool {
    /// 绑定固定 Workspace。
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}

impl Tool for GrepTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "grep".into(),
            description: "Search file contents inside the workspace".into(),
            input_schema: json!({
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"},
                    "glob": {"type": "string"},
                    "ignoreCase": {"type": "boolean"},
                    "literal": {"type": "boolean"},
                    "context": {"type": "integer"},
                    "limit": {"type": "integer"}
                },
                "additionalProperties": false
            }),
            strict: false,
        }
    }

    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError> {
        let object = require_object(arguments)?;
        let pattern = object
            .get("pattern")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("pattern must be a non-empty string"))?;
        if pattern.is_empty() {
            return Err(ToolError::new("pattern must be a non-empty string"));
        }
        for key in ["path", "glob"] {
            if object.get(key).is_some_and(|value| !value.is_string()) {
                return Err(ToolError::new(format!("{key} must be a string")));
            }
        }
        if object
            .get("glob")
            .and_then(Value::as_str)
            .is_some_and(|glob| glob.contains('[') || glob.contains(']'))
        {
            return Err(ToolError::new("glob character classes are not supported"));
        }
        for key in ["ignoreCase", "literal"] {
            if object.get(key).is_some_and(|value| !value.is_boolean()) {
                return Err(ToolError::new(format!("{key} must be a boolean")));
            }
        }
        for key in ["context", "limit"] {
            if object
                .get(key)
                .is_some_and(|value| value.as_u64().is_none())
            {
                return Err(ToolError::new(format!(
                    "{key} must be a non-negative integer"
                )));
            }
        }
        Ok(())
    }

    fn execute(&mut self, arguments: &Value) -> Result<Value, ToolError> {
        let object = require_object(arguments)?;
        let pattern = require_string(object, "pattern")?;
        let path_arg = object.get("path").and_then(Value::as_str);
        let root = self.workspace.resolve(path_arg, true)?;
        let ignore_case = object
            .get("ignoreCase")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let literal = object
            .get("literal")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let context = object.get("context").and_then(Value::as_u64).unwrap_or(0) as usize;
        let limit = object
            .get("limit")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_LIMIT);
        let glob = object.get("glob").and_then(Value::as_str).unwrap_or("**/*");

        let expression = if literal {
            regex::escape(pattern)
        } else {
            pattern.to_owned()
        };
        let regex = RegexBuilder::new(&expression)
            .case_insensitive(ignore_case)
            .multi_line(true)
            .build()
            .map_err(|error| ToolError::new(format!("invalid regex: {error}")))?;

        let files = collect_files(&root, glob)?;
        let mut matches = Vec::new();
        let mut truncated = false;
        for file in files {
            if matches.len() >= limit {
                truncated = true;
                break;
            }
            let meta = match std::fs::metadata(&file) {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            if meta.len() > MAX_FILE_BYTES {
                continue;
            }
            let text = match std::fs::read_to_string(&file) {
                Ok(text) => text,
                Err(_) => continue,
            };
            let lines: Vec<&str> = text.lines().collect();
            let rel = self.workspace.relative_to_root(&file)?;
            for (index, line) in lines.iter().enumerate() {
                if !regex.is_match(line) {
                    continue;
                }
                let line_no = index + 1;
                let start = index.saturating_sub(context);
                let end = (index + 1 + context).min(lines.len());
                let snippet = lines[start..end]
                    .iter()
                    .enumerate()
                    .map(|(offset, item)| format!("{}: {item}", start + offset + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                matches.push(json!({
                    "path": rel,
                    "line": line_no,
                    "text": line,
                    "snippet": snippet,
                }));
                if matches.len() >= limit {
                    truncated = true;
                    break;
                }
            }
        }

        Ok(json!({
            "pattern": pattern,
            "matches": matches,
            "count": matches.len(),
            "truncated": truncated,
        }))
    }
}

fn collect_files(root: &std::path::Path, glob_pat: &str) -> Result<Vec<PathBuf>, ToolError> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    if !root.is_dir() {
        return Err(ToolError::new(format!("not a file or directory: {root:?}")));
    }
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(|error| ToolError::new(error.to_string()))? {
            let entry = entry.map_err(|error| ToolError::new(error.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if crate::tools::find::glob_match(glob_pat, &rel) {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}
