"""write：在工作区内创建或覆写文件。"""

from __future__ import annotations

from typing import Any, Mapping

from tool_types import ToolSpec
from workspace import Workspace


class WriteTool:
    """写入完整文件内容。"""

    def __init__(self, workspace: Workspace) -> None:
        self.workspace = workspace
        self.spec = ToolSpec(
            name="write",
            description="Create or overwrite a file inside the workspace",
            input_schema={
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"},
                },
                "additionalProperties": False,
            },
        )

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        path = arguments.get("path")
        if not isinstance(path, str) or not path.strip():
            raise ValueError("path must be a non-empty string")
        if "content" not in arguments or not isinstance(arguments.get("content"), str):
            raise ValueError("content must be a string")

    def execute(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        path = self.workspace.resolve(str(arguments["path"]))
        path.parent.mkdir(parents=True, exist_ok=True)
        content = str(arguments["content"])
        self.workspace.atomic_write_text(path, content)
        return {
            "path": self.workspace.relative_to_root(path),
            "bytes_written": len(content.encode("utf-8")),
        }
