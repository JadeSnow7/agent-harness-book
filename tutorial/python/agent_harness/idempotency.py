from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from threading import RLock
from typing import Any
from uuid import uuid4

from ._base import AmbiguousResult, ContractError, IdentityMismatch


class Outcome(str, Enum):
    RESERVED = "reserved"
    IN_PROGRESS = "in_progress"
    EXECUTING = "executing"
    COMPLETED = "completed"
    FAILED = "failed"
    AMBIGUOUS = "ambiguous"


@dataclass(frozen=True)
class LedgerRecord:
    seq: int
    key: str
    identity: object
    intent_digest: str
    outcome: Outcome
    result: Any = None
    token: str = ""

    @property
    def terminal(self) -> bool:
        return self.outcome in {Outcome.COMPLETED, Outcome.FAILED, Outcome.AMBIGUOUS}


class IdempotencyLedger:
    """Thread-safe append-only ledger for reservation and fail-closed replay."""

    def __init__(self):
        self.records: list[LedgerRecord] = []
        self._lock = RLock()

    def _latest(self, key: str) -> LedgerRecord | None:
        return next((record for record in reversed(self.records) if record.key == key), None)

    def reserve(self, key: str, identity: object, intent_digest: str) -> LedgerRecord:
        if not key or not intent_digest:
            raise ContractError("idempotency key and intent digest are required")
        with self._lock:
            old = self._latest(key)
            if old is not None:
                if old.identity != identity or old.intent_digest != intent_digest:
                    raise IdentityMismatch("key identity or intent digest mismatch")
                if old.terminal:
                    return old
                return LedgerRecord(old.seq, old.key, old.identity, old.intent_digest,
                                    Outcome.IN_PROGRESS, old.result, old.token)
            record = LedgerRecord(len(self.records) + 1, key, identity, intent_digest,
                                  Outcome.RESERVED, token=uuid4().hex)
            self.records.append(record)
            return record

    def _transition(self, key: str, outcome: Outcome, result: Any = None,
                    token: str | None = None) -> LedgerRecord:
        with self._lock:
            old = self._latest(key)
            if old is None:
                raise ContractError(f"unknown idempotency key: {key}")
            if old.terminal:
                return old
            if token is not None and token != old.token:
                raise IdentityMismatch("reservation token mismatch")
            if outcome is Outcome.RESERVED:
                raise ContractError("a reserved operation cannot be reserved again")
            if outcome in {Outcome.COMPLETED, Outcome.FAILED} and old.outcome is Outcome.RESERVED:
                raise ContractError("operation must enter executing before terminal result")
            if outcome is Outcome.EXECUTING and old.outcome not in {
                    Outcome.RESERVED, Outcome.IN_PROGRESS}:
                raise ContractError("invalid executing transition")
            if outcome is Outcome.AMBIGUOUS:
                result = None
            record = LedgerRecord(len(self.records) + 1, key, old.identity,
                                  old.intent_digest, outcome, result, old.token)
            self.records.append(record)
            return record

    def executing(self, key: str, token: str | None = None) -> LedgerRecord:
        if token is None:
            raise IdentityMismatch("executing requires the reservation token")
        return self._transition(key, Outcome.EXECUTING, token=token)

    def complete(self, key: str, result: Any, token: str | None = None) -> LedgerRecord:
        if token is None:
            raise IdentityMismatch("complete requires the reservation token")
        return self._transition(key, Outcome.COMPLETED, result, token)

    def fail(self, key: str, error: Any, token: str | None = None) -> LedgerRecord:
        if token is None:
            raise IdentityMismatch("fail requires the reservation token")
        return self._transition(key, Outcome.FAILED, error, token)

    def ambiguous(self, key: str, token: str | None = None) -> LedgerRecord:
        return self._transition(key, Outcome.AMBIGUOUS, token=token)

    def transition(self, key: str, outcome: Outcome, result: Any = None) -> LedgerRecord:
        old = self._latest(key)
        outcome = Outcome(outcome)
        if old is not None and old.outcome is Outcome.RESERVED and outcome in {
                Outcome.COMPLETED, Outcome.FAILED}:
            self._transition(key, Outcome.EXECUTING, token=old.token)
        old = self._latest(key)
        return self._transition(key, outcome, result, old.token if old else None)

    def mark(self, key: str, outcome: Outcome, result: Any = None) -> LedgerRecord:
        return self.transition(key, outcome, result)

    def lookup(self, key: str) -> LedgerRecord | None:
        with self._lock:
            return self._latest(key)

    def unknown(self, key: str) -> LedgerRecord:
        return self.ambiguous(key)

    def require_result(self, key: str) -> Any:
        record = self.lookup(key)
        if record is None or record.outcome is not Outcome.COMPLETED:
            raise AmbiguousResult(f"result is not known to be completed: {key}")
        return record.result
