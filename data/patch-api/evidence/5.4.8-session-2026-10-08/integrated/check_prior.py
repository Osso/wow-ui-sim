"""Run exactly the prior-validator set from the recorded master tree."""
import json
from pathlib import Path
import subprocess
import sys
from run_proof import run_proof

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent

if __name__ == '__main__':
    inventory = json.loads((HERE / 'prior-validator-inventory.json').read_text())
    expected = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', inventory['revision'], 'data/patch-api/evidence'], cwd=ROOT, text=True)
    assert inventory['validators'] == [p for p in expected.splitlines() if p.endswith('/validate.py')]
    rows = []
    for index, path in enumerate(inventory['validators']):
        # Each validator source itself is pinned, not silently changed on this branch.
        original = subprocess.check_output(['git', 'show', inventory['revision'] + ':' + path], cwd=ROOT)
        assert (ROOT / path).read_bytes() == original, path
        code = run_proof('prior-' + str(index), [sys.executable, '-B', str(ROOT / path)])
        rows.append({'validator': path, 'exit': code, 'receipt': 'prior-' + str(index) + '.proof.json'})
        (HERE / 'prior-results.json').write_text(json.dumps(rows, indent=2) + '\n')
    assert all(row['exit'] == 0 for row in rows)
    print('PASS:', len(rows), 'prior validators')
