"""M2 Tool Runtime 离线测试：仅使用临时 workspace，不访问网络。"""

from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridge import spec_to_tool_definition
from detect import Expectation, ExpectationKind, detect
from one_step import OneStepError, run_one_tool_step
from registry import ToolRegistry
from tools import build_default_registry
from tools.bash import BashTool
from tool_types import ToolCall, ToolStatus
from workspace import Workspace, WorkspaceError

from chat_once import Config, HttpResponse
from protocol import ModelRequest, Role, text_message

FIXTURES = Path(__file__).resolve().parent / "fixtures"


class WorkspaceTestCase(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="m2-test-")
        self.root = Path(self._tmp.name) / "ws"
        shutil.copytree(FIXTURES, self.root)
        self.workspace = Workspace(self.root)
        self.registry = build_default_registry(self.workspace, enable_bash=False)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def execute(self, name: str, arguments: dict, call_id: str = "c1"):
        return self.registry.execute(
            ToolCall(call_id=call_id, name=name, arguments=arguments)
        )


class WorkspacePathTests(WorkspaceTestCase):
    def test_rejects_escape(self):
        with self.assertRaises(WorkspaceError):
            self.workspace.resolve("../outside.txt")

    def test_read_escape_is_structured_failure(self):
        result = self.execute("read", {"path": "../secret.txt"})
        self.assertEqual(result.status, ToolStatus.FAILED)
        self.assertIn("escapes workspace", result.error or "")

    def test_rejects_absolute_escape_and_symlink_parent(self):
        with self.assertRaises(WorkspaceError):
            self.workspace.resolve(str(Path(self._tmp.name) / "outside.txt"))

        outside = Path(self._tmp.name) / "outside"
        outside.mkdir()
        link = self.root / "external-link"
        try:
            link.symlink_to(outside, target_is_directory=True)
        except OSError as error:
            self.skipTest(f"symlink unavailable: {error}")
        with self.assertRaises(WorkspaceError):
            self.workspace.resolve("external-link/new.txt")


class ReadWriteEditTests(WorkspaceTestCase):
    def test_read_with_offset_limit(self):
        result = self.execute("read", {"path": "hello.txt", "offset": 1, "limit": 1})
        self.assertTrue(result.succeeded)
        self.assertIn("hello workspace", result.output["content"])
        self.assertEqual(result.output["line_count"], 1)

    def test_write_and_read_round_trip(self):
        written = self.execute(
            "write",
            {"path": "notes/a.txt", "content": "payload"},
        )
        self.assertTrue(written.succeeded)
        read = self.execute("read", {"path": "notes/a.txt"})
        self.assertTrue(read.succeeded)
        self.assertIn("payload", read.output["content"])

    def test_edit_unique_match(self):
        result = self.execute(
            "edit",
            {
                "path": "src/app.py",
                "edits": [
                    {"oldText": 'MESSAGE = "alpha"', "newText": 'MESSAGE = "beta"'}
                ],
            },
        )
        self.assertTrue(result.succeeded)
        text = (self.root / "src/app.py").read_text(encoding="utf-8")
        self.assertIn('MESSAGE = "beta"', text)
        self.assertNotIn('MESSAGE = "alpha"', text)

    def test_edit_non_unique_fails(self):
        target = self.root / "dup.txt"
        target.write_text("x\nx\n", encoding="utf-8")
        result = self.execute(
            "edit",
            {"path": "dup.txt", "edits": [{"oldText": "x", "newText": "y"}]},
        )
        self.assertEqual(result.status, ToolStatus.FAILED)
        self.assertIn("unique", result.error or "")

    def test_overlapping_edit_and_replace_failure_leave_original_unchanged(self):
        target = self.root / "overlap.txt"
        target.write_text("abc", encoding="utf-8")
        overlap = self.execute(
            "edit",
            {
                "path": "overlap.txt",
                "edits": [
                    {"oldText": "abc", "newText": "x"},
                    {"oldText": "bc", "newText": "y"},
                ],
            },
        )
        self.assertFalse(overlap.succeeded)
        self.assertEqual(target.read_text(encoding="utf-8"), "abc")

        with mock.patch("workspace.os.replace", side_effect=OSError("replace failed")):
            failed = self.execute("write", {"path": "overlap.txt", "content": "new"})
        self.assertFalse(failed.succeeded)
        self.assertEqual(target.read_text(encoding="utf-8"), "abc")


