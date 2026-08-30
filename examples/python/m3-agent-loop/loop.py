"""M3：最小 Agent Loop。

独立于 P0 的 `DeterministicRunner` 组合（那是 Context/Policy/Runtime/
Validation/Evidence 一起验证的参考切片，见 book/src/ch5.md §5.6），这里
是 M3 自己的、只做控制流本身的实现。

每一轮：
  1. 先查取消——协作式取消在下一次模型调用之前生效，不打断正在执行的工具。
  2. 再查预算——预算耗尽必须发生在"下一次模型动作"之前，不能先多调用
     一次模型再退出。
  3. 构造输入、询问模型、记录动作。
  4. `Finish` 必须通过 Validation 才变成 `Completed`；不通过就是 `Failed`。
  5. `CallTool` 在真正执行前依次检查：重复 `call_id`（结构性非法）、
     未知工具（结构性非法）、Policy（业务性拒绝）——前两者终止为 `Failed`，
     第三者终止为 `PolicyDenied`。只有通过全部检查才会真正执行；执行失败
     是结构化观察，会被写回历史并继续循环，不会终止 run。
"""

from __future__ import annotations

import sys
from pathlib import Path

_M2 = Path(__file__).resolve().parents[1] / "m2-tool-runtime"
if str(_M2) not in sys.path:
    sys.path.append(str(_M2))

from registry import ToolRegistry  # noqa: E402

from loop_types import (
    CallTool,
    CancelToken,
    Event,
    Finish,
    Outcome,
    RunLimits,
    RunResult,
)
from policy import AllowListPolicy
from validator import RequiredOutputValidator


def run_loop(
    model,
    registry: ToolRegistry,
    policy: AllowListPolicy,
    validator: RequiredOutputValidator,
    limits: RunLimits,
    cancel_token: CancelToken | None = None,
) -> RunResult:
    """跑一次 Agent Loop，直到五种终态之一。"""

    token = cancel_token if cancel_token is not None else CancelToken()
    history: list[dict] = []
    events: list[Event] = []
    seen_call_ids: set[str] = set()
    steps = 0

    while True:
        if token.is_cancelled:
            events.append(Event("cancelled", None))
            return RunResult(
                outcome=Outcome.CANCELLED,
                reason="cancelled",
                events=tuple(events),
                model_call_count=steps,
            )

        if steps >= limits.max_steps:
            events.append(Event("budget_exhausted", {"max_steps": limits.max_steps}))
            return RunResult(
                outcome=Outcome.BUDGET_EXHAUSTED,
                reason="max_steps_exceeded",
                events=tuple(events),
                model_call_count=steps,
            )

        events.append(Event("model_input", list(history)))
        action = model.next_action(list(history))
        steps += 1
        events.append(Event("model_action", action))

        if isinstance(action, Finish):
            if validator.validate(action.output):
                events.append(Event("validation_passed", action.output))
                return RunResult(
                    outcome=Outcome.COMPLETED,
                    output=action.output,
                    events=tuple(events),
                    model_call_count=steps,
                )
            events.append(Event("validation_failed", action.output))
            return RunResult(
                outcome=Outcome.FAILED,
                reason="validation_failed",
                events=tuple(events),
                model_call_count=steps,
            )

        assert isinstance(action, CallTool)
        call = action.call

        if call.call_id in seen_call_ids:
            events.append(
                Event(
                    "invalid_action",
                    {"reason": "duplicate_call_id", "call_id": call.call_id},
                )
            )
            return RunResult(
                outcome=Outcome.FAILED,
                reason="duplicate_call_id",
                events=tuple(events),
                model_call_count=steps,
            )
        seen_call_ids.add(call.call_id)

        if not registry.contains(call.name):
            events.append(
                Event("invalid_action", {"reason": "unknown_tool", "tool": call.name})
            )
            return RunResult(
                outcome=Outcome.FAILED,
                reason="unknown_tool",
                events=tuple(events),
                model_call_count=steps,
            )

        if not policy.check(call.name):
            events.append(Event("policy_denied", {"tool": call.name}))
            return RunResult(
                outcome=Outcome.POLICY_DENIED,
                reason="policy_denied",
                events=tuple(events),
                model_call_count=steps,
            )

        result = registry.execute(call)
        events.append(Event("tool_result", result))
        history.append(
            {
                "role": "tool",
                "call_id": result.call_id,
                "name": result.name,
                "status": result.status.value,
                "output": result.output,
                "error": result.error,
            }
        )
