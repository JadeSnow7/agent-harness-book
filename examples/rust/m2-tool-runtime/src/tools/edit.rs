use crate::registry::{require_object, require_string};
use crate::tool_types::{Tool, ToolError, ToolSpec};
use crate::workspace::Workspace;
use serde_json::{Value, json};

/// 对唯一、非重叠 oldText 做精确替换。
pub struct EditTool {
    workspace: Workspace,
}

impl EditTool {
    /// 绑定固定 Workspace。
    pub fn new(workspace: Workspace) -> Self {
        Self { workspace }
    }
}

impl Tool for EditTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "edit".into(),
            description: "Apply exact text replacements inside a workspace file".into(),
            input_schema: json!({
                "type": "object",
                "required": ["path", "edits"],
                "properties": {
                    "path": {"type": "string"},
                    "edits": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["oldText", "newText"],
                            "properties": {
                                "oldText": {"type": "string"},
                                "newText": {"type": "string"}
                            },
                            "additionalProperties": false
                        }
                    }
                },
                "additionalProperties": false
            }),
            strict: false,
        }
    }

    fn validate_arguments(&self, arguments: &Value) -> Result<(), ToolError> {
        let object = require_object(arguments)?;
        let _ = require_string(object, "path")?;
        let edits = object
            .get("edits")
            .and_then(Value::as_array)
            .ok_or_else(|| ToolError::new("edits must be a non-empty list"))?;
        if edits.is_empty() {
            return Err(ToolError::new("edits must be a non-empty list"));
        }
        for (index, edit) in edits.iter().enumerate() {
            let item = edit
                .as_object()
                .ok_or_else(|| ToolError::new(format!("edits[{index}] must be an object")))?;
            let old = item.get("oldText").and_then(Value::as_str).ok_or_else(|| {
                ToolError::new(format!("edits[{index}].oldText must be a string"))
            })?;
            if old.is_empty() {
                return Err(ToolError::new(format!(
                    "edits[{index}].oldText must not be empty"
                )));
            }
            if item.get("newText").and_then(Value::as_str).is_none() {
                return Err(ToolError::new(format!(
                    "edits[{index}].newText must be a string"
                )));
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
        let original =
            std::fs::read_to_string(&path).map_err(|error| ToolError::new(error.to_string()))?;
        let mut updated = original.clone();
        let edits = object
            .get("edits")
            .and_then(Value::as_array)
            .ok_or_else(|| ToolError::new("edits must be a non-empty list"))?;
        let mut replacements = 0usize;
        for (index, edit) in edits.iter().enumerate() {
            let item = edit.as_object().unwrap();
            let old = item.get("oldText").and_then(Value::as_str).unwrap();
            let new = item.get("newText").and_then(Value::as_str).unwrap();
            let count = original.matches(old).count();
            if count == 0 {
                return Err(ToolError::new(format!("edits[{index}].oldText not found")));
            }
            if count > 1 {
                return Err(ToolError::new(format!(
                    "edits[{index}].oldText matches {count} times; must be unique"
                )));
            }
            if !updated.contains(old) {
                return Err(ToolError::new(format!(
                    "edits[{index}] overlaps a previous edit on the working copy"
                )));
            }
            updated = updated.replacen(old, new, 1);
            replacements += 1;
        }
        self.workspace.atomic_write(&path, updated.as_bytes())?;
        Ok(json!({
            "path": self.workspace.relative_to_root(&path)?,
            "replacements": replacements,
            "changed": updated != original,
        }))
    }
}
