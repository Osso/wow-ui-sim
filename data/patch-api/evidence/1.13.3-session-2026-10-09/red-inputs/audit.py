"""RED scaffold: literal source accounting not implemented yet."""
OCCURRENCE_FIELDS = ()
def build():
    return {}
def validate_source(response=None, raw=None):
    raise AssertionError("source validation unimplemented")
def validate_ledger(ledger):
    raise AssertionError("ledger validation unimplemented")
