"""Small, dependency-free primitives shared by the chapter examples.

This package deliberately models contracts in memory. It is a teaching
implementation, not a durable database or an operating-system sandbox.
"""

from __future__ import annotations

import dataclasses
import hashlib
import json
import re
import types
from dataclasses import is_dataclass
from enum import Enum
from typing import Any, Mapping, Union, get_args, get_origin, get_type_hints


class HarnessError(Exception):
    """Base class for expected harness failures."""


class ContractError(HarnessError):
    pass


class IdentityMismatch(ContractError):
    pass


class AmbiguousResult(ContractError):
    pass


class SchemaError(ContractError):
    pass


class PolicyDenied(ContractError):
    pass


def to_data(value: Any) -> Any:
    """Convert public model values to deterministic JSON-compatible values."""
    if isinstance(value, Enum):
        return value.value
    if is_dataclass(value):
        return {field.name: to_data(getattr(value, field.name)) for field in dataclasses.fields(value)}
    if isinstance(value, Mapping):
        return {str(key): to_data(value[key]) for key in sorted(value, key=str)}
    if isinstance(value, (tuple, list)):
        return [to_data(item) for item in value]
    if isinstance(value, (set, frozenset)):
        return [to_data(item) for item in sorted(value, key=repr)]
    return value


def canonical(value: Any) -> str:
    """Canonical JSON used for replay and digest binding."""
    return json.dumps(to_data(value), ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def digest(value: Any) -> str:
    """Return a stable SHA-256 digest of a canonical representation."""
    return hashlib.sha256(canonical(value).encode("utf-8")).hexdigest()


def _convert(value: Any, annotation: Any) -> Any:
    if annotation is Any or annotation is None:
        return value
    origin = get_origin(annotation)
    args = get_args(annotation)
    if origin in (Union, types.UnionType):
        for option in args:
            if option is type(None) and value is None:
                return None
            if option is type(None):
                continue
            try:
                return _convert(value, option)
            except (TypeError, ValueError, KeyError):
                continue
        return value
    if origin in (list, tuple, set, frozenset):
        item_type = args[0] if args else Any
        converted = [_convert(item, item_type) for item in value]
        if origin is tuple:
            return tuple(converted)
        if origin is set:
            return set(converted)
        if origin is frozenset:
            return frozenset(converted)
        return converted
    if origin in (dict, Mapping):
        key_type, value_type = args if len(args) == 2 else (Any, Any)
        return {_convert(key, key_type): _convert(item, value_type) for key, item in value.items()}
    if isinstance(annotation, type) and issubclass(annotation, Enum):
        return annotation(value)
    if isinstance(annotation, type) and is_dataclass(annotation):
        return from_data(annotation, value)
    return value


def from_data(cls: type, value: Any) -> Any:
    """Rehydrate a dataclass from a wire dictionary."""
    if not isinstance(value, Mapping):
        raise ContractError(f"expected object for {cls.__name__}")
    hints = get_type_hints(cls)
    return cls(**{field.name: _convert(value[field.name], hints.get(field.name, field.type))
                  for field in dataclasses.fields(cls)})


_SECRET_KEY = re.compile(r"(authorization|api[-_]?key|token|secret|password|credential)", re.I)
_SECRET_VALUE = re.compile(
    r"(?i)\bBearer\s+[A-Za-z0-9._~+/=-]+|"
    r"\b(?:api[-_]?key|token|secret|password)\s*=\s*[^,\s]+"
)
_ABSOLUTE_PATH = re.compile(r"""(?<![A-Za-z0-9])/(?:Users|private|tmp|home|var)/[^\s'"]+""")


def redact(value: Any) -> Any:
    """Deeply redact common credential-shaped keys and values."""
    if isinstance(value, Mapping):
        return {key: "[REDACTED]" if _SECRET_KEY.search(str(key)) else redact(item)
                for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        converted = [redact(item) for item in value]
        return tuple(converted) if isinstance(value, tuple) else converted
    if isinstance(value, str):
        def replace(match: re.Match[str]) -> str:
            return "Bearer [REDACTED]" if match.group(0).lower().startswith("bearer") else match.group(0).split("=")[0] + "=[REDACTED]"
        value = _SECRET_VALUE.sub(replace, value)
        return _ABSOLUTE_PATH.sub("[REDACTED_PATH]", value)
    return value
