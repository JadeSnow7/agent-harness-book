"""M3：最小 Agent Loop 的类型。

这些类型独立于 P0 的 `DeterministicRunner` 组合，也独立于 M1 的 HTTP 协议
机制——M3 只关心"模型动作 → 工具/校验 → 下一步"这条控制流本身，不关心
真实模型的请求/响应格式。工具相关类型直接复用 M2 的 `tool_types`（同一份
`ToolCall`/`ToolResult`/`ToolStatus`），避免重复定义同一个概念。
"""

from __future__ import annotations

import sys
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Any

_M2 = Path(__file__).resolve().parents[1] / "m2-tool-runtime"
if str(_M2) not in sys.path:
    sys.path.append(str(_M2))

from tool_types import ToolCall  # noqa: E402


class Outcome(str, Enum):
    """五种互斥终态；一次 run 只会返回其中之一。

    互斥性是结构性的：`RunResult.outcome` 是单个字段，循环里每条终止路径
    只执行一次 `return`，不存在同时设置两个终态的代码路径。
    """

    COMPLETED = "completed"
    FAILED = "failed"
    BUDGET_EXHAUSTED = "budget_exhausted"
    POLICY_DENIED = "policy_denied"
    CANCELLED = "cancelled"


@dataclass(frozen=True)
class Finish:
    """模型声明"我想结束"。这只是意图，还需要通过 Validation 才变成 Completed。"""

    output: str


@dataclass(frozen=True)
class CallTool:
    """模型请求一次工具调用。"""

    call: ToolCall


ModelAction = Finish | CallTool


@dataclass(frozen=True)
class RunLimits:
    """预算边界；M3 只建模步数，token/时间/成本预算留给后续里程碑。"""

    max_steps: int


@dataclass(frozen=True)
class Event:
    """一次可观察的循环内部事件（模型输入、模型动作、工具结果、终止原因等）。"""

    kind: str
    detail: Any = None


@dataclass(frozen=True)
class RunResult:
    """一次 run 的最终结果；`outcome` 决定其余字段如何解读。"""

    outcome: Outcome
    output: str | None = None
    reason: str | None = None
    events: tuple[Event, ...] = field(default_factory=tuple)
    model_call_count: int = 0


class CancelToken:
    """协作式取消标志：循环在每轮开始时检查，不做抢占式中断。"""

    def __init__(self) -> None:
        self._cancelled = False

    def cancel(self) -> None:
        """请求取消；可在工具执行过程中调用，模拟运行中途收到取消信号。"""

        self._cancelled = True

    @property
    def is_cancelled(self) -> bool:
        """是否已被请求取消。"""

        return self._cancelled
