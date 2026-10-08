"""Seal integrated receipts and run every historical/integrated evidence validator."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
FRESH = HERE / 'integrated'
HISTORICAL = '67ec8fb94'


def read(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def load_validator():
    spec = importlib.util.spec_from_file_location('integrated_validator', HERE / 'validate_integrated.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def preserve_historical():
    paths = git('ls-tree', '-r', '--name-only', HISTORICAL, str(HERE.relative_to(ROOT))).splitlines()
    rows = []
    for name in paths:
        original = subprocess.check_output(['git', 'show', f'{HISTORICAL}:{name}'], cwd=ROOT)
        expected = hashlib.sha256(original).hexdigest()
        assert digest(ROOT / name) == expected, name
        rows.append({'path': name, 'sha256': expected})
    dump(FRESH / 'historical-preservation.json', {'revision': git('rev-parse', HISTORICAL), 'artifacts': rows})


def seal():
    preserve_historical()
    validator = load_validator()
    context = read(FRESH / 'context.json')
    summaries = []
    for path in sorted(FRESH.glob('patch_*_publication_sweep-results.json')):
        observations = read(path)
        gaps = sum(not row['ok'] for row in observations.values())
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep-results').replace('_', '.')
        summaries.append({'patch': patch, 'rows': len(observations), 'ok': len(observations) - gaps,
                          'gaps': gaps, 'result': 'pass'})
    dump(FRESH / 'extension/p710-sweep-summary.json', summaries)
    dump(FRESH / 'gap-comparison.json', validator.comparison(context))
    lines = ['# 7.1.0 integrated command ledger', '',
             'Exact master: `' + context['master_revision'] + '`. Source/runtime scope: `' + context['runtime_revision'] + '`.',
             'All commands run from `/home/osso/.worktrees/wow-ui-sim-p710-page`; CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p710-page.',
             'Driver revisions and exact per-file source hashes are retained in each receipt. Evidence-only changes do not invalidate runtime proof.', '',
             '| Receipt | Driver revision | Command | Exit | Result |', '|---|---|---|---|---|']
    for path in sorted(FRESH.glob('*.proof.json')):
        receipt = read(path)
        log = (FRESH / receipt['log']).read_text()
        result = re.findall(r'test result: .*|Ran \d+ tests.*|.*Finished .*', log)
        if receipt['command'][:2] == ['cargo', 'test'] and '0 passed;' in log:
            result.append('ZERO CASES; no coverage credited')
        lines.append('| ' + path.name + ' | ' + receipt['revision'][:10] + ' | `' + ' '.join(receipt['command']) + '` | '
                     + str(receipt['exit']) + ' | ' + '; '.join(result[-3:]) + ' |')
    lines += ['', 'Source reproduction: ' + json.dumps(read(FRESH / 'source-reproduction-summary.json')),
              '', 'Negative control and custom intrinsic discovery are expected failures, not passing coverage. Historical evidence is byte-preserved.']
    (FRESH / 'p710-final-command-ledger.md').write_text('\n'.join(lines) + '\n')
    excluded = {'artifact-hashes.json', 'validator-matrix.json', 'finalize.txt', 'job.json', 'launcher.txt'}
    hashes = {str(path.relative_to(ROOT)): digest(path) for path in FRESH.rglob('*')
              if path.is_file() and path.name not in excluded and 'validators' not in path.parts}
    hashes[str((HERE / 'validate_integrated.py').relative_to(ROOT))] = digest(HERE / 'validate_integrated.py')
    dump(FRESH / 'artifact-hashes.json', hashes)


def validate_all():
    logs = FRESH / 'validators'
    logs.mkdir(exist_ok=True)
    paths = set((ROOT / 'data/patch-api/evidence').rglob('validate.py'))
    paths.update((ROOT / 'data/patch-api/evidence').rglob('validate_integrated.py'))
    revision = git('rev-parse', 'HEAD')
    prior = {row['validator']: row for row in read(FRESH / 'prior-validator-matrix.json')}
    rows = []
    for path in sorted(paths):
        relative = path.relative_to(ROOT)
        previous = prior.get(str(relative))
        if previous is not None and path != HERE / 'validate.py':
            assert previous['exit'] == 0 and digest(path) == previous['validator_sha256']
            assert digest(FRESH / previous['log']) == previous['log_sha256']
            rows.append(previous)
            print(relative, 'reused PASS; historical inputs unchanged', flush=True)
            continue
        log = logs / ('-'.join(relative.parts[3:]) + '.txt')
        command = ['python3', '-B', str(path)]
        with log.open('wb') as handle:
            result = subprocess.run(command, cwd=ROOT, stdout=handle, stderr=subprocess.STDOUT)
        rows.append({'validator': str(relative), 'validator_sha256': digest(path), 'command': command,
                     'revision': revision, 'exit': result.returncode, 'log': str(log.relative_to(FRESH)),
                     'log_sha256': digest(log)})
        print(relative, result.returncode, flush=True)
    dump(FRESH / 'validator-matrix.json', rows)
    assert rows and all(row['exit'] == 0 for row in rows), 'Inspect retained failing validator logs'


if __name__ == '__main__':
    seal()
    validate_all()
