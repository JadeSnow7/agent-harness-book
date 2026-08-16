"""单步 postcondition checker：检查一次 ToolResult 与 workspace 状态。"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any

from tool_types import ToolResult
from workspace import Workspace


class ExpectationKind(str, Enum):
    TOOL_SUCCEEDED = "tool_succeeded"
    TOOL_FAILED = "tool_failed"
    OUTPUT_CONTAINS = "output_contains"
    FILE_CONTAINS = "file_contains"
    FILE_NOT_CONTAINS = "file_not_contains"
    FILE_EXISTS = "file_exists"


@dataclass(frozen=True)
class Expectation:
    kind: ExpectationKind
    text: str | None = None
    path: str | None = None
    call_id: str | None = None
    error_substring: str | None = None


@dataclass(frozen=True)
class CheckResult:
    expectation: Expectation
    passed: bool
    detail: str


@dataclass(frozen=True)
class DetectionReport:
    checks: tuple[CheckResult, ...]

    @property
    def passed(self) -> bool:
        return all(check.passed for check in self.checks)

    def summary(self) -> str:
        status = "passed" if self.passed else "failed"
        lines = [f"detection={status} checks={len(self.checks)}"]
        for check in self.checks:
            mark = "ok" if check.passed else "FAIL"
            lines.append(f"  [{mark}] {check.expectation.kind.value}: {check.detail}")
        return "\n".join(lines)


def detect(
    workspace: Workspace,
    result: ToolResult,
    expectations: list[Expectation],
) -> DetectionReport:
    """对单次工具结果运行局部后置条件；不生成 M7 任务级 Evidence。"""

    checks = [
        _evaluate(workspace, result, expectation) for expectation in expectations
    ]
    return DetectionReport(checks=tuple(checks))


def _evaluate(
    workspace: Workspace,
    result: ToolResult,
    expectation: Expectation,
) -> CheckResult:
    kind = expectation.kind

    if kind is ExpectationKind.TOOL_SUCCEEDED:
        if expectation.call_id and result.call_id != expectation.call_id:
            return CheckResult(expectation, False, "call_id mismatch")
        ok = result.succeeded
        return CheckResult(
            expectation,
            ok,
            "tool succeeded" if ok else f"tool failed: {result.error}",
        )

    if kind is ExpectationKind.TOOL_FAILED:
        if expectation.call_id and result.call_id != expectation.call_id:
            return CheckResult(expectation, False, "call_id mismatch")
        if result.succeeded:
            return CheckResult(expectation, False, "expected failure, got success")
        if expectation.error_substring:
            haystack = result.error or ""
            ok = expectation.error_substring in haystack
            return CheckResult(
                expectation,
                ok,
                "error matched" if ok else f"error did not contain {expectation.error_substring!r}",
            )
        return CheckResult(expectation, True, f"failed as expected: {result.error}")

    if kind is ExpectationKind.OUTPUT_CONTAINS:
        text = expectation.text or ""
        blob = _output_blob(result)
        ok = text in blob
        return CheckResult(
            expectation,
            ok,
            "output contains text" if ok else f"output missing {text!r}",
        )

    if kind is ExpectationKind.FILE_EXISTS:
        path = expectation.path or ""
        try:
            target = workspace.resolve(path, must_exist=False)
            ok = target.exists()
        except Exception as error:  # noqa: BLE001
            return CheckResult(expectation, False, str(error))
        return CheckResult(
            expectation,
            ok,
            "file exists" if ok else f"missing file {path!r}",
        )

    if kind is ExpectationKind.FILE_CONTAINS:
        return _file_text_check(workspace, expectation, should_contain=True)

    if kind is ExpectationKind.FILE_NOT_CONTAINS:
        return _file_text_check(workspace, expectation, should_contain=False)

    return CheckResult(expectation, False, f"unknown expectation: {kind}")


def _file_text_check(
    workspace: Workspace,
    expectation: Expectation,
    *,
    should_contain: bool,
) -> CheckResult:
    path = expectation.path or ""
    text = expectation.text or ""
    try:
        target = workspace.resolve(path, must_exist=True)
        content = target.read_text(encoding="utf-8")
    except Exception as error:  # noqa: BLE001
        return CheckResult(expectation, False, str(error))

    contains = text in content
    ok = contains if should_contain else not contains
    if should_contain:
        detail = "file contains text" if ok else f"file missing {text!r}"
    else:
        detail = "file does not contain text" if ok else f"file still contains {text!r}"
    return CheckResult(expectation, ok, detail)


def _output_blob(result: ToolResult) -> str:
    if not result.succeeded:
        return result.error or ""
    output: Any = result.output
    if output is None:
        return ""
    if isinstance(output, str):
        return output
    import json

    return json.dumps(output, ensure_ascii=False)
