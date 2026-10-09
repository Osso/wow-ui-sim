#!/usr/bin/env python3
"""Relocated SOURCE replay and serialized ledger/log tamper restoration only."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

EVIDENCE = Path(__file__).resolve().parent


def digest(data):
    return hashlib.sha256(data).hexdigest()


class PortableSource(unittest.TestCase):
    def test_copied_no_git_no_target_and_serialized_tamper_restore(self):
        self.assertTrue(
            (EVIDENCE / "seals.json").exists(), "historical seals are missing"
        )
        seals = json.loads((EVIDENCE / "seals.json").read_bytes())
        with tempfile.TemporaryDirectory(prefix="p220-source-") as directory:
            destination = Path(directory) / "history"
            destination.mkdir()
            for name in list(seals) + ["seals.json"]:
                shutil.copyfile(EVIDENCE / name, destination / name)
            self.assertFalse((destination / ".git").exists())
            self.assertFalse((destination / "target").exists())
            self.assertLess(
                max(p.stat().st_size for p in destination.iterdir()), 5_000_000
            )
            argv = [sys.executable, "-B", str(destination / "validate.py")]

            def replay():
                return subprocess.run(
                    argv,
                    cwd=destination,
                    capture_output=True,
                    text=True,
                    env={"PATH": "/no-git-no-cargo-no-network-tools"},
                    timeout=30,
                )

            clean = replay()
            self.assertEqual(clean.returncode, 0, clean.stderr)
            counts = json.loads(clean.stdout)
            self.assertEqual(counts["inventory_occurrences"], 36)
            self.assertEqual(counts["signature_limits"], 33)
            self.assertEqual(counts["registry_pages"], 101)
            self.assertEqual(counts["model_credit"], 0)
            self.assertEqual(counts["sealed_inputs"], len(seals))
            # Mutable later/runtime-shaped state outside the archive cannot affect replay.
            outside = destination.parent / "data/patch-api/sources"
            outside.mkdir(parents=True)
            for patch in ["2.3.0", "2.4.0", "2.5.0", "3.4.0"]:
                (outside / f"{patch}-wikitext-register.json").write_text(
                    '{"entries": [{"symbol": "IsModifiedClick", "direction": "removed"}]}'
                )
            self.assertEqual(replay().stdout, clean.stdout)
            controls = []
            for name in ["historical-page-coverage.json", "source-green.log"]:
                path = destination / name
                original = path.read_bytes()
                original_hash = digest(original)
                with self.subTest(tampered=name):
                    try:
                        if name.endswith(".json"):
                            changed_ledger = json.loads(original)
                            changed_ledger["inventory_rows"].pop()
                            path.write_text(json.dumps(changed_ledger))
                        else:
                            path.write_bytes(original + b"\nFORGED SOURCE PASS\n")
                        changed = replay()
                        self.assertNotEqual(changed.returncode, 0)
                        self.assertIn("sealed input: " + name, changed.stderr)
                        controls.append(
                            {
                                "file": name,
                                "rejected": True,
                                "original_sha256": original_hash,
                            }
                        )
                    finally:
                        path.write_bytes(original)
                    self.assertEqual(digest(path.read_bytes()), original_hash)
                    restored = replay()
                    self.assertEqual(restored.returncode, 0, restored.stderr)
                    self.assertEqual(restored.stdout, clean.stdout)
            print(
                json.dumps(
                    {
                        "counts": counts,
                        "controls": controls,
                        "copied_no_git_no_target": True,
                        "mutable_later_state_ignored": True,
                        "restored_bytes": True,
                        "archive_bytes": sum(
                            p.stat().st_size for p in destination.iterdir()
                        ),
                        "max_file_bytes": max(
                            p.stat().st_size for p in destination.iterdir()
                        ),
                    },
                    sort_keys=True,
                )
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
