"""Pi 风格的七个基础工具实现。"""

from __future__ import annotations

from registry import ToolRegistry
from tools.bash import BashTool
from tools.edit import EditTool
from tools.find import FindTool
from tools.grep import GrepTool
from tools.ls import LsTool
from tools.read import ReadTool
from tools.write import WriteTool
from workspace import Workspace


def build_default_registry(
    workspace: Workspace,
    *,
    enable_bash: bool = False,
    bash_allowlist: list[str] | None = None,
) -> ToolRegistry:
    """注册七工具；bash 默认关闭。"""

    registry = ToolRegistry()
    registry.register(ReadTool(workspace))
    registry.register(WriteTool(workspace))
    registry.register(EditTool(workspace))
    registry.register(GrepTool(workspace))
    registry.register(FindTool(workspace))
    registry.register(LsTool(workspace))
    registry.register(
        BashTool(
            workspace,
            enabled=enable_bash,
            allowlist=bash_allowlist,
        )
    )
    return registry
