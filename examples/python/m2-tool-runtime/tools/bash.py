"""bash：参数化进程执行；默认关闭，也绝不经过 shell。"""

from __future__ import annotations

import subprocess
from typing import Any, Callable, Mapping, Sequence

from tool_types import ToolSpec
from workspace import Workspace

DEFAULT_TIMEOUT_S = 10.0
DEFAULT_MAX_OUTPUT_CHARS = 20_000
DEFAULT_ALLOWLIST = ("echo", "/bin/echo", "pwd", "/bin/pwd")
ProcessRunner = Callable[..., subprocess.CompletedProcess[str]]


class BashTool:
    """在 workspace cwd 下启动一个精确白名单程序。

    ``args`` 会逐项传给进程，不解释 ``;``、``|``、重定向或变量展开。
    allowlist、cwd、超时和输出截断只是教学级进程契约，不是 OS 沙箱。
    """

    def __init__(
        self,
        workspace: Workspace,
        *,
        enabled: bool = False,
        allowlist: Sequence[str] | None = None,
        default_timeout_s: float = DEFAULT_TIMEOUT_S,
        max_output_chars: int = DEFAULT_MAX_OUTPUT_CHARS,
        runner: ProcessRunner = subprocess.run,
    ) -> None:
        self.workspace = workspace
        self.enabled = enabled
        self.allowlist = tuple(allowlist) if allowlist is not None else DEFAULT_ALLOWLIST
        self.default_timeout_s = default_timeout_s
        self.max_output_chars = max_output_chars
        self._runner = runner
        self.spec = ToolSpec(
            name="bash",
            description="Run one allowlisted program in the workspace without a shell",
            input_schema={
                "type": "object",
                "required": ["program", "args"],
                "properties": {
                    "program": {"type": "string"},
                    "args": {"type": "array", "items": {"type": "string"}},
                    "timeout_s": {"type": "number"},
                },
                "additionalProperties": False,
            },
        )

    def validate_arguments(self, arguments: Mapping[str, Any]) -> None:
        """校验程序名、字符串参数数组和正超时。"""

        program = arguments.get("program")
        if not isinstance(program, str) or not program.strip():
            raise ValueError("program must be a non-empty string")
        args = arguments.get("args")
        if not isinstance(args, list) or any(not isinstance(item, str) for item in args):
            raise ValueError("args must be an array of strings")
        timeout = arguments.get("timeout_s")
        if timeout is not None and (
            not isinstance(timeout, (int, float))
            or isinstance(timeout, bool)
            or float(timeout) <= 0
        ):
            raise ValueError("timeout_s must be a positive number")

    def execute(self, arguments: Mapping[str, Any]) -> dict[str, Any]:
        """启动进程；非零退出和超时都抛错，由 Registry 收成结构化失败。"""

        if not self.enabled:
            raise RuntimeError("bash is disabled; enable explicitly for trusted demos only")

        program = str(arguments["program"])
        if program not in self.allowlist:
            raise RuntimeError("program rejected by exact allowlist")
        args = [str(item) for item in arguments["args"]]
        timeout = float(arguments.get("timeout_s") or self.default_timeout_s)
        try:
            completed = self._runner(
                [program, *args],
                shell=False,
                cwd=str(self.workspace.root),
                capture_output=True,
                text=True,
                timeout=timeout,
                check=False,
            )
        except subprocess.TimeoutExpired as error:
            raise RuntimeError(f"program timed out after {timeout}s") from error

        stdout, stdout_cut = self._truncate(completed.stdout or "")
        stderr, stderr_cut = self._truncate(completed.stderr or "")
        if completed.returncode != 0:
            detail = stderr.strip() or stdout.strip() or "no output"
            raise RuntimeError(f"exit {completed.returncode}: {detail}")
        return {
            "program": program,
            "args": args,
            "exit_code": completed.returncode,
            "stdout": stdout,
            "stderr": stderr,
            "truncated": stdout_cut or stderr_cut,
        }

    def _truncate(self, text: str) -> tuple[str, bool]:
        """限制单个输出流大小，避免将巨量日志塞回模型上下文。"""

        if len(text) <= self.max_output_chars:
            return text, False
        return text[: self.max_output_chars], True
