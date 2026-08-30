"""M3：最小 Validator。

`Finish` 只是模型的意图，不是结果。这里用子串匹配模拟"结果是否可接受"的
校验步骤——真实实现会检查 schema、必需字段或业务规则，但控制流的关键点
是一样的：不通过就不能变成 `Completed`。
"""

from __future__ import annotations


class RequiredOutputValidator:
    """要求 `Finish` 的输出包含某个子串才算通过。"""

    def __init__(self, required_substring: str) -> None:
        self._required = required_substring

    def validate(self, output: str) -> bool:
        return self._required in output
