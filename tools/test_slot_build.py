"""Test the stable-path launcher without launching systemd or Cargo."""
import contextlib
import importlib.util
import io
import hashlib
import json
import os
import time
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/slot-build.py"



def full_identity(slot):
    """Receipts are summaries; the latest attempt's full identity stays beside the slot."""
    return json.loads((slot / ".source-identity.json").read_text())

class SlotBuildTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(SCRIPT.is_file(), "stable-path helper is not implemented")
        spec = importlib.util.spec_from_file_location("slot_build", SCRIPT)
        self.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.module)
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.slot = Path(directory.name)

    def test_change_matching_preserves_empty_direct_and_directory_inputs(self):
        tree = self.slot / "worktree"
        (tree / "src/nested").mkdir(parents=True)
        (tree / "src/direct.rs").write_text("direct input")
        (tree / "src/nested/child.rs").write_text("directory input")
        changes = {"src/direct.rs", "src/nested/child.rs", "unrelated.rs"}
        empty = str(self.slot / "debug/external")
        direct = str(self.slot / "debug/direct")
        directory = str(self.slot / "debug/directory")
        current = {
            "worktree": str(tree), "slot": str(self.slot),
            "changed_paths": sorted(changes),
            "content_changed_paths": sorted(changes),
            "target_inputs": {
                direct: ["src/direct.rs"], directory: ["src/nested"],
            },
            "reported_artifacts": [
                {"executable": path, "fresh": True}
                for path in [empty, direct, directory]
            ],
        }
        self.assertEqual(self.module.matched_input_changes(current, [], changes), set())
        self.assertEqual(self.module.matched_input_changes(
            current, ["src/direct.rs", "src/nested"], changes),
            {"src/direct.rs", "src/nested/child.rs"})
        self.assertEqual(self.module.find_stale_artifacts(current, {}, {}),
                         ["debug/direct", "debug/directory"])
        for record in current["reported_artifacts"]:
            record["fresh"] = False
        previous = {"debug/direct": "before", "debug/directory": "before"}
        self.assertEqual(self.module.find_stale_artifacts(current, previous, previous),
                         ["debug/direct", "debug/directory"])
        rebuilt = {"debug/direct": "after", "debug/directory": "after"}
        self.assertEqual(self.module.find_stale_artifacts(current, previous, rebuilt), [])

    def test_launch_argv_keeps_source_and_target_paths_stable(self):
        self.assertEqual(
            self.module.build_argv(3, Path("/home/osso/.worktrees/lane"),
                                   "lane-build", ["cargo", "test", "--no-run"]),
            ["/usr/bin/systemd-run", "--user", "--no-block", "--remain-after-exit",
             "--unit=lane-build", "--slice=agents.slice",
             "--property=MemoryMax=9G", "--property=MemorySwapMax=0",
             "--property=CPUAccounting=yes",
             "--setenv=CARGO_INCREMENTAL=1", "--setenv=RUSTC_WRAPPER=",
             "--setenv=CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/slot-3",
             "--property=BindPaths=/home/osso/.worktrees/lane:/home/osso/.cache/wow-ui-sim-srcmount/slot-3",
             "--working-directory=/home/osso/.cache/wow-ui-sim-srcmount/slot-3",
             "--", "cargo", "test", "--no-run"],
        )

    def test_retired_slot_two_is_rejected_before_launch(self):
        with contextlib.redirect_stderr(io.StringIO()) as stderr, \
                self.assertRaises(SystemExit) as exit_info:
            self.module.main(["--slot", "2", "--worktree", str(self.slot),
                              "--unit", "retired-slot", "--", "cargo", "build"])
        self.assertEqual(exit_info.exception.code, 2)
        self.assertRegex(stderr.getvalue(), r"--slot: invalid choice: '?2'?")

    def test_command_not_starting_with_cargo_is_rejected_before_launch(self):
        with contextlib.redirect_stderr(io.StringIO()) as stderr, \
                self.assertRaises(SystemExit) as exit_info:
            self.module.main(["--slot", "0", "--worktree", str(self.slot),
                              "--unit", "bare-test", "--owner", "lane-owner",
                              "--capacity-id", "cap", "--", "test", "--test", "x"])
        self.assertEqual(exit_info.exception.code, 2)
        self.assertIn("must start with cargo", stderr.getvalue())

    def test_cargo_without_json_messages_is_rejected_before_launch(self):
        for cargo in [["cargo", "check"],
                      ["cargo", "test", "--test", "x", "--", "--message-format=json"]]:
            with contextlib.redirect_stderr(io.StringIO()) as stderr, \
                    self.assertRaises(SystemExit) as exit_info:
                self.module.main(["--slot", "0", "--worktree", str(self.slot),
                                  "--unit", "plain-test", "--owner", "lane-owner",
                                  "--capacity-id", "cap", "--", *cargo])
            self.assertEqual(exit_info.exception.code, 2)
            self.assertIn("--message-format=json", stderr.getvalue())

    def test_launch_disables_configured_rustc_wrapper(self):
        argv = self.module.build_argv(0, self.slot, "wrapper-test", ["cargo", "build"])
        self.assertIn("--setenv=RUSTC_WRAPPER=", argv[:argv.index("--")])

    def test_matching_owner_is_accepted(self):
        (self.slot / ".owner").write_text("lane-owner")
        self.module.check_owner(self.slot, "lane-owner")

    def test_missing_or_different_owner_is_rejected(self):
        with self.assertRaises(FileNotFoundError):
            self.module.check_owner(self.slot, "lane-owner")
        (self.slot / ".owner").write_text("other-owner")
        with self.assertRaisesRegex(ValueError, "owner mismatch"):
            self.module.check_owner(self.slot, "lane-owner")

    def test_owner_comparison_is_exact(self):
        (self.slot / ".owner").write_text("lane-owner\n")
        with self.assertRaisesRegex(ValueError, "owner mismatch"):
            self.module.check_owner(self.slot, "lane-owner")

    def test_capacity_requires_literal_add_verdict(self):
        for output in ["agents 1/32 | ADD: available\n", "ADD\n"]:
            with self.subTest(output=output), patch.object(
                self.module.subprocess, "run",
                return_value=subprocess.CompletedProcess([], 0, output, ""),
            ) as run:
                self.module.check_capacity("lane-capacity")
                self.assertEqual(run.call_args.kwargs["env"]["CAPACITY_ID"],
                                 "lane-capacity")
                self.assertNotIn("CAPACITY_PEEK", run.call_args.kwargs["env"])
        for output in ["agents 1/32 | HOLD: ADD later", "NOT_ADD", "SHED"]:
            with self.subTest(output=output), patch.object(
                self.module.subprocess, "run",
                return_value=subprocess.CompletedProcess([], 0, output, ""),
            ):
                with self.assertRaisesRegex(ValueError, "capacity refused"):
                    self.module.check_capacity("lane-capacity")

    def test_status_reports_finished_wall_time(self):
        output = ("SubState=exited\nResult=success\nCPUUsageNSec=434560000\n"
                  "ExecMainStartTimestampMonotonic=1000000\n"
                  "ExecMainExitTimestampMonotonic=2075731\n")
        with patch.object(self.module.subprocess, "run", return_value=
                          subprocess.CompletedProcess([], 0, output, "")), \
                contextlib.redirect_stdout(io.StringIO()) as stdout:
            self.module.print_status("lane-build")
        self.assertEqual(stdout.getvalue(),
                         "SubState=exited\nResult=success\nCPUUsageNSec=434560000\n"
                         "WallTimeSec=1.075731\n")


