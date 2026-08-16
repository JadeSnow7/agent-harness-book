# P0 Evaluation Test Matrix

Status: contract-only, pre-implementation

This matrix maps each fixture in [`fixtures/p0-scenarios.json`](../fixtures/p0-scenarios.json) to the behavior it must exercise. “Expected” is an acceptance target for a future evaluator; it is not a report that the behavior is implemented today.

| Scenario | Model branch | Tool/policy branch | Validation/session branch | Terminal outcome | Required Evidence focus | Diagnostic focus |
|---|---|---|---|---|---|---|
| `plain_response` | One deterministic `final_text` action | No tool call | No validator or replay | `completed / plain_response` | request and final model action | no diagnostic; no secret leakage |
| `single_tool_success` | Tool call, then deterministic final response | Tool allowed and succeeds | Tool result is returned to the model | `completed / tool_success` | policy allow, tool result, final response | no error diagnostic |
| `tool_denied_by_policy` | Tool call | Tool is available but explicitly denied; no execution | Denial terminates the run | `terminated / policy_denied` | policy decision and blocked call; zero executions | `policy_denied` |
| `tool_execution_failure` | Tool call | Tool is allowed, starts, and returns a deterministic error | Failure terminates the run | `terminated / tool_execution_failure` | allow decision, start, declared error | `tool_execution_failed` |
| `invalid_model_action` | Malformed tool action | Allow rule exists, but action shape is invalid before dispatch | No tool execution | `terminated / invalid_model_action` | invalid action and rejection point | `invalid_model_action` |
| `validator_failure` | Tool call | Tool is allowed and returns a deterministic payload | Result fails the declared validator | `terminated / validation_failure` | raw result and validator failure | `validation_failed` |
| `max_steps_exceeded` | Three reachable tool calls; fourth is unreachable | All reachable calls allowed and succeed | Step budget ends the run before turn four | `terminated / max_steps_exceeded` | three completed steps and budget boundary | `max_steps_exceeded` |
| `session_replay` | Initial tool call; resumed script emits final response | Side-effecting tool is allowed once | Restore after tool success and reuse result on replay | `completed / replay_consistent` | checkpoint, reused result, side-effect count 1 | `session_interrupted` and `session_resumed` |
| `deterministic_repeated_run` | Same tool call and final response on two fresh runs | Same deterministic allow rule and result | Compare normalized runs | `completed / deterministic` | equality of normalized events, outcome, Evidence | no nondeterminism diagnostic |

## Coverage assertions

The matrix is complete for the P0 contract when the fixture set covers all of these dimensions:

- a no-tool success path;
- a successful tool round trip;
- policy denial before execution;
- execution failure after dispatch;
- invalid model output before dispatch;
- post-execution validation failure;
- a hard step boundary;
- interruption and replay without duplicate side effects;
- repeatability across fresh deterministic runs.

The later integration evaluator should report at least one pass/fail result per matrix row and preserve the scenario ID in its output. It should also report whether the failure came from fixture parsing, event ordering, outcome mismatch, Evidence mismatch, or diagnostics mismatch.

## Suggested execution order after implementation

Run the scenarios in dependency order: `plain_response`, `single_tool_success`, `tool_denied_by_policy`, `tool_execution_failure`, `invalid_model_action`, `validator_failure`, `max_steps_exceeded`, `session_replay`, and finally `deterministic_repeated_run`. This order moves from a model-only path through tool and failure boundaries before exercising persistence and cross-run comparison.
