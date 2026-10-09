"""RED scaffold: bounded redirect accounting not implemented."""


def build():
    return {"source_rows": [], "history": {"separate_retail_successor_references": []}}


def validate_source(**kwargs):
    raise AssertionError("frozen source validation missing")


def validate_ledger(ledger):
    return ledger


def is_retail_successor(version):
    return False


def replay_defaults():
    raise AssertionError("historical default replay missing")