class MountedDepInfoTests(unittest.TestCase):
    def setUp(self):
        SlotBuildTests.setUp(self)
        self.tree = self.slot / "worktree"
        self.tree.mkdir()
        self.target = self.slot / "targets/slot-2"
        self.target.mkdir(parents=True)
        self.source_root = self.slot / "source-mounts"
        self.mount = self.source_root / "slot-2"
        source_patch = patch.object(self.module, "SOURCE_ROOT", self.source_root)
        source_patch.start()
        self.addCleanup(source_patch.stop)
        self.current = {
            "worktree": str(self.tree), "slot": str(self.target),
            "package_roots": {"probe": [str(self.mount)]},
            "changed_paths": [], "content_changed_paths": [], "reported_artifacts": [],
        }

    def add_dep_info(self, name, content):
        binary = self.target / name
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_bytes(b"unchanged executable")
        binary.chmod(0o755)
        binary.with_suffix(".d").write_text(content)
        self.current["reported_artifacts"].append({
            "executable": str(binary), "fresh": True, "package_id": "probe",
            "manifest_path": str(self.mount / "Cargo.toml"),
        })
        return str(binary)

    def load_real_fixtures(self):
        fixtures = Path(__file__).parent / "fixtures/slot-build-dep-info"
        names = ["debug/wow-sim", "debug/wow-cli",
                 "debug/deps/integration-7df7921f32c569e7"]
        for name in names:
            content = (fixtures / (Path(name).name + ".d")).read_text()
            content = content.replace("/home/osso/.cache/wow-ui-sim-targets/slot-0",
                                      str(self.target))
            content = content.replace("/home/osso/.cache/wow-ui-sim-srcmount/slot-0",
                                      str(self.mount))
            self.add_dep_info(name, content)

    def read_inputs(self, changed):
        self.current["changed_paths"] = changed
        self.current["content_changed_paths"] = changed
        snapshot = self.module.read_dep_info_snapshot(self.target, self.tree)
        inputs, errors = self.module.read_target_inputs(self.current, snapshot, snapshot)
        self.current.update(target_inputs=inputs, dep_info_errors=errors)
        return inputs, errors

    def test_real_mounted_lib_diff_judges_both_application_targets(self):
        self.load_real_fixtures()
        inputs, errors = self.read_inputs(["src/lib.rs"])
        self.assertEqual(errors, [])
        self.assertIn("src/lib.rs", inputs[str(self.target / "debug/wow-sim")])
        hashes = {name: "same" for name in ["debug/wow-sim", "debug/wow-cli",
                  "debug/deps/integration-7df7921f32c569e7"]}
        self.assertEqual(self.module.find_stale_artifacts(self.current, hashes, hashes),
                         ["debug/wow-cli", "debug/wow-sim"])

    def test_real_mounted_test_diff_judges_only_integration_target(self):
        self.load_real_fixtures()
        inputs, errors = self.read_inputs(["tests/fast_handler_late_bind.rs"])
        self.assertEqual(errors, [])
        self.assertEqual(self.module.find_stale_artifacts(self.current, {}, {}),
                         ["debug/deps/integration-7df7921f32c569e7"])

    def test_mounted_and_worktree_escaped_inputs_are_repo_relative(self):
        binary = self.target / "debug/escaped"
        content = (f"{binary}: {self.mount}/src/space\\ name$$.rs \\\n"
                   f" {self.tree}/src/worktree\\ name$$.rs \\\n"
                   " src/relative\\ name$$.rs /registry/external.rs "
                   "/toolchain/lib.rs /home/osso/.cargo/git/checkouts/lib.rs "
                   f"{self.mount}-other/src/outside.rs\n")
        executable = self.add_dep_info("debug/escaped", content)
        inputs, errors = self.read_inputs(["src/space name$.rs"])
        self.assertEqual(errors, [])
        self.assertEqual(set(inputs[executable]), {
            "src/space name$.rs", "src/worktree name$.rs", "src/relative name$.rs",
            "build.rs", "Cargo.toml", "Cargo.lock",
        })
        self.assertEqual(self.module.find_stale_artifacts(self.current, {}, {}),
                         ["debug/escaped"])

    def test_real_utility_dep_info_maps_ct_bins_slot_mount(self):
        fixture = Path(__file__).parent / "fixtures/slot-build-dep-info/ct-bins-utility.d"
        content = fixture.read_text().replace(
            "/home/osso/.cache/wow-ui-sim-targets/slot-2", str(self.target))
        content = content.replace("/home/osso/.cache/wow-ui-sim-srcmount",
                                  str(self.source_root))
        for name in ["bench_spellbook", "bench_steady_state", "bench_talents",
                     "panel-visual-metrics"]:
            self.add_dep_info("debug/" + name, content.replace("bench_spellbook", name))
        inputs, errors = self.read_inputs(["src/lib.rs"])
        self.assertEqual(errors, [])
        for record in self.current["reported_artifacts"]:
            self.assertIn("src/lib.rs", inputs[record["executable"]])
        self.assertEqual(len(self.module.find_stale_artifacts(self.current, {}, {})), 4)

    def load_slot1_integration_bins(self):
        fixture = Path(__file__).parent / "fixtures/slot-build-dep-info/slot-1-integration-bins.json"
        entries = json.loads(fixture.read_text())
        for entry in entries:
            serialized = json.dumps(entry).replace(
                "/home/osso/.cache/wow-ui-sim-targets/slot-1", str(self.target))
            serialized = serialized.replace(
                "/home/osso/.cache/wow-ui-sim-srcmount/slot-1", str(self.mount))
            entry = json.loads(serialized)
            dep_info = Path(entry["dep_info_path"])
            dep_info.parent.mkdir(parents=True, exist_ok=True)
            dep_info.write_text(entry["dep_info"])
            hashed_binary = dep_info.with_suffix("")
            hashed_binary.write_bytes(b"rebuilt utility")
            hashed_binary.chmod(0o755)
            Path(entry["record"]["executable"]).hardlink_to(hashed_binary)
            self.current["reported_artifacts"].append(entry["record"])

    def test_real_slot1_integration_bins_use_hashed_dep_info(self):
        self.load_slot1_integration_bins()
        inputs, errors = self.read_inputs(["src/bin/bench_spellbook.rs"])
        self.assertEqual(errors, [])
        executable = str(self.target / "debug/bench_spellbook")
        self.assertIn("src/bin/bench_spellbook.rs", inputs[executable])
        self.assertEqual(self.module.find_stale_artifacts(self.current, {}, {}),
                         ["debug/bench_spellbook"])
        hashes = {"debug/bench_spellbook": "rebuilt"}
        self.assertEqual(self.module.find_stale_artifacts(self.current, {}, hashes), [])

    def test_hashed_dep_info_takes_precedence_over_uplifted_dep_info(self):
        self.load_slot1_integration_bins()
        for record in self.current["reported_artifacts"]:
            Path(record["executable"]).with_suffix(".d").write_text("invalid uplift")
        _, errors = self.read_inputs(["src/bin/bench_spellbook.rs"])
        self.assertEqual(errors, [])

    def test_reported_hashed_filename_selects_dep_info_without_hardlink(self):
        self.load_slot1_integration_bins()
        record = self.current["reported_artifacts"][-1]
        binary = Path(record["executable"])
        hashed = next((binary.parent / "deps").glob("bench_spellbook-*"))
        if hashed.suffix == ".d":
            hashed = hashed.with_suffix("")
        record["filenames"].append(str(hashed))
        binary.unlink()
        binary.write_bytes(b"copied uplift")
        binary.chmod(0o755)
        _, errors = self.read_inputs(["src/bin/bench_spellbook.rs"])
        self.assertEqual(errors, [])

    def test_unrelated_repo_package_without_dep_info_fails_closed(self):
        self.add_dep_info("debug/known", "known: src/lib.rs\n")
        self.current["reported_artifacts"].append({
            "executable": str(self.target / "debug/unrelated"), "fresh": False,
            "package_id": "unrelated", "manifest_path": str(self.mount / "other/Cargo.toml"),
        })
        self.current["package_roots"]["unrelated"] = [str(self.mount / "other")]
        _, errors = self.read_inputs(["src/lib.rs"])
        self.assertIn("missing dep-info", "; ".join(errors))

    def test_make_continuation_separates_adjacent_input_words(self):
        binary = self.target / "debug/continued"
        content = (f"{binary}: {self.mount}/src/first.rs\\\n"
                   f"{self.mount}/src/second.rs\n")
        executable = self.add_dep_info("debug/continued", content)
        inputs, errors = self.read_inputs(["src/second.rs"])
        self.assertEqual(errors, [])
        self.assertIn("src/first.rs", inputs[executable])
        self.assertIn("src/second.rs", inputs[executable])
        self.assertEqual(self.module.find_stale_artifacts(self.current, {}, {}),
                         ["debug/continued"])

    def test_zero_repo_inputs_fail_closed_before_package_inputs_are_added(self):
        for changed in [["src/lib.rs"], ["Cargo.toml"]]:
            with self.subTest(changed=changed):
                self.current["reported_artifacts"] = []
                binary = self.target / "debug/external"
                self.add_dep_info("debug/external", f"{binary}: /registry/input.rs\n")
                inputs, errors = self.read_inputs(changed)
                self.assertTrue(errors, "external-only dep-info must fail closed")
                self.assertIn("zero repo-relative inputs", "; ".join(errors))
                with self.assertRaisesRegex(ValueError, "zero repo-relative inputs"):
                    self.module.finish_build(self.target, self.target / "receipt.json",
                                             self.current, None, {}, 0)

    def test_unmappable_dep_info_rejects_even_identical_content(self):
        binary = self.target / "debug/external"
        self.add_dep_info("debug/external", f"{binary}: /registry/input.rs\n")
        self.current["changed_paths"] = ["src/lib.rs"]
        snapshot = self.module.read_dep_info_snapshot(self.target, self.tree)
        inputs, errors = self.module.read_target_inputs(self.current, snapshot, snapshot)
        self.assertIn("zero repo-relative inputs", "; ".join(errors))
        self.current.update(target_inputs=inputs, dep_info_errors=errors)
        receipt = self.target / "receipt.json"
        with self.assertRaisesRegex(ValueError, "zero repo-relative inputs"):
            self.module.finish_build(self.target, receipt, self.current, None, {}, 0)

    def test_complete_dep_info_allows_diff_outside_compiler_inputs(self):
        binary = self.target / "debug/unrelated"
        self.add_dep_info("debug/unrelated", f"{binary}: {self.mount}/src/other.rs\n")
        for changed in ["src/unused.rs", "docs/spec.md", "runtime/not-embedded.lua"]:
            with self.subTest(changed=changed):
                _, errors = self.read_inputs([changed])
                self.assertEqual(errors, [])
                self.module.finish_build(self.target, self.target / "receipt.json",
                                         self.current, None, {}, 0)

    def load_slot0_ptr_build(self):
        fixture = Path(__file__).parent / "fixtures/slot-build-dep-info/slot-0-ptr-i4b2-five-build.json"
        value = json.loads(fixture.read_text())
        serialized = json.dumps(value).replace(value["worktree"], str(self.tree))
        serialized = serialized.replace(value["slot"], str(self.target))
        serialized = serialized.replace("/home/osso/.cache/wow-ui-sim-srcmount/slot-0",
                                        str(self.mount))
        value = json.loads(serialized)
        for name, content in value.pop("dep_info").items():
            path = self.target / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        for name, aliases in value.pop("aliases").items():
            binary = self.target / name
            binary.write_bytes(b"rebuilt")
            binary.chmod(0o755)
            for alias in aliases:
                (self.target / alias).hardlink_to(binary)
        self.current = value
        return value["previous"], value["artifacts"]

    def test_real_slot0_library_diff_accepts_rebuilt_library_on_own_inputs(self):
        previous, artifacts = self.load_slot0_ptr_build()
        inputs, errors = self.read_inputs(self.current["content_changed_paths"])
        self.assertEqual(errors, [])
        library = self.current["reported_artifacts"][0]
        library_inputs = inputs[library["filenames"][0]]
        self.assertIn("src/c_api/ptr_i4_game_rules.rs", library_inputs)
        self.assertIn("src/lua_api/globals/game_rules.rs", library_inputs)
        for record in self.current["reported_artifacts"][1:]:
            self.assertNotIn("src/c_api/ptr_i4_game_rules.rs", inputs[record["executable"]])
        self.module.finish_build(self.target, self.target / "receipt.json",
                                 self.current, previous, artifacts, 0)
        self.assertEqual(json.loads((self.target / "receipt.json").read_text())["result"],
                         "success")

    def test_real_slot0_executable_own_diff_rejects_each_unchanged_hash(self):
        previous, artifacts = self.load_slot0_ptr_build()
        for name, source in [("debug/wow-sim", "src/bin/wow_sim/main.rs"),
                             ("debug/deps/integration-0702448e9a6092df", "tests/integration.rs")]:
            with self.subTest(name=name):
                self.read_inputs([source])
                unchanged = dict(artifacts, **{name: previous["artifacts"][name]})
                self.assertEqual(self.module.find_stale_artifacts(
                    self.current, previous["artifacts"], unchanged), [name])

    def test_real_slot0_embedded_lua_is_a_compiler_input(self):
        previous, artifacts = self.load_slot0_ptr_build()
        changed = ["src/lua_api/env_init/runtime_surface_bootstrap.lua"]
        inputs, errors = self.read_inputs(changed)
        self.assertEqual(errors, [])
        library = self.current["reported_artifacts"][0]
        self.assertIn(changed[0], inputs[library["filenames"][0]])
        library["fresh"] = True
        self.assertEqual(self.module.find_stale_artifacts(
            self.current, previous["artifacts"], previous["artifacts"]),
            ["debug/deps/integration-0702448e9a6092df",
             "debug/deps/libwow_ui_sim-45d2a5ba882881e5.rlib"])

    def test_real_slot0_fresh_library_requires_previous_dep_info(self):
        self.load_slot0_ptr_build()
        library = self.current["reported_artifacts"][0]
        self.assertEqual(library["target"]["name"], "wow_ui_sim")
        library["fresh"] = True
        before = self.module.read_dep_info_snapshot(self.target, self.tree)
        del before["files"][str(self.target / "debug/deps/wow_ui_sim-45d2a5ba882881e5.d")]
        after = self.module.read_dep_info_snapshot(self.target, self.tree)
        _, errors = self.module.read_target_inputs(self.current, before, after)
        self.assertTrue(errors, "fresh library cannot certify missing previous inputs")

    def test_transitive_path_library_is_judged_without_executable_mapping(self):
        binary = self.target / "debug/probe"
        self.add_dep_info("debug/probe", f"{binary}: {self.mount}/src/main.rs\n")
        self.current["package_roots"]["probe"].append(str(self.mount / "dep"))
        self.current["package_roots"]["dep"] = [
            str(self.mount / "dep"), str(self.mount / "leaf")]

        self.current["package_roots"]["leaf"] = [str(self.mount / "leaf")]
        for name, source in [("dep", "dep/src/lib.rs"), ("leaf", "leaf/data/config.lua")]:
            artifact = self.target / f"debug/deps/lib{name}-123.rlib"
            artifact.parent.mkdir(parents=True, exist_ok=True)
            artifact.with_name(f"{name}-123.d").write_text(
                f"{artifact}: {self.mount}/{source}\n")
            self.current["reported_artifacts"].append({
                "package_id": name, "manifest_path": str(self.mount / name / "Cargo.toml"),
                "target": {"name": name, "kind": ["lib"]},
                "filenames": [str(artifact)], "executable": None, "fresh": False,
            })
        inputs, errors = self.read_inputs(["leaf/data/config.lua"])
        self.assertEqual(errors, [])
        self.assertNotIn("leaf/data/config.lua", inputs[str(binary)])
        self.assertIn("leaf/data/config.lua", inputs[str(self.target / "debug/deps/libleaf-123.rlib")])
        hashes = {"debug/probe": "same"}
        self.assertEqual(self.module.find_stale_artifacts(self.current, hashes, hashes), [])
        self.current["reported_artifacts"][-1]["fresh"] = True
        self.assertEqual(self.module.find_stale_artifacts(self.current, hashes, hashes),
                         ["debug/deps/libleaf-123.rlib"])

    def test_fix11_real_sim_toc_fresh_library_rejects_its_own_change(self):
        previous, artifacts = self.load_slot0_ptr_build()
        artifact = self.target / "debug/deps/libsim_toc-c9659c1ae6dcefe3.rlib"
        artifact.with_name("sim_toc-c9659c1ae6dcefe3.d").write_text(
            f"{artifact}: crates/sim-toc/src/lib.rs\n")
        self.current["reported_artifacts"].append({
            "package_id": f"path+file://{self.mount}/crates/sim-toc#0.1.0",
            "manifest_path": str(self.mount / "crates/sim-toc/Cargo.toml"),
            "target": {"name": "sim_toc", "kind": ["lib"]},
            "filenames": [str(artifact)], "executable": None, "fresh": True,
        })
        self.read_inputs(["crates/sim-toc/src/lib.rs"])
        with self.assertRaisesRegex(ValueError, "stale artifacts"):
            self.module.finish_build(self.target, self.target / "receipt.json",
                                     self.current, previous, previous["artifacts"], 0)
        self.assertEqual(json.loads((self.target / "receipt.json").read_text())["result"],
                         "stale-artifacts")

    def test_fix11_dev_only_library_does_not_require_normal_bin_rebuild(self):
        (self.tree / "src").mkdir()
        (self.tree / "src/main.rs").write_text("fn main() {}")
        (self.tree / "dep/src").mkdir(parents=True)
        (self.tree / "dep/src/lib.rs").write_text("pub fn helper() {}")
        (self.tree / "Cargo.toml").write_text(
            '[package]\nname="app"\nversion="0.1.0"\nedition="2021"\n'
            '[dev-dependencies]\ndep={path="dep"}\n')
        (self.tree / "dep/Cargo.toml").write_text(
            '[package]\nname="dep"\nversion="0.1.0"\nedition="2021"\n')
        roots = self.module.read_package_roots(self.tree)
        app_id = next(key for key in roots if "#app@" in key)
        binary = self.target / "debug/app"
        self.add_dep_info("debug/app", f"{binary}: {self.mount}/src/main.rs\n")
        self.current["reported_artifacts"][0]["package_id"] = app_id
        self.current["package_roots"] = {
            key: [path.replace(str(self.tree), str(self.mount)) for path in paths]
            for key, paths in roots.items()}
        artifact = self.target / "debug/deps/libdep-123.rlib"
        artifact.parent.mkdir(parents=True, exist_ok=True)
        artifact.with_name("dep-123.d").write_text(
            f"{artifact}: {self.mount}/dep/src/lib.rs\n")
        self.current["reported_artifacts"].append({
            "package_id": f"path+file://{self.mount}/dep#0.1.0",
            "manifest_path": str(self.mount / "dep/Cargo.toml"),
            "target": {"name": "dep", "kind": ["lib"]},
            "filenames": [str(artifact)], "executable": None, "fresh": False,
        })
        self.read_inputs(["dep/src/lib.rs"])
        hashes = {"debug/app": "same"}
        try:
            self.module.finish_build(self.target, self.target / "receipt.json",
                                     self.current, {"artifacts": hashes}, hashes, 0)
        except ValueError as error:
            self.fail(f"dev-only library must not require a normal bin rebuild: {error}")
        self.assertEqual(json.loads((self.target / "receipt.json").read_text())["result"],
                         "success")

    def test_fix11_explicit_external_records_are_excluded(self):
        for source in ["registry+https://index.crates.io/dep#0.1.0",
                       "git+https://example.org/dep#123"]:
            with self.subTest(source=source):
                self.current["reported_artifacts"] = [{
                    "package_id": source, "manifest_path": "/external/dep/Cargo.toml",
                    "target": {"name": "dep", "src_path": "/external/dep/src/lib.rs"},
                    "filenames": [str(self.target / "debug/deps/libdep-123.rlib")],
                    "executable": None, "fresh": True,
                }]
                _, errors = self.read_inputs(["src/lib.rs"])
                self.assertEqual(errors, [])
                self.module.finish_build(self.target, self.target / "receipt.json",
                                         self.current, None, {}, 0)

    def test_fix11_repo_owned_libraries_fail_closed_for_zero_or_missing_inputs(self):
        utility = self.source_root / "ct-bins-slot-2"
        for root in [self.tree, self.mount, utility]:
            for ownership_field in ["manifest_path", "src_path"]:
                for bad_inputs in [None, "/registry/external.rs"]:
                    with self.subTest(root=root, field=ownership_field, inputs=bad_inputs):
                        artifact = self.target / "debug/deps/libdep-123.rlib"
                        dep_info = artifact.with_name("dep-123.d")
                        artifact.parent.mkdir(parents=True, exist_ok=True)
                        dep_info.unlink(missing_ok=True)
                        if bad_inputs:
                            dep_info.write_text(f"{artifact}: {bad_inputs}\n")
                        record = {
                            "package_id": f"path+file://{root}#dep@0.1.0",
                            "target": {"name": "dep", "kind": ["lib"]},
                            "filenames": [str(artifact)], "executable": None, "fresh": False,
                        }
                        if ownership_field == "manifest_path":
                            record["manifest_path"] = str(root / "Cargo.toml")
                        else:
                            record["target"]["src_path"] = str(root / "src/lib.rs")
                        self.current["reported_artifacts"] = [record]
                        self.read_inputs([])
                        with self.assertRaisesRegex(ValueError, "dep-info|zero repo-relative"):
                            self.module.finish_build(self.target, self.target / "receipt.json",
                                                     self.current, None, {}, 0)

    def test_fix11_unread_source_change_is_informational(self):
        binary = self.target / "debug/app"
        self.add_dep_info("debug/app", f"{binary}: {self.mount}/src/main.rs\n")
        self.read_inputs(["src/unused.rs", "docs/spec.md", "tools/helper.py"])
        self.module.finish_build(self.target, self.target / "receipt.json",
                                 self.current, None, {}, 0)
        receipt = json.loads((self.target / "receipt.json").read_text())
        self.assertEqual(receipt["result"], "success")
        self.assertIn("src/unused.rs", receipt["unread_package_paths"])

    def test_real_slot0_missing_library_dep_info_fails_closed(self):
        self.load_slot0_ptr_build()
        (self.target / "debug/deps/wow_ui_sim-45d2a5ba882881e5.d").unlink()
        _, errors = self.read_inputs(self.current["content_changed_paths"])
        self.assertTrue(errors)


