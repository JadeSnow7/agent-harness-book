"""M3 demo：脚本化模型先调用 echo，再 Finish，打印终态和事件数。

对应 Rust 侧 `examples/rust/m3-agent-loop`（`cargo run -p m3-agent-loop`）
的同一条确定性路径，方便两侧对照。
"""

from __future__ import annotations

import sys
from pathlib import Path

_M2 = Path(__file__).resolve().parent.parent / "m2-tool-runtime"
if str(_M2) not in sys.path:
    sys.path.append(str(_M2))

from tool_types import ToolCall  # noqa: E402
from registry import ToolRegistry  # noqa: E402

from loop import run_loop
from loop_types import CallTool, Finish, Outcome, RunLimits
from model import ScriptedMockModel
from policy import AllowListPolicy
from loop_tools import EchoTool
from validator import RequiredOutputValidator


def main(argv: list[str] | None = None) -> int:
    registry = ToolRegistry()
    registry.register(EchoTool())

    script = [
        CallTool(ToolCall(call_id="call-1", name="echo", arguments={"value": 7})),
        Finish(output="echo completed"),
    ]
    model = ScriptedMockModel(script)

    result = run_loop(
        model=model,
        registry=registry,
        policy=AllowListPolicy(["echo"]),
        validator=RequiredOutputValidator("completed"),
        limits=RunLimits(max_steps=3),
    )

    print(f"outcome={result.outcome.value}")
    print(f"event_count={len(result.events)}")
    print(f"model_call_count={result.model_call_count}")
    return 0 if result.outcome is Outcome.COMPLETED else 1


if __name__ == "__main__":
    raise SystemExit(main())
