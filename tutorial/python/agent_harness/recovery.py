from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from .idempotency import Outcome


class RecoveryAction(str, Enum):
    RETRY = "retry"
    RESUME = "resume"
    COMPENSATE = "compensate"
    STOP = "stop"


class FailPoint(str, Enum):
    BEFORE = "before"
    AFTER = "after"
    UNKNOWN = "unknown"


@dataclass(frozen=True)
class RecoveryDecision:
    action: RecoveryAction
    reason: str


def decide(outcome: Outcome, fail_point: FailPoint | None = None) -> RecoveryDecision:
    outcome = Outcome(outcome)
    if outcome is Outcome.AMBIGUOUS or fail_point is FailPoint.UNKNOWN:
        return RecoveryDecision(RecoveryAction.STOP, "unknown side-effect result; do not replay")
    if fail_point is FailPoint.AFTER or outcome is Outcome.COMPLETED:
        return RecoveryDecision(RecoveryAction.RESUME, "effect may already have happened")
    if outcome is Outcome.FAILED:
        return RecoveryDecision(RecoveryAction.RETRY, "failure is known before completion")
    return RecoveryDecision(RecoveryAction.STOP, "run is not retryable")
