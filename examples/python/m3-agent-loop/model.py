"""M3：脚本化模型，用于确定性测试。

真实模型接口在 M0/M1 已经建模（`chat_once.complete`）；M3 的循环不关心
模型是如何产生动作的，测试里只需要一个按预定顺序回放 `ModelAction` 的
替身，并记录每次收到的输入，便于断言"上一轮工具结果是否进入了下一轮输入"。
"""

from __future__ import annotations

from typing import Sequence

from loop_types import ModelAction


class UnscriptedCallError(RuntimeError):
    """脚本已经用尽，但循环又调用了一次模型。"""


class ScriptedMockModel:
    """按顺序回放预先写好的动作序列。"""

    def __init__(self, script: Sequence[ModelAction]) -> None:
        self._script = list(script)
        self._cursor = 0
        self.received_inputs: list[list[dict]] = []

    @property
    def call_count(self) -> int:
        """模型被调用的次数——用于断言预算耗尽/取消后不再多调用一次。"""

        return len(self.received_inputs)

    def next_action(self, context: list[dict]) -> ModelAction:
        self.received_inputs.append(context)
        if self._cursor >= len(self._script):
            raise UnscriptedCallError(
                f"model called a {self._cursor + 1}-th time but script only "
                f"has {len(self._script)} scripted actions"
            )
        action = self._script[self._cursor]
        self._cursor += 1
        return action
