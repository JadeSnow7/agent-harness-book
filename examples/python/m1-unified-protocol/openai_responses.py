"""OpenAI Responses API 与统一协议之间的编解码。

上层只看见 Message / ModelRequest / ModelResponse；本模块负责 Provider JSON。
"""

from __future__ import annotations

import json
from typing import Any, Mapping

from protocol import (
    ApiError,
    ContentBlock,
    DecodeError,
    EncodeError,
    Message,
    ModelRequest,
    ModelResponse,
    Role,
    TextBlock,
    ToolResultBlock,
    ToolUseBlock,
)


def encode_request(request: ModelRequest) -> dict[str, Any]:
    """将统一请求编码为 Responses API JSON Body。"""

    items: list[dict[str, Any]] = []
    if request.system and request.system.strip():
        items.append(
            {
                "type": "message",
                "role": "system",
                "content": [{"type": "input_text", "text": request.system}],
            }
        )

    for message in request.messages:
        items.extend(_encode_message(message))

    if not items:
        raise EncodeError("encoded request input must not be empty")

    payload: dict[str, Any] = {
        "model": request.model,
        "input": items,
    }
    if request.tools:
        payload["tools"] = [
            {
                "type": "function",
                "name": tool.name,
                "description": tool.description,
                "parameters": tool.input_schema,
                "strict": tool.strict,
            }
            for tool in request.tools
        ]
    return payload


def decode_response(body: Mapping[str, Any]) -> ModelResponse:
    """将 Responses API JSON 解码为统一响应。"""

    output = body.get("output")
    if not isinstance(output, list):
        raise DecodeError("API response did not contain an output list")

    blocks: list[ContentBlock] = []
    for item in output:
        if not isinstance(item, dict):
            continue
        item_type = item.get("type")
        if item_type == "message":
            blocks.extend(_decode_message_item(item))
        elif item_type == "function_call":
            blocks.append(_decode_function_call(item))
        # reasoning 及其他未知项目直接跳过，不当作协议成功内容。

    if not blocks:
        raise DecodeError("API response contained no decodable content blocks")

    response_id = body.get("id")
    model = body.get("model")
    status = body.get("status")
    if status is not None and not isinstance(status, str):
        status = None

    return ModelResponse(
        id=response_id if isinstance(response_id, str) else None,
        model=model if isinstance(model, str) else None,
        message=Message(role=Role.ASSISTANT, content=tuple(blocks)),
        status=status,
    )


def decode_response_json(text: str) -> ModelResponse:
    """解析 JSON 字符串并解码为统一响应。"""

    try:
        body = json.loads(text)
    except json.JSONDecodeError as error:
        raise DecodeError("API returned invalid JSON") from error

    if not isinstance(body, dict):
        raise DecodeError("API response must be a JSON object")
    return decode_response(body)


def parse_http_response(status_code: int, body: str) -> ModelResponse:
    """检查 HTTP 状态后解码响应；错误不回显正文。"""

    if not 200 <= status_code < 300:
        raise ApiError(f"API request failed with HTTP {status_code}")
    return decode_response_json(body)


def _encode_message(message: Message) -> list[dict[str, Any]]:
    """把一条统一消息编码成一个或多个 Responses input 项目。"""

    if message.role == Role.TOOL:
        return [_encode_tool_result_item(block) for block in message.content]

    if message.role == Role.ASSISTANT:
        return _encode_assistant_items(message)

    role = message.role.value
    content_items: list[dict[str, Any]] = []
    for block in message.content:
        if isinstance(block, TextBlock):
            content_items.append({"type": "input_text", "text": block.text})
        elif isinstance(block, ToolResultBlock):
            raise EncodeError("tool_result blocks require role=tool")
        elif isinstance(block, ToolUseBlock):
            raise EncodeError("tool_use blocks require role=assistant")
        else:
            raise EncodeError(f"unsupported content block: {type(block)!r}")

    if not content_items:
        raise EncodeError("message produced no provider content items")

    return [{"type": "message", "role": role, "content": content_items}]


def _encode_assistant_items(message: Message) -> list[dict[str, Any]]:
    """assistant 消息可能同时包含文本和 tool_use，需拆成多个 output 形态 input。"""

    items: list[dict[str, Any]] = []
    text_parts: list[dict[str, Any]] = []

    def flush_text() -> None:
        nonlocal text_parts
        if text_parts:
            items.append(
                {
                    "type": "message",
                    "role": "assistant",
                    "content": text_parts,
                }
            )
            text_parts = []

    for block in message.content:
        if isinstance(block, TextBlock):
            text_parts.append({"type": "output_text", "text": block.text})
        elif isinstance(block, ToolUseBlock):
            flush_text()
            items.append(
                {
                    "type": "function_call",
                    "call_id": block.id,
                    "name": block.name,
                    "arguments": json.dumps(block.input, ensure_ascii=False),
                }
            )
        elif isinstance(block, ToolResultBlock):
            raise EncodeError("assistant message cannot contain tool_result")
        else:
            raise EncodeError(f"unsupported content block: {type(block)!r}")

    flush_text()
    if not items:
        raise EncodeError("assistant message produced no provider items")
    return items


def _encode_tool_result_item(block: ContentBlock) -> dict[str, Any]:
    """将 tool_result 编码为 function_call_output。"""

    if not isinstance(block, ToolResultBlock):
        raise EncodeError("role=tool messages may only contain tool_result blocks")

    return {
        "type": "function_call_output",
        "call_id": block.tool_use_id,
        "output": block.content,
    }


def _decode_message_item(item: Mapping[str, Any]) -> list[ContentBlock]:
    """从 message 输出项中提取文本块。"""

    content = item.get("content")
    if not isinstance(content, list):
        return []

    blocks: list[ContentBlock] = []
    for content_item in content:
        if not isinstance(content_item, dict):
            continue
        content_type = content_item.get("type")
        if content_type in {"output_text", "text", "input_text"}:
            text = content_item.get("text")
            if isinstance(text, str) and text.strip():
                blocks.append(TextBlock(text=text))
        elif content_type == "refusal":
            refusal = content_item.get("refusal")
            if isinstance(refusal, str) and refusal.strip():
                blocks.append(TextBlock(text=refusal))
    return blocks


def _decode_function_call(item: Mapping[str, Any]) -> ToolUseBlock:
    """把 Responses function_call 项目解码为 ToolUseBlock。"""

    call_id = item.get("call_id") or item.get("id")
    name = item.get("name")
    if not isinstance(call_id, str) or not call_id.strip():
        raise DecodeError("function_call is missing call_id")
    if not isinstance(name, str) or not name.strip():
        raise DecodeError("function_call is missing name")

    arguments = item.get("arguments", "{}")
    parsed = _parse_arguments(arguments)
    return ToolUseBlock(id=call_id, name=name, input=parsed)


def _parse_arguments(arguments: Any) -> dict[str, Any]:
    """arguments 可能是 JSON 字符串或对象；统一成 dict。"""

    if isinstance(arguments, dict):
        return dict(arguments)
    if arguments is None:
        return {}
    if not isinstance(arguments, str):
        raise DecodeError("function_call arguments must be an object or JSON string")

    text = arguments.strip()
    if not text:
        return {}

    try:
        value = json.loads(text)
    except json.JSONDecodeError as error:
        raise DecodeError("function_call arguments are not valid JSON") from error

    if value is None:
        return {}
    if not isinstance(value, dict):
        raise DecodeError("function_call arguments JSON must be an object")
    return value
