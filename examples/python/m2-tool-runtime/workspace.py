"""工作区路径约束与同目录原子替换：教学级边界，不是 OS 沙箱。"""

from __future__ import annotations

import os
from pathlib import Path
import tempfile


class WorkspaceError(ValueError):
    """路径越界或工作区配置错误。"""


class Workspace:
    """把用户提供的相对/绝对路径解析并限制在 root 内。"""

    def __init__(self, root: str | Path) -> None:
        self.root = Path(root).resolve()
        if not self.root.exists():
            raise WorkspaceError(f"workspace root does not exist: {self.root}")
        if not self.root.is_dir():
            raise WorkspaceError(f"workspace root is not a directory: {self.root}")

    def resolve(self, user_path: str | None = None, *, must_exist: bool = False) -> Path:
        """解析路径，并检查现有路径或最近现有父目录的真实位置。

        明确拒绝任何 ``..`` 分量。对于尚不存在的写入目标，从最近的现有
        父目录开始 canonicalize，因此外部 symlink 父目录也不能绕过边界。
        """

        if user_path is None or user_path.strip() == "" or user_path.strip() == ".":
            candidate = self.root
        else:
            raw = Path(user_path)
            if ".." in raw.parts:
                raise WorkspaceError(f"path escapes workspace root: {user_path!r}")
            candidate = raw if raw.is_absolute() else self.root / raw

        if candidate.exists():
            resolved = candidate.resolve(strict=True)
        else:
            missing: list[str] = []
            ancestor = candidate
            while not ancestor.exists():
                if ancestor.parent == ancestor:
                    raise WorkspaceError(f"cannot resolve path: {user_path!r}")
                missing.append(ancestor.name)
                ancestor = ancestor.parent
            resolved = ancestor.resolve(strict=True).joinpath(*reversed(missing))

        try:
            resolved.relative_to(self.root)
        except ValueError as error:
            raise WorkspaceError(
                f"path escapes workspace root: {user_path!r}"
            ) from error

        if must_exist and not resolved.exists():
            raise WorkspaceError(f"path does not exist: {user_path!r}")
        return resolved

    def relative_to_root(self, path: Path) -> str:
        """把绝对路径格式化为相对 root 的 posix 路径。"""

        return path.resolve().relative_to(self.root).as_posix() or "."

    def atomic_write_text(self, path: Path, content: str) -> None:
        """在目标同目录写临时文件，再用 ``os.replace`` 一次替换目标。

        同目录保证替换不会跨文件系统；失败时清理临时文件，原文件保持不变。
        这仍不解决 TOCTOU、挂载点变化或进程权限过大的生产级问题。
        """

        parent = self.resolve(str(path.parent))
        parent.mkdir(parents=True, exist_ok=True)
        checked = self.resolve(str(path))
        temp_name: str | None = None
        try:
            with tempfile.NamedTemporaryFile(
                mode="w",
                encoding="utf-8",
                dir=parent,
                prefix=f".{path.name}.",
                suffix=".tmp",
                delete=False,
            ) as handle:
                temp_name = handle.name
                handle.write(content)
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temp_name, checked)
            temp_name = None
        finally:
            if temp_name is not None:
                Path(temp_name).unlink(missing_ok=True)
