"""Provider-neutral wire objects used by the Python teaching harness."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Protocol, TYPE_CHECKING

from ._base import ContractError, from_data

if TYPE_CHECKING:
    from .tool_descriptor import ToolDescriptor


class Role(str, Enum):
    SYSTEM = "system"
    USER = "user"
    ASSISTANT = "assistant"
    TOOL = "tool"


class EffectKind(str, Enum):
    READ = "read"
    EFFECT = "effect"


ToolKind = EffectKind


class CallStatus(str, Enum):
    OK = "ok"
    ERROR = "error"
    SUCCEEDED = "ok"  # compatibility spelling used by the first examples
    FAILED = "error"


@dataclass(frozen=True)
class ConversationContinuation:
    """Opaque provider state; the neutral core never interprets state."""

    provider: str
    state: Any


@dataclass(frozen=True)
class Message:
    role: Role
    content: str
    name: str | None = None

    def to_wire(self) -> dict[str, Any]:
        result = {"role": self.role.value, "content": self.content}
        if self.name is not None:
            result["name"] = self.name
        return result

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "Message":
        return cls(Role(value["role"]), value["content"], value.get("name"))


@dataclass(frozen=True)
class ToolCall:
    call_id: str
    tool_id: str
    arguments: dict[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.call_id or not self.tool_id:
            raise ValueError("tool calls require call_id and tool_id")

    def to_wire(self) -> dict[str, Any]:
        return {"callId": self.call_id, "toolId": self.tool_id, "arguments": self.arguments}

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "ToolCall":
        return cls(value["callId"], value["toolId"], value.get("arguments", {}))


@dataclass(frozen=True)
class ToolResult:
    call_id: str
    status: CallStatus
    output: Any = None
    error: dict[str, Any] | None = None

    @property
    def ok(self) -> bool:
        return self.status is CallStatus.OK

    def __post_init__(self) -> None:
        if self.ok and self.error is not None:
            raise ContractError("successful tool result cannot contain error")
        if not self.ok and self.error is None:
            raise ContractError("failed tool result requires structured error")

    def to_wire(self) -> dict[str, Any]:
        return {"callId": self.call_id, "status": self.status.value,
                "output": self.output, "error": self.error}

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "ToolResult":
        return cls(value["callId"], CallStatus(value["status"]),
                   value.get("output"), value.get("error"))


@dataclass(frozen=True)
class EffectIntent:
    call_id: str
    tool_id: str
    domain: str
    kind: EffectKind
    scope: str
    policy_digest: str
    run_id: str
    arguments: dict[str, Any] = field(default_factory=dict)
    change_set_hash: str | None = None
    validation_digest: str | None = None
    evidence_digest: str | None = None

    def __post_init__(self) -> None:
        if not all(isinstance(value, str) and value for value in
                   (self.call_id, self.tool_id, self.domain, self.scope,
                    self.policy_digest, self.run_id)):
            raise ValueError("effect intent identifiers must be non-empty")
        if not isinstance(self.kind, EffectKind):
            object.__setattr__(self, "kind", EffectKind(self.kind))

    def to_wire(self) -> dict[str, Any]:
        return {"callId": self.call_id, "toolId": self.tool_id, "domain": self.domain,
                "kind": self.kind.value, "scope": self.scope, "policyDigest": self.policy_digest,
                "runId": self.run_id, "arguments": self.arguments,
                "changeSetHash": self.change_set_hash, "validationDigest": self.validation_digest,
                "evidenceDigest": self.evidence_digest}

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "EffectIntent":
        return cls(value["callId"], value["toolId"], value["domain"], EffectKind(value["kind"]),
                   value["scope"], value["policyDigest"], value["runId"],
                   value.get("arguments", {}), value.get("changeSetHash"),
                   value.get("validationDigest"), value.get("evidenceDigest"))


@dataclass(frozen=True)
class PolicyDecision:
    call_id: str
    decision: str
    reason: str
    approval_id: str | None = None

    def __post_init__(self) -> None:
        if self.decision not in {"allow", "deny", "ask"}:
            raise ValueError("decision must be allow, deny, or ask")
        if self.decision == "deny" and not self.reason:
            raise ContractError("deny decisions require a reason")

    def to_wire(self) -> dict[str, Any]:
        return {"callId": self.call_id, "decision": self.decision,
                "reason": self.reason, "approvalId": self.approval_id}

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "PolicyDecision":
        return cls(value["callId"], value["decision"], value["reason"], value.get("approvalId"))


@dataclass(frozen=True)
class EffectResult:
    call_id: str
    status: CallStatus
    output: Any = None
    error: dict[str, Any] | None = None
    domain_payload: dict[str, Any] | None = None

    @property
    def ok(self) -> bool:
        return self.status is CallStatus.OK

    def to_wire(self) -> dict[str, Any]:
        return {"callId": self.call_id, "status": self.status.value, "output": self.output,
                "error": self.error, "domainPayload": self.domain_payload}

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "EffectResult":
        return cls(value["callId"], CallStatus(value["status"]), value.get("output"),
                   value.get("error"), value.get("domainPayload"))


@dataclass(frozen=True)
class ModelRequest:
    messages: tuple[Message, ...]
    model: str = "fake"
    instruction: str = ""
    tools: tuple["ToolDescriptor", ...] = ()

    def __post_init__(self) -> None:
        object.__setattr__(self, "messages", tuple(self.messages))
        object.__setattr__(self, "tools", tuple(self.tools))

    def to_wire(self) -> dict[str, Any]:
        return {
            "messages": [message.to_wire() for message in self.messages],
            "model": self.model,
            "instruction": self.instruction,
            "tools": [descriptor.to_wire() for descriptor in self.tools],
        }

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "ModelRequest":
        from .tool_descriptor import ToolDescriptor
        return cls(tuple(Message.from_wire(item) for item in value.get("messages", ())),
                   value.get("model", "fake"), value.get("instruction", ""),
                   tuple(ToolDescriptor.from_wire(item) for item in value.get("tools", ())))


@dataclass(frozen=True)
class ModelResponse:
    text: str = ""
    tool_calls: tuple[ToolCall, ...] = ()
    stop_reason: str = "stop"
    usage: dict[str, int] = field(default_factory=dict)
    continuation: ConversationContinuation | None = None

    def __post_init__(self) -> None:
        object.__setattr__(self, "tool_calls", tuple(self.tool_calls))

    def to_wire(self) -> dict[str, Any]:
        result = {"text": self.text, "toolCalls": [call.to_wire() for call in self.tool_calls],
                  "stopReason": self.stop_reason, "usage": self.usage}
        if self.continuation is not None:
            result["continuation"] = {"provider": self.continuation.provider,
                                      "state": self.continuation.state}
        return result

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "ModelResponse":
        continuation = value.get("continuation")
        opaque = (ConversationContinuation(continuation["provider"], continuation["state"])
                  if continuation is not None else None)
        return cls(value.get("text", ""),
                   tuple(ToolCall.from_wire(item) for item in value.get("toolCalls", ())),
                   value.get("stopReason", "stop"), value.get("usage", {}), opaque)


class ModelProvider(Protocol):
    def complete(self, request: ModelRequest) -> ModelResponse:
        ...


def request_payload(request: ModelRequest) -> dict[str, Any]:
    return request.to_wire()


def response_from_wire(value: dict[str, Any]) -> ModelResponse:
    return ModelResponse.from_wire(value)
