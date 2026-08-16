# P0 Evaluation Fixture Contract

Status: contract-only, pre-implementation

This document defines the stable, implementation-neutral shape of the P0 evaluation fixtures. It does not describe an implemented Rust API and does not claim that any scenario currently passes. The fixture data in [`fixtures/p0-scenarios.json`](../../fixtures/p0-scenarios.json) is the source of truth for the nine P0 scenarios; the matrix in [`evals/p0-test-matrix.md`](../../evals/p0-test-matrix.md) is the review and coverage view.

## Goals and boundaries

The fixtures must be executable-looking enough for a later evaluator to consume them, while remaining independent of final crate names, Rust types, transport details, and provider APIs. A future adapter may translate the logical vocabulary below into concrete implementation events.

The P0 slice is deterministic and local:

- the model is a scripted Mock Model;
- tools have fixture-defined results and never require network access;
- policy decisions are explicit in the fixture;
- timestamps, UUIDs, and other run-specific identifiers are not part of equality checks;
- no API key, authorization header, real provider, or external service belongs in a fixture.

## Fixture envelope

The JSON document has this shape:

```text
{
  "schema_version": "p0-eval-1",
  "status": "contract_only",
  "scenarios": [ Scenario, ... ]
}
```

Each `Scenario` must contain these fields:

| Field | Meaning |
|---|---|
| `id` | Stable snake_case scenario identifier. |
| `purpose` | Human-readable contract being exercised. |
| `request` | Deterministic request input supplied to the harness. |
| `mock_model` | Ordered model actions and the conditions under which each action is emitted. |
| `available_tools` | Complete tool catalog visible to the scenario, including deterministic behavior. |
| `policy` | Default decision, per-tool rules, and terminal behavior for a denial. |
| `limits` | Optional execution limits, such as `max_steps`. |
| `validators` | Optional output or action contracts evaluated by the harness. |
| `expected` | Event sequence, terminal outcome, and Evidence assertions. |
| `diagnostics` | Required diagnostic codes and redaction/negative assertions. |

### Request

`request.message` is the user-visible task input. `request.metadata` is optional deterministic metadata and must not contain secrets or machine-specific paths.

### Mock Model script

`mock_model.script` is an ordered list. Each entry has a `turn`, a `when` condition, and an `action`:

- `final_text` ends the model interaction normally;
- `tool_call` proposes a named tool and JSON-compatible `arguments`;
- `invalid` deliberately emits a malformed or unsupported model action for negative testing.

The `when` value is either `start`, a prior `tool_result` match, or a prior `validation` match. A later implementation must fail the fixture if an action is emitted out of order, if a condition does not match, or if a script entry marked unreachable is executed.

### Available tools and policy

`available_tools` is the scenario-local allowlist/catalog, not an assertion that a production registry already exists. Each tool has an `input_contract` and a `deterministic_result`. A result has `status` `success` or `error` and may include an output or an error code.

`policy.default` is the decision when no rule matches. A rule can explicitly `allow` or `deny` a tool. `on_denied: terminate` means the denied call must not execute and the run must reach the declared terminal denial outcome without inventing a model response.

### Expected event sequence

`expected.event_sequence` is an ordered list of logical event names with optional assertions. Event names are contract vocabulary, not Rust enum names. The P0 vocabulary is:

```text
run.started
request.accepted
model.action.emitted
model.action.invalid
policy.evaluated
tool.execution.started
tool.execution.succeeded
tool.execution.failed
tool.execution.blocked
validation.failed
session.interrupted
session.resume_requested
session.state.restored
tool.result.reused
step_limit.exceeded
run.completed
run.terminated
diagnostic.emitted
```

The sequence is checked after implementation-specific identifiers and timestamps are normalized. An event assertion may constrain `action`, `tool`, `decision`, `reason`, or another JSON field without prescribing the enclosing implementation type.

### Final outcome and Evidence

`expected.final_outcome` declares the externally observable terminal status and reason. The status vocabulary used by P0 is `completed` or `terminated`; the reason identifies the contract branch, such as `policy_denied` or `validation_failure`.

`expected.evidence.required_records` lists Evidence records that must be observable. Each record has a logical `kind` and optional field assertions. `expected.evidence.invariants` expresses cross-event requirements, such as “a denied tool has no execution start” or “the side effect count is exactly one”. Evidence is an expected output contract, not a claim that an Evidence implementation already exists.

### Diagnostics

`diagnostics.required_codes` lists stable diagnostic categories, not log strings. `diagnostics.must_not_contain` prevents secrets and misleading details from appearing in diagnostic output. `diagnostics.notes` explains what a human should be able to distinguish when reviewing a failure.

## Acceptance rules

A P0 evaluator may mark a scenario contract-compatible only when all of the following hold:

1. The fixture file parses as JSON and contains exactly the nine required IDs.
2. Every scenario contains all required sections listed above, including an explicit policy and diagnostics block.
3. Every scripted tool call names an available tool and uses JSON-compatible arguments.
4. Every possible tool call has an explicit policy result, either through a matching rule or the declared default.
5. The observed normalized event sequence matches the declared order. Extra implementation events are allowed only when the evaluator has an explicit projection from concrete events to the logical vocabulary; missing or reordered logical events are failures.
6. A negative scenario reaches its declared terminal reason and does not also claim a successful completion.
7. `tool_denied_by_policy` proves that policy denial happens before tool execution.
8. `tool_execution_failure` proves that the tool was allowed and started, but returned the declared failure.
9. `invalid_model_action` proves that invalid model output is rejected before tool execution.
10. `validator_failure` proves that the tool execution result was received but did not satisfy the declared validator.
11. `max_steps_exceeded` does not execute its marked-unreachable script turn and emits the step-limit diagnostic.
12. `session_replay` restores the recorded state, reuses the prior tool result, and performs the side effect exactly once.
13. `deterministic_repeated_run` compares two fresh runs after removing only the listed run-specific fields; outcomes, logical events, and Evidence must remain equal.
14. Diagnostics never expose an authorization header, API key, or other secret-like value.

## Evolution rules

Changes to scenario IDs, event meaning, outcome reasons, or Evidence invariants require a contract review and a version change or an explicit compatibility note. Adding optional assertions is compatible when it does not alter the existing required sequence. Concrete Rust APIs may map to this contract later, but this fixture layer must not be rewritten merely to mirror an implementation detail.
