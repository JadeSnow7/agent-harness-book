from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from ._base import ContractError, digest, redact
from .events import EventEnvelope
from .identity import Identity


@dataclass(frozen=True)
class Evidence:
    evidence_id: str
    run_id: str
    kind: str
    value: Any
    supporting_events: tuple[int, ...] = ()
    identity: Identity | None = None

    def __post_init__(self) -> None:
        object.__setattr__(self, "supporting_events", tuple(self.supporting_events))

    @property
    def digest(self) -> str:
        return digest(self)


@dataclass(frozen=True)
class EvidenceRef:
    evidence_id: str
    evidence_digest: str
    run_id: str


@dataclass(frozen=True)
class Trace:
    run_id: str
    events: tuple[EventEnvelope, ...]


@dataclass(frozen=True)
class Summary:
    run_id: str
    evidence_ids: tuple[str, ...]
    text: str
    trace_digest: str = ""


@dataclass(frozen=True)
class ReviewBundle:
    """Digest binding for review: ChangeSet + Validation + Evidence."""

    change_set_digest: str
    validation_digest: str
    evidence_digest: str

    @property
    def bundle_digest(self) -> str:
        return digest(self)

    @classmethod
    def bind(cls, change_set: Any, validation: Any,
             evidence: list[Evidence] | tuple[Evidence, ...] | Any) -> "ReviewBundle":
        change_digest = change_set.hash() if hasattr(change_set, "hash") else digest(change_set)
        validation_digest = getattr(validation, "digest", None) or digest(validation)
        if isinstance(evidence, (list, tuple)):
            evidence_digest = digest(evidence)
        else:
            evidence_digest = getattr(evidence, "digest", None) or digest(evidence)
        return cls(change_digest, validation_digest, evidence_digest)


class EvidenceStore:
    def __init__(self):
        self._values: dict[str, Evidence] = {}

    def add(self, evidence: Evidence) -> Evidence:
        safe = Evidence(evidence.evidence_id, evidence.run_id, evidence.kind,
                        redact(evidence.value), evidence.supporting_events, evidence.identity)
        old = self._values.get(safe.evidence_id)
        if old is not None and old != safe:
            raise ContractError("evidence id already contains different content")
        self._values[safe.evidence_id] = safe
        return safe

    def for_run(self, run_id: str) -> tuple[Evidence, ...]:
        return tuple(value for value in self._values.values() if value.run_id == run_id)


def project_evidence(evidence: Evidence, events: list[EventEnvelope] | tuple[EventEnvelope, ...]) -> Evidence:
    matching = {event.seq: event for event in events if event.identity.run_id == evidence.run_id}
    if evidence.identity is not None:
        matching = {seq: event for seq, event in matching.items()
                    if event.identity == evidence.identity}
    if any(event.identity.run_id != evidence.run_id for event in events) and not matching:
        raise ValueError("evidence belongs to another run")
    missing = set(evidence.supporting_events) - set(matching)
    if missing:
        raise ValueError(f"supporting event sequence is missing: {sorted(missing)}")
    return Evidence(evidence.evidence_id, evidence.run_id, evidence.kind,
                    redact(evidence.value), evidence.supporting_events, evidence.identity)


def trace_projection(run_id: str, events: list[EventEnvelope] | tuple[EventEnvelope, ...]) -> Trace:
    projected = []
    for event in events:
        if event.identity.run_id == run_id:
            projected.append(EventEnvelope(event.seq, event.kind, event.identity,
                                            redact(event.payload), event.terminal,
                                            event.event_id, event.schema))
    return Trace(run_id, tuple(projected))


def summary_projection(run_id: str, evidence: list[Evidence] | tuple[Evidence, ...],
                       trace: Trace | None = None) -> Summary:
    selected = tuple(item for item in evidence if item.run_id == run_id)
    return Summary(run_id, tuple(item.evidence_id for item in selected),
                   "\n".join(f"{item.kind}: {redact(item.value)}" for item in selected),
                   digest(trace.events) if trace is not None else "")
