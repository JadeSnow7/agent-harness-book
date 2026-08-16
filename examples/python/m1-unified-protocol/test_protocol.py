"""M1 统一协议与 Responses 适配器的离线测试。"""

from __future__ import annotations

import json
import unittest

from chat_once import (
    Config,
    HttpResponse,
    build_model_request,
    chat_once,
    complete,
    format_response,
    load_config,
)
from openai_responses import (
    decode_response,
    encode_request,
    parse_http_response,
)
from protocol import (
    ApiError,
    ConfigError,
    DecodeError,
    EncodeError,
    Message,
    ModelRequest,
    ModelResponse,
    Role,
    TextBlock,
    ToolDefinition,
    ToolResultBlock,
    ToolUseBlock,
    text_message,
)


class FakeTransport:
    """记录请求并返回预置响应。"""

    def __init__(self, response: HttpResponse) -> None:
        self.response = response
        self.last_request: dict[str, object] | None = None

    def post_json(self, url, headers, payload, timeout_s):
        self.last_request = {
            "url": url,
            "headers": dict(headers),
            "payload": dict(payload),
            "timeout_s": timeout_s,
        }
        return self.response


def message_output(text: str) -> dict[str, object]:
    return {
        "type": "message",
        "role": "assistant",
        "content": [{"type": "output_text", "text": text}],
    }


class ProtocolTypeTests(unittest.TestCase):
    def test_empty_message_is_rejected(self):
        with self.assertRaisesRegex(EncodeError, "content must not be empty"):
            Message(role=Role.USER, content=())

    def test_text_helper_builds_user_message(self):
        message = text_message(Role.USER, "hello")
        self.assertEqual(message.role, Role.USER)
        self.assertEqual(message.content, (TextBlock(text="hello"),))

    def test_invalid_and_duplicate_tool_definitions_are_rejected(self):
        with self.assertRaisesRegex(EncodeError, "tool name"):
            ToolDefinition("not a tool", "invalid", {"type": "object"})
        tool = ToolDefinition("echo", "Echo text", {"type": "object"})
        with self.assertRaisesRegex(EncodeError, "unique"):
            ModelRequest(
                model="gpt-test",
                messages=(text_message(Role.USER, "hello"),),
                tools=(tool, tool),
            )

    def test_response_text_and_tool_uses(self):
        response = ModelResponse(
            id="resp_1",
            model="gpt-test",
            message=Message(
                role=Role.ASSISTANT,
                content=(
                    TextBlock(text="first"),
                    ToolUseBlock(id="call_1", name="echo", input={"text": "hi"}),
                    TextBlock(text="second"),
                ),
            ),
        )
        self.assertEqual(response.text(), "first\nsecond")
        self.assertEqual(len(response.tool_uses()), 1)
        self.assertEqual(response.tool_uses()[0].name, "echo")

    # ANCHOR: m1-reading-case
    def test_reading_case_is_decoded_as_a_candidate(self):
        """M1 能表示 read 候选，但这里只解析，不执行文件读取。"""

        response = ModelResponse(
            id="resp_read",
            model="gpt-test",
            message=Message(
                role=Role.ASSISTANT,
                content=(
                    ToolUseBlock(
                        id="call_42",
                        name="read",
                        input={"path": "hello.txt"},
                    ),
                ),
            ),
        )
        self.assertEqual(response.tool_uses()[0].id, "call_42")
        self.assertEqual(response.tool_uses()[0].input, {"path": "hello.txt"})
    # ANCHOR_END: m1-reading-case


