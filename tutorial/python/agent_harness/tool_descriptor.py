from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from ._base import ContractError, digest
from .protocol import EffectKind


@dataclass(frozen=True)
class ToolDescriptor:
    """The explicit capability declaration shown to a model and policy layer."""

    id: str
    domain: str
    kind: EffectKind
    input_schema: dict[str, Any]
    output_schema: dict[str, Any]
    source: str = "tutorial"
    version: str = "1"

    def __post_init__(self) -> None:
        if not self.id or not self.domain or not self.source or not self.version:
            raise ValueError("tool descriptor identifiers must be non-empty")
        if not isinstance(self.kind, EffectKind):
            object.__setattr__(self, "kind", EffectKind(self.kind))
        for name, schema in (("input_schema", self.input_schema), ("output_schema", self.output_schema)):
            if not isinstance(schema, dict) or schema.get("type", "object") not in {
                "object", "array", "string", "integer", "number", "boolean", "null"
            }:
                raise ContractError(f"{name} must be a supported JSON schema")

    def to_wire(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "domain": self.domain,
            "kind": self.kind.value,
            "inputSchema": self.input_schema,
            "outputSchema": self.output_schema,
            "source": self.source,
            "version": self.version,
        }

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> "ToolDescriptor":
        return cls(value["id"], value["domain"], EffectKind(value["kind"]),
                   value["inputSchema"], value["outputSchema"],
                   value.get("source", "tutorial"), value.get("version", "1"))

    @property
    def digest(self) -> str:
        return digest(self.to_wire())
