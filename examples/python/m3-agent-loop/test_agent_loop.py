"""M3 Agent Loop 离线测试：脚本化 Transport，不访问真实网络或凭据。"""

from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

# 先导入 agent_loop，让它完成 sys.path 插入（m2-tool-runtime，进而 m1-unified-protocol），
# 后面几行才能直接按名字导入相邻里程碑目录里的模块。
from agent_loop import (
    AgentLoopLimits,
    StopReason,
    run_agent_loop,
)

from chat_once import Config, HttpResponse  # noqa: E402
from protocol import Message, ModelRequest, ModelResponse, Role, ToolResultBlock, text_message  # noqa: E402
from registry import ToolRegistry  # noqa: E402
from tools.read import ReadTool  # noqa: E402
from workspace import Workspace  # noqa: E402


class ScriptedTransport:
    """按顺序返回离线响应，并记录每次请求供断言；耗尽后抛 IndexError。

    耗尽后抛错是有意的：如果循环在预算之外多打了一次模型调用，测试会
    立即因为 ``bodies.pop(0)`` 报错而失败，而不是安静地返回空结果。
    """

    def __init__(self, bodies: list[dict]) -> None:
        self.bodies = list(bodies)
        self.calls: list[dict] = []

    def post_json(self, url, headers, payload, timeout_s):
        self.calls.append({"url": url, "headers": headers, "payload": payload})
        return HttpResponse(200, json.dumps(self.bodies.pop(0)))


def function_response(*calls: tuple[str, str, dict]) -> dict:
    return {
        "id": "resp",
        "model": "gpt-test",
        "status": "completed",
        "output": [
            {
                "type": "function_call",
                "call_id": call_id,
                "name": name,
                "arguments": json.dumps(arguments),
            }
            for call_id, name, arguments in calls
        ],
    }


def text_response(text: str) -> dict:
    return {
        "id": "resp",
        "model": "gpt-test",
        "status": "completed",
        "output": [
            {
                "type": "message",
                "role": "assistant",
                "content": [{"type": "output_text", "text": text}],
            }
        ],
    }


def empty_output_response() -> dict:
    """decode_response 会因为 blocks 为空而拒绝这种响应，不是本模块编造的场景。"""

    return {"id": "resp", "model": "gpt-test", "status": "completed", "output": []}


class AgentLoopTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="m3-test-")
        root = Path(self._tmp.name) / "ws"
        root.mkdir()
        (root / "hello.txt").write_text("hello workspace\n", encoding="utf-8")
        self.workspace = Workspace(root)
        self.registry = ToolRegistry()
        self.registry.register(ReadTool(self.workspace))
        self.config = Config("test-key", "gpt-test", "https://example.test/v1", 1)
        self.request = ModelRequest(
            model="gpt-test", messages=(text_message(Role.USER, "read hello.txt"),)
        )
        self.generous_limits = AgentLoopLimits(max_steps=5, max_tool_calls=5)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    # ANCHOR: m3-agent-loop-case
    def test_two_round_loop_completes_and_preserves_call_id(self):
        transport = ScriptedTransport(
            [
                function_response(("call_42", "read", {"path": "hello.txt"})),
                text_response("done"),
            ]
        )
        result = run_agent_loop(
            self.request, self.config, transport, self.registry, self.generous_limits
        )
        self.assertEqual(result.stop_reason, StopReason.COMPLETED)
        self.assertEqual(result.final_text, "done")
        self.assertEqual(len(result.steps), 2)
        self.assertEqual(result.steps[0].tool_result.call_id, "call_42")
        self.assertTrue(result.steps[0].tool_result.succeeded)
        self.assertIn("hello workspace", result.steps[0].tool_result.output["content"])
        self.assertEqual(len(transport.calls), 2)
        outputs = transport.calls[1]["payload"]["input"]
        self.assertTrue(
            any(
                item.get("type") == "function_call_output" and item.get("call_id") == "call_42"
                for item in outputs
            )
        )
    # ANCHOR_END: m3-agent-loop-case

    def test_step_budget_stops_before_an_extra_model_call(self):
        # 只放一个响应体：如果循环在预算耗尽后还多打一次模型调用，
        # ScriptedTransport 会在第二次 pop 时因为列表已空而抛 IndexError。
        transport = ScriptedTransport(
            [function_response(("c1", "read", {"path": "hello.txt"}))]
        )
        limits = AgentLoopLimits(max_steps=1, max_tool_calls=5)
        result = run_agent_loop(self.request, self.config, transport, self.registry, limits)
        self.assertEqual(result.stop_reason, StopReason.BUDGET_EXHAUSTED)
        self.assertEqual(len(transport.calls), 1)

    def test_tool_call_budget_stops_without_executing_beyond_the_limit(self):
        transport = ScriptedTransport(
            [
                function_response(("c1", "read", {"path": "hello.txt"})),
                function_response(("c2", "read", {"path": "hello.txt"})),
            ]
        )
        limits = AgentLoopLimits(max_steps=5, max_tool_calls=1)
        result = run_agent_loop(self.request, self.config, transport, self.registry, limits)
        self.assertEqual(result.stop_reason, StopReason.BUDGET_EXHAUSTED)
        # 第二轮的工具候选被观察到但没有被执行：steps 里只有一条 tool_result。
        executed = [step for step in result.steps if step.tool_result is not None]
        self.assertEqual(len(executed), 1)

    def test_multiple_tool_candidates_require_human_review(self):
        transport = ScriptedTransport(
            [
                function_response(
                    ("a", "read", {"path": "hello.txt"}),
                    ("b", "read", {"path": "hello.txt"}),
                )
            ]
        )
        result = run_agent_loop(
            self.request, self.config, transport, self.registry, self.generous_limits
        )
        self.assertEqual(result.stop_reason, StopReason.AMBIGUOUS_TOOL_REQUEST)
        # 循环没有替读者挑一个候选执行：这一轮不产生任何 tool_result。
        self.assertIsNone(result.steps[-1].tool_result)
        self.assertEqual(len(transport.calls), 1)

    def test_tool_failure_becomes_an_observation_and_loop_continues(self):
        transport = ScriptedTransport(
            [
                function_response(("c1", "read", {"path": "missing.txt"})),
                text_response("done"),
            ]
        )
        result = run_agent_loop(
            self.request, self.config, transport, self.registry, self.generous_limits
        )
        self.assertEqual(result.stop_reason, StopReason.COMPLETED)
        self.assertFalse(result.steps[0].tool_result.succeeded)
        outputs = transport.calls[1]["payload"]["input"]
        failure_items = [
            item
            for item in outputs
            if item.get("type") == "function_call_output" and item.get("call_id") == "c1"
        ]
        self.assertEqual(len(failure_items), 1)

    def test_decode_failure_stops_safely_instead_of_crashing(self):
        transport = ScriptedTransport([empty_output_response()])
        result = run_agent_loop(
            self.request, self.config, transport, self.registry, self.generous_limits
        )
        self.assertEqual(result.stop_reason, StopReason.UNRECOGNIZED_ACTION)
        self.assertIsNotNone(result.error)

    def test_defensive_branch_for_a_response_with_neither_tool_use_nor_text(self):
        # M1 的 decode_response 保证成功解码的响应必然有工具候选或非空文本
        # 之一，这条分支用真实 Transport 造不出来；直接构造一个不经过解码的
        # ModelResponse（内容块是 tool_result 而不是 text/tool_use）来验证
        # run_agent_loop 自身的防御逻辑，而不是伪装成端到端场景。
        odd_response = ModelResponse(
            id="resp",
            model="gpt-test",
            message=Message(
                role=Role.ASSISTANT,
                content=(ToolResultBlock(tool_use_id="x", content="unexpected"),),
            ),
        )
        with mock.patch("agent_loop._complete", return_value=odd_response):
            result = run_agent_loop(
                self.request,
                self.config,
                ScriptedTransport([]),
                self.registry,
                self.generous_limits,
            )
        self.assertEqual(result.stop_reason, StopReason.UNRECOGNIZED_ACTION)
        self.assertIsNone(result.error)


if __name__ == "__main__":
    unittest.main()
