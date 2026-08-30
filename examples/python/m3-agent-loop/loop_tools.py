"""M3 测试用工具：确定性成功、确定性失败、执行时触发取消。

复用 M2 的 `Tool` 协议（`spec` 属性 + `validate_arguments` + `execute`），
这样 `ToolRegistry.execute` 的失败收敛行为（未知工具/参数错误/执行异常都
变成结构化 `ToolResult`）可以直接给 M3 用，不用重新实现一遍。
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any, Mapping

_M2 = Path(__file__).resolve().parents[1] / "m2-tool-runtime"
if str(_M2) not in sys.path:
    sys.path.append(str(_M2))

from tool_types import ToolSpec  # noqa: E402

from loop_types import CancelToken


class EchoTool:
    """把输入的 value 原样返回；确定性成功路径。"""

    @property
    def spec(self) -> ToolSpec:
        return ToolSpec(name="echo", description="Echo back the given value.")

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        if "value" not in arguments:
            raise ValueError("echo requires 'value'")

    def execute(self, arguments: Mapping[str, Any]) -> Any:
        return arguments["value"]


class FailingTool:
    """总是失败；用于验证工具失败会变成结构化观察，而不是被异常吞掉整个 run。"""

    @property
    def spec(self) -> ToolSpec:
        return ToolSpec(name="fail", description="Always fails deterministically.")

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        return None

    def execute(self, arguments: Mapping[str, Any]) -> Any:
        raise RuntimeError("deterministic failure for testing")


class CancellingTool:
    """执行时翻转给定的 `CancelToken`；用于测试运行中途被取消。"""

    def __init__(self, token: CancelToken) -> None:
        self._token = token

    @property
    def spec(self) -> ToolSpec:
        return ToolSpec(
            name="cancel_trigger", description="Flips the cancel token on execute."
        )

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        return None

    def execute(self, arguments: Mapping[str, Any]) -> Any:
        self._token.cancel()
        return "cancel requested"
