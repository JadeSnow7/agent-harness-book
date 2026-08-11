"""Fake Transport 教学用的固定一步闭环；它不是 Agent Loop。

M1 不保留 reasoning 等未知输出项，因此本模块不应直接用于生产级真实多轮
Responses 调用。完整上下文保留或 previous_response_id 将在 M3 设计。
"""

from __future__ import annotations

from dataclasses import dataclass
import sys
from pathlib import Path

from bridge import result_to_message, spec_to_tool_definition, tool_use_to_call
from registry import ToolRegistry
from tool_types import ToolResult

_M1 = Path(__file__).resolve().parents[1] / "m1-unified-protocol"
if str(_M1) not in sys.path:
    sys.path.insert(0, str(_M1))

from chat_once import Config, HttpTransport, complete  # noqa: E402
from protocol import ModelRequest, ModelResponse  # noqa: E402


class OneStepError(RuntimeError):
    """固定闭环的形状不符合“恰好一次工具调用”时抛出。"""


@dataclass(frozen=True)
class OneStepResult:
    """保留闭环的关键证据，便于测试 call_id 和第二次请求。"""

    first_response: ModelResponse
    tool_result: ToolResult
    followup_request: ModelRequest
    final_response: ModelResponse


def request_with_registry_tools(
    request: ModelRequest, registry: ToolRegistry
) -> ModelRequest:
    """从 Registry 生成有序的模型工具声明，并复制其余统一请求字段。"""

    return ModelRequest(
        model=request.model,
        system=request.system,
        messages=request.messages,
        tools=tuple(spec_to_tool_definition(spec) for spec in registry.specs()),
    )


# ANCHOR: m2-one-step
def run_one_tool_step(
    request: ModelRequest,
    config: Config,
    transport: HttpTransport,
    registry: ToolRegistry,
) -> OneStepResult:
    """用可控教学传输调用两次，中间只允许并执行恰好一个工具候选。

    第一次没有工具、返回多个工具，或第二次再次要工具都会立即失败；本函数
    不重试、不循环，也不定义预算和停止策略。生产级真实响应还必须保留 M1
    未建模的输出项，本函数刻意不承担该职责。
    """

    first_request = request_with_registry_tools(request, registry)
    first = complete(first_request, config, transport)
    tool_uses = first.tool_uses()
    if len(tool_uses) != 1:
        raise OneStepError(
            f"first model call must request exactly one tool; got {len(tool_uses)}"
        )

    result = registry.execute(tool_use_to_call(tool_uses[0]))
    followup = ModelRequest(
        model=first_request.model,
        system=first_request.system,
        messages=first_request.messages + (first.message, result_to_message(result)),
        tools=first_request.tools,
    )
    final = complete(followup, config, transport)
    if final.tool_uses():
        raise OneStepError("second model call requested another tool; Agent Loop is M3")
    if not final.text():
        raise OneStepError("second model call returned no final text")
    return OneStepResult(first, result, followup, final)
# ANCHOR_END: m2-one-step
