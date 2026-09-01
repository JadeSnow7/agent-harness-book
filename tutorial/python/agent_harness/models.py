from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass(frozen=True)
class Session:
    session_id: str
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class Task:
    task_id: str
    session_id: str
    goal: str


@dataclass(frozen=True)
class Run:
    run_id: str
    task_id: str
    session_id: str
    status: str = "running"


@dataclass(frozen=True)
class Step:
    run_id: str
    step_id: str
    seq: int
    status: str = "running"
