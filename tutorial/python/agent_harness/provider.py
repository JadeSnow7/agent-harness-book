from __future__ import annotations

from dataclasses import dataclass, field

from .protocol import ModelRequest, ModelResponse


@dataclass
class DeterministicFakeModel:
    """Offline provider: each request consumes one predeclared response."""

    responses: list[ModelResponse]
    calls: int = 0
    requests: list[ModelRequest] = field(default_factory=list)

    def complete(self, request: ModelRequest) -> ModelResponse:
        self.calls += 1
        self.requests.append(request)
        if not self.responses:
            raise RuntimeError("provider_exhausted")
        response = self.responses.pop(0)
        if not isinstance(response, ModelResponse):
            raise TypeError("fake responses must be ModelResponse values")
        return response


FakeProvider = DeterministicFakeModel