class SearchListTests(WorkspaceTestCase):
    def test_ls_lists_entries(self):
        result = self.execute("ls", {"path": "."})
        self.assertTrue(result.succeeded)
        self.assertIn("hello.txt", result.output["entries"])
        self.assertIn("src/", result.output["entries"])

    def test_find_glob(self):
        result = self.execute("find", {"pattern": "**/*.py"})
        self.assertTrue(result.succeeded)
        self.assertTrue(any(item.endswith("app.py") for item in result.output["matches"]))

    def test_shared_glob_subset_and_find_limit(self):
        (self.root / "top.py").write_text("TOP", encoding="utf-8")
        (self.root / "src/a1.txt").write_text("a", encoding="utf-8")
        result = self.execute("find", {"pattern": "**/*.py"})
        self.assertEqual(
            result.output["matches"], ["src/app.py", "src/util.py", "top.py"]
        )
        question = self.execute("find", {"pattern": "src/a?.txt"})
        self.assertEqual(question.output["matches"], ["src/a1.txt"])
        limited = self.execute("find", {"pattern": "**/*", "limit": 1})
        self.assertEqual(limited.output["count"], 1)
        self.assertTrue(limited.output["truncated"])
        unsupported = self.execute("find", {"pattern": "src/[ab].txt"})
        self.assertFalse(unsupported.succeeded)
        self.assertIn("not supported", unsupported.error or "")

    def test_find_skips_unrelated_cross_boundary_symlink(self):
        outside = Path(self._tmp.name) / "outside-file.txt"
        outside.write_text("outside", encoding="utf-8")
        link = self.root / "escape-link.txt"
        try:
            link.symlink_to(outside)
        except OSError as error:
            self.skipTest(f"symlink unavailable: {error}")

        result = self.execute("find", {"pattern": "**/*"})
        self.assertTrue(result.succeeded)
        self.assertIn("hello.txt", result.output["matches"])
        self.assertNotIn("escape-link.txt", result.output["matches"])

    def test_grep_finds_symbol(self):
        result = self.execute(
            "grep",
            {"pattern": "MESSAGE", "glob": "**/*.py"},
        )
        self.assertTrue(result.succeeded)
        self.assertGreaterEqual(result.output["count"], 1)
        self.assertTrue(
            any(match["path"].endswith("app.py") for match in result.output["matches"])
        )

    def test_grep_limit(self):
        result = self.execute(
            "grep",
            {"pattern": ".", "glob": "**/*", "limit": 1},
        )
        self.assertTrue(result.succeeded)
        self.assertEqual(result.output["count"], 1)
        self.assertTrue(result.output["truncated"])

    def test_grep_regex_literal_ignore_case_and_context(self):
        target = self.root / "search.txt"
        target.write_text("before\nAlpha.\nafter\n", encoding="utf-8")
        regex = self.execute(
            "grep", {"pattern": r"alpha\.$", "ignoreCase": True, "context": 1}
        )
        self.assertEqual(regex.output["count"], 1)
        self.assertIn("before", regex.output["matches"][0]["snippet"])
        literal = self.execute("grep", {"pattern": "Alpha.", "literal": True})
        self.assertEqual(literal.output["count"], 1)

    def test_grep_skips_unrelated_cross_boundary_symlink(self):
        outside = Path(self._tmp.name) / "outside-file.txt"
        outside.write_text("MESSAGE outside", encoding="utf-8")
        link = self.root / "escape-link.txt"
        try:
            link.symlink_to(outside)
        except OSError as error:
            self.skipTest(f"symlink unavailable: {error}")

        result = self.execute("grep", {"pattern": "MESSAGE", "glob": "**/*"})
        self.assertTrue(result.succeeded)
        self.assertTrue(
            any(match["path"].endswith("app.py") for match in result.output["matches"])
        )


class BashAndRegistryTests(WorkspaceTestCase):
    def test_unknown_tool(self):
        result = self.execute("missing", {})
        self.assertEqual(result.status, ToolStatus.FAILED)
        self.assertIn("unknown tool", result.error or "")

    def test_argument_and_execution_errors_are_structured(self):
        invalid = self.execute("read", {})
        self.assertEqual(invalid.status, ToolStatus.FAILED)
        self.assertIn("invalid arguments", invalid.error or "")
        missing = self.execute("read", {"path": "missing.txt"})
        self.assertEqual(missing.status, ToolStatus.FAILED)
        self.assertIn("does not exist", missing.error or "")

    def test_bash_disabled_by_default(self):
        result = self.execute("bash", {"program": "echo", "args": ["hi"]})
        self.assertEqual(result.status, ToolStatus.FAILED)
        self.assertIn("disabled", result.error or "")

    def test_bash_allowlist_and_reject(self):
        registry = ToolRegistry()
        registry.register(
            BashTool(
                self.workspace,
                enabled=True,
                allowlist=["echo", "/bin/echo"],
            )
        )
        ok = registry.execute(
            ToolCall(
                call_id="b1", name="bash", arguments={"program": "echo", "args": ["hi; pwd"]}
            )
        )
        self.assertTrue(ok.succeeded)
        self.assertIn("hi; pwd", ok.output["stdout"])

        bad = registry.execute(
            ToolCall(call_id="b2", name="bash", arguments={"program": "rm", "args": ["-rf", "/"]})
        )
        self.assertEqual(bad.status, ToolStatus.FAILED)
        self.assertIn("allowlist", bad.error or "")

    def test_bash_timeout_nonzero_and_output_truncation(self):
        def timeout_runner(*_args, **_kwargs):
            raise subprocess.TimeoutExpired(cmd="fake", timeout=0.1)

        registry = ToolRegistry()
        registry.register(
            BashTool(
                self.workspace,
                enabled=True,
                allowlist=["fake"],
                runner=timeout_runner,
            )
        )
        result = registry.execute(
            ToolCall(
                call_id="t1",
                name="bash",
                arguments={"program": "fake", "args": [], "timeout_s": 0.1},
            )
        )
        self.assertEqual(result.status, ToolStatus.FAILED)
        self.assertIn("timed out", result.error or "")

        def completed(returncode=0, stdout=""):
            return lambda *_args, **_kwargs: subprocess.CompletedProcess(
                ["fake"], returncode, stdout=stdout, stderr="boom"
            )

        nonzero = BashTool(
            self.workspace, enabled=True, allowlist=["fake"], runner=completed(7)
        )
        registry.register(nonzero)
        failed = registry.execute_dict(
            call_id="n1", name="bash", arguments={"program": "fake", "args": []}
        )
        self.assertIn("exit 7", failed.error or "")

        registry.register(
            BashTool(
                self.workspace,
                enabled=True,
                allowlist=["fake"],
                max_output_chars=4,
                runner=completed(stdout="123456"),
            )
        )
        cut = registry.execute_dict(
            call_id="o1", name="bash", arguments={"program": "fake", "args": []}
        )
        self.assertEqual(cut.output["stdout"], "1234")
        self.assertTrue(cut.output["truncated"])


