"""M1：通过统一协议完成一次非流式模型调用。

流程：统一 ModelRequest → Responses JSON → HTTP → 统一 ModelResponse。
本章不执行工具，只解析并展示文本与 tool_use 候选。
"""

from __future__ import annotations

import os
import sys
from dataclasses import dataclass
from typing import Mapping, Protocol

from openai_responses import encode_request, parse_http_response
from protocol import (
    ApiError,
    ConfigError,
    ModelRequest,
    ModelResponse,
    ProtocolError,
    Role,
    ToolDefinition,
    TransportError,
    text_message,
)


DEFAULT_BASE_URL = "https://api.openai.com/v1"
DEFAULT_TIMEOUT_S = 60.0
DEFAULT_PROMPT = "请用一句话解释：为什么统一协议有助于后续接入工具？"


@dataclass(frozen=True)
class Config:
    """一次模型调用所需的配置；API Key 只在内存中传递。"""

    api_key: str
    model: str
    base_url: str
    timeout_s: float

    @property
    def endpoint(self) -> str:
        """拼出 Responses API 端点。"""

        return f"{self.base_url.rstrip('/')}/responses"


@dataclass(frozen=True)
class HttpResponse:
    """传输层返回的最小数据，不携带请求 Header。"""

    status_code: int
    body: str


class HttpTransport(Protocol):
    """网络传输的最小协议，Fake Transport 可实现它做离线测试。"""

    def post_json(
        self,
        url: str,
        headers: Mapping[str, str],
        payload: Mapping[str, object],
        timeout_s: float,
    ) -> HttpResponse:
        """发送 JSON POST，并返回状态码与正文。"""


def load_config(environ: Mapping[str, str] | None = None) -> Config:
    """从环境变量读取配置，不执行网络请求，也不打印 API Key。"""

    values = os.environ if environ is None else environ
    api_key = values.get("OPENAI_API_KEY")
    if not api_key:
        raise ConfigError(
            "missing OPENAI_API_KEY; export it before making a live request"
        )

    model = values.get("OPENAI_MODEL")
    if not model:
        raise ConfigError(
            "missing OPENAI_MODEL; set it to a model available to your account"
        )

    base_url = values.get("OPENAI_BASE_URL", DEFAULT_BASE_URL).strip()
    if not base_url:
        raise ConfigError("OPENAI_BASE_URL must not be empty")

    timeout_text = values.get("OPENAI_TIMEOUT_S", str(DEFAULT_TIMEOUT_S))
    try:
        timeout_s = float(timeout_text)
    except ValueError as error:
        raise ConfigError("OPENAI_TIMEOUT_S must be a positive number") from error
    if timeout_s <= 0:
        raise ConfigError("OPENAI_TIMEOUT_S must be a positive number")

    return Config(
        api_key=api_key,
        model=model,
        base_url=base_url.rstrip("/"),
        timeout_s=timeout_s,
    )


class RequestsTransport:
    """生产传输实现：使用 requests 发送一次 HTTPS POST。"""

    def post_json(
        self,
        url: str,
        headers: Mapping[str, str],
        payload: Mapping[str, object],
        timeout_s: float,
    ) -> HttpResponse:
        """发送请求；错误只保留类别，不回显响应正文或认证信息。"""

        try:
            import requests
        except ModuleNotFoundError as error:
            raise TransportError(
                "requests is not installed; run python -m pip install requests"
            ) from error

        try:
            response = requests.post(
                url,
                headers=dict(headers),
                json=dict(payload),
                timeout=timeout_s,
            )
        except requests.RequestException as error:
            raise TransportError(
                f"request failed: {error.__class__.__name__}"
            ) from error

        return HttpResponse(status_code=response.status_code, body=response.text)


def send_request(
    config: Config,
    payload: Mapping[str, object],
    transport: HttpTransport | None = None,
) -> HttpResponse:
    """编排传输层：准备 Header，委托 HTTP 客户端发送请求。"""

    client = RequestsTransport() if transport is None else transport
    return client.post_json(
        url=config.endpoint,
        headers={
            "Authorization": f"Bearer {config.api_key}",
            "Content-Type": "application/json",
        },
        payload=payload,
        timeout_s=config.timeout_s,
    )


# ANCHOR: m1-build-request
def build_model_request(
    model: str,
    prompt: str,
    *,
    tools: tuple[ToolDefinition, ...] = (),
) -> ModelRequest:
    """把简单提示词和可用工具提升为统一协议请求。"""

    return ModelRequest(
        model=model,
        messages=(text_message(Role.USER, prompt),),
        tools=tools,
    )
# ANCHOR_END: m1-build-request


# ANCHOR: m1-complete
def complete(
    request: ModelRequest,
    config: Config,
    transport: HttpTransport | None = None,
) -> ModelResponse:
    """发送一个已经构造好的统一请求，并返回统一响应。

    ``request.model`` 是本次请求的模型来源；``config`` 只负责认证、端点和
    timeout。调用者可以复用同一 Config 连续发送不同的统一请求。
    """

    payload = encode_request(request)
    response = send_request(config, payload, transport=transport)
    return parse_http_response(response.status_code, response.body)
# ANCHOR_END: m1-complete


def chat_once(
    prompt: str,
    config: Config | None = None,
    transport: HttpTransport | None = None,
    *,
    tools: tuple[ToolDefinition, ...] = (),
) -> ModelResponse:
    """便捷入口：从提示词构造请求，再完成一次非流式模型调用。"""

    active_config = load_config() if config is None else config
    request = build_model_request(active_config.model, prompt, tools=tools)
    return complete(request, active_config, transport=transport)


# ANCHOR: m1-format-response
def format_response(response: ModelResponse) -> str:
    """把统一响应格式化为可打印文本；工具候选单独标注。"""

    lines: list[str] = []
    text = response.text()
    if text:
        lines.append(text)

    for tool_use in response.tool_uses():
        lines.append(
            f"[tool_use id={tool_use.id} name={tool_use.name} input={tool_use.input}]"
        )

    if not lines:
        raise ApiError("decoded response contained neither text nor tool_use")
    return "\n".join(lines)
# ANCHOR_END: m1-format-response


def main() -> int:
    """命令行入口：成功打印统一响应摘要，失败打印安全错误。"""

    try:
        response = chat_once(DEFAULT_PROMPT)
        print(format_response(response))
    except ProtocolError as error:
        print(f"m1-unified-protocol: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
