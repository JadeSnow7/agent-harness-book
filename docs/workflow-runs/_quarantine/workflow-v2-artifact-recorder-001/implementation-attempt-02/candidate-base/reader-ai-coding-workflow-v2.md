# Reader AI coding workflow v2

Breaking append-only successor to v1. v1 remains byte-identical. v2 adds a minimum-write Artifact Recorder: raw export -> record -> verify -> state transition. Payloads are never overwritten; corrections are revisions. Sensitive material fails closed. Controller verification is deterministic, not semantic audit. Events 006/008 are explicitly approved encrypted platform records: plaintext unavailable and unauditable.
