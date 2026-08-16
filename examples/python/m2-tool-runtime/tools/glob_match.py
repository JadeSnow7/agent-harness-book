"""Python/Rust 共享的教学 glob 子集：``*``、``**`` 与 ``?``。"""

from __future__ import annotations

import re


def glob_matches(pattern: str, relative_path: str) -> bool:
    """按 POSIX 相对路径匹配；字符类和平台扩展不在本章范围内。"""

    if "[" in pattern or "]" in pattern:
        raise ValueError("glob character classes are not supported")
    pattern = pattern.replace("\\", "/")
    relative_path = relative_path.replace("\\", "/")
    while relative_path.startswith("./"):
        relative_path = relative_path[2:]
    pieces: list[str] = ["^"]
    index = 0
    while index < len(pattern):
        char = pattern[index]
        if char == "*" and index + 1 < len(pattern) and pattern[index + 1] == "*":
            index += 2
            if index < len(pattern) and pattern[index] == "/":
                pieces.append("(?:.*/)?")
                index += 1
            else:
                pieces.append(".*")
            continue
        if char == "*":
            pieces.append("[^/]*")
        elif char == "?":
            pieces.append("[^/]")
        else:
            pieces.append(re.escape(char))
        index += 1
    pieces.append("$")
    return re.fullmatch("".join(pieces), relative_path) is not None
