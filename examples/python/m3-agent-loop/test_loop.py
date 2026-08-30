"""M3 Agent Loop 离线测试：不访问网络、不需要真实 API Key。

覆盖 book/src/ch5.md §5.6 列出的最小验证合同，并额外覆盖 duplicate
call_id、unknown tool、Cancel、BudgetExhausted、PolicyDenied 终态互斥。
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from typing import Callable

_M2 = Path(__file__).resolve().parent.parent / "m2-tool-runtime"
if str(_M2) not in sys.path:
    sys.path.append(str(_M2))

from tool_types import ToolCall, ToolStatus  # noqa: E402
from registry import ToolRegistry  # noqa: E402

from loop import run_loop
from loop_types import CallTool, CancelToken, Finish, Outcome, RunLimits
from model import ScriptedMockModel
from policy import AllowListPolicy
from loop_tools import CancellingTool, EchoTool, FailingTool
from validator import RequiredOutputValidator


def _make_registry(*, with_failing: bool = False, cancel_token: CancelToken | None = None) -> ToolRegistry:
    registry = ToolRegistry()
    registry.register(EchoTool())
    if with_failing:
        registry.register(FailingTool())
    if cancel_token is not None:
        registry.register(CancellingTool(cancel_token))
    return registry


def _echo_call(call_id: str, value: int) -> CallTool:
    return CallTool(ToolCall(call_id=call_id, name="echo", arguments={"value": value}))


class BasicLoopTests(unittest.TestCase):
    """§5.6 最小验证合同：三步脚本 + 不可达第四步。"""

    def test_budget_exhausted_before_next_model_action(self):
        # 三步：CallTool, CallTool, Finish；第四步（另一次 CallTool）不可达。
        # max_steps=2 意味着预算在第三次模型调用（Finish）之前就耗尽。
        script = [
            _echo_call("c1", 1),
            _echo_call("c2", 2),
            Finish("completed"),
            _echo_call("c4", 4),  # 不可达
        ]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(),
            policy=AllowListPolicy(["echo"]),
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=2),
        )

        self.assertIs(result.outcome, Outcome.BUDGET_EXHAUSTED)
        # 模型只被调用了两次：预算耗尽发生在下一次模型动作之前，不是之后。
        self.assertEqual(model.call_count, 2)
        self.assertEqual(result.model_call_count, 2)

    def test_tool_result_visible_in_next_model_input(self):
        script = [_echo_call("c1", 42), Finish("completed")]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(),
            policy=AllowListPolicy(["echo"]),
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=5),
        )

        self.assertIs(result.outcome, Outcome.COMPLETED)
        self.assertEqual(model.call_count, 2)
        first_input, second_input = model.received_inputs
        self.assertEqual(first_input, [])
        self.assertTrue(
            any(item.get("call_id") == "c1" and item.get("output") == 42 for item in second_input),
            f"second model input did not carry the first tool result: {second_input!r}",
        )

    def test_tool_failure_is_observable_not_uncaught(self):
        script = [
            CallTool(ToolCall(call_id="c1", name="fail", arguments={})),
            Finish("completed"),
        ]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(with_failing=True),
            policy=AllowListPolicy(["echo", "fail"]),
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=5),
        )

        # 工具失败不会终止循环或抛出未捕获异常；run 照常走到 Finish。
        self.assertIs(result.outcome, Outcome.COMPLETED)
        tool_results = [e.detail for e in result.events if e.kind == "tool_result"]
        self.assertEqual(len(tool_results), 1)
        self.assertEqual(tool_results[0].status, ToolStatus.FAILED)

    def test_finish_requires_validation_to_become_completed(self):
        script = [Finish("nope")]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(),
            policy=AllowListPolicy(["echo"]),
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=5),
        )

        self.assertIs(result.outcome, Outcome.FAILED)
        self.assertEqual(result.reason, "validation_failed")


class InvalidActionTests(unittest.TestCase):
    def test_duplicate_call_id_is_rejected(self):
        script = [_echo_call("c1", 1), _echo_call("c1", 2)]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(),
            policy=AllowListPolicy(["echo"]),
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=5),
        )

        self.assertIs(result.outcome, Outcome.FAILED)
        self.assertEqual(result.reason, "duplicate_call_id")
        # 第二次调用在被判定重复后不会真的执行工具。
        tool_results = [e for e in result.events if e.kind == "tool_result"]
        self.assertEqual(len(tool_results), 1)

    def test_unknown_tool_is_rejected_without_executing(self):
        script = [CallTool(ToolCall(call_id="c1", name="does-not-exist", arguments={}))]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(),
            policy=AllowListPolicy(["echo", "does-not-exist"]),
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=5),
        )

        self.assertIs(result.outcome, Outcome.FAILED)
        self.assertEqual(result.reason, "unknown_tool")
        tool_results = [e for e in result.events if e.kind == "tool_result"]
        self.assertEqual(tool_results, [])


class PolicyAndCancelTests(unittest.TestCase):
    def test_policy_denied_is_reachable(self):
        script = [CallTool(ToolCall(call_id="c1", name="fail", arguments={}))]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(with_failing=True),
            policy=AllowListPolicy(["echo"]),  # "fail" 已注册但不在白名单
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=5),
        )

        self.assertIs(result.outcome, Outcome.POLICY_DENIED)
        tool_results = [e for e in result.events if e.kind == "tool_result"]
        self.assertEqual(tool_results, [], "policy-denied call must not execute")

    def test_cancel_is_reachable_mid_run(self):
        token = CancelToken()
        script = [
            CallTool(ToolCall(call_id="c1", name="cancel_trigger", arguments={})),
            Finish("completed"),  # 不可达：取消发生在下一次模型调用之前
        ]
        model = ScriptedMockModel(script)
        result = run_loop(
            model=model,
            registry=_make_registry(cancel_token=token),
            policy=AllowListPolicy(["cancel_trigger"]),
            validator=RequiredOutputValidator("completed"),
            limits=RunLimits(max_steps=5),
            cancel_token=token,
        )

        self.assertIs(result.outcome, Outcome.CANCELLED)
        self.assertEqual(model.call_count, 1)


class TerminalOutcomeCoverageTests(unittest.TestCase):
    """五种终态互斥且都可达。

    互斥性是结构性的：`RunResult.outcome` 是单个字段，循环里的每条终止
    路径只执行一次 `return`（见 loop.py），不存在同一次 run 报告两个终态
    的代码路径。这里额外用一次覆盖性测试确认全部五种终态都真的被其他
    测试触发过，而不是定义了却从未被验证可达。
    """

    def test_all_five_outcomes_are_covered_by_the_suite(self):
        scenarios: dict[Outcome, Callable[[], object]] = {
            Outcome.COMPLETED: lambda: run_loop(
                model=ScriptedMockModel([Finish("completed")]),
                registry=_make_registry(),
                policy=AllowListPolicy(["echo"]),
                validator=RequiredOutputValidator("completed"),
                limits=RunLimits(max_steps=5),
            ),
            Outcome.FAILED: lambda: run_loop(
                model=ScriptedMockModel([Finish("nope")]),
                registry=_make_registry(),
                policy=AllowListPolicy(["echo"]),
                validator=RequiredOutputValidator("completed"),
                limits=RunLimits(max_steps=5),
            ),
            Outcome.BUDGET_EXHAUSTED: lambda: run_loop(
                model=ScriptedMockModel([_echo_call("c1", 1), Finish("completed")]),
                registry=_make_registry(),
                policy=AllowListPolicy(["echo"]),
                validator=RequiredOutputValidator("completed"),
                limits=RunLimits(max_steps=1),
            ),
            Outcome.POLICY_DENIED: lambda: run_loop(
                model=ScriptedMockModel(
                    [CallTool(ToolCall(call_id="c1", name="fail", arguments={}))]
                ),
                registry=_make_registry(with_failing=True),
                policy=AllowListPolicy(["echo"]),
                validator=RequiredOutputValidator("completed"),
                limits=RunLimits(max_steps=5),
            ),
            Outcome.CANCELLED: lambda: run_loop(
                model=ScriptedMockModel(
                    [CallTool(ToolCall(call_id="c1", name="cancel_trigger", arguments={}))]
                ),
                registry=_make_registry(cancel_token=(token := CancelToken())),
                policy=AllowListPolicy(["cancel_trigger"]),
                validator=RequiredOutputValidator("completed"),
                limits=RunLimits(max_steps=5),
                cancel_token=token,
            ),
        }

        achieved: dict[Outcome, Outcome] = {}
        for expected_outcome, build in scenarios.items():
            achieved[expected_outcome] = build().outcome

        for expected_outcome, actual_outcome in achieved.items():
            self.assertIs(
                actual_outcome,
                expected_outcome,
                f"scenario built for {expected_outcome} produced {actual_outcome} instead",
            )
        self.assertEqual(set(achieved.values()), set(Outcome))


if __name__ == "__main__":
    unittest.main()
