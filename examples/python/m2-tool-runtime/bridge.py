"""与 M1 统一协议内容块之间的轻量桥接。"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Mapping

from tool_types import ToolCall, ToolResult, ToolStatus

# 允许从相邻 M1 示例导入统一协议类型。
_M1 = Path(__file__).resolve().parents[1] / "m1-unified-protocol"
if str(_M1) not in sys.path:
    sys.path.insert(0, str(_M1))

from protocol import (  # noqa: E402
    Message,
    Role,
    ToolDefinition,
    ToolResultBlock,
    ToolUseBlock,
)


def spec_to_tool_definition(spec) -> ToolDefinition:
    """把 Runtime ToolSpec 变成模型可见的 provider-neutral 工具声明。"""

    return ToolDefinition(
        name=spec.name,
        description=spec.description,
        input_schema=dict(spec.input_schema),
        strict=spec.strict,
    )


def tool_use_to_call(block: ToolUseBlock) -> ToolCall:
    """把 M1 ToolUseBlock 转成 Runtime ToolCall。"""

    return ToolCall(call_id=block.id, name=block.name, arguments=dict(block.input))


def call_from_dict(data: Mapping[str, Any]) -> ToolCall:
    """从简单 JSON 对象构造 ToolCall。"""

    call_id = data.get("call_id") or data.get("id")
    name = data.get("name") or data.get("tool")
    arguments = data.get("arguments") or data.get("input") or {}
    if not isinstance(call_id, str) or not call_id.strip():
        raise ValueError("call_id is required")
    if not isinstance(name, str) or not name.strip():
        raise ValueError("name is required")
    if isinstance(arguments, str):
        arguments = json.loads(arguments)
    if not isinstance(arguments, Mapping):
        raise ValueError("arguments must be an object")
    return ToolCall(call_id=call_id, name=name, arguments=dict(arguments))


def result_to_tool_result_block(result: ToolResult) -> ToolResultBlock:
    """把 Runtime 结果编码为 M1 ToolResultBlock。"""

    return ToolResultBlock(
        tool_use_id=result.call_id,
        content=result.as_text(),
        is_error=result.status is ToolStatus.FAILED,
    )


def result_to_message(result: ToolResult) -> Message:
    """构造 role=tool 的统一消息。"""

    return Message(role=Role.TOOL, content=(result_to_tool_result_block(result),))
