"""Bounded historical artifact tests; no runtime or Git-object dependency."""
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

PAGE = Path(__file__).resolve().parent
LEDGER = "data/patch-api/sources/4.3.4-page-coverage.json"
RAW = "data/patch-api/sources/4.3.4-api-changes.wikitext"


class HistoricalValidationTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue((PAGE / "validate.py").is_file(), "historical validator missing")
        spec = importlib.util.spec_from_file_location("p434_validator", PAGE / "validate.py")
        self.validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.validator)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.page = Path(self.temp.name) / "evidence"
        shutil.copytree(PAGE, self.page, ignore=shutil.ignore_patterns("__pycache__"))

    def load(self, name):
        return json.loads((self.page / name).read_text())

    def save(self, name, value):
        (self.page / name).write_text(json.dumps(value, indent=2) + "\n")

    def reseal(self):
        seals = self.load("session-seals.json")
        for name in seals["files"]:
            data = (self.page / name).read_bytes()
            seals["files"][name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
        self.save("session-seals.json", seals)
        acceptance = self.load("validator-acceptance-pins.json")
        for name in acceptance["files"]:
            data = (self.page / name).read_bytes()
            acceptance["files"][name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
        self.save("validator-acceptance-pins.json", acceptance)

    def mutate_archive(self, path, mutate, update_identities=True):
        archive = json.loads(gzip.decompress((self.page / "historical-inputs.json.gz").read_bytes()))
        archive[path] = mutate(archive[path])
        data = gzip.compress(json.dumps(archive).encode(), mtime=0)
        (self.page / "historical-inputs.json.gz").write_bytes(data)
        pins = self.load("historical-input-pins.json")
        pins["archive_sha256"] = hashlib.sha256(data).hexdigest()
        if update_identities:
            raw = archive[path].encode()
            pins["paths"][path] = {"sha256": hashlib.sha256(raw).hexdigest(), "blob": hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()}
        self.save("historical-input-pins.json", pins)
        self.reseal()

    def reject_restore(self, mutation, message):
        before = {p.name: p.read_bytes() for p in self.page.iterdir() if p.is_file()}
        try:
            mutation()
            with self.assertRaisesRegex(ValueError, message):
                self.validator.validate(self.page)
        finally:
            for name, data in before.items():
                (self.page / name).write_bytes(data)
        self.validator.validate(self.page)

    def test_accepts_detached_historical_evidence(self):
        result = self.validator.validate(self.page)
        self.assertEqual((result["inventory"], result["ledger_ids"], result["successors"]), (11, 12, 64))
        self.assertEqual((result["gaps"], result["negative_gaps"], result["supersessions"]), (7, 8, 3))
        self.assertEqual(result["boundary"], "historical artifacts only; not fresh model/native proof")

    def test_source_response_tamper(self):
        def mutate():
            response = self.load("source-response.json")
            page = next(iter(response["query"]["pages"].values()))
            page["revisions"][0]["revid"] += 1
            self.save("source-response.json", response)
            self.reseal()
        self.reject_restore(mutate, "source response")

    def test_ledger_status_tamper(self):
        def mutate(text):
            ledger = json.loads(text)
            ledger["source_rows"][1]["status"] = "absence-only"
            return json.dumps(ledger)
        self.reject_restore(lambda: self.mutate_archive(LEDGER, mutate), "ledger status")

    def test_archived_git_blob_tamper(self):
        self.reject_restore(lambda: self.mutate_archive(RAW, lambda text: text + "\n", False), "archived identity")

    def test_archived_source_tamper(self):
        self.reject_restore(lambda: self.mutate_archive(RAW, lambda text: text.replace("GetSessionTime", "ForgedSessionTime")), "wikitext")

    def test_receipt_exit_tamper(self):
        def mutate():
            receipt = self.load("own.proof.json")
            receipt["exit"] = 1
            self.save("own.proof.json", receipt)
            self.reseal()
        self.reject_restore(mutate, "receipt exit")

    def test_log_tamper(self):
        self.reject_restore(lambda: (self.page / "own.log").write_bytes(b"forged log"), "sealed artifact")

    def test_same_cardinality_negative_tamper(self):
        def mutate():
            results = self.load("negative-results.json")
            results["negative-fabricated-global"]["ok"] = True
            self.save("negative-results.json", results)
            self.reseal()
        self.reject_restore(mutate, "observation outcome")

    def test_successor_supersession_tamper(self):
        def mutate():
            accounting = self.load("accounting.json")
            accounting["supersessions"]["wt-global-api-GMSubmitBug-16"] = "fabricated"
            self.save("accounting.json", accounting)
            self.reseal()
        self.reject_restore(mutate, "supersessions")

    def test_acceptance_seal_tamper(self):
        self.reject_restore(lambda: (self.page / "session-seals.json").write_bytes((self.page / "session-seals.json").read_bytes() + b"\n"), "acceptance artifact")


if __name__ == "__main__":
    unittest.main()
