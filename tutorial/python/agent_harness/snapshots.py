from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from ._base import canonical


@dataclass(frozen=True)
class Snapshot:
    seq: int
    schema: str
    state: dict[str, Any]

    def __post_init__(self) -> None:
        if self.seq < 0 or not self.schema:
            raise ValueError("snapshot requires a non-negative sequence and schema")

    def to_json(self) -> str:
        return canonical(self)

    def to_dict(self) -> dict[str, Any]:
        return {"seq": self.seq, "schema": self.schema, "state": self.state}

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> "Snapshot":
        return cls(int(value["seq"]), value["schema"], value["state"])
