"""grep：在工作区内搜索文件内容。"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Any, Mapping

from tool_types import ToolSpec
from tools.glob_match import glob_matches
from workspace import Workspace, WorkspaceError

DEFAULT_LIMIT = 100
MAX_FILE_BYTES = 1_000_000


class GrepTool:
    """简单 Python 实现的内容搜索；不依赖外部 rg。"""

    def __init__(self, workspace: Workspace) -> None:
        self.workspace = workspace
        self.spec = ToolSpec(
            name="grep",
            description="Search file contents inside the workspace",
            input_schema={
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"},
                    "glob": {"type": "string"},
                    "ignoreCase": {"type": "boolean"},
                    "literal": {"type": "boolean"},
                    "context": {"type": "integer"},
                    "limit": {"type": "integer"},
                },
                "additionalProperties": False,
            },
        )

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        pattern = arguments.get("pattern")
        if not isinstance(pattern, str) or pattern == "":
            raise ValueError("pattern must be a non-empty string")
        for key in ("path", "glob"):
            value = arguments.get(key)
            if value is not None and not isinstance(value, str):
                raise ValueError(f"{key} must be a string")
        for key in ("ignoreCase", "literal"):
            value = arguments.get(key)
            if value is not None and not isinstance(value, bool):
                raise ValueError(f"{key} must be a boolean")
        for key in ("context", "limit"):
            value = arguments.get(key)
            if value is not None and (
                not isinstance(value, int) or isinstance(value, bool) or value < 0
            ):
                raise ValueError(f"{key} must be a non-negative integer")

    def execute(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        root = self.workspace.resolve(arguments.get("path"), must_exist=True)
        pattern_text = str(arguments["pattern"])
        literal = bool(arguments.get("literal", False))
        ignore_case = bool(arguments.get("ignoreCase", False))
        context = int(arguments.get("context") or 0)
        limit = int(arguments.get("limit") or DEFAULT_LIMIT)
        glob_pat = arguments.get("glob") or "**/*"

        flags = re.MULTILINE
        if ignore_case:
            flags |= re.IGNORECASE
        try:
            regex = (
                re.compile(re.escape(pattern_text), flags)
                if literal
                else re.compile(pattern_text, flags)
            )
        except re.error as error:
            raise ValueError(f"invalid regex: {error}") from error

        files = self._iter_files(root, str(glob_pat))
        matches: list[dict[str, Any]] = []
        truncated = False

        for file_path in files:
            if len(matches) >= limit:
                truncated = True
                break
            try:
                if file_path.stat().st_size > MAX_FILE_BYTES:
                    continue
                text = file_path.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError):
                continue

            lines = text.splitlines()
            rel = self.workspace.relative_to_root(file_path)
            for line_no, line in enumerate(lines, start=1):
                if not regex.search(line):
                    continue
                start = max(0, line_no - 1 - context)
                end = min(len(lines), line_no + context)
                snippet = "\n".join(
                    f"{idx + 1}: {lines[idx]}" for idx in range(start, end)
                )
                matches.append(
                    {
                        "path": rel,
                        "line": line_no,
                        "text": line,
                        "snippet": snippet,
                    }
                )
                if len(matches) >= limit:
                    truncated = True
                    break

        return {
            "pattern": pattern_text,
            "matches": matches,
            "count": len(matches),
            "truncated": truncated,
        }

    def _iter_files(self, root: Path, glob_pat: str) -> list[Path]:
        if root.is_file():
            return [root]
        if not root.is_dir():
            raise WorkspaceError(f"not a file or directory: {root}")
        results: list[Path] = []
        # 与 Rust 一样先递归枚举，再使用共享的 glob 子集匹配相对路径。
        for path in sorted(root.rglob("*")):
            if path.is_file():
                self.workspace.resolve(str(path))
                relative = path.resolve().relative_to(root).as_posix()
                if glob_matches(glob_pat, relative):
                    results.append(path)
        return results
