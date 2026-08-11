"""M0 Python 示例的离线测试。

测试使用 FakeTransport，不读取真实 API Key，也不访问网络。
"""

import json
import unittest

from chat_once import (
    ApiError,
    Config,
    ConfigError,
    HttpResponse,
    ResponseFormatError,
    chat_once,
    build_request,
    extract_output_text,
    load_config,
    parse_response,
)


class FakeTransport:
    """记录请求并返回预置响应，模拟 HTTP Transport。"""

    def __init__(self, response: HttpResponse) -> None:
        """保存测试响应，并准备记录最后一次请求。"""

        self.response = response
        self.last_request: dict[str, object] | None = None

    def post_json(self, url, headers, payload, timeout_s):
        """记录请求参数，不发出真实网络请求。"""

        self.last_request = {
            "url": url,
            "headers": dict(headers),
            "payload": dict(payload),
            "timeout_s": timeout_s,
        }
        return self.response


def response_body(*items: dict[str, object]) -> str:
    """构造 Responses API 风格的 JSON 响应。"""

    return json.dumps({"output": list(items)})


class ConfigTests(unittest.TestCase):
    """验证配置读取不会悄悄依赖真实进程环境。"""

    def test_missing_api_key_fails_before_network(self):
        """没有 API Key 时返回安全错误，错误中不包含密钥。"""

        with self.assertRaisesRegex(ConfigError, "missing OPENAI_API_KEY"):
            load_config({"OPENAI_MODEL": "gpt-5.6-luna"})

    def test_missing_model_fails_before_network(self):
        """没有模型名时要求调用者显式配置模型。"""

        with self.assertRaisesRegex(ConfigError, "missing OPENAI_MODEL"):
            load_config({"OPENAI_API_KEY": "secret-value"})

    def test_loads_config_without_printing_secret(self):
        """配置对象可以读取模型和超时，Key 只保存在对象中。"""

        config = load_config(
            {
                "OPENAI_API_KEY": "secret-value",
                "OPENAI_MODEL": "gpt-5.6-luna",
                "OPENAI_TIMEOUT_S": "12.5",
            }
        )

        self.assertEqual(config.model, "gpt-5.6-luna")
        self.assertEqual(config.timeout_s, 12.5)
        self.assertEqual(config.endpoint, "https://api.openai.com/v1/responses")


class RequestTests(unittest.TestCase):
    """验证请求构造和传输边界。"""

    def setUp(self):
        """为每个测试准备相同的、虚构的配置。"""

        self.config = Config(
            api_key="secret-value",
            model="gpt-5.6-luna",
            base_url="https://example.test/v1",
            timeout_s=5.0,
        )

    def test_build_request_is_provider_shaped(self):
        """请求构造只产生 model 和 input，不接触网络。"""

        self.assertEqual(
            build_request(self.config, "Hello"),
            {"model": "gpt-5.6-luna", "input": "Hello"},
        )

    # ANCHOR: m0-reading-case
    def test_reading_case_is_still_only_a_plain_prompt(self):
        """连续案例在 M0 仍只是文本输入，不能访问 hello.txt。"""

        prompt = (
            "请读取工作区 hello.txt 的第一行并告诉我内容；"
            "如果无法访问文件，不要猜测。"
        )
        self.assertEqual(
            build_request(self.config, prompt),
            {"model": "gpt-5.6-luna", "input": prompt},
        )
    # ANCHOR_END: m0-reading-case

    def test_chat_once_sends_expected_request_and_extracts_text(self):
        """编排流程向 Fake Transport 发送正确请求并得到文本。"""

        transport = FakeTransport(
            HttpResponse(
                status_code=200,
                body=response_body(
                    {
                        "type": "message",
                        "content": [{"type": "output_text", "text": "hello"}],
                    }
                ),
            )
        )

        self.assertEqual(chat_once("Hello", self.config, transport), "hello")
        self.assertIsNotNone(transport.last_request)
        assert transport.last_request is not None
        self.assertEqual(
            transport.last_request["url"], "https://example.test/v1/responses"
        )
        self.assertEqual(transport.last_request["payload"], self.build_payload())
        self.assertEqual(
            transport.last_request["headers"],
            {
                "Authorization": "Bearer secret-value",
                "Content-Type": "application/json",
            },
        )

    def build_payload(self):
        """返回本测试期望的请求 JSON。"""

        return {"model": "gpt-5.6-luna", "input": "Hello"}


class ResponseTests(unittest.TestCase):
    """验证状态码、JSON 和多项目文本解析。"""

    def test_http_status_does_not_echo_response_body(self):
        """HTTP 错误只暴露状态码，不暴露服务端正文。"""

        response = HttpResponse(
            status_code=401,
            body='{"error":{"message":"secret-response"}}',
        )

        with self.assertRaises(ApiError) as context:
            parse_response(response)

        self.assertEqual(str(context.exception), "API request failed with HTTP 401")
        self.assertNotIn("secret-response", str(context.exception))

    def test_common_http_failures_are_classified_without_body_echo(self):
        """常见的鉴权、限流和服务端错误都只暴露状态码。"""

        for status_code in (401, 429, 500):
            with self.subTest(status_code=status_code):
                response = HttpResponse(
                    status_code=status_code,
                    body='{"error":{"message":"secret-response"}}',
                )

                with self.assertRaises(ApiError) as context:
                    parse_response(response)

                self.assertEqual(
                    str(context.exception),
                    f"API request failed with HTTP {status_code}",
                )
                self.assertNotIn("secret-response", str(context.exception))

    def test_invalid_json_is_a_format_error(self):
        """成功状态但非法 JSON 仍然属于响应格式错误。"""

        with self.assertRaisesRegex(ResponseFormatError, "invalid JSON"):
            parse_response(HttpResponse(status_code=200, body="not-json"))

    def test_skips_reasoning_before_text(self):
        """reasoning 项在前时仍能找到后面的文本。"""

        body = {
            "output": [
                {"type": "reasoning", "summary": []},
                {
                    "type": "message",
                    "content": [{"type": "output_text", "text": "answer"}],
                },
            ]
        }

        self.assertEqual(extract_output_text(body), "answer")

    def test_joins_multiple_text_items_in_order(self):
        """多个文本项目按响应顺序合并，非文本项目被跳过。"""

        body = {
            "output": [
                {
                    "type": "message",
                    "content": [
                        {"type": "output_text", "text": "first"},
                        {"type": "refusal", "refusal": "ignored"},
                        {"type": "output_text", "text": "second"},
                    ],
                }
            ]
        }

        self.assertEqual(extract_output_text(body), "first\nsecond")

    def test_missing_output_text_is_a_format_error(self):
        """没有任何非空文本时返回明确错误。"""

        with self.assertRaisesRegex(ResponseFormatError, "no output_text"):
            extract_output_text({"output": []})

    def test_missing_output_list_is_a_format_error(self):
        """响应缺少 output 列表时不能继续猜测字段路径。"""

        with self.assertRaisesRegex(ResponseFormatError, "output list"):
            extract_output_text({})


if __name__ == "__main__":
    unittest.main()
