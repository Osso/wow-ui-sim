"""Retained RED scaffold: literal SOURCE accounting not implemented."""
def build():
    return {}

def validate_ledger(ledger, *, expected=None):
    assert ledger == build(), 'serialized literal ledger'

def validate_source(**kwargs):
    return '', {}
