"""工具注册表：查找、校验参数、执行并返回结构化结果。"""

from __future__ import annotations

from typing import Any, Mapping

from tool_types import Tool, ToolCall, ToolResult, ToolSpec, ToolStatus


class ToolRegistry:
    """按名称注册工具，并统一执行入口。"""

    def __init__(self) -> None:
        self._tools: dict[str, Tool] = {}

    def register(self, tool: Tool) -> None:
        """注册或覆盖同名工具。"""

        self._tools[tool.spec.name] = tool

    def get(self, name: str) -> Tool | None:
        """查找工具。"""

        return self._tools.get(name)

    def contains(self, name: str) -> bool:
        """是否已注册。"""

        return name in self._tools

    def specs(self) -> list[ToolSpec]:
        """列出全部工具规格。"""

        return [tool.spec for tool in self._tools.values()]

    # ANCHOR: m2-registry-execute
    def execute(self, call: ToolCall) -> ToolResult:
        """执行一次调用；未知工具、参数错误和执行失败都变成 Failed 结果。"""

        tool = self._tools.get(call.name)
        if tool is None:
            return ToolResult(
                call_id=call.call_id,
                name=call.name,
                status=ToolStatus.FAILED,
                error=f"unknown tool: {call.name}",
            )

        try:
            tool.validate_arguments(call.arguments)
        except Exception as error:  # noqa: BLE001 - 收成结构化失败
            return ToolResult(
                call_id=call.call_id,
                name=call.name,
                status=ToolStatus.FAILED,
                error=f"invalid arguments: {error}",
            )

        try:
            output = tool.execute(call.arguments)
        except Exception as error:  # noqa: BLE001 - 收成结构化失败
            return ToolResult(
                call_id=call.call_id,
                name=call.name,
                status=ToolStatus.FAILED,
                error=str(error) or error.__class__.__name__,
            )

        return ToolResult(
            call_id=call.call_id,
            name=call.name,
            status=ToolStatus.SUCCEEDED,
            output=output,
        )
    # ANCHOR_END: m2-registry-execute

    def execute_dict(
        self,
        *,
        call_id: str,
        name: str,
        arguments: Mapping[str, Any] | None = None,
    ) -> ToolResult:
        """从普通 dict 字段构造 ToolCall 并执行。"""

        return self.execute(
            ToolCall(
                call_id=call_id,
                name=name,
                arguments=dict(arguments or {}),
            )
        )
