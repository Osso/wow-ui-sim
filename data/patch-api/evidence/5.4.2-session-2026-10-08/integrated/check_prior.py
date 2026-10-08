"""Retain the complete prior-validator set from the recorded master Git tree."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = '896086537a2b3c1ead5886d5ae3e430d56e7ef20'
CHECKOUT = Path('/home/osso/.worktrees/wow-ui-sim-p542-master-proof')


def main():
    assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=CHECKOUT, text=True).strip() == MASTER
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', MASTER, 'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    paths = [name for name in names if name.endswith('/validate.py')]
    assert paths
    (HERE / 'prior-validator-inventory.json').write_text(json.dumps({'revision': MASTER, 'validators': paths}, indent=2) + '\n')
    rows = []
    for index, name in enumerate(paths):
        result = subprocess.run([sys.executable, '-B', str(CHECKOUT / name)], cwd=CHECKOUT, capture_output=True, text=True)
        log = HERE / f'prior-{index}.txt'
        log.write_text(result.stdout + result.stderr)
        rows.append({'validator': name, 'exit': result.returncode, 'log': log.name,
                     'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()})
    (HERE / 'prior-results.json').write_text(json.dumps(rows, indent=2) + '\n')
    print(json.dumps({'validators': len(rows), 'passed': sum(row['exit'] == 0 for row in rows), 'failed': sum(row['exit'] != 0 for row in rows)}))
    return int(any(row['exit'] != 0 for row in rows))


if __name__ == '__main__':
    sys.exit(main())
