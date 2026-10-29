"""Exercise slot pruning using synthetic Cargo trees only."""
import contextlib
import fcntl
import importlib.util
import io
import json
import multiprocessing
import os
from pathlib import Path
import tempfile
import time
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/slot-build.py"
MIB = 1024 * 1024


def hold_build_lock(path, ready, release):
    with path.open("a") as stream:
        fcntl.flock(stream, fcntl.LOCK_EX)
        ready.set()
        release.wait(10)


class SlotPruneTests(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("slot_build", SCRIPT)
        self.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.module)
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.slot = Path(directory.name)
        (self.slot / ".owner").write_text("owner")
        self.now = time.time()

    def artifact(self, name, age=4, marker=None):
        paths = [self.slot / "debug" / suffix for suffix in (
            f".fingerprint/{name}-abc123/lib-{marker or name}",
            f"deps/{name}-abc123.d", f"deps/lib{name}-abc123.rlib",
            f"build/{name}-abc123/output", f"incremental/{name}-abc123/session/data",
        )]
        for path in paths:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"x" * MIB)
        stamp = self.now - age * 3600
        for path in paths:
            while path.name not in {".fingerprint", "deps", "build", "incremental"}:
                os.utime(path, (stamp, stamp))
                path = path.parent
        return paths

    def age_tree(self, age):
        stamp = self.now - age * 3600
        for path in self.slot.rglob("*"):
            os.utime(path, (stamp, stamp))

    def prune(self, cap=8 * MIB, **kwargs):
        self.assertTrue(callable(getattr(self.module, "prune_slot", None)),
                        "whole-unit slot pruning is not implemented")
        with contextlib.redirect_stdout(io.StringIO()) as output:
            result = self.module.prune_slot(
                self.slot, "owner", cap, now=self.now, **kwargs)
        return result, output.getvalue()

    def test_under_cap_keeps_every_member(self):
        paths = self.artifact("alpha")
        result, _ = self.prune()
        self.assertEqual(result["removed"], [])
        self.assertTrue(all(path.exists() for path in paths))

    def test_over_cap_removes_oldest_whole_units_to_watermark(self):
        oldest = self.artifact("zeta", age=8)
        middle = self.artifact("beta", age=6)
        newest = self.artifact("alpha", age=4)
        result, output = self.prune(cap=13 * MIB)
        self.assertFalse(any(path.exists() for path in oldest))
        self.assertTrue(all(path.exists() for path in middle + newest))
        self.assertLess(result["remaining_bytes"], 13 * MIB * .8)
        self.assertEqual(len(result["removed"]), 1)
        self.assertIn("zeta", output)
        for entry in result["removed"]:
            self.assertGreater(entry["size_bytes"], 0)
            self.assertTrue(entry["paths"])

    def test_recent_member_retains_entire_unit(self):
        paths = self.artifact("alpha")
        os.utime(paths[-1], (self.now, self.now))
        result, _ = self.prune(cap=MIB)
        self.assertEqual(result["removed"], [])
        self.assertTrue(all(path.exists() for path in paths))
        self.assertGreater(result["remaining_bytes"], MIB)

    def test_target_alias_with_matching_hash_is_removed_with_fingerprint(self):
        paths = self.artifact("some-package", marker="some_binary")
        alias = self.slot / "debug/deps/libsome_binary-abc123.rlib"
        alias.write_bytes(b"x" * MIB)
        self.age_tree(4)
        result, _ = self.prune(cap=MIB)
        self.assertEqual(len(result["removed"]), 1)
        self.assertFalse(alias.exists())
        self.assertFalse(any(path.exists() for path in paths))

    def test_recent_independent_incremental_hash_does_not_pin_old_artifacts(self):
        paths = self.artifact("alpha")
        incremental = self.slot / "debug/incremental/alpha-unrelated/session"
        incremental.mkdir(parents=True)
        (incremental / "data").write_bytes(b"x" * MIB)
        result, _ = self.prune(cap=2 * MIB)
        self.assertEqual(len(result["removed"]), 1)
        self.assertFalse(any(path.exists() for path in paths))
        self.assertTrue(incremental.exists())

    def test_newer_artifact_hash_is_not_coalesced_with_older_hash(self):
        paths = self.artifact("alpha")
        newer = self.slot / "debug/deps/libalpha-def456.rlib"
        newer.write_bytes(b"x" * MIB)
        result, _ = self.prune(cap=2 * MIB)
        self.assertEqual(len(result["removed"]), 1)
        self.assertFalse(any(path.exists() for path in paths))
        self.assertTrue(newer.exists())

    def test_crate_name_starting_with_lib_keeps_dep_info_in_unit(self):
        paths = self.artifact("libloading")
        os.utime(paths[1], (self.now, self.now))
        result, _ = self.prune(cap=MIB)
        self.assertEqual(result["removed"], [])
        self.assertTrue(all(path.exists() for path in paths))

    def test_previous_successful_receipt_protects_whole_unit(self):
        paths = self.artifact("alpha")
        receipt = {"result": "success", "reported_artifacts": [
            {"filenames": [str(paths[2])]}], "artifacts": {}}
        (self.slot / ".verified-source-identity.json").write_text(json.dumps(receipt))
        result, _ = self.prune(cap=MIB)
        self.assertEqual(result["removed"], [])
        self.assertTrue(all(path.exists() for path in paths))

    def build_script_unit(self, name, age):
        fingerprint = self.slot / f"debug/.fingerprint/{name}-run456/run-build-script-build-script-build"
        out = self.slot / f"debug/build/{name}-run456/out"
        for path in (fingerprint, out / "generated.rs"):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"x" * MIB)
        stamp = self.now - age * 3600
        for path in (fingerprint, fingerprint.parent, out / "generated.rs", out, out.parent):
            os.utime(path, (stamp, stamp))
        return out

    def write_receipt(self, name, age_days, out_dirs=(), filenames=()):
        receipt = self.slot / f"receipts/{name}.json"
        receipt.parent.mkdir(exist_ok=True)
        receipt.write_text(json.dumps({"build_script_out_dirs": [str(p) for p in out_dirs],
                                       "reported_artifacts": [{"filenames": [str(p) for p in filenames]}],
                                       "finished_ns": int((self.now - age_days * 86400) * 1e9)}))
        # A rewritten receipt has a fresh mtime; only the recorded finish time counts.
        os.utime(receipt, (self.now, self.now))

    def test_recent_receipt_protects_untouched_build_script_and_dependency(self):
        out = self.build_script_unit("quote", age=30)
        library = self.artifact("syn", age=30)
        stale = self.artifact("old", age=10)
        self.write_receipt("noop", 1, out_dirs=[out], filenames=[library[2]])
        result, _ = self.prune(cap=7 * MIB)
        self.assertTrue(out.exists())
        self.assertTrue(all(path.exists() for path in library))
        self.assertFalse(any(path.exists() for path in stale))
        self.assertEqual(len(result["removed"]), 1)

    def test_cumulative_verified_artifact_map_does_not_protect(self):
        paths = self.artifact("old")
        verified = {"result": "success", "reported_artifacts": [],
                    "artifacts": {str(paths[2].relative_to(self.slot)): "digest"}}
        (self.slot / ".verified-source-identity.json").write_text(json.dumps(verified))
        result, _ = self.prune(cap=MIB)
        self.assertFalse(any(path.exists() for path in paths))

    def test_receipt_older_than_live_window_does_not_protect(self):
        library = self.artifact("syn", age=30)
        self.write_receipt("old", self.module.LIVE_RECEIPT_DAYS + 1, filenames=[library[2]])
        result, _ = self.prune(cap=MIB)
        self.assertFalse(any(path.exists() for path in library))

    def test_stale_bulky_units_go_before_older_build_script_units(self):
        out = self.build_script_unit("quote", age=50)
        stale = self.artifact("old", age=10)
        result, _ = self.prune(cap=6 * MIB)
        self.assertFalse(any(path.exists() for path in stale))
        self.assertTrue(out.exists())
        self.assertEqual(len(result["removed"]), 1)

    def test_build_lock_refuses_pruning_until_other_process_releases(self):
        for profile in ("debug", "release", "aarch64-unknown-linux-gnu/custom"):
            with self.subTest(profile=profile):
                paths = self.artifact("alpha")
                profile_dir = self.slot / profile
                if profile != "debug":
                    profile_dir.parent.mkdir(parents=True, exist_ok=True)
                    (self.slot / "debug").rename(profile_dir)
                    paths = [profile_dir / path.relative_to(self.slot / "debug")
                             for path in paths]
                ready, release = multiprocessing.Event(), multiprocessing.Event()
                process = multiprocessing.Process(
                    target=hold_build_lock,
                    args=(profile_dir / ".cargo-lock", ready, release))
                process.start()
                try:
                    self.assertTrue(ready.wait(5), "build lock holder did not start")
                    with self.assertRaisesRegex(ValueError, "slot build lock busy.*cargo-lock"):
                        self.prune(cap=MIB)
                    self.assertTrue(all(path.exists() for path in paths))
                finally:
                    release.set()
                    process.join(5)
                    if process.is_alive():
                        process.terminate()
                        process.join(5)
                self.assertEqual(process.exitcode, 0)
                self.prune(cap=MIB)
                self.assertFalse(any(path.exists() for path in paths))

    def test_recent_library_sidecar_protects_every_same_hash_member(self):
        paths = self.artifact("alpha")
        library = self.slot / "debug/deps/libalpha-abc123.so"
        sidecar = library.with_suffix(".so.dwp")
        library.write_bytes(b"x" * MIB)
        sidecar.write_bytes(b"x" * MIB)
        self.age_tree(4)
        os.utime(sidecar, (self.now, self.now))
        result, _ = self.prune(cap=MIB)
        self.assertEqual(result["removed"], [])
        self.assertTrue(all(path.exists() for path in paths + [library, sidecar]))

    def test_old_library_sidecar_is_removed_with_same_hash_unit(self):
        paths = self.artifact("alpha")
        sidecar = self.slot / "debug/deps/libalpha-abc123.so.dwp"
        sidecar.write_bytes(b"x" * MIB)
        self.age_tree(4)
        result, _ = self.prune(cap=MIB)
        self.assertEqual(len(result["removed"]), 1)
        self.assertFalse(any(path.exists() for path in paths + [sidecar]))

    def test_receipt_external_hardlink_protects_whole_unit(self):
        paths = self.artifact("alpha")
        with tempfile.TemporaryDirectory() as external:
            alias = Path(external) / "verified.rlib"
            os.link(paths[2], alias)
            receipt = {"artifacts": {}, "reported_artifacts": [
                {"filenames": [str(alias)]}]}
            (self.slot / ".verified-source-identity.json").write_text(json.dumps(receipt))
            result, _ = self.prune(cap=MIB)
            self.assertEqual(result["removed"], [])
            self.assertTrue(all(path.exists() for path in paths))

    def test_mismatched_owner_never_deletes(self):
        paths = self.artifact("alpha")
        (self.slot / ".owner").write_text("someone else")
        with self.assertRaisesRegex(ValueError, "owner mismatch"):
            self.prune(cap=MIB)
        self.assertTrue(all(path.exists() for path in paths))

    def test_dry_run_needs_no_claim_and_changes_nothing(self):
        paths = self.artifact("alpha")
        (self.slot / ".owner").unlink()
        result, output = self.prune(cap=MIB, dry_run=True)
        self.assertEqual(len(result["removed"]), 1)
        self.assertIn('"dry_run": true', output)
        self.assertTrue(all(path.exists() for path in paths))

    def test_external_symlink_is_not_followed(self):
        paths = self.artifact("alpha")
        external = self.slot.parent / (self.slot.name + "-external")
        external.write_bytes(b"outside")
        self.addCleanup(external.unlink)
        (paths[-1].parent / "link").symlink_to(external)
        self.age_tree(4)
        self.prune(cap=MIB)
        self.assertEqual(external.read_bytes(), b"outside")


if __name__ == "__main__":
    unittest.main()
