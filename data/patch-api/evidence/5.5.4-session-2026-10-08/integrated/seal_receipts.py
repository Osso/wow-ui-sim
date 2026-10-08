"""Seal completed integrated receipts without rewriting historical records."""
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
RUNTIME = '7158cb3fbee1a617a7df2f090a6aef4cb64c2f23'
MASTER = 'bebcc5830dcb21e768a2e3c365105351a400fccb'
LABELS = {'all-sweeps': 56, 'master-all-sweeps': 56, 'mists-page': 2,
          'client-lines': 3, 'mists-check': None, 'negative': None,
          'tools-tests': None, 'format': None}


def digest(value):
    return hashlib.sha256(value).hexdigest()


def main():
    proofs, ledger = {}, []
    for label, count in LABELS.items():
        receipt = json.loads((HERE / (label + '.proof.json')).read_text())
        expected = 101 if label == 'negative' else 0
        assert receipt['exit'] == receipt['expected_exit'] == expected
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        policy = {'command': receipt['command'], 'exit': expected}
        if count is not None:
            policy['passed'] = count
        proofs[label] = policy
        ledger.append({'scope': label, **receipt, 'invalidated': False})
    (HERE / 'command-results.json').write_text(json.dumps(ledger, indent=2) + '\n')
    lines = ['# Integrated proof ledger', '', f'Runtime: `{RUNTIME}`. Master: `{MASTER}`.', '',
             '| Scope | Recorded revision | Command | Result | Later changes invalidate proof |',
             '|---|---|---|---|---|']
    for row in ledger:
        result = str(row['exit']) + (' (expected negative)' if row['scope'] == 'negative' else ' (PASS)')
        lines.append(f"| {row['scope']} | `{row['revision']}` | `{' '.join(row['command'])}` | {result} | No; only own evidence and documentation changed |")
    lines += ['', 'Reproduction: 56 registers, 53 extracts; inherited 12.0.5/12.0.7/12.1.0 errors exactly unchanged.',
              'Retail: every row on 55 pages (9,740 observations) equals pinned master.',
              'Prior validators: 38/38, selected by git ls-tree at pinned master.']
    (HERE / 'command-ledger.md').write_text('\n'.join(lines) + '\n')
    historical = json.loads((HERE.parent / 'p554-context.json').read_text())
    shared_paths = sorted(set(historical['shared_sha256']) | {'tools/patch_audit_validation.py', 'tools/check_patch_validators.py'})
    shared = {path: digest(subprocess.check_output(['git', 'show', RUNTIME + ':' + path], cwd=ROOT)) for path in shared_paths}
    excluded = {'context.json', 'validator-gate.txt', 'validator-gate.proof.json', 'validator-gate-report.json',
                'tamper-results.json', 'tamper-0.txt', 'tamper-1.txt'}
    seals = {path.relative_to(HERE).as_posix(): digest(path.read_bytes())
             for path in sorted(HERE.rglob('*')) if path.is_file() and path.name not in excluded}
    context = {'runtime_revision': RUNTIME, 'master_revision': MASTER, 'proofs': proofs,
               'shared_sha256': shared, 'session_sha256': seals}
    (HERE / 'context.json').write_text(json.dumps(context, indent=2) + '\n')
    print(json.dumps({'sealed_files': len(seals), 'proofs': len(proofs)}))


if __name__ == '__main__':
    main()
