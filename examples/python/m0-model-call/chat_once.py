"""M0：发送一次 Responses API 请求并提取文本。

本模块只完成一次非流式模型调用，不执行工具、不维护会话，也不验证任务结果。
配置、请求构造、网络传输和响应解析彼此分离，便于离线测试和后续复用。
"""

from __future__ import annotations

import json
import math
import os
import sys
from dataclasses import dataclass, field
from typing import Mapping, Protocol


# OpenAI API 的默认根地址；可通过 OPENAI_BASE_URL 替换为兼容服务地址。
DEFAULT_BASE_URL = "https://api.openai.com/v1"

# 没有显式配置超时时使用的秒数，避免请求无限等待。
DEFAULT_TIMEOUT_S = 60.0


class AppError(Exception):
    """所有可安全展示给调用者的应用层错误的基类。"""


class ConfigError(AppError):
    """配置缺失或配置格式不正确。"""


class TransportError(AppError):
    """HTTP 客户端无法完成请求，例如网络失败或超时。"""


class ApiError(AppError):
    """Provider 返回了非成功 HTTP 状态。"""


class ResponseFormatError(AppError):
    """Provider 返回的 JSON 不符合本示例需要的文本结构。"""


@dataclass(frozen=True)
class Config:
    """一次模型调用所需的配置；API Key 只在内存中传递。"""

    api_key: str = field(repr=False)
    model: str
    base_url: str
    timeout_s: float

    @property
    def endpoint(self) -> str:
        """拼出 Responses API 端点，不让调用流程重复处理斜杠。"""

        return f"{self.base_url.rstrip('/')}/responses"


@dataclass(frozen=True)
class HttpResponse:
    """传输层返回的最小数据，不携带请求 Header 或其他敏感信息。"""

    status_code: int
    body: str


class HttpTransport(Protocol):
    """网络传输的最小协议，Fake Transport 可以实现它进行离线测试。"""

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
    if not math.isfinite(timeout_s) or timeout_s <= 0:
        raise ConfigError("OPENAI_TIMEOUT_S must be a positive number")

    return Config(
        api_key=api_key,
        model=model,
        base_url=base_url.rstrip("/"),
        timeout_s=timeout_s,
    )


# ANCHOR: m0-build-request
def build_request(config: Config, prompt: str) -> dict[str, object]:
    """纯函数：将用户输入转换成 Responses API 请求 JSON。"""

    return {
        "model": config.model,
        "input": prompt,
    }
# ANCHOR_END: m0-build-request


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
            raise TransportError(f"request failed: {error.__class__.__name__}") from error

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


# ANCHOR: m0-extract-output
def extract_output_text(body: Mapping[str, object]) -> str:
    """从 Responses API 输出项目中提取所有非空 output_text。"""

    output = body.get("output")
    if not isinstance(output, list):
        raise ResponseFormatError("API response did not contain an output list")

    texts: list[str] = []
    for item in output:
        if not isinstance(item, dict):
            continue
        content = item.get("content")
        if not isinstance(content, list):
            continue
        for content_item in content:
            if not isinstance(content_item, dict):
                continue
            if content_item.get("type") != "output_text":
                continue
            text = content_item.get("text")
            if isinstance(text, str) and text.strip():
                texts.append(text)

    if not texts:
        raise ResponseFormatError("API response contained no output_text")
    return "\n".join(texts)
# ANCHOR_END: m0-extract-output


def parse_response(response: HttpResponse) -> str:
    """检查 HTTP 状态、解析 JSON，并返回最终文本。"""

    if not 200 <= response.status_code < 300:
        # 不把 Provider 返回正文放进错误，避免敏感内容被意外回显。
        raise ApiError(f"API request failed with HTTP {response.status_code}")

    try:
        body = json.loads(response.body)
    except json.JSONDecodeError as error:
        raise ResponseFormatError("API returned invalid JSON") from error

    if not isinstance(body, dict):
        raise ResponseFormatError("API response must be a JSON object")
    return extract_output_text(body)


# ANCHOR: m0-chat-once
def chat_once(
    prompt: str,
    config: Config | None = None,
    transport: HttpTransport | None = None,
) -> str:
    """完成一次模型调用；传入 Fake Transport 即可完全离线测试。"""

    active_config = load_config() if config is None else config
    payload = build_request(active_config, prompt)
    response = send_request(active_config, payload, transport=transport)
    return parse_response(response)
# ANCHOR_END: m0-chat-once


def main() -> int:
    """命令行入口：成功打印文本，失败打印安全错误并返回非零状态。"""

    try:
        print(chat_once("请用一句话解释：为什么单次模型调用还不是 Agent？"))
    except AppError as error:
        print(f"m0-model-call: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
