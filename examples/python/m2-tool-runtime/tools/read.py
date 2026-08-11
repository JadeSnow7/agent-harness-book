"""read：读取工作区内文件内容。"""

from __future__ import annotations

from typing import Any, Mapping

from tool_types import ToolSpec
from workspace import Workspace, WorkspaceError

DEFAULT_MAX_LINES = 2000
DEFAULT_MAX_BYTES = 512_000


class ReadTool:
    """按路径读取文本文件，支持 offset/limit 截断。"""

    def __init__(self, workspace: Workspace) -> None:
        self.workspace = workspace
        self.spec = ToolSpec(
            name="read",
            description="Read file contents inside the workspace",
            input_schema={
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": {"type": "string"},
                    "offset": {"type": "integer"},
                    "limit": {"type": "integer"},
                },
                "additionalProperties": False,
            },
        )

    # ANCHOR: m2-read-tool
    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        path = arguments.get("path")
        if not isinstance(path, str) or not path.strip():
            raise ValueError("path must be a non-empty string")
        offset = arguments.get("offset")
        if offset is not None and (not isinstance(offset, int) or isinstance(offset, bool) or offset < 1):
            raise ValueError("offset must be a positive integer")
        limit = arguments.get("limit")
        if limit is not None and (not isinstance(limit, int) or isinstance(limit, bool) or limit < 1):
            raise ValueError("limit must be a positive integer")

    def execute(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        path = self.workspace.resolve(str(arguments["path"]), must_exist=True)
        if not path.is_file():
            raise WorkspaceError(f"not a file: {arguments['path']!r}")

        data = path.read_bytes()
        truncated_bytes = False
        if len(data) > DEFAULT_MAX_BYTES:
            data = data[:DEFAULT_MAX_BYTES]
            truncated_bytes = True

        text = data.decode("utf-8", errors="replace")
        lines = text.splitlines()
        offset = int(arguments.get("offset") or 1)
        start = max(offset - 1, 0)
        limit = arguments.get("limit")
        max_lines = int(limit) if limit is not None else DEFAULT_MAX_LINES
        selected = lines[start : start + max_lines]
        truncated_lines = start + len(selected) < len(lines)

        body = "\n".join(
            f"{start + index + 1}: {line}" for index, line in enumerate(selected)
        )
        return {
            "path": self.workspace.relative_to_root(path),
            "content": body,
            "line_count": len(selected),
            "truncated": truncated_bytes or truncated_lines,
        }
    # ANCHOR_END: m2-read-tool
