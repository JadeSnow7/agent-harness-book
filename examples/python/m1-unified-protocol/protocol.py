"""M1：Provider 无关的统一协议类型。

本模块只定义消息、内容块、模型请求/响应和可安全展示的错误类型。
它不发送网络请求，也不执行工具。
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
import re
from typing import Any


class ProtocolError(Exception):
    """统一协议层可安全展示给调用者的错误基类。"""


class ConfigError(ProtocolError):
    """配置缺失或配置格式不正确。"""


class TransportError(ProtocolError):
    """HTTP 客户端无法完成请求，例如网络失败或超时。"""


class ApiError(ProtocolError):
    """Provider 返回了非成功 HTTP 状态。"""


class DecodeError(ProtocolError):
    """Provider JSON 无法解码为统一协议对象。"""


class EncodeError(ProtocolError):
    """统一协议对象无法编码为 Provider JSON。"""


class Role(str, Enum):
    """消息角色；tool 表示工具观察，不是模型自身输出。"""

    SYSTEM = "system"
    USER = "user"
    ASSISTANT = "assistant"
    TOOL = "tool"


# ANCHOR: m1-protocol-types
@dataclass(frozen=True)
class TextBlock:
    """普通文本内容块。"""

    text: str
    type: str = field(default="text", init=False)


@dataclass(frozen=True)
class ToolUseBlock:
    """模型提出的工具调用候选；本章只解析，不执行。"""

    id: str
    name: str
    input: dict[str, Any]
    type: str = field(default="tool_use", init=False)


@dataclass(frozen=True)
class ToolResultBlock:
    """工具执行后的观察；供后续轮次重新进入模型输入。"""

    tool_use_id: str
    content: str
    is_error: bool = False
    type: str = field(default="tool_result", init=False)


ContentBlock = TextBlock | ToolUseBlock | ToolResultBlock


@dataclass(frozen=True)
class ToolDefinition:
    """告诉模型“有哪些函数可用”的 Provider 无关定义。

    ``input_schema`` 使用 JSON Schema 对象描述输入。Runtime 仍必须自行校验模型
    实际返回的参数；模型侧的 strict 约束不能替代执行前校验。
    """

    name: str
    description: str
    input_schema: dict[str, Any]
    strict: bool = True

    def __post_init__(self) -> None:
        """在发请求前拒绝 Provider 无法接受或读者难以理解的定义。"""

        if re.fullmatch(r"[A-Za-z0-9_-]{1,64}", self.name) is None:
            raise EncodeError(
                "tool name must be 1-64 letters, digits, underscores, or hyphens"
            )
        if not self.description.strip():
            raise EncodeError("tool description must not be empty")
        if not isinstance(self.input_schema, dict):
            raise EncodeError("tool input_schema must be an object")
        if self.input_schema.get("type") != "object":
            raise EncodeError("tool input_schema.type must be 'object'")
        if not isinstance(self.strict, bool):
            raise EncodeError("tool strict must be a boolean")


@dataclass(frozen=True)
class Message:
    """一条带角色的统一消息。"""

    role: Role
    content: tuple[ContentBlock, ...]

    def __post_init__(self) -> None:
        """拒绝空内容，避免后续适配器把空消息编码成歧义请求。"""

        if not self.content:
            raise EncodeError("message content must not be empty")


@dataclass(frozen=True)
class ModelRequest:
    """一次模型调用的统一请求。"""

    model: str
    messages: tuple[Message, ...]
    system: str | None = None
    tools: tuple[ToolDefinition, ...] = ()

    def __post_init__(self) -> None:
        """模型名和消息列表都必须明确存在。"""

        if not self.model.strip():
            raise EncodeError("model must not be empty")
        if not self.messages:
            raise EncodeError("messages must not be empty")
        names = [tool.name for tool in self.tools]
        if len(names) != len(set(names)):
            raise EncodeError("tool names must be unique within a request")
# ANCHOR_END: m1-protocol-types


@dataclass(frozen=True)
class ModelResponse:
    """一次模型调用的统一响应。"""

    id: str | None
    model: str | None
    message: Message
    status: str | None = None

    def text(self) -> str:
        """按顺序合并所有非空文本块；没有文本时返回空字符串。"""

        parts = [
            block.text
            for block in self.message.content
            if isinstance(block, TextBlock) and block.text.strip()
        ]
        return "\n".join(parts)

    def tool_uses(self) -> tuple[ToolUseBlock, ...]:
        """返回响应中的全部工具调用候选。"""

        return tuple(
            block for block in self.message.content if isinstance(block, ToolUseBlock)
        )


def text_message(role: Role, text: str) -> Message:
    """构造只含一个文本块的消息。"""

    return Message(role=role, content=(TextBlock(text=text),))
