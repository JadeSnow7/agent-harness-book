"""ls：列出工作区目录内容。"""

from __future__ import annotations

from typing import Any, Mapping

from tool_types import ToolSpec
from workspace import Workspace, WorkspaceError

DEFAULT_LIMIT = 500


class LsTool:
    """按字母序列出目录项；目录名带 / 后缀。"""

    def __init__(self, workspace: Workspace) -> None:
        self.workspace = workspace
        self.spec = ToolSpec(
            name="ls",
            description="List directory contents inside the workspace",
            input_schema={
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "limit": {"type": "integer"},
                },
                "additionalProperties": False,
            },
        )

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        path = arguments.get("path")
        if path is not None and not isinstance(path, str):
            raise ValueError("path must be a string")
        limit = arguments.get("limit")
        if limit is not None and (not isinstance(limit, int) or isinstance(limit, bool) or limit < 1):
            raise ValueError("limit must be a positive integer")

    def execute(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        path = self.workspace.resolve(arguments.get("path"), must_exist=True)
        if not path.is_dir():
            raise WorkspaceError(f"not a directory: {arguments.get('path')!r}")

        limit = int(arguments.get("limit") or DEFAULT_LIMIT)
        names = sorted(path.iterdir(), key=lambda item: item.name.lower())
        entries: list[str] = []
        for item in names[:limit]:
            label = item.name + ("/" if item.is_dir() else "")
            entries.append(label)

        return {
            "path": self.workspace.relative_to_root(path),
            "entries": entries,
            "truncated": len(names) > limit,
            "count": len(entries),
        }
