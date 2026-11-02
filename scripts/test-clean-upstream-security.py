"""Regression tests for the explicitly inventoried security migrations."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

WORKSPACE = Path(__file__).resolve().parents[1]
CHECKER = WORKSPACE / "scripts/verify-clean-upstream.py"
SOURCE = WORKSPACE / "amante-viewer"


class SecurityInventoryTests(unittest.TestCase):
    def run_checker(self, data, update_expected=False, upstream_hash=None, approve_data=False, override=None):
        original = json.loads((SOURCE / "amante-upstream-manifest.json").read_text())
        hashes = dict(original["files"]["Cargo.lock"])
        if update_expected:
            hashes["amante"] = hashlib.sha256(data).hexdigest()
        if upstream_hash:
            hashes["amante"] = upstream_hash
        security = json.loads((SOURCE / "amante-security-manifest.json").read_text())
        security["files"] = {"Cargo.lock": dict(security["files"]["Cargo.lock"])}
        security["added"] = {}
        if approve_data:
            security["files"]["Cargo.lock"]["sha256"] = hashlib.sha256(data).hexdigest()
        if override:
            security["files"][override] = {"base": "0" * 64, "sha256": "0" * 64}
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "Cargo.lock").write_bytes(data)
            (root / "amante-security-manifest.json").write_text(json.dumps(security))
            (root / "amante-upstream-manifest.json").write_text(
                json.dumps({"upstream": original["upstream"],
                            "files": {"Cargo.lock": hashes}, "added": {}})
            )
            return subprocess.run(
                [sys.executable, str(CHECKER), str(root)],
                capture_output=True, text=True,
            )

    def test_only_approved_updates_pass(self):
        result = self.run_checker((SOURCE / "Cargo.lock").read_bytes())
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_windows_checkout_passes(self):
        data = (SOURCE / "Cargo.lock").read_bytes().replace(b"\r\n", b"\n")
        result = self.run_checker(data.replace(b"\n", b"\r\n"))
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_unapproved_drift_fails_even_with_updated_inventory(self):
        data = (SOURCE / "Cargo.lock").read_bytes() + b"\n# unrelated change\n"
        result = self.run_checker(data, update_expected=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Changed security baseline", result.stderr)

    def test_changing_the_upstream_hash_fails(self):
        data = (SOURCE / "Cargo.lock").read_bytes()
        result = self.run_checker(data, upstream_hash="0" * 64)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Changed security baseline", result.stderr)

    def test_missing_security_patch_fails(self):
        data = (SOURCE / "Cargo.lock").read_bytes().replace(
            b'name = "rustls-webpki"\nversion = "0.103.16"',
            b'name = "rustls-webpki"\nversion = "0.103.15"',
        )
        result = self.run_checker(data, approve_data=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Missing approved security patch", result.stderr)

    def test_unmaintained_crates_cannot_be_reintroduced(self):
        for name in ["ttf-parser", "zvariant", "zbus_names", "zvariant_derive"]:
            with self.subTest(name=name):
                data = (SOURCE / "Cargo.lock").read_bytes() + (
                    f'\n[[package]]\nname = "{name}"\nversion = "0.1.0"\n'
                ).encode()
                result = self.run_checker(data, approve_data=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Unmaintained dependency reintroduced", result.stderr)

    def test_runtime_cannot_be_overridden_by_security_inventory(self):
        result = self.run_checker((SOURCE / "Cargo.lock").read_bytes(), override="crates/aurora-render/src/lib.rs")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unapproved security override", result.stderr)


if __name__ == "__main__":
    unittest.main()
