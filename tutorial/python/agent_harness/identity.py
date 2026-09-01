from __future__ import annotations

from dataclasses import dataclass

from ._base import IdentityMismatch, digest

SessionId = str
TaskId = str
RunId = str
StepId = str
EventId = str


@dataclass(frozen=True, order=True)
class Identity:
    """The immutable session/task/run scope attached to every event and effect."""

    session_id: str
    task_id: str
    run_id: str

    def __post_init__(self) -> None:
        if not all(isinstance(value, str) and value for value in
                   (self.session_id, self.task_id, self.run_id)):
            raise ValueError("identity fields must be non-empty strings")

    def matches(self, other: object) -> bool:
        return self == other

    def to_wire(self) -> dict[str, str]:
        return {"sessionId": self.session_id, "taskId": self.task_id, "runId": self.run_id}

    @classmethod
    def from_wire(cls, value: dict[str, str]) -> "Identity":
        return cls(value["sessionId"], value["taskId"], value["runId"])


def require_same(expected: Identity, actual: Identity) -> None:
    if expected != actual:
        raise IdentityMismatch(f"identity mismatch: expected {expected}, got {actual}")


def intent_digest(intent: object) -> str:
    return digest(intent)
