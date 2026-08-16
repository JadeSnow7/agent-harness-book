# P0 Independent Review

Date: 2026-08-05 (historical review)
Scope: integrated workspace after Wave 3

## Result

No Blocker or Major issue was found in the reviewed deterministic slice. The review is based on the integrated diff and the commands recorded in the execution board; it does not claim remote CI has run.

## Checks

| Area | Result | Evidence |
|---|---|---|
| Dependency direction | Pass | `cargo tree --workspace`; implementation crates depend inward on `agent-core` |
| Loop termination | Pass | `DeterministicRunner` tests for finish, denial, budget, model, tool, and validation failures |
| Policy bypass | Pass | Runner emits `PolicyEvaluated` before `ToolStarted`; denial integration test sees no tool event |
| Evidence provenance | Pass | Runner emits `EvidenceRecorded` from prior event IDs; observability only projects recorded facts |
| Determinism | Pass | core event-ID tests and p0-demo repeated-run test |
| Replay ordering | Pass | `InMemoryEventStore::validate_run` checks identity, contiguous sequence, and deterministic IDs |
| Documentation honesty | Pass | README/book list only the in-memory deterministic scope and current limitations |
| Placeholder scan | Pass with test-only `expect` | no `todo!`, `unimplemented!`, or library `unwrap`; test `expect` calls are diagnostic assertions |

## Findings

### Minor

- `JsonValue::Number` is currently `i64` only.
- The event store is in-memory; it is not a crash-safe durable recovery protocol.
- Historical result: mdBook was unavailable in the 2026-08-05 review environment, so that review did not include a book build. Supplemental verification on 2026-08-16 with mdBook 0.5.4 built the book to a temporary directory and verified the generated home page and representative chapter files. This supplemental result does not rewrite the historical review.

## Supplemental verification

This section records a later local check rather than a change to the historical review result:

- `mdbook build book --dest-dir <temporary-directory>` passed with mdBook 0.5.4.
- The generated `index.html`, `ch0.html`, `ch4.html`, and `ch16.html` files were present.
- The temporary output was outside the repository and was not submitted.

## Follow-up

The accepted repository baseline still describes a narrower early P0/M0 scope. ADR-0001 records this current-task expansion without modifying the baseline; a future lead decision should reconcile the roadmap before the next milestone.
