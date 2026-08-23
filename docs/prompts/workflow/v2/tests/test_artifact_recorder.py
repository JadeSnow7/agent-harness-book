import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
TOOL = ROOT / "docs/prompts/workflow/v2/tools/artifact_recorder.py"


class RecorderTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = Path(self.tmp.name)
        self.repo = self.base / "repo"
        self.repo.mkdir()
        self.source = self.base / "source.md"
        self.source.write_bytes("标题\r\n第二行".encode("utf-8"))

    def tearDown(self):
        self.tmp.cleanup()

    def descriptor(self, sequence=1, attempt=1, role="task-analysis", status="passed", source=None, extra=None):
        source = source or self.source
        data = source.read_bytes()
        value = {
            "source_path": str(source), "expected_byte_length": len(data),
            "expected_sha256": hashlib.sha256(data).hexdigest(), "task_id": "test-task",
            "chapter": "meta", "sequence": sequence, "artifact_key": role,
            "role": role, "attempt": attempt, "status": status,
            "payload_extension": "md", "media_type": "text/markdown",
            "target_path": f"{sequence:03d}-{role}-attempt-{attempt:02d}.payload.md",
            "source_provenance": "platform_raw_export", "source_reference": "fixture",
            "identity_source": "payload_header", "historical": False,
            "recorded_by": "test", "revision_of": None,
        }
        if extra:
            value.update(extra)
        path = self.base / f"d{sequence}.json"
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    def invoke(self, *args):
        return subprocess.run([sys.executable, str(TOOL), *args], text=True, capture_output=True)

    def test_record_preserves_bytes_and_verify_is_read_only(self):
        descriptor = self.descriptor()
        result = self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(descriptor))
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        payload = self.repo / "run/001-task-analysis-attempt-01.payload.md"
        self.assertEqual(payload.read_bytes(), self.source.read_bytes())
        before = {p: p.read_bytes() for p in (self.repo / "run").iterdir()}
        result = self.invoke("verify", "--repo-root", str(self.repo), "--artifact-root", "run", "--task-id", "test-task", "--chapter", "meta")
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertEqual(before, {p: p.read_bytes() for p in (self.repo / "run").iterdir()})

    def test_sequence_and_revision_are_enforced(self):
        first = self.descriptor()
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(first)).returncode, 0)
        bad = self.descriptor(sequence=3, role="planning")
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(bad)).returncode, 3)

    def test_canonical_manifest_tamper_and_symlink_fail_closed(self):
        d = self.descriptor()
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(d)).returncode, 0)
        manifest = next((self.repo / "run").glob("*.manifest.yaml"))
        obj = json.loads(manifest.read_text())
        manifest.write_text(json.dumps(obj), encoding="utf-8")
        self.assertEqual(self.invoke("verify", "--repo-root", str(self.repo), "--artifact-root", "run", "--task-id", "test-task", "--chapter", "meta").returncode, 3)

    def test_invalid_utf8_and_secret_are_rejected(self):
        bad = self.base / "bad.bin"; bad.write_bytes(b"\xff")
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(self.descriptor(source=bad))).returncode, 2)
        secret = self.base / "secret.txt"; secret.write_text("Authorization: Bearer abcdefghijklmnopqrstuvwxyz", encoding="utf-8")
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run2", "--descriptor", str(self.descriptor(source=secret))).returncode, 4)

    def test_unknown_and_orphan_fail(self):
        root = self.repo / "run"; root.mkdir(); (root / "noise.txt").write_text("x")
        self.assertEqual(self.invoke("verify", "--repo-root", str(self.repo), "--artifact-root", "run", "--task-id", "test-task", "--chapter", "meta").returncode, 3)

    def test_path_traversal_and_symlink_are_security_failures(self):
        descriptor = self.descriptor(extra={"target_path": "../escape.payload.md"})
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(descriptor)).returncode, 3)
        link = self.base / "link.md"
        link.symlink_to(self.source)
        linked = self.descriptor(source=link)
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run2", "--descriptor", str(linked)).returncode, 4)

    def test_encrypted_exception_requires_full_single_json_record(self):
        for sequence in range(1, 6):
            prior = self.descriptor(sequence=sequence, role="controller-state", status="planning",
                                    extra={"artifact_key": f"state-{sequence}",
                                           "target_path": f"{sequence:03d}-controller-state-attempt-01.payload.md"})
            self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(prior)).returncode, 0)
        encrypted = self.base / "encrypted.jsonl"
        encrypted.write_bytes(b'{"line_id":"fixture-006","sequence":6,"role":"controller-correction","status":"returned","platform_metadata":{"provider":"fixture"},"encrypted_content":"ciphertext-value"}')
        descriptor = self.descriptor(sequence=6, role="controller-correction", status="returned", source=encrypted,
                                     extra={"artifact_key": "controller-correction", "target_path": "006-controller-correction-attempt-01.payload.json",
                                            "payload_extension": "json", "representation": "encrypted_platform_record",
                                            "plaintext_available": False, "plaintext_verified": False,
                                            "content_auditability": "unavailable", "identity_source": "platform_metadata",
                                            "source_provenance": "platform_jsonl_encrypted_content", "exception_reason": "plaintext unavailable",
                                            "exception_approval_ref": "019-user-gate-implementation-attempt-02.manifest.yaml",
                                            "claim_source": "platform metadata", "claimed_role": "controller-correction", "claimed_status": "returned"})
        result = self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "run", "--descriptor", str(descriptor))
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('"artifact_count": 6', self.invoke("verify", "--repo-root", str(self.repo), "--artifact-root", "run", "--task-id", "test-task", "--chapter", "meta").stdout)
        encrypted.write_bytes(b'{"encrypted_content":"one"}\n{"encrypted_content":"two"}')
        bad = self.descriptor(sequence=6, role="controller-correction", status="returned", source=encrypted,
                              extra={"artifact_key": "controller-correction", "target_path": "006-controller-correction-attempt-01.payload.json",
                                     "payload_extension": "json", "representation": "encrypted_platform_record",
                                     "plaintext_available": False, "plaintext_verified": False, "content_auditability": "unavailable",
                                     "identity_source": "platform_metadata", "source_provenance": "platform_jsonl_encrypted_content",
                                     "exception_reason": "x", "exception_approval_ref": "ref", "claim_source": "x",
                                     "claimed_role": "controller-correction", "claimed_status": "returned"})
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "other", "--descriptor", str(bad)).returncode, 3)

    def test_legacy_inspect_is_read_only_and_never_passed(self):
        root = self.repo / "legacy"
        root.mkdir()
        (root / "old.payload").write_bytes("旧协议".encode())
        before = sorted(p.relative_to(self.repo) for p in self.repo.rglob("*"))
        result = self.invoke("legacy-inspect", "--artifact-root", str(root))
        self.assertEqual(result.returncode, 0)
        self.assertIn("legacy_unverified", result.stdout)
        self.assertNotIn("passed", result.stdout)
        self.assertEqual(before, sorted(p.relative_to(self.repo) for p in self.repo.rglob("*")))

    def test_bearer_placeholder_is_allowed_but_real_value_is_rejected(self):
        placeholder = self.base / "placeholder.txt"
        placeholder.write_text("Authorization: Bearer <TOKEN>", encoding="utf-8")
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "placeholder", "--descriptor", str(self.descriptor(source=placeholder))).returncode, 0)
        secret = self.base / "real.txt"
        secret.write_text("Authorization: Bearer abcdefghijklmnopqrstuvwxyz", encoding="utf-8")
        self.assertEqual(self.invoke("record", "--repo-root", str(self.repo), "--artifact-root", "secret", "--descriptor", str(self.descriptor(source=secret))).returncode, 4)

    def test_inspect_does_not_print_source(self):
        result = self.invoke("inspect", "--source", str(self.source))
        self.assertEqual(result.returncode, 0)
        self.assertNotIn("标题", result.stdout)
        self.assertEqual(json.loads(result.stdout)["byte_length"], len(self.source.read_bytes()))

    def test_install_failpoints_leave_no_payload_manifest_or_temp(self):
        for failpoint in ("payload:file_fsync", "payload:link", "payload:dir_fsync",
                          "manifest:file_fsync", "manifest:link", "manifest:dir_fsync"):
            root = self.repo / failpoint.replace(":", "-")
            env = dict(os.environ, ARTIFACT_RECORDER_FAILPOINT=failpoint,
                       PYTHONPYCACHEPREFIX="/tmp")
            result = subprocess.run(
                [sys.executable, str(TOOL), "record", "--repo-root", str(self.repo),
                 "--artifact-root", str(root.relative_to(self.repo)),
                 "--descriptor", str(self.descriptor())],
                text=True, capture_output=True, env=env)
            self.assertNotEqual(result.returncode, 0, failpoint)
            if root.exists():
                self.assertEqual([], list(root.iterdir()), failpoint)


if __name__ == "__main__":
    unittest.main()
