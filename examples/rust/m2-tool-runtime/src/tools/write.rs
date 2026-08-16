use crate::registry::{require_object, require_string};
use crate::tool_types::{Tool, ToolError, ToolSpec};
use crate::workspace::Workspace;
use serde_json::{Value, json};

/// 以同目录原子替换方式写入完整文件。
pub struct WriteTool {
    workspace: Workspace,
}

impl WriteTool {
    /// 绑定固定 Workspace。
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}

impl Tool for WriteTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "write".into(),
            description: "Create or overwrite a file inside the workspace".into(),
            input_schema: json!({
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                },
                "additionalProperties": false
            }),
            strict: false,
        }
    }

    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError> {
        let object = require_object(arguments)?;
        let _ = require_string(object, "path")?;
        object
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("content must be a string"))?;
        Ok(())
    }

    fn execute(&mut self, arguments: &Value) -> Result<Value, ToolError> {
        let object = require_object(arguments)?;
        let path_arg = require_string(object, "path")?;
        let content = object
            .get("content")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("content must be a string"))?;
        let path = self.workspace.resolve(Some(path_arg), false)?;
        self.workspace.atomic_write(&path, content.as_bytes())?;
        Ok(json!({
            "path": self.workspace.relative_to_root(&path)?,
            "bytes_written": content.len(),
        }))
    }
}
