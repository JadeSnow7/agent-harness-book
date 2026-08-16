use crate::registry::require_object;
use crate::tool_types::{Tool, ToolError, ToolSpec};
use crate::workspace::Workspace;
use serde_json::{Value, json};

const DEFAULT_LIMIT: usize = 500;

/// 按名称列出 Workspace 中的一层目录项。
pub struct LsTool {
    workspace: Workspace,
}

impl LsTool {
    /// 绑定固定 Workspace。
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}

impl Tool for LsTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "ls".into(),
            description: "List directory contents inside the workspace".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
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
        let path_arg = object.get("path").and_then(Value::as_str);
        let path = self.workspace.resolve(path_arg, true)?;
        if !path.is_dir() {
            return Err(ToolError::new(format!("not a directory: {path_arg:?}")));
        }
        let limit = object
            .get("limit")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_LIMIT);
        let mut names = std::fs::read_dir(&path)
            .map_err(|error| ToolError::new(error.to_string()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        names.sort_by_key(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default()
        });
        let truncated = names.len() > limit;
        let entries = names
            .into_iter()
            .take(limit)
            .map(|path| {
                let mut label = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if path.is_dir() {
                    label.push('/');
                }
                Value::String(label)
            })
            .collect::<Vec<_>>();
        Ok(json!({
            "path": self.workspace.relative_to_root(&path)?,
            "entries": entries,
            "truncated": truncated,
            "count": entries.len(),
        }))
    }
}
