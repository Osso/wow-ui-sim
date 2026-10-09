"""Test-first absent SOURCE implementation; retained RED boundary."""
def build():
    return {"source_rows": [], "references": [], "contracts": [], "history": {"separate_retail_successor_references": []}}
def validate_source(**kwargs):
    raise AssertionError("frozen source validation missing")
def validate_ledger(ledger):
    return {}
def replay_defaults():
    raise AssertionError("historical default replay missing")
