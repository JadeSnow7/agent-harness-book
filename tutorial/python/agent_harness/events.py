from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
from typing import Any

from ._base import ContractError, digest
from .identity import Identity, require_same


class EventKind:
    RUN_STARTED = "run.started"
    MODEL_REQUESTED = "model.requested"
    MODEL_RESPONDED = "model.responded"
    TOOL_STARTED = "tool.started"
    TOOL_RESULT = "tool.result"
    EFFECT_INTENT = "effect.intent"
    POLICY_DECISION = "policy.decision"
    RUN_FINISHED = "run.finished"


@dataclass(frozen=True)
class EventEnvelope:
    seq: int
    kind: str
    identity: Identity
    payload: dict[str, Any]
    terminal: bool = False
    event_id: str = ""
    schema: str = "v1"

    def __post_init__(self) -> None:
        if self.seq < 1 or not self.kind or not isinstance(self.payload, dict):
            raise ValueError("event requires a positive seq, kind, and object payload")
        if not self.event_id:
            object.__setattr__(self, "event_id", f"{self.identity.run_id}:{self.seq}")
        if not self.schema:
            raise ValueError("event schema is required")

    def to_wire(self) -> dict[str, Any]:
        return {"seq": self.seq, "eventId": self.event_id, "schema": self.schema,
                "kind": self.kind, "identity": self.identity.to_wire(),
                "payload": deepcopy(self.payload), "terminal": self.terminal}

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "EventEnvelope":
        terminal = value.get("terminal", False)
        if not isinstance(terminal, bool):
            raise ContractError("event terminal must be boolean")
        return cls(int(value["seq"]), value["kind"], Identity.from_wire(value["identity"]),
                   value.get("payload", {}), terminal,
                   value.get("eventId", ""), value.get("schema", "v1"))


class EventLog:
    """Append-only, identity-scoped event stream.

    The log is intentionally in-memory.  Its invariants are the lesson; a
    durable store belongs to a later runtime implementation.
    """

    def __init__(self, identity: Identity | None = None):
        self._events: list[EventEnvelope] = []
        self.identity = identity

    @property
    def events(self) -> tuple[EventEnvelope, ...]:
        return tuple(self._events)

    @property
    def seq(self) -> int:
        return len(self._events)

    @property
    def terminal(self) -> bool:
        return bool(self._events and self._events[-1].terminal)

    def append(self, event: EventEnvelope) -> EventEnvelope:
        if self.identity is None:
            self.identity = event.identity
        require_same(self.identity, event.identity)
        if event.seq != self.seq + 1:
            raise ContractError("sequence gap or out-of-order event")
        if self.terminal:
            raise ContractError("cannot append after terminal event")
        if event.terminal and event.kind not in {EventKind.RUN_FINISHED, "done", "finished"}:
            raise ContractError("only a run-finished event may be terminal")
        self._events.append(EventEnvelope(event.seq, event.kind, event.identity,
                                           deepcopy(event.payload), event.terminal,
                                           event.event_id, event.schema))
        return event

    def validate(self) -> bool:
        probe = EventLog(self.identity)
        for event in self._events:
            probe.append(event)
        return True

    def replay(self, provider=None, tool_registry=None, validator=None) -> tuple[EventEnvelope, ...]:
        # Replay never calls a provider or tool.  It validates and returns the
        # recorded facts for a deterministic state projection.
        self.validate()
        return tuple(self._events)

    def rebuild_state(self) -> dict[str, Any]:
        self.validate()
        state: dict[str, Any] = {}
        for event in self._events:
            if isinstance(event.payload.get("state"), dict):
                state = deepcopy(event.payload["state"])
            state.update({key: value for key, value in event.payload.items() if key != "state"})
        return state

    def digest(self) -> str:
        return digest([event.to_wire() for event in self._events])


EventStore = EventLog
