# P0 Execution Board

Status: completed
Started: 2026-08-05
Scope authority: the current P0 orchestration task, subject to `AGENTS.md` and accepted decisions.

## Wave 0 audit

- Repository state: initial commit `ec2f258`, branch `main`, clean working tree at audit time.
- Existing implementation: no Rust workspace, crates, examples, book, evals, fixtures, or CI files yet.
- Existing guidance: `AGENTS.md`, `README.md`, and `docs/decisions/repository-baseline.md`.
- Known conflict: the baseline decision describes P0-1 as governance/M0 only and excludes later runtime work; the current task explicitly expands P0 to the end-to-end harness slice. The current task is authoritative for this execution. The baseline decision remains unchanged until a follow-up ADR records the expanded scope.
- Initial blocker: none known before toolchain validation.

## Work packages

| Task ID | Work Package | Owner | Depends On | Writable Paths | Status | Evidence |
|---|---|---|---|---|---|---|
| W0-01 | Repository and toolchain audit | Lead | — | read-only | done | Initial repository snapshot recorded above |
| W0-02 | P0 execution board | Lead | W0-01 | `docs/status/p0-execution-board.md` | done | This file |
| W0-03 | Path and interface ownership freeze | Lead | W0-01 | board only | done | Ownership table below |
| P0-A | Workspace foundation | Foundation agent | W0-03 | Root Cargo/config, `.cargo/`, `.github/`, crate skeletons | done | `cargo fmt`, `check`, `test`, and `clippy` passed on Rust 1.93.0; MSRV not installed locally |
| P0-B | Architecture and contracts | Architecture agent | W0-03 | `docs/architecture/`, `docs/specs/`, `docs/adr/` | done | Contract proposal and ADR-0001 reviewed; accepted for this execution's implementation freeze |
| P0-C | Evaluation contracts and fixtures | Evaluation agent | W0-03 | `evals/`, `fixtures/`, `docs/testing/` | done | JSON/schema/semantic fixture checks passed; 9 scenarios |
| P0-D | Core loop | Core agent / Lead integration | Wave 1 contract freeze | `crates/agent-core/` | done | 12 core tests passed; deterministic runner and evidence lifecycle integrated |
| P0-E | Model adapters and context | Model agent / Lead integration | Wave 1 contract freeze | `crates/model-adapters/`, `crates/context-engine/` | done | 5 + 5 tests passed; deterministic script and bounded context |
| P0-F | Runtime, policy, validation | Runtime agent / Lead integration | Wave 1 contract freeze | `crates/tool-runtime/`, `crates/policy-engine/`, `crates/validators/` | done | 4 + 3 + 3 tests passed; echo/schema/policy paths |
| P0-G | Session and observability | Lead integration | Wave 1 contract freeze | `crates/session-store/`, `crates/observability/` | done | 2 + 1 tests passed; in-memory isolation/replay and projections |
| P0-H | Examples and integration evals | Lead | Wave 2 integration | `examples/`, integration tests, eval fixtures | done | `p0-demo` run output verified; 3 E2E tests passed; fixture contracts present |
| P0-I | Book and developer docs | Lead | Wave 2 integration | `book/`, `README.md` | done | README and mdBook skeleton match actual APIs; temporary-output mdBook build passed on 2026-08-16 |
| P0-J | Independent review | Lead | Wave 3 implementation | review report only | done | `docs/review/p0-independent-review.md`; no Blocker/Major found |
| Lead | Integration, blocker fixes, final gate | Lead | All waves | Shared files only when required | done | Rust workspace, E2E example, fixture checks, dependency review, clippy, and temporary-output mdBook build passed; MSRV toolchain was not installed locally |

## File ownership and integration order

| Shared or scoped area | Owner | Integration rule |
|---|---|---|
| Root `Cargo.toml`, toolchain, CI | P0-A, then Lead | No other agent edits during Wave 1 |
| `crates/agent-core/` | P0-D | Core contract changes require Lead decision |
| `crates/model-adapters/`, `crates/context-engine/` | P0-E | Consume frozen core types |
| `crates/tool-runtime/`, `crates/policy-engine/`, `crates/validators/` | P0-F | Policy precedes execution; consume frozen core types |
| `crates/session-store/`, `crates/observability/` | P0-G | Evidence references recorded events |
| `docs/architecture/`, `docs/specs/`, `docs/adr/` | P0-B, then Lead | Documentation may describe only frozen contracts |
| `evals/`, `fixtures/`, `docs/testing/` | P0-C, then P0-H | P0-H may extend fixtures only after API mapping |
| `examples/`, integration tests | P0-H | Must run against actual integrated APIs |
| `book/`, `README.md` | P0-I | Must be reconciled with final executable commands |

## Gate criteria

Wave 1 gate: passed on 2026-08-05. The P0-B contract is frozen for this execution with the following implementation choices: core owns the shared value types and runner; the implementation uses only the Rust standard library; IDs are deterministic string newtypes; the canonical demo tool is `echo`; policy denial is terminal; tool failures are structured results; validation is required before completion; the in-memory event log is the P0 persistence boundary. Any deviation requires Lead review and a board/ADR update. Every later gate records commands and results here. A claim of `done` requires a diff, a validation command, and its result.

## Final gate evidence

- `git diff --check`: passed.
- `python3 -m json.tool fixtures/p0-scenarios.json`: passed.
- `cargo fmt --all --check`: passed.
- `cargo check --workspace`: passed.
- `cargo test --workspace`: passed, including 12 core tests and 3 p0-demo integration tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo tree --workspace --edges normal`: passed; all implementation crates depend inward on `agent-core`, with no third-party dependencies.
- `cargo run -p p0-demo`: passed; output included `Completed`, `event_count=11`, and `evidence_count=1`.
- `mdbook build book --dest-dir <temporary-directory>`: passed on 2026-08-16 with mdBook 0.5.4; generated `index.html`, `ch0.html`, `ch4.html`, and `ch16.html` were verified outside the repository.

## Open issues / backlog

- Resolve the P0 scope conflict with a follow-up ADR after the implementation scope is stable.
- Keep real provider integrations, network tools, GUI, Forge Studio/Godot domain types, persistence services, and distributed execution outside this P0 slice.
