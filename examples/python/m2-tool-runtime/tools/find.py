"""find：在工作区内按 glob 查找文件。"""

from __future__ import annotations

from typing import Any, Mapping

from tool_types import ToolSpec
from tools.glob_match import glob_matches
from workspace import Workspace, WorkspaceError

DEFAULT_LIMIT = 1000


class FindTool:
    """递归枚举后使用教学 glob 子集匹配 POSIX 相对路径。"""

    def __init__(self, workspace: Workspace) -> None:
        self.workspace = workspace
        self.spec = ToolSpec(
            name="find",
            description="Find files by glob pattern inside the workspace",
            input_schema={
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"},
                    "limit": {"type": "integer"},
                },
                "additionalProperties": False,
            },
        )

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        pattern = arguments.get("pattern")
        if not isinstance(pattern, str) or not pattern.strip():
            raise ValueError("pattern must be a non-empty string")
        path = arguments.get("path")
        if path is not None and not isinstance(path, str):
            raise ValueError("path must be a string")
        limit = arguments.get("limit")
        if limit is not None and (not isinstance(limit, int) or isinstance(limit, bool) or limit < 1):
            raise ValueError("limit must be a positive integer")

    def execute(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        base = self.workspace.resolve(arguments.get("path"), must_exist=True)
        if not base.is_dir():
            raise WorkspaceError(f"not a directory: {arguments.get('path')!r}")

        pattern = str(arguments["pattern"])
        # 在开始遍历前验证不支持的字符类，失败不会产生半截结果。
        glob_matches(pattern, "")
        limit = int(arguments.get("limit") or DEFAULT_LIMIT)
        all_matches: list[str] = []
        for path in sorted(base.rglob("*")):
            if not path.is_file() and not path.is_dir():
                continue
            try:
                self.workspace.resolve(str(path))
            except WorkspaceError:
                continue
            base_rel = path.resolve().relative_to(base).as_posix()
            if not glob_matches(pattern, base_rel):
                continue
            rel = path.resolve().relative_to(self.workspace.root).as_posix()
            if path.is_dir():
                rel += "/"
            all_matches.append(rel)

        truncated = len(all_matches) > limit
        matches = all_matches[:limit]
        return {
            "path": self.workspace.relative_to_root(base),
            "pattern": pattern,
            "matches": matches,
            "count": len(matches),
            "truncated": truncated,
        }
