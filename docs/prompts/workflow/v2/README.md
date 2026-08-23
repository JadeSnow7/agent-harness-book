# Workflow v2

v2 coexists with v1. `raw export -> Recorder record -> Controller verify -> state transition`.

The read-only `legacy-inspect` command reports old assets as
`legacy_unverified`; it never emits `passed` or writes a file. `record` is the
only writer and uses same-directory temporary files, fsync, no-overwrite
installation and failure cleanup. It may recover only an exact payload-only
orphan. Unknown/temp files, symlinks, gaps, non-canonical manifests and
unverifiable sources fail closed. Exit codes are 2 input, 3 integrity and 4
security.

Roles: task-analysis, planning, implementation, post-implementation-audit, summary, commit, commit-audit, artifact-recorder, foundation-audit. Recorder never summarizes, redacts, overwrites, edits business files, or records its own output.

The first seven roles retain v1 permissions and five human gates. Long-term assets include task packages, complete structured and natural-language outputs, raw user gates, approved prompts, implementation/audit/summary/commit records, Controller snapshots and Recorder manifests. Raw payloads are stored without wrapping or newline normalization; metadata is separate canonical JSON-as-YAML. Attempts never overwrite; each correction points via `revision_of` to the previous manifest. Source, secret, path, identity, sequence, pair, length, SHA-256 or revision failures stop progression. The 006/008 encrypted platform-record exception is the only non-plaintext representation and is explicitly not plaintext-auditable. v2 does not modify ch3 or Rust/Cargo files.

Legacy migration is deliberately separate: an old run may only be inspected with
`legacy-inspect`, remains `legacy_unverified`, and can never be promoted by that
command or accepted by final `verify`. Rebuild B is blocked until the independent
Foundation Audit (`roles/foundation-audit.md`) passes and a new user approval is
recorded. Foundation Audit never trusts the legacy run's own status claims; it
re-hashes the quarantine inventory, re-diffs the rebuild candidate against the
quarantined version, and re-runs verification commands itself. A passed audit
does not resume the frozen legacy run — recording continues under a brand-new
`task_id` starting at sequence 1. A failed foundation repair cannot be repaired
by appending or trusting the old run assets.
