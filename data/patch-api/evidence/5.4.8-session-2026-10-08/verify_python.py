"""Run each requested Python fixture script once, retaining individual receipts."""
import json
from pathlib import Path
import sys

from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[4]


def main():
    outcomes = {}
    for path in sorted((ROOT / 'tools').glob('test_*.py')):
        outcomes[path.name] = run_proof('p548-' + path.stem, ['python3', '-B', str(path)])
    print(json.dumps(outcomes))
    return int(any(outcomes.values()))


if __name__ == '__main__':
    sys.exit(main())
