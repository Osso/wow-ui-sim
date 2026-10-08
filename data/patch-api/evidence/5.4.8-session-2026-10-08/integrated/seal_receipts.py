"""Seal completed proof artifacts only; no transient or ignored validation inputs."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from run_checks import SELECTORS

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent

if __name__ == '__main__':
    for label in ['branch-checks', 'master-checks', 'prefork-checks', 'master-prefork-checks', 'prior-validators', 'lib-checks', 'master-missing-checks']:
        assert (HERE / (label + '.proof.json')).exists(), 'active driver: ' + label
    # Correct the copied runner's old descriptive target field. The command driver
    # and actual Running/binary paths establish the master-ref allocation.
    corrections = []
    for path in sorted(HERE.glob('*.proof.json')):
        receipt = json.loads(path.read_text())
        command = receipt['command']
        if command[0] != 'cargo':
            continue
        target = '/home/osso/.cache/wow-ui-sim-targets/' + ('master-ref' if '--manifest-path' in command else 'p548-page')
        if receipt['target'] != target:
            corrections.append({'receipt': path.name, 'before': receipt['target'], 'after': target,
                                'basis': 'committed run_master/run_prefork driver target assignment and recorded executable paths'})
            receipt['target'] = target
            receipt['environment']['CARGO_TARGET_DIR'] = target
            path.write_text(json.dumps(receipt, indent=2) + '\n')
    (HERE / 'target-metadata-corrections.json').write_text(json.dumps(corrections, indent=2) + '\n')
    subprocess.run([sys.executable, '-B', str(HERE / 'summarize_proofs.py')], cwd=ROOT, check=True)
    receipts = json.loads((HERE / 'receipts.json').read_text())
    required = {'own-sweep', 'all-sweeps', 'master-all-sweeps', 'negative', 'format', 'mists-check',
                'retail-build', 'master-retail-build', 'branch-startup', 'master-startup',
                'integration-patch_5_4_8', 'prefork_full_ui-patch_5_4_8', 'integration-regressions',
                'master-integration-regressions', 'mists-integration-regressions',
                'master-mists-integration-regressions', 'prior-validators'}
    required.update(path.stem for path in (ROOT / 'tools').glob('test_*.py'))
    for prefix in ['', 'master-']:
        required.update(prefix + 'prefork-' + selector for selector in SELECTORS)
    assert required <= receipts.keys(), required - receipts.keys()
    expected = {'negative': 1, 'integration-regressions': 101, 'master-integration-regressions': 101,
                'mists-integration-regressions': 101, 'master-mists-integration-regressions': 101}
    for prefix in ['', 'master-']:
        for profile in ['retail', 'mists']:
            required.update([prefix + 'lib-' + profile + '-list', prefix + 'lib-' + profile + '-regressions'])
    for row in json.loads((HERE / 'regression-comparison.json').read_text()):
        if row['target'] == 'lib':
            for side in ['branch', 'master']:
                label = row[side]['receipt'].removesuffix('.proof.json')
                if row[side]['failures']:
                    expected[label] = 101
    for label in required:
        assert receipts[label]['exit'] == expected.get(label, 0), label
    runtime = subprocess.check_output(['git', 'rev-parse', 'b77d0b5db'], cwd=ROOT, text=True).strip()
    master = subprocess.check_output(['git', 'rev-parse', 'a9d7c9566'], cwd=ROOT, text=True).strip()
    context = {'runtime_revision': runtime, 'master_revision': master, 'required_receipts': sorted(required),
               'wrapper_sha256': hashlib.sha256((HERE.parent / 'validate.py').read_bytes()).hexdigest(),
               'expected_failures': expected, 'seals': {}}
    ledger = ['# Integrated 5.4.8 command/proof ledger', '',
              'Runtime `' + runtime + '`; master `' + master + '`.',
              'All commands run from the owned p548-page worktree; master uses immutable Git archives.',
              'Targets: p548-page for branch, master-ref for master. No later source/test/tool changes invalidate these receipts.',
              'Rejected multi-filter prefork invocations have no coverage; single-filter receipts supersede them.',
              'Mists prefork is unavailable (requires client-retail); its rejection logs are not passing behavior evidence.', '',
              '| Receipt | Command | Exit | Result |', '|---|---|---:|---|']
    import re
    for label, receipt in sorted(receipts.items()):
        text = (HERE / receipt['log']).read_text()
        counts = re.findall(r'^test result: .*$', text, re.M)
        python = re.findall(r'Ran \d+ tests? in [^\n]+', text)
        result = (counts[-1] if counts else '; '.join(python)) or ('[]' if text.rstrip().endswith('[]') else 'see sealed log')
        ledger.append('| ' + label + ' | `' + ' '.join(receipt['command']) + '` | ' + str(receipt['exit']) + ' | ' + result + ' |')
    (HERE / 'command-ledger.md').write_text('\n'.join(ledger) + '\n')
    excluded = {'context.json', 'check_tamper.py', 'tamper-check.txt', 'tamper-check.proof.json',
                'validator-gate.txt', 'validator-gate.proof.json', 'validator-gate-report.json'}
    for path in sorted(HERE.rglob('*')):
        if not path.is_file() or path.name in excluded or path.name.endswith('.launcher.txt'):
            continue
        context['seals'][path.relative_to(HERE).as_posix()] = hashlib.sha256(path.read_bytes()).hexdigest()
    (HERE / 'context.json').write_text(json.dumps(context, indent=2) + '\n')
    print('SEALED', len(context['seals']), 'artifacts;', len(required), 'required receipts')
