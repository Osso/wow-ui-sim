import hashlib
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SOURCE = Path(__file__).resolve().parents[1] / "gen_cvar_info.py"
SPEC = importlib.util.spec_from_file_location("gen_cvar_info", SOURCE)
GENERATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GENERATOR)


class CVarInfoGeneratorTests(unittest.TestCase):
    def test_captured_zero_mask_remains_distinct_from_missing_metadata(self):
        flags = GENERATOR.parse_flags(
            b"zero:\n  default: '1'\n  account: false\nscoped:\n  character: true\n"
        )
        self.assertEqual(flags, {"zero": 0, "scoped": 2})
        self.assertNotIn("missing", flags)

    def test_invalid_and_duplicate_flags_fail(self):
        invalid = (
            b"name:\n  account: yes\n",
            b"name:\n  account: true\n  account: false\n",
            b"name:\nname:\n",
            b"name:\n  unknown: true\n",
            b"Name:\n  secure: true\n",
        )
        for source in invalid:
            with self.subTest(source=source), self.assertRaises(ValueError):
                GENERATOR.parse_flags(source)

    def test_unsupported_indented_flag_syntax_fails(self):
        invalid = (
            b"name:\n    account: true\n",
            b"name:\n  account true\n",
        )
        for source in invalid:
            with self.subTest(source=source), self.assertRaisesRegex(
                ValueError, "unsupported CVar attribute at line 2"
            ):
                GENERATOR.parse_flags(source)

    def test_snapshot_pins_reject_version_or_build_configuration_changes(self):
        config = GENERATOR.PROFILES["wowforever"]
        version = config["snapshot_version"]
        major, build_number = version.rsplit(".", 1)
        build = {"version": major, "build": build_number, "hash": config["snapshot_hash"]}
        self.assertEqual(GENERATOR.validate_snapshot("wowforever", build), "1.60.1.70532")
        for changed in ({**build, "build": "70707"}, {**build, "hash": "wrong"}):
            with self.subTest(build=changed), self.assertRaises(ValueError):
                GENERATOR.validate_snapshot("wowforever", changed)

    def test_rust_output_and_provenance_preserve_snapshot_and_captured_masks(self):
        config = GENERATOR.PROFILES["retail"]
        version = config["snapshot_version"]
        build_hash = config["snapshot_hash"]
        major, build_number = version.rsplit(".", 1)
        build_source = (
            f"version: '{major}'\nbuild: '{build_number}'\nhash: '{build_hash}'\n"
        ).encode()
        flag_source = b"zero:\n  account: false\nscoped:\n  account: true\n"
        license_source = b"MIT license bytes\r\n"

        def source(path, revision, cache_dir):
            if path.endswith("cvars.yaml"):
                return flag_source
            if path.endswith("build.yaml"):
                return build_source
            if path == "LICENSE":
                return license_source
            return b"native capture contract"

        with (
            patch.dict(GENERATOR.PROFILES, {"retail": config}, clear=True),
            patch.object(GENERATOR, "source", side_effect=source),
        ):
            outputs = GENERATOR.generate(None)
        output_path = "crates/sim-data-profile/data/cvar_info_retail.rs"
        rust = outputs[output_path]
        self.assertIn(b"snapshot 12.1.5.70575", rust)
        self.assertIn(GENERATOR.REVISION.encode(), rust)
        self.assertIn(b"data/cvars-info/LICENSE.wowless", rust)
        self.assertIn(b'    "scoped" => 1u8,\n    "zero" => 0u8,\n', rust)
        self.assertNotIn(b'"missing"', rust)
        self.assertNotIn("data/cvars-info/retail.json", outputs)
        self.assertEqual(outputs["data/cvars-info/LICENSE.wowless"], license_source)
        provenance = json.loads(outputs["data/cvars-info/provenance.json"])
        profile = provenance["profiles"]["retail"]
        self.assertEqual(provenance["schema"], 2)
        self.assertEqual(profile["snapshot_version"], "12.1.5.70575")
        self.assertNotIn("target_manifest_version", profile)
        self.assertEqual(profile["output_path"], output_path)
        self.assertEqual(profile["output_sha256"], hashlib.sha256(rust).hexdigest())
        self.assertEqual(profile["source_sha256"], hashlib.sha256(flag_source).hexdigest())
        self.assertEqual(profile["catalog_last_change_revision"], config["last_change_revision"])
        self.assertEqual(profile["entries"], 2)

    def test_check_mode_covers_repo_relative_rust_license_and_provenance_without_writing(self):
        outputs = {
            "crates/sim-data-profile/data/cvar_info_retail.rs": b"rust output",
            "data/cvars-info/LICENSE.wowless": b"license",
            "data/cvars-info/provenance.json": b"provenance",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.object(GENERATOR, "ROOT", root):
                with patch.object(Path, "write_bytes", side_effect=AssertionError("check wrote a file")):
                    self.assertEqual(GENERATOR.write_outputs(outputs, True), list(outputs))
                self.assertFalse((root / "crates").exists())
                for filename, data in outputs.items():
                    path = root / filename
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(data)
                with patch.object(Path, "write_bytes", side_effect=AssertionError("check wrote a file")):
                    self.assertEqual(GENERATOR.write_outputs(outputs, True), [])
                license_path = "data/cvars-info/LICENSE.wowless"
                (root / license_path).write_bytes(b"modified license")
                with patch.object(Path, "write_bytes", side_effect=AssertionError("check wrote a file")):
                    self.assertEqual(GENERATOR.write_outputs(outputs, True), [license_path])

    def test_license_line_endings_are_preserved_without_relaxing_rust_output_checks(self):
        license_path = GENERATOR.LICENSE_PATH
        rust_path = "crates/sim-data-profile/data/cvar_info_retail.rs"
        outputs = {license_path: b"MIT\nCopyright\n", rust_path: b"rust row\n"}
        existing_license = b"MIT\r\nCopyright\r\n"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for filename, data in outputs.items():
                path = root / filename
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data.replace(b"\n", b"\r\n"))
            with patch.object(GENERATOR, "ROOT", root):
                self.assertEqual(GENERATOR.write_outputs(outputs, True), [rust_path])
                self.assertEqual(GENERATOR.write_outputs(outputs, False), [])
                self.assertEqual((root / license_path).read_bytes(), existing_license)
                self.assertEqual((root / rust_path).read_bytes(), outputs[rust_path])
                (root / license_path).write_bytes(b"different notice")
                GENERATOR.write_outputs({license_path: outputs[license_path]}, False)
                self.assertEqual((root / license_path).read_bytes(), outputs[license_path])


if __name__ == "__main__":
    unittest.main()
