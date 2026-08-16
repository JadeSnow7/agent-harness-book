"""M2：Tool Runtime 的最小类型。

失败以结构化 ToolResult 表示，不把工具业务失败抬升为未捕获异常。
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Mapping, Protocol


class ToolStatus(str, Enum):
    """单次工具调用的结果状态。"""

    SUCCEEDED = "succeeded"
    FAILED = "failed"


@dataclass(frozen=True)
class ToolSpec:
    """工具对外描述；`strict` 会原样桥接到 M1 ToolDefinition。"""

    name: str
    description: str
    input_schema: dict[str, Any] = field(default_factory=dict)
    strict: bool = False


@dataclass(frozen=True)
class ToolCall:
    """一次工具调用请求，通常来自统一协议中的 tool_use。"""

    call_id: str
    name: str
    arguments: Mapping[str, Any]


@dataclass(frozen=True)
class ToolResult:
    """一次工具执行的结构化结果。"""

    call_id: str
    name: str
    status: ToolStatus
    output: Any | None = None
    error: str | None = None

    @property
    def succeeded(self) -> bool:
        """是否成功。"""

        return self.status is ToolStatus.SUCCEEDED

    def as_text(self) -> str:
        """把结果压成可供 tool_result 内容块使用的文本。"""

        if self.succeeded:
            if self.output is None:
                return ""
            if isinstance(self.output, str):
                return self.output
            import json

            return json.dumps(self.output, ensure_ascii=False, indent=2)
        return self.error or "tool failed"


class Tool(Protocol):
    """单个工具的最小协议。"""

    @property
    def spec(self) -> ToolSpec:
        """返回工具规格。"""

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        """参数不合法时抛出 ValueError。"""

    def execute(self, arguments: Mapping[str, Any]) -> Any:
        """执行工具；业务失败应抛出 Exception，由 Registry 收成 Failed。"""
