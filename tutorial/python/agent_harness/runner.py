from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from ._base import canonical
from .events import EventEnvelope, EventKind, EventLog
from .identity import Identity, intent_digest
from .idempotency import IdempotencyLedger, Outcome
from .policy import authorize
from .protocol import (CallStatus, EffectIntent, EffectKind, Message, ModelRequest,
                       Role, ToolResult)
from .stop_policy import ProgressOracle, StopPolicy, StopReason
from .tool_definition import ToolRegistry, validate_schema
from .validation import Validator, finish


@dataclass(frozen=True)
class RunResult:
    reason: StopReason
    text: str
    steps: int
    errors: tuple[dict[str, Any], ...] = ()
    terminal: dict[str, Any] | None = None


class Runner:
    """A deterministic, serial model/tool loop for CH13-CH16.

    It intentionally has no network, credentials, background workers, or
    side-effect executor. Effect tools stop at the policy seam unless a
    caller explicitly configures a policy allowing them.
    """

    def __init__(self, provider: Any, tools: ToolRegistry | None = None,
                 validator: Validator | None = None,
                 event_log: EventLog | None = None,
                 policy: StopPolicy | None = None,
                 allow_effects: bool = False,
                 ledger: IdempotencyLedger | None = None):
        self.provider = provider
        self.tools = tools
        self.validator = validator or Validator({"nonempty": lambda value: bool(value)})
        self.event_log = event_log
        self.policy = policy or StopPolicy(step_limit=8)
        self.allow_effects = allow_effects
        self.ledger = ledger or IdempotencyLedger()

    def _log(self, kind: str, identity: Identity, payload: dict[str, Any],
             terminal: bool = False) -> None:
        if self.event_log is None:
            return
        self.event_log.append(EventEnvelope(self.event_log.seq + 1, kind, identity,
                                            payload, terminal))

    def _terminal(self, reason: StopReason, text: str, steps: int,
                  errors: tuple[dict[str, Any], ...], identity: Identity,
                  details: dict[str, Any] | None = None) -> RunResult:
        terminal = {"status": reason.value, "details": details or {}}
        self._log(EventKind.RUN_FINISHED, identity, terminal, terminal=True)
        return RunResult(reason, text, steps, errors, terminal)

    def run(self, prompt: str, run_id: str = "run", cancelled: bool = False) -> RunResult:
        identity = self.event_log.identity if self.event_log and self.event_log.identity else Identity(
            "session", "task", run_id)
        if self.event_log is not None and self.event_log.identity is not None:
            identity = self.event_log.identity
        oracle = ProgressOracle(self.policy.step_limit or 8, self.policy.tool_limit)
        messages: list[Message] = [Message(Role.USER, prompt)]
        seen_call_ids: set[str] = set()
        self._log(EventKind.RUN_STARTED, identity, {"prompt": prompt})

        for _ in range(oracle.step_limit):
            if cancelled:
                return self._terminal(StopReason.CANCELLED, "", oracle.steps, (), identity)
            try:
                request = ModelRequest(tuple(messages),
                                       tools=self.tools.descriptor_list() if self.tools else ())
                self._log(EventKind.MODEL_REQUESTED, identity,
                          {"messages": [message.to_wire() for message in messages]})
                response = self.provider.complete(request)
            except Exception as exc:
                error = {"code": "provider_error", "message": str(exc)}
                return self._terminal(StopReason.ESCALATE, "", oracle.steps,
                                      (error,), identity)

            oracle.observe(("model", response.text, tuple(call.call_id for call in response.tool_calls)))
            self._log(EventKind.MODEL_RESPONDED, identity, response.to_wire())
            if response.text:
                messages.append(Message(Role.ASSISTANT, response.text))

            if response.tool_calls:
                response_ids = [call.call_id for call in response.tool_calls]
                duplicates = {call_id for call_id in response_ids
                              if response_ids.count(call_id) > 1 or call_id in seen_call_ids}
                if duplicates:
                    error = {"code": "duplicate_call_id",
                             "message": "duplicate call id: " + ", ".join(sorted(duplicates))}
                    return self._terminal(StopReason.ESCALATE, response.text, oracle.steps,
                                          (error,), identity)
                seen_call_ids.update(response_ids)
                for call in response.tool_calls:
                    oracle.observe(("tool", call.call_id), tool=True)
                    if self.tools is None:
                        error = {"code": "no_tool_registry", "message": call.tool_id}
                        return self._terminal(StopReason.ESCALATE, "", oracle.steps,
                                              (error,), identity)
                    spec = self.tools.get(call.tool_id)
                    if spec is None:
                        result = self.tools.call(call.call_id, call.tool_id, call.arguments)
                        error = result.error or {"code": "unknown_tool"}
                        self._log(EventKind.POLICY_DECISION, identity, {
                            "callId": call.call_id, "decision": "deny",
                            "reason": "unknown tool"
                        })
                        self._log(EventKind.TOOL_RESULT, identity, result.to_wire())
                        messages.append(Message(Role.TOOL, canonical(result.to_wire()), call.call_id))
                        continue
                    schema_error = validate_schema(spec.descriptor.input_schema, call.arguments)
                    if schema_error:
                        result = ToolResult(call.call_id, CallStatus.ERROR,
                                            error={"code": "schema_error", "message": schema_error})
                        self._log(EventKind.TOOL_RESULT, identity, result.to_wire())
                        messages.append(Message(Role.TOOL, canonical(result.to_wire()), call.call_id))
                        continue
                    intent = EffectIntent(call.call_id, call.tool_id, spec.descriptor.domain,
                                          spec.descriptor.kind, "run:" + identity.run_id,
                                          spec.descriptor.digest, identity.run_id,
                                          call.arguments)
                    self._log(EventKind.EFFECT_INTENT, identity, {
                        "intent": intent.to_wire(), "descriptor": spec.descriptor.to_wire()})
                    decision = authorize(intent, allow_effects=self.allow_effects)
                    self._log(EventKind.POLICY_DECISION, identity, decision.to_wire())
                    if decision.decision != "allow":
                        error = {"code": "approval_required" if decision.decision == "ask"
                                 else "policy_denied", "message": decision.reason}
                        messages.append(Message(Role.TOOL, canonical({
                            "callId": call.call_id, "status": "error", "error": error
                        }), call.call_id))
                        return self._terminal(StopReason.ESCALATE, response.text, oracle.steps,
                                              (error,), identity)
                    ledger_key = f"{identity.run_id}:{call.call_id}"
                    reservation = None
                    if spec.descriptor.kind is EffectKind.EFFECT:
                        reservation = self.ledger.reserve(ledger_key, identity,
                                                           intent_digest(intent))
                        if reservation.outcome is Outcome.COMPLETED:
                            cached = ToolResult.from_wire(reservation.result)
                            messages.append(Message(Role.TOOL, canonical(cached.to_wire()), call.call_id))
                            continue
                        if reservation.outcome is Outcome.IN_PROGRESS:
                            error = {"code": "in_progress", "message": "effect is already reserved"}
                            return self._terminal(StopReason.ESCALATE, response.text,
                                                  oracle.steps, (error,), identity)
                        self.ledger.executing(ledger_key, reservation.token)
                    self._log(EventKind.TOOL_STARTED, identity,
                              {"callId": call.call_id, "toolId": call.tool_id})
                    result = self.tools.call(call.call_id, call.tool_id, call.arguments)
                    self._log(EventKind.TOOL_RESULT, identity, result.to_wire())
                    messages.append(Message(Role.TOOL, canonical(result.to_wire()), call.call_id))
                    if reservation is not None:
                        if result.ok:
                            self.ledger.complete(ledger_key, result.to_wire(), reservation.token)
                        elif result.error and result.error.get("code") in {
                                "tool_exception", "output_schema_error"}:
                            self.ledger.ambiguous(ledger_key, reservation.token)
                            return self._terminal(
                                StopReason.ESCALATE, response.text, oracle.steps,
                                (result.error,), identity)
                        else:
                            self.ledger.fail(ledger_key, result.error, reservation.token)
                reason = self.policy.decide(oracle)
                if reason is not StopReason.CONTINUE:
                    return self._terminal(reason, response.text, oracle.steps, (), identity)
                continue

            if response.text:
                report = self.validator.validate(response.text)
                if report.passed:
                    return self._terminal(StopReason.COMPLETED, response.text, oracle.steps,
                                          (), identity, finish(report))
                errors = tuple(check.__dict__ for check in report.failures())
                return self._terminal(StopReason.ESCALATE, response.text, oracle.steps,
                                      errors, identity, finish(report))

            reason = self.policy.decide(oracle)
            if reason is not StopReason.CONTINUE:
                return self._terminal(reason, "", oracle.steps, (), identity)

        return self._terminal(StopReason.BUDGET_EXHAUSTED, "", oracle.steps, (), identity)
