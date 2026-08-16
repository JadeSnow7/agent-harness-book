"""M2 demo：脚本化 tool_use → 执行 → tool_result → 检测反馈。

默认不启用 bash。所有文件操作发生在临时复制的 fixture workspace 中。
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

from bridge import call_from_dict, result_to_tool_result_block
from detect import Expectation, ExpectationKind, detect
from tools import build_default_registry
from tool_types import ToolResult
from workspace import Workspace

FIXTURES = Path(__file__).resolve().parent / "fixtures"


def copy_fixtures(target: Path) -> None:
    """把只读 fixtures 复制到可写临时目录。"""

    if target.exists():
        shutil.rmtree(target)
    shutil.copytree(FIXTURES, target)


def run_scenario(workspace_root: Path, *, enable_bash: bool = False) -> int:
    """运行单步教学场景并打印检测结果。"""

    workspace = Workspace(workspace_root)
    registry = build_default_registry(
        workspace,
        enable_bash=enable_bash,
        bash_allowlist=["echo", "/bin/echo", "pwd", "/bin/pwd"],
    )

    steps: list[tuple[dict, list[Expectation]]] = [
        (
            {
                "call_id": "c1",
                "name": "ls",
                "arguments": {"path": "."},
            },
            [
                Expectation(ExpectationKind.TOOL_SUCCEEDED, call_id="c1"),
                Expectation(ExpectationKind.OUTPUT_CONTAINS, text="hello.txt"),
            ],
        ),
        (
            {
                "call_id": "c2",
                "name": "read",
                "arguments": {"path": "hello.txt"},
            },
            [
                Expectation(ExpectationKind.TOOL_SUCCEEDED, call_id="c2"),
                Expectation(ExpectationKind.OUTPUT_CONTAINS, text="hello workspace"),
            ],
        ),
        (
            {
                "call_id": "c3",
                "name": "edit",
                "arguments": {
                    "path": "src/app.py",
                    "edits": [
                        {
                            "oldText": 'MESSAGE = "alpha"',
                            "newText": 'MESSAGE = "beta"',
                        }
                    ],
                },
            },
            [
                Expectation(ExpectationKind.TOOL_SUCCEEDED, call_id="c3"),
                Expectation(
                    ExpectationKind.FILE_CONTAINS,
                    path="src/app.py",
                    text='MESSAGE = "beta"',
                ),
                Expectation(
                    ExpectationKind.FILE_NOT_CONTAINS,
                    path="src/app.py",
                    text='MESSAGE = "alpha"',
                ),
            ],
        ),
        (
            {
                "call_id": "c4",
                "name": "grep",
                "arguments": {"pattern": "beta", "glob": "**/*.py"},
            },
            [
                Expectation(ExpectationKind.TOOL_SUCCEEDED, call_id="c4"),
                Expectation(ExpectationKind.OUTPUT_CONTAINS, text="src/app.py"),
            ],
        ),
        (
            {
                "call_id": "c5",
                "name": "find",
                "arguments": {"pattern": "**/*.txt"},
            },
            [
                Expectation(ExpectationKind.TOOL_SUCCEEDED, call_id="c5"),
                Expectation(ExpectationKind.OUTPUT_CONTAINS, text="hello.txt"),
            ],
        ),
        (
            {
                "call_id": "c6",
                "name": "write",
                "arguments": {
                    "path": "notes/out.md",
                    "content": "# done\n",
                },
            },
            [
                Expectation(ExpectationKind.TOOL_SUCCEEDED, call_id="c6"),
                Expectation(ExpectationKind.FILE_EXISTS, path="notes/out.md"),
                Expectation(
                    ExpectationKind.FILE_CONTAINS,
                    path="notes/out.md",
                    text="# done",
                ),
            ],
        ),
    ]

    if enable_bash:
        steps.append(
            (
                {
                    "call_id": "c7",
                    "name": "bash",
                    "arguments": {"program": "echo", "args": ["ok-from-bash"]},
                },
                [
                    Expectation(ExpectationKind.TOOL_SUCCEEDED, call_id="c7"),
                    Expectation(ExpectationKind.OUTPUT_CONTAINS, text="ok-from-bash"),
                ],
            )
        )
    else:
        steps.append(
            (
                {
                    "call_id": "c7",
                    "name": "bash",
                    "arguments": {"program": "echo", "args": ["should-fail"]},
                },
                [
                    Expectation(
                        ExpectationKind.TOOL_FAILED,
                        call_id="c7",
                        error_substring="disabled",
                    ),
                ],
            )
        )

    # 额外验证：未知工具与路径逃逸都是结构化失败。
    steps.append(
        (
            {
                "call_id": "c8",
                "name": "not-a-tool",
                "arguments": {},
            },
            [
                Expectation(
                    ExpectationKind.TOOL_FAILED,
                    call_id="c8",
                    error_substring="unknown tool",
                ),
            ],
        )
    )
    steps.append(
        (
            {
                "call_id": "c9",
                "name": "read",
                "arguments": {"path": "../outside.txt"},
            },
            [
                Expectation(
                    ExpectationKind.TOOL_FAILED,
                    call_id="c9",
                    error_substring="escapes workspace",
                ),
            ],
        )
    )

    all_passed = True
    for raw_call, expectations in steps:
        call = call_from_dict(raw_call)
        result = registry.execute(call)
        report = detect(workspace, result, expectations)
        _print_step(result, report)
        all_passed = all_passed and report.passed

    print("scenario=" + ("passed" if all_passed else "failed"))
    return 0 if all_passed else 1


def _print_step(result: ToolResult, report) -> None:
    block = result_to_tool_result_block(result)
    print("---")
    print(
        json.dumps(
            {
                "call_id": result.call_id,
                "name": result.name,
                "status": result.status.value,
                "error": result.error,
                "tool_result_is_error": block.is_error,
            },
            ensure_ascii=False,
        )
    )
    print(report.summary())


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="M2 tool runtime demo")
    parser.add_argument(
        "--enable-bash",
        action="store_true",
        help="enable whitelisted bash for this demo run",
    )
    parser.add_argument(
        "--workspace",
        type=Path,
        default=None,
        help="optional workspace directory; default uses a temp copy of fixtures",
    )
    args = parser.parse_args(argv)

    if args.workspace is not None:
        args.workspace.mkdir(parents=True, exist_ok=True)
        if not any(args.workspace.iterdir()):
            copy_fixtures(args.workspace)
        return run_scenario(args.workspace, enable_bash=args.enable_bash)

    with tempfile.TemporaryDirectory(prefix="m2-tool-runtime-") as tmp:
        root = Path(tmp) / "workspace"
        copy_fixtures(root)
        return run_scenario(root, enable_bash=args.enable_bash)


if __name__ == "__main__":
    raise SystemExit(main())
