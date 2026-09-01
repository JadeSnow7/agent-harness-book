from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any

from ._base import ContractError, digest, redact
from .tool_definition import validate_schema


class Lifecycle(str, Enum):
    REGISTERED = "registered"
    ACTIVE = "active"
    REVOKED = "revoked"
    FAILED = "failed"


@dataclass(frozen=True)
class CapabilityManifest:
    name: str
    version: str
    source: str
    capabilities: frozenset[str]
    schema: dict[str, Any]

    def __post_init__(self) -> None:
        if not self.name or not self.version or not self.source or not isinstance(self.schema, dict):
            raise ContractError("invalid manifest")
        object.__setattr__(self, "capabilities", frozenset(self.capabilities))

    def fingerprint(self) -> str:
        return digest(self)


ExtensionManifest = CapabilityManifest


@dataclass(frozen=True)
class HarnessProfile:
    name: str
    tool_ids: frozenset[str]
    extension_names: frozenset[str]
    allow_effects: bool = False

    def __post_init__(self) -> None:
        if not self.name:
            raise ContractError("profile name is required")
        object.__setattr__(self, "tool_ids", frozenset(self.tool_ids))
        object.__setattr__(self, "extension_names", frozenset(self.extension_names))


def compose_profile(name: str, tool_ids: list[str] | tuple[str, ...],
                    extensions: list[Extension] | tuple[Extension, ...],
                    allow_effects: bool = False) -> HarnessProfile:
    names = [extension.manifest.name for extension in extensions]
    if len(set(names)) != len(names):
        raise ContractError("duplicate extension identity")
    return HarnessProfile(name, frozenset(tool_ids), frozenset(names), allow_effects)


@dataclass
class Extension:
    manifest: CapabilityManifest
    lifecycle: Lifecycle = Lifecycle.REGISTERED

    def activate(self) -> None:
        self.lifecycle = Lifecycle.ACTIVE

    def revoke(self) -> None:
        self.lifecycle = Lifecycle.REVOKED

    def fail(self) -> None:
        self.lifecycle = Lifecycle.FAILED

    def invoke(self, payload: dict[str, Any], timeout: float | None = None) -> dict[str, Any]:
        if self.lifecycle is not Lifecycle.ACTIVE:
            raise RuntimeError("extension unavailable")
        if timeout is not None and timeout <= 0:
            raise TimeoutError("extension timeout")
        if not isinstance(payload, dict):
            raise ValueError("schema requires object payload")
        error = validate_schema(self.manifest.schema, payload)
        if error:
            raise ValueError(error)
        return {"ok": True, "extension": self.manifest.name,
                "payload": redact(payload)}


def capability_diff(old: CapabilityManifest, new: CapabilityManifest) -> dict[str, Any]:
    return {
        "added": sorted(new.capabilities - old.capabilities),
        "removed": sorted(old.capabilities - new.capabilities),
        "version_changed": old.version != new.version,
        "source_changed": old.source != new.source,
        "manifest_changed": old.fingerprint() != new.fingerprint(),
    }


class FakeMCP(Extension):
    """Deterministic fake MCP adapter used by CH16 composition tests."""


class FakeSkill(Extension):
    """Deterministic fake skill adapter."""


class FakeHook(Extension):
    """Deterministic fake hook adapter."""


class FakePlugin(Extension):
    """Deterministic fake plugin adapter."""


class ExtensionGateway:
    """The only composition seam exposed to an extension in this lesson.

    The gateway makes policy, validation, and evidence explicit. Calling an
    Extension directly remains useful for the CH15 lifecycle demo, but it is
    not presented as a complete runtime path.
    """

    def __init__(self, extension: Extension, validator: Any, evidence_store: Any):
        self.extension = extension
        self.validator = validator
        self.evidence_store = evidence_store

    def invoke(self, payload: dict[str, Any], decision: Any,
               run_id: str, evidence_id: str) -> tuple[dict[str, Any], Any]:
        if getattr(decision, "decision", decision) != "allow":
            raise PermissionError("extension invocation denied by policy")
        value = self.extension.invoke(payload)
        report = self.validator.validate(value)
        if not report.passed:
            raise ValueError("extension output failed validation")
        from .evidence import Evidence
        evidence = self.evidence_store.add(
            Evidence(evidence_id, run_id, "extension.output", value))
        return value, evidence
