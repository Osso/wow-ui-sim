"""Correction RED scaffold: historical category tracker leaks across headings."""
import audit


def build():
    return audit.build()


def validate_ledger(ledger):
    return ledger["totals"]
