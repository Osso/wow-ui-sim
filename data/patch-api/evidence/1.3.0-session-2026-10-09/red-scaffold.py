"""Own RED scaffold: accounting/identity/default-replay not implemented."""
def build():
    return {}
def validate_source(**kwargs):
    return None
def validate_ledger(ledger):
    return None
def replay_defaults():
    raise AssertionError("own default replay absent")
def is_retail_successor(version):
    return False
