from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any


class StopReason(str, Enum):
    CONTINUE = "continue"
    COMPLETED = "completed"
    BUDGET_EXHAUSTED = "budget_exhausted"
    STALLED = "stalled"
    ESCALATE = "escalate"
    CANCELLED = "cancelled"


class ProgressSignal(str, Enum):
    PROGRESS = "progress"
    NO_PROGRESS = "no_progress"


NoProgress = ProgressSignal.NO_PROGRESS
BudgetExhausted = StopReason.BUDGET_EXHAUSTED
DuplicateAction = StopReason.STALLED
EscalationRequired = StopReason.ESCALATE


@dataclass(frozen=True)
class StopDecision:
    reason: StopReason
    message: str = ""


class ProgressOracle:
    def __init__(self, step_limit: int, tool_limit: int = 100):
        if step_limit <= 0 or tool_limit <= 0:
            raise ValueError("progress limits must be positive")
        self.step_limit = step_limit
        self.tool_limit = tool_limit
        self.steps = 0
        self.tools = 0
        self.last_observation: Any = None
        self.repeated = 0
        self.evidence_count = 0

    def observe(self, observation: Any, tool: bool = False, evidence: bool = False) -> None:
        self.steps += 1
        if tool:
            self.tools += 1
        if evidence:
            self.evidence_count += 1
        if observation == self.last_observation and not evidence:
            self.repeated += 1
        else:
            self.repeated = 0
        self.last_observation = observation


class StopPolicy:
    def __init__(self, max_stalled: int = 2, step_limit: int | None = None,
                 tool_limit: int = 100):
        self.max_stalled = max_stalled
        self.step_limit = step_limit
        self.tool_limit = tool_limit

    def decide(self, oracle: ProgressOracle, cancelled: bool = False,
               escalated: bool = False, completed: bool = False) -> StopReason:
        if cancelled:
            return StopReason.CANCELLED
        if completed:
            return StopReason.COMPLETED
        if escalated:
            return StopReason.ESCALATE
        if oracle.steps >= oracle.step_limit or oracle.tools >= oracle.tool_limit:
            return StopReason.BUDGET_EXHAUSTED
        if oracle.repeated >= self.max_stalled:
            return StopReason.STALLED
        return StopReason.CONTINUE
