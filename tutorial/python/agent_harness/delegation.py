from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable

from ._base import ContractError


@dataclass(frozen=True)
class TaskSpec:
    task_id: str
    parent_run_id: str
    goal: str
    capabilities: frozenset[str]
    budget: int

    def __post_init__(self) -> None:
        if not self.task_id or not self.parent_run_id or not self.goal or self.budget <= 0:
            raise ValueError("task spec requires ids, a goal, and a positive budget")
        object.__setattr__(self, "capabilities", frozenset(self.capabilities))


@dataclass(frozen=True)
class SubRun:
    run_id: str
    parent_run_id: str
    task: TaskSpec


@dataclass(frozen=True)
class TaskResult:
    task_id: str
    run_id: str
    completed: bool
    evidence: tuple[Any, ...] = ()
    text: str = ""
    parent_run_id: str = ""
    steps: int = 0
    tools: int = 0

    def __post_init__(self) -> None:
        object.__setattr__(self, "evidence", tuple(self.evidence))


class SerialDelegateExecutor:
    """Deterministic serial delegation; child runs cannot impersonate parents."""

    def __init__(self, fn: Callable[[TaskSpec, SubRun], TaskResult],
                 allowed_capabilities: frozenset[str] | None = None):
        self.fn = fn
        self.allowed_capabilities = allowed_capabilities

    def run(self, parent_run_id: str, specs: list[TaskSpec] | tuple[TaskSpec, ...]) -> tuple[TaskResult, ...]:
        output: list[TaskResult] = []
        for number, spec in enumerate(specs, 1):
            if spec.parent_run_id != parent_run_id:
                raise ContractError("parent identity mismatch")
            if (self.allowed_capabilities is not None and
                    not spec.capabilities <= self.allowed_capabilities):
                raise ContractError("child requested a capability outside the parent grant")
            child = SubRun(f"{parent_run_id}.child.{number}", parent_run_id, spec)
            result = self.fn(spec, child)
            if result.run_id != child.run_id or result.task_id != spec.task_id:
                raise ContractError("child result identity mismatch")
            if result.parent_run_id not in ("", parent_run_id):
                raise ContractError("child result parent identity mismatch")
            if result.steps > spec.budget or result.tools > spec.budget:
                raise ContractError("child exceeded its independent budget")
            output.append(result)
        return tuple(output)


def aggregate(parent_run_id: str, results: list[TaskResult] | tuple[TaskResult, ...]) -> TaskResult:
    if not results:
        raise ContractError("cannot aggregate an empty delegation")
    if any(result.run_id == parent_run_id for result in results):
        raise ContractError("child identity required")
    if any(result.parent_run_id and result.parent_run_id != parent_run_id for result in results):
        raise ContractError("child result parent identity mismatch")
    if len({result.task_id for result in results}) != len(results):
        raise ContractError("duplicate child task result")
    if any(not result.completed or not result.evidence for result in results):
        raise ContractError("missing evidence prevents aggregation")
    return TaskResult(parent_run_id, parent_run_id, True,
                      tuple(item for result in results for item in result.evidence),
                      "; ".join(result.text for result in results if result.text))
