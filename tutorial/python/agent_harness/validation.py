from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable

from ._base import digest


@dataclass(frozen=True)
class ValidationCheck:
    name: str
    passed: bool
    code: str = "validation_failed"
    message: str = ""


@dataclass(frozen=True)
class ValidationReport:
    checks: tuple[ValidationCheck, ...]

    @property
    def passed(self) -> bool:
        return bool(self.checks) and all(check.passed for check in self.checks)

    def failures(self) -> list[ValidationCheck]:
        return [check for check in self.checks if not check.passed]

    @property
    def digest(self) -> str:
        return digest(self)

    def to_dict(self) -> dict[str, Any]:
        return {"passed": self.passed, "checks": [check.__dict__ for check in self.checks],
                "digest": self.digest}


class Validator:
    def __init__(self, checks: dict[str, Callable[[Any], Any]] | list[Callable[[Any], Any]]):
        self.checks = checks

    def validate(self, value: Any) -> ValidationReport:
        output: list[ValidationCheck] = []
        if isinstance(self.checks, dict):
            checks = [(name, self.checks[name]) for name in sorted(self.checks)]
        else:
            checks = [(getattr(check, "__name__", f"check_{index}"), check)
                      for index, check in enumerate(self.checks)]
        for name, check in checks:
            try:
                passed = bool(check(value))
                output.append(ValidationCheck(name, passed,
                                              "ok" if passed else "validation_failed",
                                              "" if passed else f"{name} failed"))
            except Exception as exc:
                output.append(ValidationCheck(name, False, "validator_error", str(exc)))
        return ValidationReport(tuple(output))


def finish(report: ValidationReport) -> dict[str, Any]:
    """The only terminal success gate in the teaching runner."""
    if report.passed:
        return {"status": "Completed", "validation": report.to_dict(), "failures": []}
    return {"status": "Failed", "validation": report.to_dict(),
            "failures": [check.__dict__ for check in report.failures()]}
