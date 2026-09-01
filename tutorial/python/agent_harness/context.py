from __future__ import annotations

from dataclasses import dataclass
from enum import Enum, IntEnum

from ._base import ContractError


class Source(str, Enum):
    SYSTEM = "system"
    USER = "user"
    MEMORY = "memory"
    TOOL = "tool"
    MODEL = "model"


ContextSource = Source


class ContextPriority(IntEnum):
    LOW = 1
    NORMAL = 10
    HIGH = 100
    REQUIRED = 1000


@dataclass(frozen=True)
class ContextBudget:
    max_bytes: int

    def __post_init__(self) -> None:
        if self.max_bytes < 0:
            raise ContractError("context budget cannot be negative")


@dataclass(frozen=True)
class ContextItem:
    key: str
    text: str
    source: Source
    priority: int = 0
    freshness: int = 0
    required: bool = False

    def __post_init__(self) -> None:
        if not self.key or not isinstance(self.text, str):
            raise ValueError("context items require a key and text")
        if not isinstance(self.source, Source):
            object.__setattr__(self, "source", Source(self.source))

    @property
    def size(self) -> int:
        return len(self.text.encode("utf-8"))


@dataclass(frozen=True)
class ContextDecision:
    key: str
    action: str
    reason: str


@dataclass(frozen=True)
class ContextResult:
    items: tuple[ContextItem, ...]
    omitted: tuple[str, ...]
    summarized: tuple[str, ...]
    decisions: tuple[ContextDecision, ...]
    used_bytes: int
    budget_bytes: int

    @property
    def text(self) -> str:
        # The rendered form is intentionally small and deterministic. The
        # keys remain available in items/decisions for provenance.
        return "".join(item.text for item in self.items)


def _summary(item: ContextItem, budget: int) -> str:
    encoded = item.text.encode("utf-8")
    if len(encoded) <= budget:
        return item.text
    if budget <= 3:
        return "." * budget
    # Keep the boundary valid for UTF-8 instead of slicing encoded bytes.
    return item.text.encode("utf-8")[:budget - 3].decode("utf-8", "ignore") + "..."


def build_context(items: list[ContextItem] | tuple[ContextItem, ...],
                  budget_bytes: int | ContextBudget) -> ContextResult:
    if isinstance(budget_bytes, ContextBudget):
        budget_bytes = budget_bytes.max_bytes
    if budget_bytes < 0:
        raise ContractError("context budget cannot be negative")
    ordered = sorted(items, key=lambda item: (-int(item.required), -item.priority,
                                               -item.freshness, item.source.value, item.key))
    selected: list[ContextItem] = []
    omitted: list[str] = []
    summarized: list[str] = []
    decisions: list[ContextDecision] = []
    used = 0
    for item in ordered:
        if used + item.size <= budget_bytes:
            selected.append(item)
            used += item.size
            decisions.append(ContextDecision(item.key, "included", "fits budget"))
            continue
        remaining = budget_bytes - used
        if item.required:
            raise ContractError(f"required context item does not fit: {item.key}")
        short = _summary(item, remaining)
        # Summaries are useful for genuinely long context, not a reason to
        # squeeze a short value (especially multi-byte text) into a budget.
        can_summarize = len(item.text) > 32
        if can_summarize and remaining > 3 and len(short.encode("utf-8")) <= remaining:
            summarized.append(item.key)
            selected.append(ContextItem(item.key, short, item.source, item.priority,
                                        item.freshness, item.required))
            used += len(short.encode("utf-8"))
            decisions.append(ContextDecision(item.key, "summarized", "compressed to remaining budget"))
        else:
            omitted.append(item.key)
            decisions.append(ContextDecision(item.key, "omitted", "no remaining budget"))
    return ContextResult(tuple(selected), tuple(omitted), tuple(summarized),
                         tuple(decisions), used, budget_bytes)


class ContextBuilder:
    def __init__(self, budget_bytes: int | ContextBudget):
        self.budget_bytes = budget_bytes

    def build(self, items: list[ContextItem] | tuple[ContextItem, ...]) -> ContextResult:
        return build_context(items, self.budget_bytes)
