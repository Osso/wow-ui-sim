"""Current literal ledger: reset category at section boundaries.

Original audit/ledger/seals/archive remain immutable historical artifacts.
This correction changes metadata only, not contracts or proof credit.
"""

import json
from pathlib import Path
import sys
import audit

EVIDENCE = Path(__file__).resolve().parent


def build():
    ledger = audit.build()
    categories = {}
    category = None
    for row in ledger["source_rows"]:
        literal = row["literal"]
        if literal.startswith("=="):
            category = None
        elif literal.startswith(";"):
            category = literal[1:].strip()
        categories[row["line"]] = category
    for entry in ledger["inventory"]:
        entry["group"] = categories[entry["line"]]
    return ledger


def validate_ledger(ledger):
    assert ledger == build(), "current literal ledger"
    return ledger["totals"]


def main():
    if sys.argv[1:] == ["capture"]:
        output = EVIDENCE / "current-ledger.json"
        assert not output.exists(), "refuse current ledger overwrite"
        output.write_text(json.dumps(build(), indent=2) + "\n")
    else:
        original_seals = audit.check_seals()
        totals = validate_ledger(json.loads((EVIDENCE / "current-ledger.json").read_bytes()))
        audit.replay_defaults()
        print(json.dumps({"scope": "current SOURCE metadata correction, original artifacts unchanged", "original_seals": original_seals, "totals": totals}, indent=2))


if __name__ == "__main__":
    main()