class ScriptedTransport:
    """按顺序返回离线响应，并保存两次请求供断言。"""

    def __init__(self, bodies: list[dict]) -> None:
        self.bodies = list(bodies)
        self.calls: list[dict] = []

    def post_json(self, url, headers, payload, timeout_s):
        self.calls.append({"url": url, "headers": headers, "payload": payload})
        return HttpResponse(200, json.dumps(self.bodies.pop(0)))


class OneStepClosureTests(WorkspaceTestCase):
    def setUp(self) -> None:
        super().setUp()
        self.config = Config("test-key", "gpt-test", "https://example.test/v1", 1)
        self.request = ModelRequest(
            model="gpt-test", messages=(text_message(Role.USER, "read hello.txt"),)
        )

    @staticmethod
    def function_response(*calls: tuple[str, str, dict]) -> dict:
        return {
            "id": "resp_1",
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

    @staticmethod
    def text_response(text: str = "done") -> dict:
        return {
            "id": "resp_2",
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

    # ANCHOR: m2-reading-case
    def test_fixed_two_call_closure_preserves_call_id(self):
        transport = ScriptedTransport(
            [self.function_response(("call_42", "read", {"path": "hello.txt"})), self.text_response()]
        )
        result = run_one_tool_step(self.request, self.config, transport, self.registry)
        self.assertEqual(result.tool_result.call_id, "call_42")
        self.assertIn("hello workspace", result.tool_result.output["content"])
        self.assertEqual(result.final_response.text(), "done")
        self.assertEqual(len(transport.calls), 2)
        self.assertEqual(len(transport.calls[0]["payload"]["tools"]), 7)
        outputs = transport.calls[1]["payload"]["input"]
        self.assertTrue(
            any(item.get("type") == "function_call_output" and item.get("call_id") == "call_42" for item in outputs)
        )
    # ANCHOR_END: m2-reading-case

    def test_rejects_zero_multiple_and_second_round_tool_calls(self):
        cases = [
            [self.text_response("no tool")],
            [self.function_response(("a", "read", {"path": "hello.txt"}), ("b", "ls", {"path": "."}))],
            [self.function_response(("a", "read", {"path": "hello.txt"})), self.function_response(("b", "ls", {"path": "."}))],
        ]
        for bodies in cases:
            with self.subTest(bodies=len(bodies)):
                with self.assertRaises(OneStepError):
                    run_one_tool_step(self.request, self.config, ScriptedTransport(bodies), self.registry)

    def test_bridge_exposes_provider_neutral_definition(self):
        definition = spec_to_tool_definition(self.registry.specs()[0])
        self.assertEqual(definition.name, "read")
        self.assertFalse(definition.strict)


class DetectionTests(WorkspaceTestCase):
    def test_detection_pass_and_fail(self):
        result = self.execute(
            "edit",
            {
                "path": "src/app.py",
                "edits": [
                    {"oldText": 'MESSAGE = "alpha"', "newText": 'MESSAGE = "beta"'}
                ],
            },
        )
        report = detect(
            self.workspace,
            result,
            [
                Expectation(ExpectationKind.TOOL_SUCCEEDED),
                Expectation(
                    ExpectationKind.FILE_CONTAINS,
                    path="src/app.py",
                    text="beta",
                ),
                Expectation(
                    ExpectationKind.FILE_NOT_CONTAINS,
                    path="src/app.py",
                    text="alpha",
                ),
            ],
        )
        self.assertTrue(report.passed)

        failed = detect(
            self.workspace,
            result,
            [
                Expectation(
                    ExpectationKind.FILE_CONTAINS,
                    path="src/app.py",
                    text="not-present",
                )
            ],
        )
        self.assertFalse(failed.passed)


class DemoImportTests(unittest.TestCase):
    def test_demo_scenario_passes(self):
        from demo import main

        self.assertEqual(main([]), 0)


if __name__ == "__main__":
    unittest.main()
