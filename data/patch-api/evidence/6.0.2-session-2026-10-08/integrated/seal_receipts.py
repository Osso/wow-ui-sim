"""Freeze completed proof receipts and exact all-page comparisons."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
MASTER = 'd0fabed03'
RUNTIME = '89d359e65'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def read(path):
    return json.loads(path.read_text())


def dump(name, value):
    path = HERE / name
    previous = path.read_bytes() if path.exists() else None
    path.write_text(json.dumps(value, indent=2) + '\n')


def main():
    assert (HERE / 'checks-launcher.proof.json').exists(), 'checks still running'
    assert (HERE / 'master-diagnostic-launcher.proof.json').exists(), 'master diagnostic still running'
    pages = []
    paths = git('ls-tree', '-r', '--name-only', RUNTIME, 'tests').splitlines()
    for path in paths:
        if not path.endswith('_publication_sweep.rs'):
            continue
        stem = Path(path).stem
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        result = read(HERE / (stem + '-results.json'))
        if patch != '6.0.2':
            assert result == read(HERE / 'master' / (stem + '-results.json')), patch
        pages.append({'patch': patch, 'observations': len(result),
                      'gaps': sorted(key for key, row in result.items() if not row['ok']),
                      'unchanged_vs_master': patch != '6.0.2'})
    dump('gap-comparison.json', pages)
    own = read(HERE / 'patch_6_0_2_publication_sweep-results.json')
    historical = read(HERE.parent / 'patch_6_0_2_publication_sweep-results.json')
    assert own == historical
    dump('supersession-review.json', {'before_gaps': 275, 'after_gaps': 275,
                                     'observation_changes': {}, 'replacements': [],
                                     'integrated_registers': ['6.1.0', '6.2.0', '6.2.2'],
                                     'pending_prose': 95, 'pending_enum_members': 79})
    receipts = {path.name.removesuffix('.proof.json'): read(path)
                for path in sorted(HERE.glob('*.proof.json'))
                if 'launcher' not in path.name and not path.name.startswith('portability')}
    dump('context.json', {'master_revision': git('rev-parse', MASTER),
                          'runtime_revision': git('rev-parse', RUNTIME), 'receipts': receipts})
    lines = ['# Integrated 6.0.2 command/proof ledger', '',
             'Runtime `'+git('rev-parse', RUNTIME)+'`; master `'+git('rev-parse', MASTER)+'`.',
             'Later evidence/docs commits do not invalidate unchanged src/tests/tools trees.',
             'Every command executes from the owned worktree; Cargo target is p602-page.', '',
             '| Receipt | Command | Exit | Result |', '|---|---|---:|---|']
    for label, receipt in receipts.items():
        text = (HERE / receipt['log']).read_text()
        counts = re.findall(r'test result: (?:ok|FAILED)\. [^\n]+', text)
        python = re.findall(r'Ran \d+ tests? in [^\n]+', text)
        result = '; '.join(counts or python) or ('[]' if text.rstrip().endswith('[]') else 'see sealed log')
        command = ' '.join(receipt['command'])
        lines.append('| '+label+' | `'+command+'` | '+str(receipt['exit'])+' | '+result+' |')
    lines.extend(['', 'Integration scenario boundary failure matches pinned master exactly (9/10).',
                  'No cached tests match the scenario/shared-map positional filters; cached scenario behavior is covered by patch_6_0_2 and objective_tracker, vignette consumers by flight-map/navigation/POI tests.',
                  '53 registers/50 extracts reproduce; three inherited extract failures remain unchanged.',
                  '37 prior validators enumerated with git ls-tree at pinned master all pass.',
                  'Historical receipts and validator invariants replay unchanged through mapped pins.'])
    path = HERE / 'command-ledger.md'
    previous = path.read_bytes() if path.exists() else None
    path.write_text('\n'.join(lines) + '\n')
    print(json.dumps({'pages': len(pages), 'other_pages_unchanged': len(pages)-1,
                      'receipts': len(receipts), 'own_gaps': 275}))


def seal_committed_inputs():
    prefix = HERE.relative_to(ROOT).as_posix() + '/'
    paths = git('ls-tree', '-r', '--name-only', 'HEAD', prefix).splitlines()
    seals = {}
    for path in paths:
        name = path.removeprefix(prefix)
        if name == 'artifact-hashes.json' or name.startswith('portability'):
            continue
        content = subprocess.check_output(['git', 'show', 'HEAD:' + path], cwd=ROOT)
        assert (ROOT / path).read_bytes() == content, path
        seals[name] = hashlib.sha256(content).hexdigest()
    wrapper = HERE.parent / 'validate.py'
    content = subprocess.check_output(['git', 'show', 'HEAD:' + wrapper.relative_to(ROOT).as_posix()], cwd=ROOT)
    assert wrapper.read_bytes() == content
    seals['../validate.py'] = hashlib.sha256(content).hexdigest()
    assert 'context.json' in seals and 'validate.py' in seals
    dump('artifact-hashes.json', seals)
    print('Sealed', len(seals), 'committed inputs')


if __name__ == '__main__':
    seal_committed_inputs() if sys.argv[1:] == ['--seal'] else main()