class EncodeTests(unittest.TestCase):
    def test_encodes_system_and_user_text(self):
        request = ModelRequest(
            model="gpt-test",
            system="be brief",
            messages=(text_message(Role.USER, "Hello"),),
        )
        payload = encode_request(request)
        self.assertEqual(payload["model"], "gpt-test")
        self.assertEqual(
            payload["input"],
            [
                {
                    "type": "message",
                    "role": "system",
                    "content": [{"type": "input_text", "text": "be brief"}],
                },
                {
                    "type": "message",
                    "role": "user",
                    "content": [{"type": "input_text", "text": "Hello"}],
                },
            ],
        )

    def test_encodes_function_tool_definitions_in_order(self):
        request = ModelRequest(
            model="gpt-test",
            messages=(text_message(Role.USER, "Use a tool"),),
            tools=(
                ToolDefinition(
                    name="echo",
                    description="Echo text",
                    input_schema={
                        "type": "object",
                        "properties": {"text": {"type": "string"}},
                        "required": ["text"],
                        "additionalProperties": False,
                    },
                ),
                ToolDefinition(
                    name="pwd",
                    description="Return the workspace path",
                    input_schema={
                        "type": "object",
                        "properties": {},
                        "additionalProperties": False,
                    },
                    strict=False,
                ),
            ),
        )
        payload = encode_request(request)
        self.assertEqual([item["name"] for item in payload["tools"]], ["echo", "pwd"])
        self.assertEqual(payload["tools"][0]["type"], "function")
        self.assertTrue(payload["tools"][0]["strict"])
        self.assertFalse(payload["tools"][1]["strict"])

    def test_encodes_tool_result_as_function_call_output(self):
        request = ModelRequest(
            model="gpt-test",
            messages=(
                Message(
                    role=Role.TOOL,
                    content=(
                        ToolResultBlock(
                            tool_use_id="call_1",
                            content='{"ok":true}',
                        ),
                    ),
                ),
            ),
        )
        payload = encode_request(request)
        self.assertEqual(
            payload["input"],
            [
                {
                    "type": "function_call_output",
                    "call_id": "call_1",
                    "output": '{"ok":true}',
                }
            ],
        )

    def test_encodes_assistant_tool_use(self):
        request = ModelRequest(
            model="gpt-test",
            messages=(
                Message(
                    role=Role.ASSISTANT,
                    content=(
                        TextBlock(text="calling"),
                        ToolUseBlock(
                            id="call_9",
                            name="echo",
                            input={"text": "ping"},
                        ),
                    ),
                ),
            ),
        )
        payload = encode_request(request)
        self.assertEqual(payload["input"][0]["role"], "assistant")
        self.assertEqual(payload["input"][1]["type"], "function_call")
        self.assertEqual(payload["input"][1]["call_id"], "call_9")
        self.assertEqual(
            json.loads(payload["input"][1]["arguments"]),
            {"text": "ping"},
        )


class DecodeTests(unittest.TestCase):
    def test_skips_reasoning_and_joins_text(self):
        body = {
            "id": "resp_abc",
            "model": "gpt-test",
            "status": "completed",
            "output": [
                {"type": "reasoning", "summary": []},
                message_output("first"),
                {
                    "type": "message",
                    "content": [
                        {"type": "output_text", "text": "second"},
                        {"type": "refusal", "refusal": "ignored-as-text-if-present"},
                    ],
                },
            ],
        }
        response = decode_response(body)
        self.assertEqual(response.id, "resp_abc")
        self.assertEqual(response.model, "gpt-test")
        self.assertEqual(response.status, "completed")
        self.assertEqual(
            response.text(),
            "first\nsecond\nignored-as-text-if-present",
        )

    def test_decodes_function_call_arguments_string(self):
        body = {
            "output": [
                {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "echo",
                    "arguments": '{"text":"hi"}',
                }
            ]
        }
        response = decode_response(body)
        tool_use = response.tool_uses()[0]
        self.assertEqual(tool_use.id, "call_1")
        self.assertEqual(tool_use.name, "echo")
        self.assertEqual(tool_use.input, {"text": "hi"})

    def test_decodes_function_call_arguments_object(self):
        body = {
            "output": [
                {
                    "type": "function_call",
                    "call_id": "call_2",
                    "name": "echo",
                    "arguments": {"text": "object"},
                }
            ]
        }
        response = decode_response(body)
        self.assertEqual(response.tool_uses()[0].input, {"text": "object"})

    def test_missing_output_is_decode_error(self):
        with self.assertRaisesRegex(DecodeError, "output list"):
            decode_response({})

    def test_empty_output_is_decode_error(self):
        with self.assertRaisesRegex(DecodeError, "no decodable content"):
            decode_response({"output": [{"type": "reasoning"}]})

    def test_invalid_arguments_json_is_decode_error(self):
        body = {
            "output": [
                {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "echo",
                    "arguments": "not-json",
                }
            ]
        }
        with self.assertRaisesRegex(DecodeError, "not valid JSON"):
            decode_response(body)

    def test_function_call_requires_call_id_name_and_object_arguments(self):
        cases = [
            ({"type": "function_call", "name": "echo", "arguments": "{}"}, "call_id"),
            (
                {"type": "function_call", "call_id": "call_1", "arguments": "{}"},
                "name",
            ),
            (
                {
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "echo",
                    "arguments": "[]",
                },
                "must be an object",
            ),
        ]
        for item, message in cases:
            with self.subTest(message=message):
                with self.assertRaisesRegex(DecodeError, message):
                    decode_response({"output": [item]})

    def test_unknown_output_is_skipped_but_cannot_be_the_only_output(self):
        response = decode_response(
            {"output": [{"type": "future_item"}, message_output("known")]}
        )
        self.assertEqual(response.text(), "known")
        with self.assertRaisesRegex(DecodeError, "no decodable content"):
            decode_response({"output": [{"type": "future_item"}]})

    def test_http_error_does_not_echo_body(self):
        with self.assertRaises(ApiError) as context:
            parse_http_response(401, '{"error":{"message":"secret-response"}}')
        self.assertEqual(str(context.exception), "API request failed with HTTP 401")
        self.assertNotIn("secret-response", str(context.exception))

    def test_invalid_json_is_decode_error(self):
        with self.assertRaisesRegex(DecodeError, "invalid JSON"):
            parse_http_response(200, "not-json")


