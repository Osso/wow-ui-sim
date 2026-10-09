"""RED fixture scaffold: literal accounting not implemented."""


def build():
    return {}


def validate_ledger(ledger):
    assert ledger == build(), 'serialized literal ledger'
