"""Absent 1.6.0 accounting scaffold for retained SOURCE RED."""
import importlib.util
from pathlib import Path
EVIDENCE = Path(__file__).resolve().parent

def load_historical(name):
    spec = importlib.util.spec_from_file_location(name, EVIDENCE / "historical-tools" / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def build():
    return {"source_rows": [], "history": {"separate_retail_successor_references": []}}

def validate_source(**kwargs):
    raise AssertionError("1.6.0 frozen identity accounting absent")

def validate_ledger(ledger):
    raise AssertionError("1.6.0 serialized accounting absent")

def replay_defaults():
    raise AssertionError("1.6.0 default replay absent")