class DirectoryInputDigestTests(unittest.TestCase):
    setUp = SlotBuildTests.setUp

    def digest(self, path):
        return self.module.read_input_hashes(self.slot, [str(path)])[str(path)]

    def test_tree_digest_is_relative_and_independent_of_creation_order(self):
        left, right = self.slot / "left", self.slot / "right"
        for root, names in [(left, ["a", "b"]), (right, ["b", "a"])]:
            (root / "nested").mkdir(parents=True)
            for name in names:
                (root / "nested" / name).write_text(name)
        self.assertEqual(self.digest(left), self.digest(right))
        (right / "nested/a").rename(right / "nested/c")
        self.assertNotEqual(self.digest(left), self.digest(right))

    def test_tree_digest_follows_file_and_directory_symlinks(self):
        root, target = self.slot / "tree", self.slot / "target"
        root.mkdir()
        target.mkdir()
        value = target / "value"
        value.write_text("original")
        for name, referent in [("file", value), ("directory", target)]:
            with self.subTest(name=name):
                link = root / name
                link.symlink_to(referent)
                before = self.digest(root)
                value.write_text("changed")
                self.assertNotEqual(before, self.digest(root))
                value.write_text("original")
                link.unlink()

    def test_directory_symlink_parent_traversal_preserves_real_input(self):
        (self.slot / "actual/nested").mkdir(parents=True)
        (self.slot / "actual/input").mkdir()
        (self.slot / "actual/input/value").write_text("actual")
        (self.slot / "input").mkdir()
        (self.slot / "input/value").write_text("decoy")
        link = self.slot / "link"
        link.symlink_to(self.slot / "actual/nested")
        spelling = link / "../input"
        self.assertEqual(self.digest(spelling), self.digest(self.slot / "actual/input"))
        self.assertNotEqual(self.digest(spelling), self.digest(self.slot / "input"))
        link.unlink()
        self.assertIsNone(self.digest(spelling))

    def test_directory_digest_cannot_equal_file_content_digest(self):
        root = self.slot / "tree"
        root.mkdir()
        directory_digest = self.digest(root)
        root.rmdir()
        root.write_bytes(b"directory\0")
        self.assertNotEqual(directory_digest, self.digest(root))

    def test_cyclic_directory_symlink_fails_closed(self):
        root = self.slot / "tree"
        root.mkdir()
        (root / "loop").symlink_to(root)
        with self.assertRaisesRegex(ValueError, "cyclic directory input"):
            self.digest(root)


