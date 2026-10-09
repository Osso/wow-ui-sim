"""RED scaffold: bounded source accounting not implemented."""
from pathlib import Path
import json
EVIDENCE = Path(__file__).resolve().parent
def build():
    return {"source_rows": [], "references": [], "contracts": [], "history": {"separate_retail_successor_references": []}}
def validate_source(**kwargs):
    raise AssertionError("frozen identity not implemented")
def validate_ledger(ledger):
    raise AssertionError("literal accounting not implemented")
def replay_defaults():
    raise AssertionError("default byte/error replay not implemented")
def is_retail_successor(version):
    return False
