# M3 Agent Loop Test Matrix

Status: implemented and verified — both a Python-first reference and an
independent Rust counterpart satisfy every row below.

This matrix is the explicit correspondence the HUSH execution plan's Phase 1
(HB-M3) asks for: "同步 Rust 对照实现...测试矩阵与 Python 一一对应." Unlike
[`p0-test-matrix.md`](p0-test-matrix.md) (a contract for P0's in-memory
composition of Context/Policy/Tool/Validation/Events), this matrix is
M3's own contract — see `book/src/ch5.md` §5.6 for why M3 must stay an
independent code increment rather than reusing P0's `DeterministicRunner`.

| Behavior | Python test (`examples/python/m3-agent-loop/test_loop.py`) | Rust test (`examples/rust/m3-agent-loop/tests/m3_e2e.rs`) | Terminal outcome |
| --- | --- | --- | --- |
| Three-step script (tool, tool, `Finish`) plus an unreachable fourth step; budget termination happens strictly before the next model action | `test_budget_exhausted_before_next_model_action` | `budget_exhausted_before_next_model_action` | `BudgetExhausted` |
| A tool result from one turn is visible in the model's input on the next turn | `test_tool_result_visible_in_next_model_input` | `tool_result_visible_in_next_model_input` | `Completed` |
| A tool failure produces a structured, observable result instead of an uncaught exception; the loop continues | `test_tool_failure_is_observable_not_uncaught` | `tool_failure_is_observable_not_uncaught` | `Completed` |
| `Finish` only becomes `Completed` after passing Validation — a failing validator does not short-circuit to `Completed` | `test_finish_requires_validation_to_become_completed` | `finish_requires_validation_to_become_completed` | `Failed` (`validation_failed`) |
| Duplicate `call_id` across the run is rejected before the second call executes | `test_duplicate_call_id_is_rejected` | `duplicate_call_id_is_rejected` | `Failed` (`duplicate_call_id`) |
| Unknown tool ID is rejected before dispatch, without executing | `test_unknown_tool_is_rejected_without_executing` | `unknown_tool_is_rejected_without_executing` | `Failed` (`unknown_tool`) |
| Policy denial for a *registered* but disallowed tool — distinct from unknown-tool — with no execution | `test_policy_denied_is_reachable` | `policy_denied_is_reachable` | `PolicyDenied` |
| Cooperative cancellation triggered mid-run (during a tool's `execute`) stops the loop before the next model call | `test_cancel_is_reachable_mid_run` | `cancel_is_reachable_mid_run` | `Cancelled` |
| All five terminal outcomes are mutually exclusive (structural: one `outcome` field, one `return` per terminating branch) and each is actually reachable by the suite | `test_all_five_outcomes_are_covered_by_the_suite` | `all_five_outcomes_are_covered_by_the_suite` | all five |

## Verification commands

```bash
# Python
python3 examples/python/m3-agent-loop/test_loop.py -v

# Rust
cargo test -p m3-agent-loop
cargo clippy -p m3-agent-loop --all-targets
```

Both suites are offline: no network access, no real API keys, no
filesystem access outside normal process memory.

## Reuse boundary (both implementations)

Neither implementation depends on P0's `DeterministicRunner`,
`ContextBuilder`, `EventSink`/`AgentEvent`, or the P0-flavored `Validator`/
`ModelProvider` traits — that composition is explicitly out of bounds per
`book/src/ch5.md` §5.6. Both implementations do reuse small, orchestration-
independent building blocks that predate M3:

- Python reuses `examples/python/m2-tool-runtime`'s `ToolCall`/`ToolResult`/
  `ToolStatus`/`ToolRegistry` (the same tool-execution primitives M2
  already established).
- Rust reuses `agent_core`'s plain data types (`ModelAction`, `ToolCall`,
  `ToolResult`, `ToolResultStatus`) and `tool_runtime::{Tool, ToolRegistry}`
  — the same failure-collapsing tool executor P0 uses, but not P0's runner.

Everything M3-specific — the loop control flow, the five-outcome model, the
scripted mock model, the allow-list policy, and the output validator — is
written fresh in both languages.
