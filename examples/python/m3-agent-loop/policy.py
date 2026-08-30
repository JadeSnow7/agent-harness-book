"""M3：最小 allow-list Policy。

只回答"这个工具名允许执行吗"，不做参数级策略、速率限制或审批流——那些是
后续里程碑的问题。
"""

from __future__ import annotations

from typing import Iterable


class AllowListPolicy:
    """按工具名的白名单。"""

    def __init__(self, allowed: Iterable[str]) -> None:
        self._allowed = set(allowed)

    def check(self, tool_name: str) -> bool:
        """工具名是否在白名单中。"""

        return tool_name in self._allowed