class ChatOnceTests(unittest.TestCase):
    def setUp(self):
        self.config = Config(
            api_key="secret-value",
            model="gpt-test",
            base_url="https://example.test/v1",
            timeout_s=5.0,
        )

    def test_missing_api_key(self):
        with self.assertRaisesRegex(ConfigError, "missing OPENAI_API_KEY"):
            load_config({"OPENAI_MODEL": "gpt-test"})

    def test_missing_model(self):
        with self.assertRaisesRegex(ConfigError, "missing OPENAI_MODEL"):
            load_config({"OPENAI_API_KEY": "secret-value"})

    def test_timeout_must_be_finite(self):
        """非有限超时（inf、nan）必须被拒绝，避免请求无限等待。"""

        base = {"OPENAI_API_KEY": "secret-value", "OPENAI_MODEL": "gpt-test"}

        for bad in ("inf", "nan", "0", "-1", "abc"):
            with self.subTest(bad=bad):
                with self.assertRaisesRegex(ConfigError, "positive number"):
                    load_config({**base, "OPENAI_TIMEOUT_S": bad})

    def test_chat_once_round_trip_with_fake_transport(self):
        transport = FakeTransport(
            HttpResponse(
                status_code=200,
                body=json.dumps(
                    {
                        "id": "resp_1",
                        "model": "gpt-test",
                        "output": [message_output("hello protocol")],
                    }
                ),
            )
        )
        response = chat_once("Hello", self.config, transport)
        self.assertEqual(response.text(), "hello protocol")
        self.assertIsNotNone(transport.last_request)
        assert transport.last_request is not None
        self.assertEqual(
            transport.last_request["url"],
            "https://example.test/v1/responses",
        )
        self.assertEqual(
            transport.last_request["headers"],
            {
                "Authorization": "Bearer secret-value",
                "Content-Type": "application/json",
            },
        )
        expected = encode_request(build_model_request("gpt-test", "Hello"))
        self.assertEqual(transport.last_request["payload"], expected)

    def test_complete_sends_the_exact_unified_request(self):
        transport = FakeTransport(
            HttpResponse(
                status_code=200,
                body=json.dumps({"output": [message_output("done")]}),
            )
        )
        request = ModelRequest(
            model="different-model",
            messages=(text_message(Role.USER, "Hello"),),
            tools=(
                ToolDefinition(
                    "echo",
                    "Echo text",
                    {"type": "object", "properties": {}},
                ),
            ),
        )
        response = complete(request, self.config, transport)
        self.assertEqual(response.text(), "done")
        assert transport.last_request is not None
        self.assertEqual(transport.last_request["payload"], encode_request(request))

    def test_format_response_includes_tool_use(self):
        response = ModelResponse(
            id=None,
            model=None,
            message=Message(
                role=Role.ASSISTANT,
                content=(
                    TextBlock(text="done"),
                    ToolUseBlock(id="c1", name="echo", input={"text": "x"}),
                ),
            ),
        )
        rendered = format_response(response)
        self.assertIn("done", rendered)
        self.assertIn("tool_use", rendered)
        self.assertIn("echo", rendered)


if __name__ == "__main__":
    unittest.main()
