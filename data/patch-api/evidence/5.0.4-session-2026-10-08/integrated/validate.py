"""Read-only integrated 5.0.4 accounting with exact shared revision inputs."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def read(name):
    return json.loads((HERE / name).read_text())


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', revision + ':' + path)


def pinned(revision, path):
    return json.loads(blob(revision, path))


def check_history(context):
    historical = read('historical-preservation.json')
    for name, expected in historical.items():
        path = HERE / 'historical-validator.py.txt' if name == 'validate.py' else HISTORY / name
        assert digest(path.read_bytes()) == expected, ('historical artifact drift', name)
    assert digest((HISTORY / 'validate.py').read_bytes()) == context['dispatcher_sha256']
    mapping = read('rebase-mapping.json')
    assert mapping['master'] == context['master_revision']
    for row in mapping['commits']:
        for directory, trees in row['trees'].items():
            assert git('rev-parse', row['rebased'] + ':' + directory).decode().strip() == trees['rebased']
    for path, expected in context['prior_validator_blobs'].items():
        assert digest(blob(context['master_revision'], path)) == digest(git('show', expected))


def check_reproduction(context):
    revision = context['runtime_revision']
    paths = git('ls-tree', '-r', '--name-only', revision, 'data/patch-api/sources').decode().splitlines()
    registers = read('p504-register-reproduction.json')
    assert {path for path in paths if path.endswith('-wikitext-register.json')} == {
        'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json' for row in registers}
    assert all(row['byte_identical'] and row['exit'] == 0 for row in registers)
    for row in registers:
        assert row['sha256'] == digest(blob(revision, 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json'))
    extracts = read('p504-saved-extract-reproduction.json')
    assert {row['patch'] for row in extracts if not row['byte_identical']} == {'12.0.5', '12.0.7', '12.1.0'}
    for row in extracts:
        assert row['saved_sha256'] == digest(blob(revision, 'data/patch-api/sources/' + row['patch'] + '-api-changes.txt'))
    preservation = read('p504-input-preservation.json')
    master_paths = git('ls-tree', '-r', '--name-only', context['master_revision'], 'data/patch-api/sources').decode().splitlines()
    assert {row['path'] for row in preservation['rows']} == set(master_paths)
    for row in preservation['rows']:
        assert row['before_sha256'] == row['after_sha256'] == digest(blob(context['master_revision'], row['path']))
        assert row['after_sha256'] == digest(blob(revision, row['path']))
    assert all(row['before'] == row['after'] for row in read('p504-extract-preservation.json'))
    supplemental = read('supplemental-extract-reproduction.json')
    assert supplemental['byte_identical'] and supplemental['sha256'] == digest(blob(revision, supplemental['path']))
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_accounting(context):
    revision = context['runtime_revision']
    rows = read('gap-comparison.json')
    paths = git('ls-tree', '-r', '--name-only', revision, 'tests').decode().splitlines()
    assert {Path(path).stem for path in paths if re.fullmatch(r'tests/patch_.*_publication_sweep.rs', path)} == {
        'patch_' + row['patch'].replace('.', '_') + '_publication_sweep' for row in rows}
    for row in rows:
        stem = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        observed = read(stem + '-results.json')
        register = pinned(revision, 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json')
        assert set(observed) == {entry['id'] for entry in register['entries']}
        gaps = sorted(key for key, value in observed.items() if not value['ok'])
        fixture = 'tests/data/' + stem.removesuffix('_publication_sweep') + '_sweep_known_gaps.json'
        assert gaps == row['gaps'] == sorted(pinned(revision, fixture))
        assert len(observed) == row['observations']
        if row['patch'] != '5.0.4':
            assert row['unchanged_vs_master'] and observed == read('master/' + stem + '-results.json')
            assert blob(revision, fixture) == blob(context['master_revision'], fixture)
    own = read('patch_5_0_4_publication_sweep-results.json')
    historical = json.loads((HISTORY / 'patch_5_0_4_publication_sweep-results.json').read_text())
    review = read('supersession-review.json')
    assert {key for key in own if own[key] != historical[key]} == {row['source_id'] for row in review['replacements']}
    old_gaps = {key for key, value in historical.items() if not value['ok']}
    gaps = {key for key, value in own.items() if not value['ok']}
    assert review['historical_gaps'] == len(old_gaps) and review['integrated_gaps'] == len(gaps)
    assert review['new_gaps'] == sorted(gaps - old_gaps) == []
    ledger = pinned(revision, 'data/patch-api/sources/5.0.4-page-coverage.json')['source_rows']
    by_id = {row['source_id']: row for row in ledger}
    assert len(by_id) == len(ledger) == 704
    for key, value in own.items():
        assert by_id[key]['status'] == ('bounded-coverage' if value['ok'] else 'audit-pending')
    gap_review = read('gap-review.json')
    assert {row['source_id'] for row in gap_review['publication_mismatches']} == gaps
    assert len(gap_review['prose_pending']) == 61 and len(gap_review['signatures_pending']) == 5
    summary = read('accounting-summary.json')
    assert summary['publication_mismatches'] == len(gaps) and summary['inventory'] == len(own)
    assert summary['statuses'] == {status: sum(row['status'] == status for row in ledger) for status in summary['statuses']}
    negative = read('negative-results.json')
    control = context['negative_id']
    assert set(negative) == set(own) and own[control]['ok'] and not negative[control]['ok']
    assert all(negative[key] == value for key, value in own.items() if key != control)
    assert {key for key, value in negative.items() if not value['ok']} == gaps | {control}
    assert read('mists-line-control-results.json') == read('master/mists-line-control-results.json')
    return len(rows), len(gaps)


def check_proofs(context):
    revision = context['runtime_revision']
    for name, expected_exit in context['proofs'].items():
        receipt = read(name + '.proof.json')
        assert receipt['command'] and not receipt['invalidated'] and receipt['exit'] == expected_exit, name
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256'], ('log drift', name)
        scope = ['src', 'tests', 'Cargo.toml', 'Cargo.lock', 'build.rs']
        if name.startswith(('test_', 'historical-', 'preservation', 'reproduction')):
            scope = ['tools']
        changed = git('diff', '--name-only', receipt['revision'], revision, '--', *scope).decode().splitlines()
        allowed = {'src/c_api/c_pet_battles.rs', 'tests/patch_5_0_4_behavior.rs',
                   'tests/patch_5_0_4_publication_sweep.rs', 'tests/data/patch_5_0_4_sweep_known_gaps.json'}
        if name.startswith('master-'):
            permitted = allowed
        elif name in ('all-sweeps', 'own-prefork', 'negative', 'default-check', 'mists-check', 'mists-all-sweeps', 'mists-pet-type', 'startup-build', 'startup', 'format', 'resolved-sweeps'):
            permitted = set()
        else:
            # The resolved-gap JSON fixture is not selected by pet state, caller,
            # library, historical or parser proof commands.
            permitted = {'tests/data/patch_5_0_4_sweep_known_gaps.json'}
        assert set(changed) <= permitted, (name, changed)
        if name in context['test_counts']:
            results = re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed', (HERE / receipt['log']).read_text())
            assert results and list(map(int, results[-1])) == context['test_counts'][name], (name, results)
    assert (HERE / 'startup.txt').read_text().splitlines()[-1] == '[]'
    for name in ('mists-check', 'default-check'):
        text = (HERE / (name + '.txt')).read_text()
        assert 'Finished `dev` profile' in text
        warnings = re.findall(r'^warning: (.*)$', text, re.M)
        assert all(value.startswith(('iced-wgpu-patched/', '`iced_wgpu` (manifest)')) for value in warnings), warnings
    for name in ('pet-battles-integration', 'lib-namespace', 'retail-line-controls'):
        pattern = r'test (\S+) \.\.\. (ok|FAILED)'
        assert sorted(re.findall(pattern, (HERE / (name + '.txt')).read_text())) == sorted(re.findall(pattern, (HERE / ('master-' + name + '.txt')).read_text()))


def main():
    context = read('context.json')
    for name, expected in context['own_files'].items():
        assert digest((HERE / name).read_bytes()) == expected, ('integrated artifact drift', name)
    for path, expected in context['current_source_guards'].items():
        assert digest((ROOT / path).read_bytes()) == expected, ('own source drift', path)
    for directory, expected in context['directory_trees'].items():
        assert git('rev-parse', context['runtime_revision'] + ':' + directory).decode().strip() == expected
    assert git('diff', '--name-only', context['master_revision'], context['runtime_revision'], '--', 'src', 'Interface').decode().splitlines() == ['src/c_api/c_pet_battles.rs']
    check_history(context)
    registers, extracts = check_reproduction(context)
    pages, gaps = check_accounting(context)
    check_proofs(context)
    assert all(path.stat().st_size < 5_000_000 for path in HISTORY.rglob('*') if path.is_file())
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts, 'pages': pages, 'gaps': gaps, 'retirements': 0}))


if __name__ == '__main__':
    main()
