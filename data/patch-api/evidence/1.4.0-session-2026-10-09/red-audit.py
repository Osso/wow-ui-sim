"""Absent-accounting RED scaffold, retained separately after own RED run."""
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent


def build():
    return {"source_rows": [], "inventory": [], "signatures": [], "prose": [], "headers": [], "templates": [], "references": [], "navigation": [], "contracts": [], "defaults": [], "count_claims": [], "measurements": {"model": 0, "runtime": 0, "native": 0}, "history": {"applied_successors": [], "queued_successor_references": []}, "totals": {}}


def validate_ledger(ledger):
    return ledger["totals"]


def validate_source(**kwargs):
    return b"", {}, []


def is_retail_successor(version):
    return False


def replay_defaults():
    pass
