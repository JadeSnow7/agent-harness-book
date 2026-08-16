# ADR-0001: P0 End-to-End Deterministic Harness Scope

Status: Accepted for the current P0 execution (scope exception)
Date: 2026-08-05
Scope: P0-B architecture and contract freeze

## Context

The accepted [`repository-baseline`](../decisions/repository-baseline.md) establishes the repository mission, dependency direction, API stability policy, and M0–M10 roadmap. In that roadmap, model calls are M0, while Tool Runtime, Agent Loop, Sessions & Recovery, Policy & Sandbox, and Validation & Evidence are later milestones.

The current P0 orchestration task expands the P0 slice and asks for a contract covering a deterministic end-to-end harness: request and session identity, an agent loop, model actions, policy decisions, tool lifecycle, validation, evidence, outcomes, errors, and replay invariants. The execution board already records this as a known conflict and says the current task is authoritative for this execution.

## Decision

For P0-B, define and freeze the smallest end-to-end deterministic harness contract in:

- [`p0-component-boundaries.md`](../architecture/p0-component-boundaries.md)
- [`p0-deterministic-harness-contract.md`](../specs/p0-deterministic-harness-contract.md)

The expanded P0 contract is limited to an in-process deterministic profile. It covers boundary contracts for the runner, model provider, context builder, policy evaluator, tool executor, validator, event sink/log, and read-only observability. It does not require live integrations, durable production recovery, external side effects, streaming, parallel tools, human approval, or Forge Studio types.

This ADR records the accepted scope exception for the current orchestration and implementation handoff. It does **not** amend, supersede, or silently edit the accepted repository baseline. The baseline remains the repository-level source of truth until a later lead-approved ADR explicitly reconciles the milestone roadmap with the expanded P0 scope.

## Consequences

Positive consequences:

- P0 implementation agents can work against one shared vocabulary and one dependency direction.
- The deterministic profile makes event replay and fixture-based evaluation possible without real model credentials or network access.
- Policy, execution, validation, session logging, and observability remain replaceable implementations behind core-owned boundaries.
- The contract makes terminal states and tool ordering explicit before code is written.

Costs and constraints:

- The P0 implementation spans concerns that the baseline roadmap originally assigned to later milestones.
- The contract is experimental under the baseline's M0–M4 API policy; later implementation evidence may justify a new ADR or a contract revision.
- Session multiplicity, event fingerprints, context overflow semantics, and durable recovery remain unresolved and cannot be inferred from this ADR.

## Alternatives considered

1. Keep P0 limited to M0 model calls. This would preserve the roadmap literally but would not satisfy the current orchestration task or provide the requested end-to-end handoff.
2. Edit the accepted baseline in place. Rejected because the repository rules prohibit silently changing accepted decisions and the task explicitly says not to edit the baseline ADR.
3. Define a broad production-style platform contract. Rejected because it would add premature persistence, external side effects, distributed execution, and provider-specific abstractions beyond the deterministic P0 goal.

## Follow-up

After the implementation scope and integration results stabilize, the Lead should decide whether a repository-level ADR should reconcile the roadmap and this P0 scope assumption. Until then, later agents should treat the contract specification as the implementation boundary for this orchestration only.
