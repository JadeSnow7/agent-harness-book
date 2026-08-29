"""M3：一个最小的、有预算的 Agent Loop 教学原型；不是生产实现。

第 4 章的 ``one_step`` 只处理固定两次调用：第一次必须恰好请求一个工具，
第二次必须给出最终文本，否则立即失败。本模块把这个固定形状换成一个循环，
但仍然刻意保持最小：串行、每轮至多一个工具候选、没有并行工具调用、没有
Context 裁剪、没有 Policy、没有持久化。这些限制会分别在后续章节被讨论。

循环遇到"这一轮该怎么办"这类它没有被授权自行处理的情况时，一律安全停止，
不猜测、不重试：一轮出现多个工具候选需要人工复核；模型响应解码失败（比如
Provider 返回了无法识别的 JSON 形状）会被捕获而不是让循环崩溃退出。
"""

from __future__ import annotations

import sys
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

_M2 = Path(__file__).resolve().parents[1] / "m2-tool-runtime"
if str(_M2) not in sys.path:
    sys.path.insert(0, str(_M2))

from bridge import result_to_message, tool_use_to_call  # noqa: E402
from registry import ToolRegistry  # noqa: E402
from tool_types import ToolResult  # noqa: E402

# one_step 模块被导入时会把相邻的 m1-unified-protocol 目录插入 sys.path；
# 复用它已经验证过的插入时机和 request_with_registry_tools，不重新实现一遍。
from one_step import request_with_registry_tools  # noqa: E402

from chat_once import Config, HttpTransport  # noqa: E402
from chat_once import complete as _complete  # noqa: E402
from protocol import ModelRequest, ModelResponse, ProtocolError  # noqa: E402


class StopReason(str, Enum):
    """循环结束的显式原因；是返回结果的一部分，不是只写进日志里。

    这一版只有以下四个变体，明确不包含 PolicyDenied 和 Cancelled：
    PolicyDenied 依赖第 10 章才会引入的 Policy 引擎，Cancelled 依赖第 7、13
    章才会引入的 Session/取消控制。读者现在不需要理解也不需要实现它们——
    它们不是本章遗漏，而是有意留给对应章节。
    """

    COMPLETED = "completed"
    BUDGET_EXHAUSTED = "budget_exhausted"
    AMBIGUOUS_TOOL_REQUEST = "ambiguous_tool_request"
    UNRECOGNIZED_ACTION = "unrecognized_action"


@dataclass(frozen=True)
class AgentLoopLimits:
    """循环预算：两个独立边界，缺一个都可能让循环失控。

    ``max_steps`` 限制模型调用总轮数；``max_tool_calls`` 限制实际执行的
    工具次数。两者分别检查，因为"模型被问了很多轮但工具用得很少"和"工具
    用得很多但轮数不多"是两种不同的失控方式。
    """

    max_steps: int
    max_tool_calls: int


@dataclass(frozen=True)
class StepRecord:
    """一轮迭代留下的证据：这一轮模型看到了什么、循环做了什么。

    ``tool_result`` 只在这一轮实际执行了工具时才非空；停止的那一轮（无论
    因为完成、预算耗尽、多候选还是解码失败）都不会再执行工具。
    """

    response: ModelResponse | None
    tool_result: ToolResult | None


@dataclass(frozen=True)
class AgentLoopResult:
    """循环的最终结果。

    ``error`` 只在 ``stop_reason`` 是 ``UNRECOGNIZED_ACTION`` 且由 Provider
    解码失败触发时非空，保留可安全展示的异常信息，不回显请求头或凭据。
    """

    stop_reason: StopReason
    steps: tuple[StepRecord, ...]
    final_text: str | None
    last_response: ModelResponse | None
    error: str | None = None


# ANCHOR: m3-agent-loop
def run_agent_loop(
    request: ModelRequest,
    config: Config,
    transport: HttpTransport,
    registry: ToolRegistry,
    limits: AgentLoopLimits,
) -> AgentLoopResult:
    """在预算内反复调用模型、执行恰好一个工具候选，直到显式停止。

    每一轮都必须能回答"模型看到了什么、提出了什么、Runtime 做了什么"：
    工具执行失败会被 Registry 收敛成结构化 ``ToolResult``，作为下一轮的观察
    正常进入循环，不会中断循环；一轮出现多个工具候选，或者这一轮的模型
    响应根本无法解码，都被视为循环没有被授权自行处理的情况，立即安全
    停止，不选择、不重试、不吞掉异常。
    """

    messages = list(request.messages)
    steps: list[StepRecord] = []
    tool_calls_made = 0
    step_count = 0

    while True:
        if step_count >= limits.max_steps:
            return AgentLoopResult(
                stop_reason=StopReason.BUDGET_EXHAUSTED,
                steps=tuple(steps),
                final_text=None,
                last_response=None,
            )
        step_count += 1

        step_request = request_with_registry_tools(
            ModelRequest(
                model=request.model,
                system=request.system,
                messages=tuple(messages),
                tools=request.tools,
            ),
            registry,
        )

        try:
            response = _complete(step_request, config, transport)
        except ProtocolError as error:
            steps.append(StepRecord(response=None, tool_result=None))
            return AgentLoopResult(
                stop_reason=StopReason.UNRECOGNIZED_ACTION,
                steps=tuple(steps),
                final_text=None,
                last_response=None,
                error=str(error),
            )

        tool_uses = response.tool_uses()

        if len(tool_uses) > 1:
            steps.append(StepRecord(response=response, tool_result=None))
            return AgentLoopResult(
                stop_reason=StopReason.AMBIGUOUS_TOOL_REQUEST,
                steps=tuple(steps),
                final_text=None,
                last_response=response,
            )

        if len(tool_uses) == 1:
            if tool_calls_made >= limits.max_tool_calls:
                steps.append(StepRecord(response=response, tool_result=None))
                return AgentLoopResult(
                    stop_reason=StopReason.BUDGET_EXHAUSTED,
                    steps=tuple(steps),
                    final_text=None,
                    last_response=response,
                )
            call = tool_use_to_call(tool_uses[0])
            result = registry.execute(call)
            tool_calls_made += 1
            messages.append(response.message)
            messages.append(result_to_message(result))
            steps.append(StepRecord(response=response, tool_result=result))
            continue

        text = response.text()
        if text:
            steps.append(StepRecord(response=response, tool_result=None))
            return AgentLoopResult(
                stop_reason=StopReason.COMPLETED,
                steps=tuple(steps),
                final_text=text,
                last_response=response,
            )

        # 防御分支：M1 当前的 decode_response 保证成功解码的响应必然满足
        # "有工具候选或有非空文本"之一，这条分支在真实 complete() 链路下
        # 不可达；保留它是因为 run_agent_loop 不应该假设未来所有 decoder
        # 都维持同一条不变量。测试直接构造 ModelResponse 覆盖这条分支，
        # 不通过 ScriptedTransport，因为经由真实链路造不出这种响应。
        steps.append(StepRecord(response=response, tool_result=None))
        return AgentLoopResult(
            stop_reason=StopReason.UNRECOGNIZED_ACTION,
            steps=tuple(steps),
            final_text=None,
            last_response=response,
        )
# ANCHOR_END: m3-agent-loop