class SourceFreshnessTests(unittest.TestCase):
    def setUp(self):
        SlotBuildTests.setUp(self)
        self.tree = self.slot / "tree"
        self.tree.mkdir()
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.com")
        self.git("config", "user.name", "Test")
        (self.tree / "src").mkdir()
        (self.tree / "src/main.rs").write_text('fn main() { println!("A"); }')
        (self.tree / "Cargo.toml").write_text('[package]\nname="probe"\nversion="0.1.0"\n')
        (self.tree / "Cargo.lock").write_text('version = 3\n[[package]]\nname = "probe"\nversion = "0.1.0"\n')
        self.git("add", ".")
        self.git("commit", "-qm", "A")

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.tree), *args])

    def prepare(self, previous=None):
        self.assertTrue(callable(getattr(self.module, "prepare_source", None)),
                        "launcher does not invalidate older source mtimes")
        return self.module.prepare_source(self.tree, previous)

    def run_library_step(self, profile, fresh=False, mutation="", executable=False):
        """Run a separate worker against persistent outputs, without Cargo."""
        import sys
        (self.slot / ".owner").write_text("test-owner")
        filename = f"probe-{profile}" if executable else f"libprobe-{profile}.rmeta"
        binary = self.slot / "debug/deps" / filename
        emitter = (
            'import json,sys; from pathlib import Path\n'
            'binary,source=map(Path,sys.argv[1:3])\n'
            'fresh=json.loads(sys.argv[3])\n'
            'if not fresh:\n'
            '    binary.parent.mkdir(parents=True,exist_ok=True)\n'
            '    binary.write_bytes(source.read_bytes())\n'
            '    binary.with_suffix(".d").write_text(str(binary)+": src/main.rs\\n")\n'
            'exec(sys.argv[4])\n'
            f'print(json.dumps({{"reason":"compiler-artifact","package_id":{f"path+file://{self.tree}#probe@0.1.0"!r},"filenames":[str(binary)],\n'
            '    "executable":str(binary) if json.loads(sys.argv[5]) else None,"fresh":fresh}))\n'
        )
        request = self.slot / "library-request.json"
        receipt = self.slot / "library-receipt.json"
        request.write_text(json.dumps({
            "slot": str(self.slot), "worktree": str(self.tree), "owner": "test-owner",
            "unit": "library-step", "receipt": str(receipt),
            "cargo": [sys.executable, "-c", emitter, str(binary),
                      str(self.tree / "src/main.rs"), json.dumps(fresh), mutation,
                      json.dumps(executable)]}))
        result = subprocess.run(
            [sys.executable, "-B", str(SCRIPT), "--execute", str(request)],
            capture_output=True)
        return result, json.loads(receipt.read_text())

    def build_directory_library(self, executable=False):
        directory = self.tree / "tests/nested"
        directory.mkdir(parents=True)
        (directory / "case.rs").write_text("original case")
        self.git("add", ".")
        self.git("commit", "-qm", "directory build input")
        mutation = ('binary.write_bytes(source.read_bytes() + '
                    '(source.parent.parent / "tests/nested/case.rs").read_bytes()); '
                    'binary.with_suffix(".d").write_text('
                    'str(binary)+": src/main.rs tests\\n")')
        result, receipt = self.run_library_step("retail", mutation=mutation,
                                                executable=executable)
        self.assertEqual(result.returncode, 0, result.stderr)
        return receipt

    def test_directory_input_unchanged_tree_reuses_certificate(self):
        first = self.build_directory_library()
        # Content, not timestamps, determines freshness.
        os.utime(self.tree / "tests/nested/case.rs", (1, 1))
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["artifact_certificates_sha256"],
                         first["artifact_certificates_sha256"])
        output = self.slot / "debug/deps/libprobe-retail.rmeta"
        self.assertTrue(output.read_bytes().endswith(b"original case"))

    def test_receipt_summarizes_certificates_kept_in_verified_identity(self):
        receipt = self.build_directory_library()
        verified = json.loads((self.slot / ".verified-source-identity.json").read_text())
        self.assertNotIn("artifact_certificates", receipt)
        certificates = verified["artifact_certificates"]
        self.assertGreater(len(certificates), 0)
        self.assertEqual(receipt["artifact_certificate_count"], len(certificates))
        self.assertEqual(receipt["artifact_certificates_sha256"], hashlib.sha256(
            json.dumps(certificates, sort_keys=True).encode()).hexdigest())

    def test_stat_cache_reuses_unchanged_bytes_and_rereads_rewrites(self):
        self.module.RACY_NS = 0
        path = self.slot / "input.rs"
        path.write_text("first")
        reads = []
        read = lambda: reads.append(path.read_text()) or reads[-1]
        self.assertEqual(self.module.stat_cached("text", path, read), "first")
        self.assertEqual(self.module.stat_cached("text", path, read), "first")
        self.assertEqual(reads, ["first"])
        time.sleep(0.02)
        path.write_text("other")
        self.assertEqual(self.module.stat_cached("text", path, read), "other")
        self.assertEqual(reads, ["first", "other"])

    def test_stat_cache_rereads_values_read_inside_racy_window(self):
        path = self.slot / "input.rs"
        path.write_text("just written")
        self.module.file_sha256(path)
        cache = self.slot / ".stat-cache.json"
        self.module.save_stat_cache(cache)
        self.assertEqual(json.loads(cache.read_text()), {})

    def test_persisted_stat_cache_cannot_hide_same_size_source_edit(self):
        self.build_directory_library()
        # The first build touches changed sources; once they age past the racy
        # window, an unrelated build stores their digests.
        time.sleep(self.module.RACY_NS / 1e9 + 0.1)
        result, _ = self.run_library_step("forever")
        self.assertEqual(result.returncode, 0, result.stderr)
        directory = self.tree / "tests/nested"
        cached = json.loads((self.slot / ".stat-cache.json").read_text())
        self.assertIn(hashlib.sha256(b"original case").hexdigest(),
                      [value for _, value in cached.values()])
        self.assert_cached_change_rejected(
            lambda: (directory / "case.rs").write_text("Original case"))

    def assert_directory_change_rejected(self, change):
        self.build_directory_library()
        self.assert_cached_change_rejected(lambda: change(self.tree / "tests/nested"))

    def assert_cached_change_rejected(self, change):
        output = self.slot / "debug/deps/libprobe-retail.rmeta"
        change()
        # Advance the slot-wide source baseline with an unrelated profile.
        result, _ = self.run_library_step("forever")
        self.assertEqual(result.returncode, 0, result.stderr)
        verified = (self.slot / ".verified-source-identity.json").read_bytes()
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertEqual(receipt["changed_paths"], [])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertFalse(output.exists())
        self.assertEqual((self.slot / ".verified-source-identity.json").read_bytes(), verified)

    def test_directory_input_changed_nested_file_rejects_reuse(self):
        self.assert_directory_change_rejected(
            lambda directory: (directory / "case.rs").write_text("changed case"))

    def test_directory_input_added_file_rejects_reuse(self):
        self.assert_directory_change_rejected(
            lambda directory: (directory / "added.rs").write_text("added case"))

    def test_directory_input_removed_file_rejects_reuse(self):
        self.assert_directory_change_rejected(
            lambda directory: (directory / "case.rs").unlink())

    def test_directory_nested_edit_requires_changed_rebuilt_executable_hash(self):
        first = self.build_directory_library(executable=True)
        output = Path(first["reported_artifacts"][0]["executable"])
        original = output.read_bytes()
        verified = (self.slot / ".verified-source-identity.json").read_bytes()
        (self.tree / "tests/nested/case.rs").write_text("changed case")
        mutation = (f'binary.write_bytes({original!r}); '
                    'binary.with_suffix(".d").write_text('
                    'str(binary)+": src/main.rs tests\\n")')
        result, receipt = self.run_library_step("retail", mutation=mutation, executable=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertEqual(output.read_bytes(), original)
        self.assertEqual((self.slot / ".verified-source-identity.json").read_bytes(), verified)
        self.assertNotIn("tests/nested/case.rs", receipt["unread_package_paths"])

    def test_directory_mtime_only_refresh_rejects_fresh_record(self):
        self.build_directory_library()
        output = str(self.slot / "debug/deps/libprobe-retail.rmeta")
        current = {"worktree": str(self.tree), "slot": str(self.slot),
                   "changed_paths": ["tests/nested/case.rs"],
                   "content_changed_paths": [],
                   "reported_artifacts": [{"filenames": [output], "fresh": True}],
                   "target_inputs": {output: ["tests"]}}
        self.assertEqual(self.module.find_stale_artifacts(current, {}, {}),
                         ["debug/deps/libprobe-retail.rmeta"])

    def test_profile_switch_cannot_certify_an_older_library(self):
        result, _ = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        source = self.tree / "src/main.rs"
        source.write_text('struct PetBattlePet { display_id: u32 }')
        self.git("commit", "-qam", "new model")
        os.utime(source, (1, 1))
        result, _ = self.run_library_step("forever")
        self.assertEqual(result.returncode, 0, result.stderr)
        verified = (self.slot / ".verified-source-identity.json").read_bytes()
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertEqual(receipt["changed_paths"], [])
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertEqual((self.slot / ".verified-source-identity.json").read_bytes(), verified)

    def test_rebuilt_library_with_fewer_inputs_allows_immediate_fresh_reuse(self):
        old = self.tree / "src/old.rs"
        old.write_text("pub fn old() {}")
        self.git("add", ".")
        self.git("commit", "-qm", "add old module")
        result, _ = self.run_library_step(
            "retail", mutation='binary.with_suffix(".d").write_text('
            'str(binary)+": src/main.rs src/old.rs\\n")')
        self.assertEqual(result.returncode, 0, result.stderr)
        old.unlink()
        self.git("commit", "-qam", "remove old module")
        result, receipt = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        output = str(self.slot / "debug/deps/libprobe-retail.rmeta")
        self.assertIn("src/old.rs", receipt["target_inputs"][output])
        rebuilt = (self.slot / "debug/deps/libprobe-retail.rmeta").read_bytes()
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["result"], "success")
        self.assertEqual(Path(output).read_bytes(), rebuilt)

    def build_generated_library(self, generated, prefix=""):
        mutation = (prefix + f'generated=Path({str(generated)!r}); '
                    'binary.write_bytes(source.read_bytes()+generated.read_bytes()); '
                    'binary.with_suffix(".d").write_text('
                    'str(binary)+": src/main.rs "+str(generated)+"\\n")')
        result, receipt = self.run_library_step("retail", mutation=mutation)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["result"], "success")

    def test_fresh_library_rejects_changed_generated_compiler_input(self):
        generated = self.slot / "debug/build/test-harness-hash/out/integration_tests.rs"
        generated.parent.mkdir(parents=True)
        generated.write_text("pub const VALUE: u32 = 1;")
        self.build_generated_library(generated)
        output = self.slot / "debug/deps/libprobe-retail.rmeta"
        verified = (self.slot / ".verified-source-identity.json").read_bytes()
        generated.write_text("pub const VALUE: u32 = 2;")
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertFalse(output.exists())
        self.assertEqual(receipt["changed_paths"], [])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertEqual((self.slot / ".verified-source-identity.json").read_bytes(), verified)

    def assert_resolved_generated_input_is_guarded(self, spelling, generated):
        self.build_generated_library(spelling)
        output = self.slot / "debug/deps/libprobe-retail.rmeta"
        verified = (self.slot / ".verified-source-identity.json").read_bytes()
        generated.write_text("pub const VALUE: u32 = 2;")
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertFalse(output.exists())
        self.assertEqual(receipt["changed_paths"], [])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertEqual((self.slot / ".verified-source-identity.json").read_bytes(), verified)
        self.build_generated_library(spelling)
        # Changing only dep-info spelling must preserve the resolved certificate key.
        mutation = ('binary.with_suffix(".d").write_text('
                    f'str(binary)+": src/main.rs {generated}\\n")')
        result, receipt = self.run_library_step("retail", fresh=True, mutation=mutation)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["result"], "success")
        generated.unlink()
        for fresh in [True, False]:
            with self.subTest(fresh=fresh):
                mutation = ('binary.with_suffix(".d").write_text('
                            f'str(binary)+": src/main.rs {spelling}\\n")')
                result, receipt = self.run_library_step("retail", fresh=fresh, mutation=mutation)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(receipt["result"], "stale-artifacts")

    def test_symlink_parent_input_into_source_rejects_stale_reuse(self):
        nested = self.tree / "nested"
        nested.mkdir()
        actual = self.tree / "extra.rs"
        actual.write_text("actual v1")
        self.git("add", ".")
        self.git("commit", "-qm", "add actual input")
        bridge = self.slot / "bridge"
        bridge.symlink_to(nested, target_is_directory=True)
        spelling = bridge / ".." / "extra.rs"
        decoy = self.slot / "extra.rs"
        decoy.write_text("decoy unchanged")
        self.assertEqual(spelling.read_bytes(), actual.read_bytes())
        self.build_generated_library(spelling)
        output = self.slot / "debug/deps/libprobe-retail.rmeta"
        original = output.read_bytes()
        self.assertTrue(original.endswith(b"actual v1"))
        actual.write_text("actual v2")
        self.git("commit", "-qam", "change actual input")
        other, _ = self.run_library_step("forever")
        self.assertEqual(other.returncode, 0, other.stderr)
        verified = (self.slot / ".verified-source-identity.json").read_bytes()
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertFalse(output.exists())
        self.assertEqual(receipt["changed_paths"], [])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")
        certificate = full_identity(self.slot)["artifact_certificates"][str(output.relative_to(self.slot))]
        self.assertIn("extra.rs", certificate["inputs"])
        self.assertNotIn(str(decoy), certificate["inputs"])
        self.assertEqual((self.slot / ".verified-source-identity.json").read_bytes(), verified)

    def test_removed_input_ancestor_rejects_fresh_reuse(self):
        actual = self.slot / "present.rs"
        actual.write_text("compiler input v1")
        ancestor = self.slot / "ancestor"
        ancestor.mkdir()
        spelling = ancestor / ".." / "present.rs"
        self.assertEqual(spelling.read_bytes(), actual.read_bytes())
        self.build_generated_library(spelling)
        output = self.slot / "debug/deps/libprobe-retail.rmeta"
        original = output.read_bytes()
        self.assertTrue(original.endswith(b"compiler input v1"))
        verified = (self.slot / ".verified-source-identity.json").read_bytes()
        ancestor.rmdir()
        self.assertFalse(spelling.exists())
        with self.assertRaises(FileNotFoundError):
            spelling.read_bytes()
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertEqual(output.read_bytes(), original)
        self.assertEqual(receipt["changed_paths"], [])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertTrue(receipt["dep_info_errors"])
        self.assertEqual((self.slot / ".verified-source-identity.json").read_bytes(), verified)

    def test_target_triple_generated_input_is_guarded(self):
        generated = self.slot / "x86_64-unknown-linux-gnu/debug/build/probe-hash/out/generated.rs"
        generated.parent.mkdir(parents=True)
        generated.write_text("pub const VALUE: u32 = 1;")
        self.assert_resolved_generated_input_is_guarded(generated, generated)

    def test_external_alias_to_slot_generated_input_is_guarded(self):
        generated = self.slot / "debug/build/probe-hash/out/generated.rs"
        generated.parent.mkdir(parents=True)
        generated.write_text("pub const VALUE: u32 = 1;")
        with tempfile.TemporaryDirectory() as directory:
            alias = Path(directory) / "generated.rs"
            alias.symlink_to(generated)
            self.assert_resolved_generated_input_is_guarded(alias, generated)

    def test_slot_generated_input_outside_build_out_layout_is_guarded(self):
        generated = self.slot / "custom/nested/generated.rs"
        generated.parent.mkdir(parents=True)
        generated.write_text("pub const VALUE: u32 = 1;")
        self.assert_resolved_generated_input_is_guarded(generated, generated)

    def test_slot_alias_to_external_input_remains_external(self):
        with tempfile.TemporaryDirectory() as directory:
            external = Path(directory) / "slot-3/debug/build/probe-hash/out/generated.rs"
            external.parent.mkdir(parents=True)
            external.write_text("external v1")
            alias = self.slot / "debug/build/probe-hash/out/generated.rs"
            alias.parent.mkdir(parents=True)
            alias.symlink_to(external)
            self.build_generated_library(alias)
            external.unlink()
            result, receipt = self.run_library_step("retail", fresh=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(receipt["result"], "success")

    def test_library_requires_generated_compiler_input_to_exist(self):
        generated = self.slot / "release/build/test-harness-hash/out/prefork_full_ui_cases.rs"
        generated.parent.mkdir(parents=True)
        generated.write_text("pub fn generated() {}")
        self.build_generated_library(generated)
        generated.unlink()
        for fresh in [True, False]:
            with self.subTest(fresh=fresh):
                mutation = ('binary.with_suffix(".d").write_text('
                            f'str(binary)+": src/main.rs {generated}\\n")')
                result, receipt = self.run_library_step("retail", fresh=fresh, mutation=mutation)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(receipt["result"], "stale-artifacts")

    def test_newly_generated_compiler_input_can_be_certified_and_reused(self):
        generated = self.slot / "debug/build/test-harness-hash/out/integration_tests.rs"
        generated.parent.mkdir(parents=True)
        self.build_generated_library(
            generated, prefix=f'Path({str(generated)!r}).write_text("pub fn generated() {{}}"); ')
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["result"], "success")

    def test_absolute_compiler_input_outside_slot_and_sources_remains_external(self):
        with tempfile.TemporaryDirectory() as directory:
            external = Path(directory) / "registry.rs"
            external.write_text("external v1")
            self.build_generated_library(external)
            external.unlink()
            result, receipt = self.run_library_step("retail", fresh=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(receipt["result"], "success")

    def test_fresh_library_without_content_certificate_fails_closed(self):
        result, _ = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        verified = self.slot / ".verified-source-identity.json"
        previous = json.loads(verified.read_text())
        previous.pop("artifact_certificates", None)
        verified.write_text(json.dumps(previous))
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")

    def test_fresh_library_requires_its_certified_output_bytes(self):
        result, _ = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        result, receipt = self.run_library_step(
            "retail", fresh=True, mutation='binary.write_bytes(b"foreign output")')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")

    def test_library_cannot_certify_source_changed_during_child(self):
        result, receipt = self.run_library_step(
            "retail", mutation='source.write_text("changed after compile")')
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertFalse((self.slot / ".verified-source-identity.json").exists())

    def test_fresh_library_certificate_survives_unrelated_profile_build(self):
        for profile, fresh in [("retail", False), ("forever", False), ("retail", True)]:
            result, receipt = self.run_library_step(profile, fresh=fresh)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(receipt["result"], "success")
        self.assertEqual(len(full_identity(self.slot)["artifact_certificates"]), 2)

    def test_rebuilt_library_recovers_after_profile_reuse_rejection(self):
        result, _ = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        (self.tree / "src/main.rs").write_text("new model")
        self.git("commit", "-qam", "new model")
        result, _ = self.run_library_step("forever")
        self.assertEqual(result.returncode, 0, result.stderr)
        result, _ = self.run_library_step("retail", fresh=True)
        self.assertNotEqual(result.returncode, 0)
        for fresh in [False, True]:
            result, receipt = self.run_library_step("retail", fresh=fresh)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(receipt["result"], "success")

    def test_library_certificate_compares_content_across_worktrees(self):
        result, _ = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        other = self.slot / "other-tree"
        self.git("worktree", "add", "--detach", str(other), "HEAD")
        self.tree = other
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["changed_paths"], [])

    def test_fresh_library_rejects_dep_info_input_set_replacement(self):
        result, _ = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        other = self.tree / "src/other.rs"
        other.write_text((self.tree / "src/main.rs").read_text())
        self.git("add", ".")
        self.git("commit", "-qm", "other source")
        result, _ = self.run_library_step("forever")
        self.assertEqual(result.returncode, 0, result.stderr)
        (self.slot / "debug/deps/libprobe-retail.d").write_text(
            f"{self.slot}/debug/deps/libprobe-retail.rmeta: {other}\\n")
        result, receipt = self.run_library_step("retail", fresh=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["changed_paths"], [])
        self.assertEqual(receipt["result"], "stale-artifacts")

    def test_library_rejects_missing_compiler_input_even_if_previously_absent(self):
        result, _ = self.run_library_step("retail")
        self.assertEqual(result.returncode, 0, result.stderr)
        missing = self.tree / "src/missing.rs"
        dep_info = self.slot / "debug/deps/libprobe-retail.d"
        dep_info.write_text(f"{dep_info.with_suffix('.rmeta')}: {missing}\\n")
        # Retain the missing input in current dep-info, just as in the before snapshot.
        mutation = f'binary.with_suffix(".d").write_text({dep_info.read_text()!r})'
        result, receipt = self.run_library_step("retail", mutation=mutation)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["result"], "stale-artifacts")

    def test_older_changed_file_is_touched_without_changing_bytes(self):
        previous = self.prepare()
        source = self.tree / "src/main.rs"
        source.write_text('fn main() { println!("B"); }')
        self.git("commit", "-qam", "B")
        os.utime(source, (1, 1))
        content = source.read_bytes()
        before = time.time_ns()
        current = self.prepare(previous)
        self.assertGreaterEqual(source.stat().st_mtime_ns, before)
        self.assertEqual(source.read_bytes(), content)
        self.assertEqual(current["changed_paths"], ["src/main.rs"])
        self.assertNotEqual(current["identity"], previous["identity"])

    def test_unchanged_identity_does_not_touch_sources(self):
        previous = self.prepare()
        source = self.tree / "src/main.rs"
        os.utime(source, (1, 1))
        self.prepare(previous)
        self.assertEqual(source.stat().st_mtime_ns, 1_000_000_000)

    def test_dirty_untracked_and_reverted_paths_are_invalidated(self):
        previous = self.prepare()
        (self.tree / "src/main.rs").write_text("dirty A")
        (self.tree / "src/new.rs").write_text("untracked A")
        dirty = self.prepare(previous)
        self.git("checkout", "--", "src/main.rs")
        (self.tree / "src/new.rs").write_text("untracked B")
        for name in ["src/main.rs", "src/new.rs"]:
            os.utime(self.tree / name, (1, 1))
        current = self.prepare(dirty)
        self.assertNotEqual(current["identity"], dirty["identity"])
        self.assertEqual(current["changed_paths"], ["src/main.rs", "src/new.rs"])
        self.assertTrue(all((self.tree / p).stat().st_mtime_ns > 1_000_000_000
                            for p in current["changed_paths"]))

    def test_unknown_identity_refreshes_all_tracked_crate_inputs(self):
        source = self.tree / "src/main.rs"
        os.utime(source, (1, 1))
        current = self.prepare({"artifacts": {"debug/probe": "old"}})
        self.assertIn("src/main.rs", current["changed_paths"])
        self.assertGreater(source.stat().st_mtime_ns, 1_000_000_000)

    def test_unknown_refresh_includes_workspace_and_path_dependency_inputs(self):
        inputs = ["iced-wgpu-patched/src/lib.rs", "iced-dynamic/src/lib.rs",
                  "xtask/src/main.rs", "examples/probe.rs", "assets/icon.bin"]
        for name in inputs:
            path = self.tree / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("input")
        self.git("add", ".")
        self.git("commit", "-qm", "additional inputs")
        for name in inputs:
            os.utime(self.tree / name, (1, 1))
        current = self.prepare()
        for name in inputs:
            self.assertIn(name, current["changed_paths"])
            self.assertGreater((self.tree / name).stat().st_mtime_ns, 1_000_000_000)
            self.assertEqual((self.tree / name).read_text(), "input")

    def test_metadata_failure_refreshes_every_tracked_file(self):
        (self.tree / "Cargo.toml").write_text("invalid manifest")
        (self.tree / "outside-input.bin").write_bytes(b"input")
        self.git("add", ".")
        self.git("commit", "-qm", "unavailable metadata")
        os.utime(self.tree / "outside-input.bin", (1, 1))
        current = self.prepare()
        self.assertIn("outside-input.bin", current["changed_paths"])
        self.assertGreater((self.tree / "outside-input.bin").stat().st_mtime_ns,
                           1_000_000_000)

    def test_virtual_workspace_uses_metadata_member_and_dependency_roots(self):
        (self.tree / "Cargo.toml").write_text(
            '[workspace]\nmembers=["app"]\nexclude=["path-library"]\nresolver="2"\n')
        for name, manifest in {
            "app": '[package]\nname="app"\nversion="0.1.0"\n'
                   '[dependencies]\npath-library={path="../path-library"}\n',
            "path-library": '[package]\nname="path-library"\nversion="0.1.0"\n',
        }.items():
            root = self.tree / name
            (root / "src").mkdir(parents=True)
            (root / "src/lib.rs").write_text("pub fn input() {}")
            (root / "Cargo.toml").write_text(manifest)
        (self.tree / "README.md").write_text("docs")
        self.git("add", ".")
        self.git("commit", "-qm", "virtual workspace")
        for name in ["app/src/lib.rs", "path-library/src/lib.rs", "README.md"]:
            os.utime(self.tree / name, (1, 1))
        current = self.prepare()
        for name in ["app/src/lib.rs", "path-library/src/lib.rs"]:
            self.assertIn(name, current["changed_paths"])
            self.assertGreater((self.tree / name).stat().st_mtime_ns, 1_000_000_000)
        self.assertEqual((self.tree / "README.md").stat().st_mtime_ns, 1_000_000_000)

    def test_unknown_commit_refreshes_root_package_including_docs(self):
        (self.tree / "data").mkdir()
        (self.tree / "data/input.csv").write_text("input")
        (self.tree / "README.md").write_text("docs")
        self.git("add", ".")
        self.git("commit", "-qm", "inputs")
        previous = self.prepare()
        previous["identity"]["head"] = "0" * 40
        for name in ["src/main.rs", "data/input.csv", "README.md"]:
            os.utime(self.tree / name, (1, 1))
        current = self.prepare(previous)
        self.assertIn("data/input.csv", current["changed_paths"])
        self.assertGreater((self.tree / "src/main.rs").stat().st_mtime_ns, 1_000_000_000)
        self.assertGreater((self.tree / "README.md").stat().st_mtime_ns, 1_000_000_000)

    def test_interrupted_build_invalidates_inputs_even_at_same_identity(self):
        previous = self.prepare()
        previous["result"] = "building"
        source = self.tree / "src/main.rs"
        os.utime(source, (1, 1))
        current = self.prepare(previous)
        self.assertIn("src/main.rs", current["changed_paths"])
        self.assertGreater(source.stat().st_mtime_ns, 1_000_000_000)

    def add_reported_executable(self, current, fresh=False):
        current.update(worktree=str(self.tree), slot=str(self.slot),
                       package_roots={"probe": [str(self.tree)]},
                       target_inputs={str(self.slot / "debug/probe"): ["src/main.rs"]},
                       reported_artifacts=[{"package_id": "probe", "fresh": fresh,
                                            "executable": str(self.slot / "debug/probe")}])

    def test_identical_artifacts_after_rust_change_fail_with_receipt(self):
        previous = self.prepare()
        previous["artifacts"] = {"debug/probe": "abc"}
        (self.tree / "src/main.rs").write_text("changed")
        current = self.prepare(previous)
        receipt = self.slot / "receipt.json"
        self.add_reported_executable(current)
        with self.assertRaisesRegex(ValueError, "stale artifacts"):
            self.module.finish_build(self.slot, receipt, current, previous,
                                     {"debug/probe": "abc"}, 0)
        self.assertEqual(json.loads(receipt.read_text())["result"], "stale-artifacts")

    def test_worker_rebuilds_older_tree_in_same_persistent_target(self):
        import sys
        binary = self.slot / "debug/probe"
        binary.parent.mkdir()
        (self.slot / ".owner").write_text("test-owner")
        compiler = (
            'import json,sys; from pathlib import Path; '
            'source,binary=map(Path,sys.argv[1:3]); '
            'old=binary.exists() and source.stat().st_mtime_ns <= binary.stat().st_mtime_ns; '
            'None if old else binary.write_bytes(source.read_bytes()); '
            'binary.chmod(0o755); '
            'binary.with_suffix(".d").write_text(str(binary) + ": " + str(source) + "\\n"); '
            f'print(json.dumps({{"reason":"compiler-artifact","package_id":{f"path+file://{self.tree}#probe@0.1.0"!r},"executable":str(binary),"fresh":False}}))'
        )
        request = self.slot / "request.json"
        request.write_text(json.dumps({
            "slot": str(self.slot), "worktree": str(self.tree), "owner": "test-owner",
            "unit": "pair", "receipt": str(self.slot / "pair.json"),
            "cargo": [sys.executable, "-c", compiler, str(self.tree / "src/main.rs"),
                      str(binary), "--message-format=json"]}))
        self.assertEqual(self.module.execute_build(request), 0)
        first = json.loads((self.slot / "pair.json").read_text())
        source = self.tree / "src/main.rs"
        source.write_text('fn main() { println!("B"); }')
        self.git("commit", "-qam", "older B")
        os.utime(source, (1, 1))
        self.assertEqual(self.module.execute_build(request), 0)
        second = json.loads((self.slot / "pair.json").read_text())
        self.assertNotEqual(first["artifacts"], second["artifacts"])
        self.assertEqual(binary.read_bytes(), source.read_bytes())

    def test_worker_records_real_artifact_bytes_and_propagates_failure(self):
        import hashlib
        import sys
        binary = self.slot / "debug/probe"
        binary.parent.mkdir()
        binary.write_bytes(b"A binary")
        binary.with_suffix(".d").write_text(f"{binary}: {self.tree / 'src/main.rs'}\n")
        (self.slot / ".owner").write_text("test-owner")
        emitter = ('import json,sys,os; from pathlib import Path; '
                   'Path(sys.argv[1]).chmod(0o755); os.utime(sys.argv[1], None); '
                   'print(json.dumps({"reason":"compiler-artifact",'
                   f'"package_id":{f"path+file://{self.tree}#probe@0.1.0"!r},'
                   '"executable":sys.argv[1],"fresh":False})); sys.exit(int(sys.argv[2]))')
        request = self.slot / "request.json"
        for code in [0, 7]:
            request.write_text(json.dumps({
                "slot": str(self.slot), "worktree": str(self.tree),
                "owner": "test-owner", "unit": "test-unit",
                "cargo": [sys.executable, "-c", emitter, str(binary), str(code)],
                "receipt": str(self.slot / "receipt.json")}))
            self.assertEqual(self.module.execute_build(request), code)
            receipt = json.loads((self.slot / "receipt.json").read_text())
            self.assertEqual(receipt["artifacts"], {
                "debug/probe": hashlib.sha256(b"A binary").hexdigest()})
            self.assertEqual(receipt["returncode"], code)

    def test_capturing_wrappers_record_hashes_and_reject_identical_binary(self):
        import hashlib
        import sys
        binary = self.slot / "debug/probe"
        binary.parent.mkdir()
        (self.slot / ".owner").write_text("test-owner")
        # This process redirects its own streams, like cargo-exec.py; no Cargo
        # messages are visible to the worker. Its argv must remain untouched.
        wrapper = self.slot / "wrapper.py"
        wrapper.write_text(
            'import os,sys; from pathlib import Path\n'
            'assert len(sys.argv) == 2, sys.argv\n'
            'os.dup2(os.open(sys.argv[1], os.O_WRONLY|os.O_CREAT, 0o600), 1)\n'
            f'p=Path({str(binary)!r}); p.write_bytes(b"binary"); p.chmod(0o755)\n'
            'print("captured cargo output")\n')
        request = self.slot / "request.json"
        receipt = self.slot / "receipt.json"
        request.write_text(json.dumps({
            "slot": str(self.slot), "worktree": str(self.tree), "owner": "test-owner",
            "unit": "wrapped", "receipt": str(receipt),
            "cargo": [sys.executable, str(wrapper), str(self.slot / "compile.stdout")]}))
        with self.assertRaisesRegex(ValueError, "no Cargo compiler-artifact"):
            self.module.execute_build(request)
        self.assertEqual(json.loads(receipt.read_text())["artifacts"], {
            "debug/probe": hashlib.sha256(b"binary").hexdigest()})
        self.assertEqual((self.slot / "compile.stdout").read_text(),
                         "captured cargo output\n")
        (self.tree / "src/main.rs").write_text("changed Rust")
        with self.assertRaisesRegex(ValueError, "stale artifacts"):
            self.module.execute_build(request)

    def run_mixed_output_pair(self, changed_input, changed_outputs, fresh=False,
                              reported_names=None, dep_inputs=None, second_dep_inputs=None,
                              before_second=None):
        import sys
        (self.slot / ".owner").write_text("test-owner")
        emitter = (
            'import json,sys; from pathlib import Path\n'
            'slot=Path(sys.argv[1])\n'
            'for name,content in json.loads(sys.argv[2]).items():\n'
            '    path=slot/name\n'
            '    path.parent.mkdir(parents=True, exist_ok=True)\n'
            '    path.write_text(content)\n'
            '    if path.suffix != ".rmeta": path.chmod(0o755)\n'
            'for name,inputs in json.loads(sys.argv[6]).items():\n'
            '    path=(slot/name).with_suffix(".d")\n'
            '    if inputs is None:\n'
            '        path.unlink(missing_ok=True)\n'
            '    else:\n'
            '        path.parent.mkdir(parents=True,exist_ok=True)\n'
            '        path.write_text(str(slot/name) + ": " + " ".join(inputs) + "\\n")\n'
            'for name in json.loads(sys.argv[5]):\n'
            '    print(json.dumps({"reason":"compiler-artifact", "package_id":sys.argv[3],\n'
            '        "executable":str(slot/name) if not name.endswith(".rmeta") else None,\n'
            '        "filenames":[str(slot/name)],\n'
            '        "fresh":name in json.loads(sys.argv[4])}))\n'
        )
        request = self.slot / "request.json"
        receipt = self.slot / "receipt.json"
        metadata = json.loads(subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
            cwd=self.tree))
        package_id = metadata["packages"][0]["id"]
        outputs = {"debug/probe": "A binary", "debug/other": "A other",
                   "debug/deps/probe.rmeta": "A metadata"}
        dep_inputs = {**{name: [str(self.tree / "src/main.rs")] for name in outputs},
                      **(dep_inputs or {})}
        for phase in ["A", "B"]:
            if phase == "B":
                if changed_input:
                    comment = "//" if changed_input.endswith(".rs") else "#"
                    with (self.tree / changed_input).open("a") as stream:
                        stream.write(f"\n{comment} changed build input\n")
                outputs = changed_outputs
                if before_second:
                    before_second()
            request.write_text(json.dumps({
                "slot": str(self.slot), "worktree": str(self.tree),
                "owner": "test-owner", "unit": "mixed-output", "receipt": str(receipt),
                "cargo": [sys.executable, "-c", emitter, str(self.slot),
                          json.dumps(outputs), package_id,
                          json.dumps((list(dep_inputs) if fresh is True else fresh or [])
                                     if phase == "B" else []),
                          json.dumps(reported_names if phase == "B" and reported_names
                                     is not None else list(outputs)),
                          json.dumps(second_dep_inputs if phase == "B" and
                                     second_dep_inputs is not None else dep_inputs)]}))
            result = subprocess.run(
                [sys.executable, str(self.module.__file__), "--execute", str(request)],
                capture_output=True)
            if phase == "A":
                self.assertEqual(result.returncode, 0, result.stderr)
                first = json.loads(receipt.read_text())
        return result, first, json.loads(receipt.read_text())

    def test_same_package_test_edit_does_not_judge_fresh_bin(self):
        (self.tree / "tests").mkdir()
        (self.tree / "tests/x.rs").write_text("#[test] fn x() {}")
        self.git("add", ".")
        self.git("commit", "-qm", "test target")
        result, first, second = self.run_mixed_output_pair(
            "tests/x.rs", {"debug/other": "B test"}, fresh=["debug/probe"],
            reported_names=["debug/probe", "debug/other"],
            dep_inputs={"debug/probe": [str(self.tree / "src/main.rs")],
                        "debug/other": [str(self.tree / "tests/x.rs")]})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(first["artifacts"]["debug/probe"],
                         second["artifacts"]["debug/probe"])

    def test_reported_affected_target_requires_a_current_hash(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/deps/probe.rmeta": "B metadata"},
            reported_names=["debug/probe"],
            before_second=lambda: (self.slot / "debug/probe").unlink())
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_missing_dep_info_fails_closed(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary"},
            reported_names=["debug/probe"],
            second_dep_inputs={"debug/probe": None})
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"dep-info", result.stderr)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_missing_previous_dep_info_accepts_rebuilt_child_with_new_dep_info(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary"},
            reported_names=["debug/probe"],
            before_second=lambda: (self.slot / "debug/probe.d").unlink())
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(second["result"], "success")
        self.assertNotEqual(first["artifacts"]["debug/probe"],
                            second["artifacts"]["debug/probe"])

    def test_missing_previous_dep_info_still_rejects_fresh_child(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary"}, fresh=True,
            reported_names=["debug/probe"],
            before_second=lambda: (self.slot / "debug/probe.d").unlink())
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def run_failed_build_retry(self, remove_dep_info=False, commit_source=False):
        import sys
        (self.slot / ".owner").write_text("test-owner")
        binary = self.slot / "debug/probe"
        source = self.tree / "src/main.rs"
        request = self.slot / "request.json"
        receipt = self.slot / "receipt.json"
        emitter = (
            'import json,sys; from pathlib import Path\n'
            'binary,source=map(Path,sys.argv[1:3])\n'
            'phase=sys.argv[3]\n'
            'if phase == "A":\n'
            '    binary.parent.mkdir(parents=True)\n'
            '    binary.write_bytes(b"A binary")\n'
            '    binary.chmod(0o755)\n'
            '    binary.with_suffix(".d").write_text(f"{binary}: {source}\\n")\n'
            'if phase == "failed": sys.exit(7)\n'
            f'print(json.dumps({{"reason":"compiler-artifact", "package_id":{f"path+file://{self.tree}#probe@0.1.0"!r},\n'
            '    "executable":str(binary), "fresh":phase == "missing"}))\n'
        )
        results, receipts = [], []
        for phase in ("A", "failed", "missing" if remove_dep_info else "retry"):
            if phase == "failed":
                source.write_text('fn main() { println!("B"); }')
                if commit_source:
                    self.git("commit", "-qam", "B")
            if phase == "missing":
                binary.with_suffix(".d").unlink()
            request.write_text(json.dumps({
                "slot": str(self.slot), "worktree": str(self.tree),
                "owner": "test-owner", "unit": phase, "receipt": str(receipt),
                "cargo": [sys.executable, "-B", "-c", emitter,
                          str(binary), str(source), phase]}))
            results.append(subprocess.run(
                [sys.executable, "-B", str(self.module.__file__), "--execute", str(request)],
                capture_output=True))
            receipts.append(json.loads(receipt.read_text()))
            verified = json.loads((self.slot / ".verified-source-identity.json").read_text())
            self.assertEqual(self.module.summarize_receipt(verified), receipts[0])
        self.assertEqual(results[0].returncode, 0, results[0].stderr)
        self.assertEqual(results[1].returncode, 7, results[1].stderr)
        self.assertEqual(receipts[0]["artifacts"], receipts[2]["artifacts"])
        self.assertNotEqual(receipts[0]["identity"], receipts[2]["identity"])
        return results[2], receipts

    def test_failed_build_retry_rejects_unchanged_binary(self):
        result, receipts = self.run_failed_build_retry(commit_source=True)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(b"stale artifacts", result.stderr)
        self.assertIn("src/main.rs", receipts[2]["content_changed_paths"])
        verified = json.loads((self.slot / ".verified-source-identity.json").read_text())
        self.assertEqual(self.module.summarize_receipt(verified), receipts[0])
        attempted = json.loads((self.slot / ".source-identity.json").read_text())
        self.assertEqual(self.module.summarize_receipt(attempted), receipts[2])

    def test_failed_build_retry_rejects_missing_current_dep_info(self):
        result, receipts = self.run_failed_build_retry(remove_dep_info=True)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(b"missing dep-info", result.stderr)
        self.assertIn("src/main.rs", receipts[2]["content_changed_paths"])

    def test_rejected_build_retry_does_not_advance_verified_content(self):
        result, first, stale = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "A binary"},
            reported_names=["debug/probe"])
        self.assertNotEqual(result.returncode, 0)
        import sys
        result = subprocess.run(
            [sys.executable, "-B", str(self.module.__file__), "--execute",
             str(self.slot / "request.json")], capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"stale artifacts", result.stderr)
        verified = json.loads((self.slot / ".verified-source-identity.json").read_text())
        self.assertEqual(self.module.summarize_receipt(verified), first)
        self.assertNotEqual(verified["identity"], stale["identity"])

    def test_stale_receipt_recovers_with_identical_rebuilt_bytes(self):
        def mark_interrupted_attempt():
            state = self.slot / ".source-identity.json"
            previous = json.loads(state.read_text())
            previous["result"] = "building"
            state.write_text(json.dumps(previous))

        result, first, stale = self.run_mixed_output_pair(
            None, {"debug/probe": "A binary"}, fresh=True,
            reported_names=["debug/probe"], before_second=mark_interrupted_attempt)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(stale["result"], "stale-artifacts")
        self.assertEqual(stale["identity"], first["identity"])
        self.assertEqual(stale["content_changed_paths"], [])
        self.assertIn("src/main.rs", stale["changed_paths"])
        import sys
        request = self.slot / "request.json"
        command = json.loads(request.read_text())
        command["cargo"][6] = "[]"  # Rebuild rather than report fresh=true.
        request.write_text(json.dumps(command))
        for attempt in range(2):
            with self.subTest(attempt=attempt):
                result = subprocess.run(
                    [sys.executable, "-B", str(self.module.__file__), "--execute",
                     str(self.slot / "request.json")], capture_output=True)
                saved = json.loads((self.slot / "receipt.json").read_text())
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(saved["result"], "success")
                self.assertEqual(saved["identity"], stale["identity"])
                self.assertEqual(saved["artifacts"]["debug/probe"],
                                 stale["artifacts"]["debug/probe"])

    def test_stale_receipt_with_new_content_still_requires_changed_bytes(self):
        previous = self.prepare()
        previous.update(result="stale-artifacts", artifacts={"debug/probe": "same"})
        (self.tree / "src/main.rs").write_text("new content after stale result")
        current = self.prepare(previous)
        self.add_reported_executable(current)
        with self.assertRaisesRegex(ValueError, "stale artifacts"):
            self.module.finish_build(self.slot, self.slot / "receipt.json", current,
                                     previous, {"debug/probe": "same"}, 0)

    def test_unreadable_previous_dep_info_fails_even_when_child_replaces_it(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary"},
            reported_names=["debug/probe"],
            before_second=lambda: (self.slot / "debug/probe.d").write_bytes(bytes([255])))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"unreadable dep-info", result.stderr)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_previous_dep_info_cannot_be_erased_by_child(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary"}, fresh=True,
            reported_names=["debug/probe"],
            second_dep_inputs={"debug/probe": [str(self.tree / "unrelated.rs")]})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_new_dep_info_is_unioned_with_previous_inputs(self):
        (self.tree / "unrelated.rs").write_text("pub fn unrelated() {}")
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary"}, fresh=True,
            reported_names=["debug/probe"],
            dep_inputs={"debug/probe": [str(self.tree / "unrelated.rs")]},
            second_dep_inputs={"debug/probe": [str(self.tree / "src/main.rs")]})
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_deps_dep_info_handles_relative_escaped_paths_and_continuations(self):
        source = self.tree / "src/space #$.rs"
        source.write_text("pub fn input() {}")
        binary = self.slot / "debug/deps/probe-012345"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"binary")
        dep_info = binary.with_suffix(".d")
        dep_info.write_text(
            f"{binary}: src/main.rs \\\n src/space\\ #$$.rs\n\n"
            "src/main.rs:\n\n# env-dep:EXAMPLE=value\n")
        current = {"worktree": str(self.tree), "slot": str(self.slot),
                   "package_roots": {"probe": [str(self.tree)]},
                   "changed_paths": ["src/space #$.rs"],
                   "content_changed_paths": ["src/space #$.rs"],
                   "reported_artifacts": [{"package_id": "probe", "fresh": True,
                                           "executable": str(binary)}]}
        snapshot = self.module.read_dep_info_snapshot(self.slot, self.tree)
        current["target_inputs"], errors = self.module.read_target_inputs(
            current, snapshot, snapshot)
        self.assertEqual(errors, [])
        self.assertIn("src/space #$.rs", current["target_inputs"][str(binary)])
        self.assertEqual(self.module.find_stale_artifacts(current, {}, {}),
                         ["debug/deps/probe-012345"])

    def test_fresh_report_rejects_even_changed_executable_bytes(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary", "debug/other": "B other"},
            fresh=True)
        self.assertNotEqual(first["artifacts"]["debug/probe"],
                            second["artifacts"]["debug/probe"])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_unreported_same_package_executable_is_not_judged(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/other": "B other"})
        self.assertEqual(first["artifacts"]["debug/probe"],
                         second["artifacts"]["debug/probe"])
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_library_record_does_not_require_unreported_executables(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/deps/probe.rmeta": "B metadata"})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(second["result"], "success")

    def test_no_reported_artifact_fails_for_changed_package_inputs(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary", "debug/other": "B other"},
            reported_names=[])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"no Cargo compiler-artifact records", result.stderr)

    def test_changed_metadata_cannot_certify_unchanged_executable(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/deps/probe.rmeta": "B metadata"},
            reported_names=["debug/probe", "debug/deps/probe.rmeta"])
        self.assertEqual(second["changed_paths"], ["src/main.rs"])
        self.assertEqual(first["artifacts"]["debug/probe"],
                         second["artifacts"]["debug/probe"])
        self.assertNotEqual(first["artifacts"]["debug/deps/probe.rmeta"],
                            second["artifacts"]["debug/deps/probe.rmeta"])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"stale artifacts", result.stderr)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_changed_executable_cannot_certify_another_unchanged_executable(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary"},
            reported_names=["debug/probe", "debug/other"])
        self.assertNotEqual(first["artifacts"]["debug/probe"],
                            second["artifacts"]["debug/probe"])
        self.assertEqual(first["artifacts"]["debug/other"],
                         second["artifacts"]["debug/other"])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_build_input_change_rejects_unchanged_executable(self):
        result, first, second = self.run_mixed_output_pair(
            "Cargo.toml", {"debug/deps/probe.rmeta": "B metadata"},
            reported_names=["debug/probe", "debug/deps/probe.rmeta"])
        self.assertEqual(second["changed_paths"], ["Cargo.toml"])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_build_script_is_an_input_even_when_absent_from_dep_info(self):
        (self.tree / "build.rs").write_text("fn main() {}")
        self.git("add", ".")
        self.git("commit", "-qm", "build script")
        result, first, second = self.run_mixed_output_pair(
            "build.rs", {"debug/deps/probe.rmeta": "B metadata"},
            reported_names=["debug/probe"])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_lockfile_is_an_input_even_when_absent_from_dep_info(self):
        (self.tree / "Cargo.lock").write_text(
            'version = 4\n[[package]]\nname = "probe"\nversion = "0.1.0"\n')
        self.git("add", ".")
        self.git("commit", "-qm", "lockfile")
        result, first, second = self.run_mixed_output_pair(
            "Cargo.lock", {"debug/deps/probe.rmeta": "B metadata"},
            reported_names=["debug/probe"])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(second["result"], "stale-artifacts")

    def test_changed_executables_allow_unchanged_library_metadata(self):
        result, first, second = self.run_mixed_output_pair(
            "src/main.rs", {"debug/probe": "B binary", "debug/other": "B other"})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(first["artifacts"]["debug/deps/probe.rmeta"],
                         second["artifacts"]["debug/deps/probe.rmeta"])
        self.assertEqual(second["result"], "success")

    def run_test_only_pair(self, report_application):
        import hashlib
        import sys
        (self.slot / ".owner").write_text("test-owner")
        (self.tree / "app").mkdir()
        self.git("mv", "src", "app/src")
        self.git("mv", "Cargo.toml", "app/Cargo.toml")
        (self.tree / "Cargo.toml").write_text(
            '[workspace]\nmembers=["app", "tests"]\nresolver="2"\n')
        (self.tree / "tests").mkdir()
        source = self.tree / "tests/foo.rs"
        source.write_text("#[test] fn foo() {}")
        (self.tree / "tests/Cargo.toml").write_text(
            '[package]\nname="tests"\nversion="0.1.0"\n'
            '[[test]]\nname="foo"\npath="foo.rs"\n')
        with (self.tree / "Cargo.lock").open("a") as lock:
            lock.write('[[package]]\nname = "tests"\nversion = "0.1.0"\n')
        self.git("add", ".")
        self.git("commit", "-qm", "test A")
        metadata = json.loads(subprocess.check_output(
            ["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
            cwd=self.tree))
        packages = {p["name"]: p["id"] for p in metadata["packages"]}
        emitter = (
            'import json,sys; from pathlib import Path\n'
            'slot,phase,report,app_id,test_id=sys.argv[1:]\n'
            'binary=Path(slot)/"debug/wow-sim"\n'
            'test_binary=Path(slot)/"debug/deps/foo-test"\n'
            'if phase == "A":\n'
            '    binary.parent.mkdir(parents=True,exist_ok=True)\n'
            '    binary.write_bytes(b"unchanged application executable")\n'
            '    binary.chmod(0o755)\n'
            'test_binary.parent.mkdir(parents=True,exist_ok=True)\n'
            'test_binary.write_bytes(("test executable " + phase).encode())\n'
            'test_binary.chmod(0o755)\n'
            f'binary.with_suffix(".d").write_text(str(binary) + ": " + '
            f'{str(self.tree / "app/src/main.rs")!r} + "\\n")\n'
            f'test_binary.with_suffix(".d").write_text(str(test_binary) + ": " + '
            f'{str(source)!r} + "\\n")\n'
            'if phase == "A" or report == "yes":\n'
            '    print(json.dumps({"reason":"compiler-artifact", "package_id":app_id,\n'
            '        "executable":str(binary), "fresh":phase == "B"}))\n'
            'print(json.dumps({"reason":"compiler-artifact", "package_id":test_id,\n'
            '    "executable":str(test_binary), "fresh":False}))\n'
        )
        request = self.slot / "request.json"
        receipt = self.slot / "receipt.json"
        binary = self.slot / "debug/wow-sim"
        for phase in ["A", "B"]:
            if phase == "B":
                before = binary.stat()
                digest = hashlib.sha256(binary.read_bytes()).hexdigest()
                source.write_text("#[test] fn foo() { assert_eq!(2, 2); }")
                self.git("commit", "-qam", "test B only")
            request.write_text(json.dumps({
                "slot": str(self.slot), "worktree": str(self.tree),
                "owner": "test-owner", "unit": "test-only", "receipt": str(receipt),
                "cargo": [sys.executable, "-c", emitter, str(self.slot), phase,
                          "yes" if report_application else "no", packages["probe"],
                          packages["tests"]]}))
            result = subprocess.run([sys.executable, "-B", str(self.module.__file__),
                                     "--execute", str(request)], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
        saved = json.loads(receipt.read_text())
        self.assertEqual(saved["changed_paths"], ["tests/foo.rs"])
        self.assertEqual(saved["result"], "success")
        self.assertEqual(binary.stat().st_mtime_ns, before.st_mtime_ns)
        self.assertEqual(binary.stat().st_ctime_ns, before.st_ctime_ns)
        self.assertEqual(hashlib.sha256(binary.read_bytes()).hexdigest(), digest)

    def test_test_only_diff_ignores_unreported_application(self):
        self.run_test_only_pair(False)

    def test_test_only_diff_ignores_fresh_disjoint_application(self):
        self.run_test_only_pair(True)

    def test_worker_preserves_binary_streams_and_launcher_arguments(self):
        import sys
        (self.slot / ".owner").write_text("test-owner")
        verified = dict(self.prepare(), result="success", artifacts={})
        self.module.write_json(self.slot / ".verified-source-identity.json", verified)
        request = self.slot / "request.json"
        request.write_text(json.dumps({
            "slot": str(self.slot), "worktree": str(self.tree), "owner": "test-owner",
            "unit": "streams", "receipt": str(self.slot / "receipt.json"),
            "cargo": [sys.executable, "-c",
                      'import os,sys; assert len(sys.argv)==1; '
                      'os.write(1,b"stdout\\xff\\x00"); os.write(2,b"stderr\\xfe\\x00")']}))
        result = subprocess.run([sys.executable, str(self.module.__file__),
                                 "--execute", str(request)], capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, b"stdout\xff\x00")
        pruning_line, child_stderr = result.stderr.split(b"\n", 1)
        pruning = json.loads(pruning_line)
        self.assertEqual(pruning["slot"], str(self.slot))
        self.assertFalse(pruning["dry_run"])
        self.assertEqual(pruning["removed"], [])
        self.assertEqual(child_stderr, b"stderr\xfe\x00")

    def test_hidden_stream_allows_diff_outside_package_roots(self):
        current = self.prepare()
        self.add_reported_executable(current)
        current.update(changed_paths=["README.md"], reported_artifacts=[],
                       package_roots={"probe": [str(self.tree / "app")]})
        receipt = self.slot / "receipt.json"
        self.module.finish_build(self.slot, receipt, current, None, {}, 0)
        self.assertEqual(json.loads(receipt.read_text())["result"], "success")

    def test_changed_artifacts_and_identity_are_recorded(self):
        previous = self.prepare()
        previous["artifacts"] = {"debug/probe": "abc"}
        (self.tree / "src/main.rs").write_text("changed")
        current = self.prepare(previous)
        receipt = self.slot / "receipt.json"
        self.add_reported_executable(current)
        self.module.finish_build(self.slot, receipt, current, previous,
                                 {"debug/probe": "def"}, 0)
        saved = json.loads(receipt.read_text())
        self.assertEqual(saved["artifacts"], {"debug/probe": "def"})
        self.assertEqual(saved["identity"], current["identity"])



class LockClosureTransitionTests(unittest.TestCase):
    setUp = SourceFreshnessTests.setUp
    git = SourceFreshnessTests.git
    # The changed leaf is two edges away from probe. Unrelated is outside
    # probe's closure; switching the root dependency exercises both cases.
    def lock_text(self, version, depends):
        return (f'version = 3\n\n[[package]]\nname = "probe"\nversion = "0.1.0"\n'
                f'dependencies = ["{depends}"]\n\n'
                '[[package]]\nname = "stable"\nversion = "1.0.0"\n\n'
                '[[package]]\nname = "middle"\nversion = "1.0.0"\n'
                'dependencies = ["leaf"]\n\n'
                f'[[package]]\nname = "leaf"\nversion = "{version}"\n'
                'source = "registry+https://github.com/rust-lang/crates.io-index"\n')

    def setup_transition(self, depends="stable"):
        self.package_id = f"path+file://{self.tree}#probe@0.1.0"
        self.output = self.slot / "debug/deps/libprobe-123.rlib"
        self.fingerprint = self.slot / "debug/.fingerprint/probe-123/lib-probe"
        self.output.parent.mkdir(parents=True)
        self.output.write_bytes(b"compiled artifact")
        self.output.with_name("probe-123.d").write_text(
            f"{self.output}: {self.tree}/src/main.rs\n")
        self.fingerprint.parent.mkdir(parents=True)
        self.fingerprint.write_text("fingerprint")
        self.lock_old = self.lock_text("1.0.0", depends)
        (self.tree / "Cargo.lock").write_text(self.lock_old)
        self.git("add", ".")
        self.git("commit", "-qm", "old lock")
        self.old_revision = self.git("rev-parse", "HEAD").decode().strip()
        result, receipt = self.execute(False)
        self.assertIsNone(result, result)
        self.assertEqual(receipt["result"], "success")
        return receipt

    def execute(self, fresh):
        (self.slot / ".owner").write_text("test-owner")
        request = self.slot / "request.json"
        receipt = self.slot / "receipt.json"
        request.write_text(json.dumps({"slot": str(self.slot), "worktree": str(self.tree),
            "owner": "test-owner", "unit": "fixture", "receipt": str(receipt),
            "cargo": ["fake-compiler"]}))
        record = {"reason": "compiler-artifact", "package_id": self.package_id,
                  "manifest_path": str(self.tree / "Cargo.toml"),
                  "target": {"name": "probe", "kind": ["lib"]},
                  "filenames": [str(self.output)], "executable": None, "fresh": fresh}
        error = None
        with patch.object(self.module, "read_package_roots", return_value={
                self.package_id: [str(self.tree)]}), \
             patch.object(self.module, "run_build_command", return_value=(0, [record])):
            try:
                self.module.execute_build(request)
            except ValueError as exc:
                error = str(exc)
        return error, json.loads(receipt.read_text())

    def change_lock(self, depends="stable"):
        (self.tree / "Cargo.lock").write_text(self.lock_text("2.0.0", depends))
        self.git("add", "Cargo.lock")
        self.git("commit", "-qm", "change leaf")

    def make_legacy(self, revision=None, mismatch=False):
        import hashlib
        path = self.slot / ".verified-source-identity.json"
        previous = json.loads(path.read_text())
        certificate = previous["artifact_certificates"][str(self.output.relative_to(self.slot))]
        # Strip any new lock-closure representation to model a persisted old receipt.
        certificate = {"inputs": dict(certificate["inputs"]), "sha256": certificate["sha256"]}
        certificate["inputs"]["Cargo.lock"] = hashlib.sha256(
            b"different lock" if mismatch else self.lock_old.encode()).hexdigest()
        previous["artifact_certificates"] = {str(self.output.relative_to(self.slot)): certificate}
        if revision is not None:
            previous["identity"]["head"] = revision
        path.write_text(json.dumps(previous))

    def test_uncertified_fresh_workspace_libraries_evict_then_rebuild(self):
        names = ("sim-xml", "sim-lua-bridge")
        records = []
        for name in names:
            root = self.tree / "crates" / name
            (root / "src").mkdir(parents=True)
            (root / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="0.1.0"\n')
            (root / "src/lib.rs").write_text(f'// {name}\n')
            target = name.replace("-", "_")
            output = self.slot / f"debug/deps/lib{target}-abc123.rlib"
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_bytes(name.encode())
            output.with_name(f"{target}-abc123.d").write_text(f"{output}: {root}/src/lib.rs\n")
            fingerprint = self.slot / f"debug/.fingerprint/{name}-abc123/stamp"
            fingerprint.parent.mkdir(parents=True)
            fingerprint.write_text("fingerprint")
            records.append({"reason": "compiler-artifact", "package_id": f"path+file://{root}#0.1.0",
                            "manifest_path": str(root / "Cargo.toml"),
                            "target": {"name": target, "kind": ["lib"]},
                            "filenames": [str(output)], "executable": None, "fresh": True})
        (self.tree / "Cargo.lock").write_text('version = 3\n' + ''.join(
            f'\n[[package]]\nname="{name}"\nversion="0.1.0"\n' for name in names))
        self.git("add", ".")
        self.git("commit", "-qm", "workspace fixture")
        other = self.slot / "debug/deps/libother-456.rlib"
        other.write_bytes(b"keep")
        other_fingerprint = self.slot / "debug/.fingerprint/other-456/stamp"
        other_fingerprint.parent.mkdir(parents=True)
        other_fingerprint.write_text("keep")
        (self.slot / ".owner").write_text("test-owner")
        request = self.slot / "request.json"
        receipt_path = self.slot / "receipt.json"
        request.write_text(json.dumps({"slot": str(self.slot), "worktree": str(self.tree),
            "owner": "test-owner", "unit": "fixture", "receipt": str(receipt_path),
            "cargo": ["fake-compiler"]}))
        roots = {record["package_id"]: [str(Path(record["manifest_path"]).parent)] for record in records}

        def execute():
            with patch.object(self.module, "read_package_roots", return_value=roots), \
                 patch.object(self.module, "run_build_command", return_value=(0, records)):
                try:
                    self.module.execute_build(request)
                except ValueError as error:
                    return str(error), json.loads(receipt_path.read_text())
            return None, json.loads(receipt_path.read_text())

        error, receipt = execute()
        self.assertIsNotNone(error)
        self.assertEqual(receipt["result"], "stale-artifacts")
        self.assertEqual(len([e for e in receipt["dep_info_errors"] if "uncertified fresh artifact" in e]), 2)
        expected = {f"debug/deps/lib{name.replace('-', '_')}-abc123.rlib" for name in names}
        expected |= {f"debug/.fingerprint/{name}-abc123" for name in names}
        self.assertEqual(set(receipt["evicted_artifacts"]), expected)
        for record in records:
            self.assertFalse(Path(record["filenames"][0]).exists())
        self.assertTrue(other.exists())
        self.assertTrue(other_fingerprint.exists())

        for record in records:
            Path(record["filenames"][0]).write_bytes(b"rebuilt")
            record["fresh"] = False
        error, receipt = execute()
        self.assertIsNone(error, error)
        self.assertEqual(receipt["result"], "success")
        for record in records:
            self.assertIn(str(Path(record["filenames"][0]).relative_to(self.slot)),
                          full_identity(self.slot)["artifact_certificates"])
            record["fresh"] = True
        error, receipt = execute()
        self.assertIsNone(error, error)
        self.assertEqual(receipt["result"], "success")
        self.assertTrue(all(Path(record["filenames"][0]).exists() for record in records))
        self.assertTrue(other.exists())
        self.assertTrue(other_fingerprint.exists())

    def test_path_version_only_transition_accepts_and_evicts(self):
        self.setup_transition()
        self.package_id = f"path+file://{self.tree}/probe#0.1.0"
        # Create a certificate with the actual Cargo path-ID form.
        error, _ = self.execute(False)
        self.assertIsNone(error, error)
        self.change_lock()
        error, _ = self.execute(True)
        self.assertIsNone(error, error)
        (self.tree / "Cargo.lock").write_text(self.lock_text("2.0.0", "middle"))
        self.git("add", "Cargo.lock")
        self.git("commit", "-qm", "change closure")
        error, _ = self.execute(True)
        self.assertIsNotNone(error)
        self.assertFalse(self.output.exists())
        self.assertFalse(self.fingerprint.exists())

    def test_duplicate_name_version_resolves_source(self):
        lock = b'''version = 3
[[package]]
name = "probe"
version = "0.1.0"
dependencies = ["storage 0.2.0 (git+https://example.org/repo?rev=abc)"]
[[package]]
name = "storage"
version = "0.2.0"
source = "registry+https://example.org/index"
[[package]]
name = "storage"
version = "0.2.0"
source = "git+https://example.org/repo?rev=abc#abc"
'''
        digest = self.module.lock_closure_digest(lock, "path+file:///tmp/probe#probe@0.1.0")
        changed_other = lock.replace(b'example.org/index', b'other.org/index')
        self.assertEqual(digest, self.module.lock_closure_digest(changed_other, "path+file:///tmp/probe#probe@0.1.0"))

    def test_checksum_and_dependency_edges_change_digest(self):
        lock = b'''version = 3
[[package]]
name = "probe"
version = "0.1.0"
dependencies = ["a", "b"]
[[package]]
name = "a"
version = "1.0.0"
checksum = "one"
dependencies = ["b"]
[[package]]
name = "b"
version = "1.0.0"
'''
        package = "path+file:///tmp/probe#probe@0.1.0"
        digest = self.module.lock_closure_digest(lock, package)
        self.assertNotEqual(digest, self.module.lock_closure_digest(lock.replace(b'"one"', b'"two"'), package))
        self.assertNotEqual(digest, self.module.lock_closure_digest(lock.replace(b'dependencies = ["b"]', b'dependencies = []'), package))

    def test_build_script_parent_fingerprint_is_evicted(self):
        self.setup_transition("middle")
        self.output.unlink()
        self.fingerprint.unlink()
        self.output = self.slot / "debug/build/probe-abc123/build-script-build"
        self.output.parent.mkdir(parents=True)
        self.output.write_bytes(b"build script")
        self.output.with_suffix(".d").write_text(f"{self.output}: {self.tree}/src/main.rs\n")
        self.fingerprint = self.slot / "debug/.fingerprint/probe-abc123/build-script-build-script-build"
        self.fingerprint.parent.mkdir(parents=True)
        self.fingerprint.write_text("fingerprint")
        error, _ = self.execute(False)
        self.assertIsNone(error, error)
        self.change_lock("middle")
        error, _ = self.execute(True)
        self.assertIsNotNone(error)
        self.assertFalse(self.output.exists())
        self.assertFalse(self.fingerprint.exists())

    def test_unidentified_fingerprint_fails_without_deleting_unrelated(self):
        self.setup_transition("middle")
        self.fingerprint.unlink()
        self.fingerprint.parent.rmdir()
        other = self.slot / "debug/.fingerprint/other-456/stamp"
        other.parent.mkdir(parents=True)
        other.write_text("keep")
        self.change_lock("middle")
        error, _ = self.execute(True)
        self.assertFalse(self.output.exists())
        self.assertTrue(other.exists())
        self.assertIn(str(self.output), error)
        self.assertIn("fingerprint", error)
        self.assertIn("could not be identified", error)
        self.execute(False)
        self.assertTrue(other.exists())

    def test_uplifted_executable_evicts_hashed_source_and_fingerprint_only(self):
        receipt = self.setup_transition("middle")
        uplift = self.slot / "debug/probe"
        hashed = self.slot / "debug/deps/probe-abc123"
        uplift.write_bytes(b"binary")
        hashed.write_bytes(b"binary")
        fingerprint = self.slot / "debug/.fingerprint/probe-abc123/stamp"
        fingerprint.parent.mkdir(parents=True)
        fingerprint.write_text("fingerprint")
        other = self.slot / "debug/.fingerprint/other-456/stamp"
        other.parent.mkdir(parents=True)
        other.write_text("keep")
        receipt["reported_artifacts"] = [{"package_id": self.package_id,
            "target": {"name": "probe", "kind": ["bin"]},
            "executable": str(uplift), "filenames": [str(uplift)]}]
        receipt["rejected_artifacts"] = [str(uplift)]
        receipt["dep_info_errors"] = ["rejected fixture"]
        with self.assertRaises(ValueError):
            self.module.finish_build(self.slot, self.slot / "receipt.json", receipt, None, {}, 0)
        self.assertFalse(uplift.exists())
        self.assertFalse(hashed.exists())
        self.assertFalse(fingerprint.exists())
        self.assertTrue(other.exists())
        self.assertTrue(self.output.exists())
        self.assertTrue(self.fingerprint.exists())

    def test_unrelated_lock_change_accepts_certificate_and_stale_check(self):
        self.setup_transition()
        self.change_lock()
        error, receipt = self.execute(True)
        with self.subTest(check="certificate"):
            self.assertFalse(receipt["dep_info_errors"], error)
        with self.subTest(check="stale"):
            self.assertEqual(self.module.find_stale_artifacts(receipt, {}, receipt["artifacts"]), [])
        self.assertTrue(self.output.exists())

    def test_transitive_lock_change_rejects_fresh_artifact(self):
        self.setup_transition("middle")
        self.change_lock("middle")
        error, receipt = self.execute(True)
        self.assertIsNotNone(error)
        self.assertEqual(receipt["result"], "stale-artifacts")

    def test_legacy_certificate_migrates_from_matching_revision(self):
        self.setup_transition()
        self.make_legacy()
        self.change_lock()
        error, receipt = self.execute(True)
        self.assertIsNone(error, error)
        self.assertEqual(receipt["result"], "success")
        self.assertTrue(self.output.exists())

    def test_unverifiable_legacy_certificate_evicts_output_and_fingerprint(self):
        for case in ("missing_revision", "digest_mismatch"):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                # Isolate each persisted receipt and its git history.
                original_slot, original_tree = self.slot, self.tree
                self.slot = Path(directory)
                self.tree = self.slot / "tree"
                self.tree.mkdir()
                self.git("init", "-q")
                self.git("config", "user.email", "test@example.com")
                self.git("config", "user.name", "Test")
                (self.tree / "src").mkdir()
                (self.tree / "src/main.rs").write_text("fn main() {}")
                (self.tree / "Cargo.toml").write_text('[package]\nname="probe"\nversion="0.1.0"\n')
                self.git("add", ".")
                self.git("commit", "-qm", "initial")
                self.setup_transition()
                self.make_legacy(revision="0000000000000000000000000000000000000000"
                                 if case == "missing_revision" else None,
                                 mismatch=case == "digest_mismatch")
                self.change_lock()
                error, receipt = self.execute(True)
                self.assertIsNotNone(error)
                self.assertEqual(receipt["result"], "stale-artifacts")
                with self.subTest(artifact="output"):
                    self.assertFalse(self.output.exists(), "rejected output must be evicted")
                with self.subTest(artifact="fingerprint"):
                    self.assertFalse(self.fingerprint.exists(), "fingerprint must be evicted")
                self.slot, self.tree = original_slot, original_tree

    def test_non_lock_source_change_still_rejects_fresh_artifact(self):
        self.setup_transition()
        (self.tree / "src/main.rs").write_text("fn main() { println!(\"changed\"); }")
        self.git("add", "src/main.rs")
        self.git("commit", "-qm", "change source")
        error, receipt = self.execute(True)
        self.assertIsNotNone(error)
        self.assertEqual(receipt["result"], "stale-artifacts")


if __name__ == "__main__":
    unittest.main()
