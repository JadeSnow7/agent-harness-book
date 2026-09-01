from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any

from ._base import ContractError, digest
from .protocol import EffectKind, EffectIntent, PolicyDecision


class Decision(str, Enum):
    ALLOW = "allow"
    DENY = "deny"
    ASK = "ask"


@dataclass(frozen=True)
class Approval:
    approval_id: str
    run_id: str
    call_id: str
    changeset_hash: str
    validation_digest: str
    evidence_digest: str
    approver: str

    @property
    def digest(self) -> str:
        return digest(self)


def approval_digest(changeset_hash: str, validation_digest: str,
                    evidence_digest: str) -> str:
    return digest({"changeSet": changeset_hash, "validation": validation_digest,
                   "evidence": evidence_digest})


def bind_approval(approval: Approval, run_id: str, call_id: str,
                  changeset_hash: str, validation_digest: str,
                  evidence_digest: str) -> bool:
    expected = (run_id, call_id, changeset_hash, validation_digest, evidence_digest)
    actual = (approval.run_id, approval.call_id, approval.changeset_hash,
              approval.validation_digest, approval.evidence_digest)
    if actual != expected:
        raise ContractError("approval binding mismatch")
    if not approval.approver or not approval.approval_id:
        raise ContractError("approval requires an approver and id")
    return True


class ApprovalStore:
    def __init__(self):
        self._approvals: dict[str, Approval] = {}

    def record(self, approval: Approval) -> Approval:
        old = self._approvals.get(approval.approval_id)
        if old is not None and old != approval:
            raise ContractError("approval id already binds different content")
        self._approvals[approval.approval_id] = approval
        return approval

    def get(self, approval_id: str) -> Approval | None:
        return self._approvals.get(approval_id)


def authorize(intent: Any, allow_effects: bool = False,
              approval: Approval | None = None) -> PolicyDecision:
    if not isinstance(intent, EffectIntent):
        return PolicyDecision("", Decision.DENY.value, "invalid effect intent")
    if intent.kind is EffectKind.READ:
        return PolicyDecision(intent.call_id, Decision.ALLOW.value, "read-only")
    if approval is not None:
        if None in (intent.change_set_hash, intent.validation_digest, intent.evidence_digest):
            return PolicyDecision(intent.call_id, Decision.DENY.value,
                                  "approval is not bound to a ChangeSet, Validation, and Evidence")
        try:
            bind_approval(approval, intent.run_id, intent.call_id,
                          intent.change_set_hash, intent.validation_digest,
                          intent.evidence_digest)
        except ContractError as exc:
            return PolicyDecision(intent.call_id, Decision.DENY.value, str(exc))
        return PolicyDecision(intent.call_id, Decision.ALLOW.value,
                              "bound approval", approval.approval_id)
    if allow_effects:
        return PolicyDecision(intent.call_id, Decision.ALLOW.value, "explicit policy")
    return PolicyDecision(intent.call_id, Decision.ASK.value, "effect requires approval")
