from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable

from ._base import ContractError, SchemaError
from .protocol import CallStatus, ToolResult
from .tool_descriptor import ToolDescriptor


def validate_schema(schema: dict[str, Any], value: Any, path: str = "$") -> str | None:
    """Validate the small JSON-Schema subset used in the lessons."""
    if not isinstance(schema, dict):
        return f"{path}: schema must be an object"
    expected = schema.get("type")
    valid = {
        "object": isinstance(value, dict),
        "array": isinstance(value, list),
        "string": isinstance(value, str),
        "integer": isinstance(value, int) and not isinstance(value, bool),
        "number": isinstance(value, (int, float)) and not isinstance(value, bool),
        "boolean": isinstance(value, bool),
        "null": value is None,
    }
    if expected in valid and not valid[expected]:
        return f"{path}: expected {expected}"
    if "enum" in schema and value not in schema["enum"]:
        return f"{path}: value is not in enum"
    if expected == "object" and isinstance(value, dict):
        properties = schema.get("properties", {})
        for name in schema.get("required", []):
            if name not in value:
                return f"{path}.{name}: required"
        if schema.get("additionalProperties", True) is False:
            extra = sorted(set(value) - set(properties))
            if extra:
                return f"{path}.{extra[0]}: additional property"
        for name, child in properties.items():
            if name in value:
                error = validate_schema(child, value[name], f"{path}.{name}")
                if error:
                    return error
    if expected == "array" and isinstance(value, list) and "items" in schema:
        for index, item in enumerate(value):
            error = validate_schema(schema["items"], item, f"{path}[{index}]")
            if error:
                return error
    return None


@dataclass(frozen=True)
class ToolDefinition:
    """Provider-facing declaration; it is not a runtime permission."""

    name: str
    description: str
    parameters: dict[str, Any]

    def to_wire(self) -> dict[str, Any]:
        return {"name": self.name, "description": self.description,
                "parameters": self.parameters}

    def as_descriptor(self, domain: str, kind: Any,
                      output_schema: dict[str, Any],
                      source: str = "tutorial", version: str = "1") -> ToolDescriptor:
        return ToolDescriptor(self.name, domain, kind, self.parameters,
                              output_schema, source, version)


@dataclass(frozen=True)
class ToolSpec:
    descriptor: ToolDescriptor
    handler: Callable[[dict[str, Any]], Any]


class ToolRegistry:
    """Registry that owns schema checks and call-id replay protection."""

    def __init__(self, specs: list[ToolSpec] | tuple[ToolSpec, ...] = ()):
        self._specs: dict[str, ToolSpec] = {}
        self._calls: dict[str, ToolResult] = {}
        for spec in specs:
            self.register(spec)

    def register(self, spec: ToolSpec) -> None:
        if spec.descriptor.id in self._specs:
            raise ContractError(f"duplicate tool descriptor: {spec.descriptor.id}")
        self._specs[spec.descriptor.id] = spec

    def descriptor_list(self) -> tuple[ToolDescriptor, ...]:
        return tuple(self._specs[key].descriptor for key in sorted(self._specs))

    def get(self, tool_id: str) -> ToolSpec | None:
        return self._specs.get(tool_id)

    def call(self, call_id: str, tool_id: str, arguments: dict[str, Any]) -> ToolResult:
        if call_id in self._calls:
            return ToolResult(call_id, CallStatus.ERROR,
                              error={"code": "duplicate_call_id", "message": "call id already consumed"})
        spec = self._specs.get(tool_id)
        if spec is None:
            result = ToolResult(call_id, CallStatus.ERROR,
                                error={"code": "unknown_tool", "message": tool_id})
            self._calls[call_id] = result
            return result
        error = validate_schema(spec.descriptor.input_schema, arguments)
        if error:
            result = ToolResult(call_id, CallStatus.ERROR,
                                error={"code": "schema_error", "message": error})
            self._calls[call_id] = result
            return result
        try:
            output = spec.handler(arguments)
            error = validate_schema(spec.descriptor.output_schema, output)
            if error:
                result = ToolResult(call_id, CallStatus.ERROR,
                                    error={"code": "output_schema_error", "message": error})
            else:
                result = ToolResult(call_id, CallStatus.OK, output=output)
        except Exception as exc:
            result = ToolResult(call_id, CallStatus.ERROR,
                                error={"code": "tool_exception", "message": str(exc)})
        self._calls[call_id] = result
        return result
