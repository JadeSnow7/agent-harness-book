use crate::registry::{require_object, require_string};
use crate::tool_types::{Tool, ToolError, ToolSpec};
use crate::workspace::Workspace;
use serde_json::{Value, json};

const DEFAULT_MAX_LINES: usize = 2000;
const DEFAULT_MAX_BYTES: usize = 512_000;

/// 在 Workspace 内有界读取文本文件。
pub struct ReadTool {
    workspace: Workspace,
}

impl ReadTool {
    /// 绑定固定 Workspace。
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}

impl Tool for ReadTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "read".into(),
            description: "Read file contents inside the workspace".into(),
            input_schema: json!({
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {"type": "string"},
                    "offset": {"type": "integer"},
                    "limit": {"type": "integer"}
                },
                "additionalProperties": false
            }),
            strict: false,
        }
    }

    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError> {
        let object = require_object(arguments)?;
        let _ = require_string(object, "path")?;
        if let Some(offset) = object.get("offset") {
            let value = offset
                .as_u64()
                .ok_or_else(|| ToolError::new("offset must be a positive integer"))?;
            if value < 1 {
                return Err(ToolError::new("offset must be a positive integer"));
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
        let path_arg = require_string(object, "path")?;
        let path = self.workspace.resolve(Some(path_arg), true)?;
        if !path.is_file() {
            return Err(ToolError::new(format!("not a file: {path_arg:?}")));
        }
        let mut data = std::fs::read(&path).map_err(|error| ToolError::new(error.to_string()))?;
        let mut truncated = false;
        if data.len() > DEFAULT_MAX_BYTES {
            data.truncate(DEFAULT_MAX_BYTES);
            truncated = true;
        }
        let text = String::from_utf8_lossy(&data);
        let lines: Vec<&str> = text.lines().collect();
        let offset = object
            .get("offset")
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .max(1) as usize;
        let start = offset.saturating_sub(1);
        let limit = object
            .get("limit")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(DEFAULT_MAX_LINES);
        let end = (start + limit).min(lines.len());
        if end < lines.len() {
            truncated = true;
        }
        let body = lines[start..end]
            .iter()
            .enumerate()
            .map(|(index, line)| format!("{}: {line}", start + index + 1))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(json!({
            "path": self.workspace.relative_to_root(&path)?,
            "content": body,
            "line_count": end.saturating_sub(start),
            "truncated": truncated,
        }))
    }
}
