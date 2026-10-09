"""RED boundary scaffold; no source accounting implemented."""
def absent(*args, **kwargs):
    assert False, "1.1.0 source accounting absent"
build = validate_source = validate_ledger = replay_defaults = absent
