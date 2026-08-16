"""edit：对工作区文件做精确文本替换。"""

from __future__ import annotations

from typing import Any, Mapping

from tool_types import ToolSpec
from workspace import Workspace, WorkspaceError


class EditTool:
    """每个 oldText 必须在原始文件中唯一出现（对齐 Pi 语义）。"""

    def __init__(self, workspace: Workspace) -> None:
        self.workspace = workspace
        self.spec = ToolSpec(
            name="edit",
            description="Apply exact text replacements inside a workspace file",
            input_schema={
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
                                "newText": {"type": "string"},
                            },
                            "additionalProperties": False,
                        },
                    },
                },
                "additionalProperties": False,
            },
        )

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        path = arguments.get("path")
        if not isinstance(path, str) or not path.strip():
            raise ValueError("path must be a non-empty string")
        edits = arguments.get("edits")
        if not isinstance(edits, list) or not edits:
            raise ValueError("edits must be a non-empty list")
        for index, edit in enumerate(edits):
            if not isinstance(edit, Mapping):
                raise ValueError(f"edits[{index}] must be an object")
            if not isinstance(edit.get("oldText"), str):
                raise ValueError(f"edits[{index}].oldText must be a string")
            if not isinstance(edit.get("newText"), str):
                raise ValueError(f"edits[{index}].newText must be a string")
            if edit["oldText"] == "":
                raise ValueError(f"edits[{index}].oldText must not be empty")

    def execute(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        path = self.workspace.resolve(str(arguments["path"]), must_exist=True)
        if not path.is_file():
            raise WorkspaceError(f"not a file: {arguments['path']!r}")

        original = path.read_text(encoding="utf-8")
        updated = original
        replacements = 0

        # 所有 oldText 相对原始内容匹配，避免增量重叠语义。
        for index, edit in enumerate(arguments["edits"]):
            old = str(edit["oldText"])
            new = str(edit["newText"])
            count = original.count(old)
            if count == 0:
                raise ValueError(f"edits[{index}].oldText not found")
            if count > 1:
                raise ValueError(
                    f"edits[{index}].oldText matches {count} times; must be unique"
                )
            if old not in updated:
                raise ValueError(
                    f"edits[{index}] overlaps a previous edit on the working copy"
                )
            updated = updated.replace(old, new, 1)
            replacements += 1

        self.workspace.atomic_write_text(path, updated)
        return {
            "path": self.workspace.relative_to_root(path),
            "replacements": replacements,
            "changed": updated != original,
        }
