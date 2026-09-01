from __future__ import annotations

import difflib
from dataclasses import dataclass
from typing import Any

from ._base import ContractError, digest
from .protocol import EffectKind, EffectIntent, EffectResult, CallStatus


@dataclass(frozen=True)
class ReadEffect:
    target: str
    kind: EffectKind = EffectKind.READ


@dataclass(frozen=True)
class Projection:
    domain: str
    value: dict[str, Any]


@dataclass(frozen=True)
class Mutation:
    mutation_id: str
    domain: str
    scope: str
    changeset_hash: str


@dataclass(frozen=True)
class ChangeSet:
    target: str
    before: str
    after: str
    diff: str
    reason: str
    risk: str
    expected_hash: str
    domain: str = "generic"
    change_set_id: str = ""
    scope: str = ""

    @classmethod
    def create(cls, target: str, before: str, after: str, reason: str, risk: str,
               domain: str = "generic", change_set_id: str = "",
               scope: str = "") -> "ChangeSet":
        diff = "".join(difflib.unified_diff(
            before.splitlines(True), after.splitlines(True),
            fromfile=target + " (before)", tofile=target + " (after)"))
        return cls(target, before, after, diff, reason, risk, digest(before),
                   domain, change_set_id or digest((target, before, after))[:16], scope)

    def hash(self) -> str:
        return digest(self)

    @property
    def changeset_id(self) -> str:
        return self.change_set_id


class EffectApplier:
    """Apply a ChangeSet once against the exact expected projection."""

    def __init__(self):
        self.results: dict[str, dict[str, Any]] = {}

    def apply(self, change: ChangeSet, current: str) -> dict[str, Any]:
        key = change.hash()
        if key in self.results:
            return self.results[key]
        if digest(current) != change.expected_hash:
            raise ContractError("expected hash mismatch; stale ChangeSet rejected")
        result = {
            "effectId": key,
            "target": change.target,
            "before": current,
            "after": change.after,
            "diff": change.diff,
            "domain": change.domain,
            "status": "applied",
        }
        self.results[key] = result
        return result


class EffectPipeline:
    """Minimal explicit EffectIntent -> decision -> result seam."""

    def __init__(self, applier: EffectApplier | None = None):
        self.applier = applier or EffectApplier()

    def apply(self, intent: EffectIntent, decision: Any, change: ChangeSet,
              current: str) -> EffectResult:
        if getattr(decision, "decision", decision) != "allow":
            return EffectResult(intent.call_id, CallStatus.ERROR,
                                error={"code": "policy_denied", "message": "effect not allowed"})
        if intent.domain != change.domain and change.domain != "generic":
            return EffectResult(intent.call_id, CallStatus.ERROR,
                                error={"code": "effect_binding_mismatch", "message": "domain mismatch"})
        if change.scope and intent.scope != change.scope:
            return EffectResult(intent.call_id, CallStatus.ERROR,
                                error={"code": "effect_binding_mismatch", "message": "scope mismatch"})
        if intent.change_set_hash and intent.change_set_hash != change.hash():
            return EffectResult(intent.call_id, CallStatus.ERROR,
                                error={"code": "effect_binding_mismatch", "message": "ChangeSet digest mismatch"})
        try:
            return EffectResult(intent.call_id, CallStatus.OK,
                                output=self.applier.apply(change, current),
                                domain_payload={"domain": change.domain})
        except ContractError as exc:
            return EffectResult(intent.call_id, CallStatus.ERROR,
                                error={"code": "effect_rejected", "message": str(exc)})
