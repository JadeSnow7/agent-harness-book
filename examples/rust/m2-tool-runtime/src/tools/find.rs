use crate::registry::{require_object, require_string};
use crate::tool_types::{Tool, ToolError, ToolSpec};
use crate::workspace::Workspace;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const DEFAULT_LIMIT: usize = 1000;

/// 递归枚举并按教学 glob 子集筛选路径。
pub struct FindTool {
    workspace: Workspace,
}

impl FindTool {
    /// 绑定固定 Workspace。
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}

impl Tool for FindTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "find".into(),
            description: "Find files by glob pattern inside the workspace".into(),
            input_schema: json!({
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"},
                    "limit": {"type": "integer"}
                },
                "additionalProperties": false
            }),
            strict: false,
        }
    }

    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError> {
        let object = require_object(arguments)?;
        let pattern = require_string(object, "pattern")?;
        if pattern.contains('[') || pattern.contains(']') {
            return Err(ToolError::new("glob character classes are not supported"));
        }
        if let Some(path) = object.get("path") {
            if !path.is_string() {
                return Err(ToolError::new("path must be a string"));
            }
        }
        if let Some(limit) = object.get("limit") {
            let value = limit
                .as_u64()
                .ok_or_else(|| ToolError::new("limit must be a positive integer"))?;
            if value < 1 {
                return Err(ToolError::new("limit must be a positive integer"));
            }
        }
        Ok(())
    }

    fn execute(&mut self, arguments: &Value) -> Result<Value, ToolError> {
        let object = require_object(arguments)?;
        let pattern = require_string(object, "pattern")?;
        let path_arg = object.get("path").and_then(Value::as_str);
        let base = self.workspace.resolve(path_arg, true)?;
        if !base.is_dir() {
            return Err(ToolError::new(format!("not a directory: {path_arg:?}")));
        }
        let limit = object
            .get("limit")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_LIMIT);

        let mut matches = Vec::new();
        let mut stack = vec![base.clone()];
        while let Some(dir) = stack.pop() {
            let entries =
                std::fs::read_dir(&dir).map_err(|error| ToolError::new(error.to_string()))?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path.clone());
                }
                let rel = relative_posix(&base, &path);
                if glob_match(pattern, &rel) {
                    let mut label = self.workspace.relative_to_root(&path)?;
                    if path.is_dir() && !label.ends_with('/') {
                        label.push('/');
                    }
                    matches.push(label);
                }
            }
        }
        matches.sort();
        let truncated = matches.len() > limit;
        matches.truncate(limit);
        Ok(json!({
            "path": self.workspace.relative_to_root(&base)?,
            "pattern": pattern,
            "matches": matches,
            "count": matches.len(),
            "truncated": truncated,
        }))
    }
}

fn relative_posix(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// 按 POSIX 相对路径匹配 `*`、`**` 和 `?`。
pub fn glob_match(pattern: &str, path: &str) -> bool {
    // 教学实现：支持 *、?、** 的简化匹配。
    if pattern.contains('[') || pattern.contains(']') {
        return false;
    }
    let pattern = pattern.trim_start_matches("./");
    let path = path.trim_start_matches("./");
    match_components(&split_glob(pattern), &split_path(path))
}

fn split_glob(pattern: &str) -> Vec<&str> {
    pattern.split('/').filter(|part| !part.is_empty()).collect()
}

fn split_path(path: &str) -> Vec<&str> {
    path.split('/').filter(|part| !part.is_empty()).collect()
}

fn match_components(pattern: &[&str], path: &[&str]) -> bool {
    match (pattern.first().copied(), path.first().copied()) {
        (None, None) => true,
        (Some("**"), rest_pattern) => {
            if match_components(&pattern[1..], path) {
                return true;
            }
            if rest_pattern.is_some() {
                return match_components(pattern, &path[1..]);
            }
            false
        }
        (Some(p), Some(s)) => {
            if segment_match(p, s) {
                match_components(&pattern[1..], &path[1..])
            } else {
                false
            }
        }
        (Some(_), None) | (None, Some(_)) => false,
    }
}

fn segment_match(pattern: &str, segment: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let s: Vec<char> = segment.chars().collect();
    let mut dp = vec![vec![false; s.len() + 1]; p.len() + 1];
    dp[0][0] = true;
    for i in 1..=p.len() {
        if p[i - 1] == '*' {
            dp[i][0] = dp[i - 1][0];
        }
    }
    for i in 1..=p.len() {
        for j in 1..=s.len() {
            dp[i][j] = match p[i - 1] {
                '*' => dp[i - 1][j] || dp[i][j - 1],
                '?' => dp[i - 1][j - 1],
                ch => ch == s[j - 1] && dp[i - 1][j - 1],
            };
        }
    }
    dp[p.len()][s.len()]
}

#[allow(dead_code)]
fn join_rel(base: &Path, rel: &str) -> PathBuf {
    base.join(rel)
}
