"""Read-only integrated 5.1.0 proof, scoped to committed inputs rather than future audits."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def read(name):
    return json.loads((HERE / name).read_text())


def digest(content):
    return hashlib.sha256(content).hexdigest()


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def blob(revision, path):
    return git('show', revision + ':' + path)


def pinned(revision, path):
    return json.loads(blob(revision, path))


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def check_history(context):
    for name, expected in read('historical-preservation.json').items():
        path = HERE / 'historical-validator.py.txt' if name == 'validate.py' else HISTORY / name
        assert digest(path.read_bytes()) == expected, ('historical artifact drift', name)
    mapping = read('rebase-mapping.json')
    assert mapping['master_revision'] == context['master_revision']
    for row in mapping['commits'] + mapping['external_pins']:
        revision = row['rebased_revision']
        assert git('rev-parse', revision + '^{tree}').decode().strip() == row['rebased_tree']
        patch = git('show', '--format=', '--binary', revision)
        assert git('patch-id', '--stable', input=patch).decode().split()[0] == row['rebased_patch_id']
        for directory, trees in row['directories'].items():
            assert git('rev-parse', revision + ':' + directory).decode().strip() == trees['rebased_tree']
        if row['recorded_patch_id'] != row['rebased_patch_id']:
            assert row['conflict_reason']
            patch = (HERE / row['historical_patch']).read_bytes()
            assert git('patch-id', '--stable', input=patch).decode().split()[0] == row['recorded_patch_id']
    inventory = read('prior-validator-inventory.json')
    assert inventory['revision'] == context['master_revision']
    assert inventory['validators'] == sorted(path for path in names(inventory['revision'], 'data/patch-api/evidence')
                                            if path.endswith('/validate.py'))


def check_reproduction(context):
    revision, master = context['runtime_revision'], context['master_revision']
    registers = read('p510-register-reproduction.json')
    extracts = read('p510-saved-extract-reproduction.json')
    expected = {Path(path).name.removesuffix('-wikitext-register.json')
                for path in names(revision, 'data/patch-api/sources') if path.endswith('-wikitext-register.json')}
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    prior = {row['patch']: row for row in pinned(master,
        'data/patch-api/evidence/5.2.0-session-2026-10-08/integrated/p520-register-reproduction.json')}
    for row in registers:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        flags = provenance.get('generator_flags', prior.get(patch, {}).get('verified_flags'))
        assert row['verified_flags'] == flags
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
    prior_extracts = {row['patch']: row for row in pinned(master,
        'data/patch-api/evidence/5.2.0-session-2026-10-08/integrated/p520-saved-extract-reproduction.json')}
    for row in extracts:
        patch = row['patch']
        assert row['saved_sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-api-changes.txt'))
        if patch in prior_extracts:
            assert all(row[key] == prior_extracts[patch][key] for key in
                       ['verified_flags', 'byte_identical', 'error', 'sha256', 'saved_sha256'])
        else:
            assert row['byte_identical'] and row['error'] is None
    assert {row['patch']: row['error'] for row in extracts if not row['byte_identical']} == {
        patch: row['error'] for patch, row in prior_extracts.items() if not row['byte_identical']}
    preservation = read('p510-input-preservation.json')
    assert {row['path'] for row in preservation['rows']} == set(names(master, 'data/patch-api/sources'))
    for row in preservation['rows']:
        assert row['before_sha256'] == row['after_sha256'] == digest(blob(master, row['path']))
        assert row['after_sha256'] == digest(blob(revision, row['path']))
    assert all(row['before'] == row['after'] for row in read('p510-extract-preservation.json'))
    supplemental = read('supplemental-extract-reproduction.json')
    assert supplemental['flags'] == pinned(revision,
        'data/patch-api/sources/5.4.0-api-changes.provenance.json')['diff_source']['extractor_flags']
    assert supplemental['byte_identical'] and supplemental['sha256'] == digest(blob(revision, supplemental['path']))
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_accounting(context):
    revision = context['runtime_revision']
    pages = read('gap-comparison.json')
    expected = {Path(path).stem for path in names(revision, 'tests')
                if re.fullmatch(r'tests/patch_.*_publication_sweep.rs', path)}
    assert expected == {'patch_' + row['patch'].replace('.', '_') + '_publication_sweep' for row in pages}
    for row in pages:
        stem = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        observations = read(stem + '-results.json')
        register = pinned(revision, f'data/patch-api/sources/{row["patch"]}-wikitext-register.json')
        assert set(observations) == {entry['id'] for entry in register['entries']}
        known_path = 'tests/data/' + stem.removesuffix('_publication_sweep') + '_sweep_known_gaps.json'
        assert sorted(key for key, value in observations.items() if not value['ok']) == row['gaps'] == sorted(pinned(revision, known_path))
        assert len(observations) == row['observations']
        if row['patch'] != '5.1.0':
            assert row['unchanged_vs_master'] and observations == read('master/' + stem + '-results.json')
            assert blob(revision, known_path) == blob(context['master_revision'], known_path)
    positive = read('patch_5_1_0_publication_sweep-results.json')
    historical = json.loads((HISTORY / 'p510-own-results.json').read_text())
    review = read('supersession-review.json')
    changed = {key for key in positive if positive[key] != historical[key]}
    assert changed == {row['source_id'] for row in review['replacements']}
    old_gaps = {key for key, value in historical.items() if not value['ok']}
    gaps = {key for key, value in positive.items() if not value['ok']}
    assert review['historical_gaps'] == len(old_gaps) and review['integrated_gaps'] == len(gaps)
    assert review['new_gaps'] == sorted(gaps - old_gaps)
    ledger = pinned(revision, 'data/patch-api/sources/5.1.0-page-coverage.json')
    entries = {row['source_id']: row for row in ledger['source_rows']}
    assert len(entries) == len(ledger['source_rows'])
    for key, value in positive.items():
        assert entries[key]['status'] == ('bounded-coverage' if value['ok'] else 'audit-pending')
    assert {row['source_id'] for row in read('gap-review.json')} == gaps
    accounting = read('accounting-summary.json')
    assert accounting['inventory'] == len(positive) and accounting['publication_gaps'] == len(gaps)
    assert accounting['total_ids'] == len(entries)
    assert accounting['statuses'] == {status: sum(row['status'] == status for row in entries.values())
                                      for status in ('audit-pending', 'bounded-coverage', 'metadata-only')}
    negative = read('negative-results.json')
    assert set(negative) == set(positive)
    control = context['negative_id']
    assert positive[control]['ok'] and not negative[control]['ok']
    assert all(negative[key] == value for key, value in positive.items() if key != control)
    assert {key for key, value in negative.items() if not value['ok']} == gaps | {control}
    assert read('mists-line-control-results.json') == read('master/mists-line-control-results.json')
    return len(pages), len(gaps)


def check_proofs(context):
    revision = context['runtime_revision']
    fixtures = {Path(path).stem for path in names(revision, 'tools') if re.fullmatch(r'tools/test_.*\.py', path)}
    assert fixtures == {name for name in context['proofs'] if name.startswith('test_')}
    for name in context['proofs']:
        receipt = read(name + '.proof.json')
        assert receipt['command'] and not receipt['invalidated']
        assert receipt['exit'] in (1, 101) if name == 'negative' else receipt['exit'] == 0
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256'], ('receipt log drift', name)
        scope = ['src', 'tests', 'Cargo.toml', 'Cargo.lock', 'build.rs']
        if name == 'reproduction':
            scope = ['tools/gen_patch_wikitext_register.py', 'tools/extract_patch_non_inventory.py']
        changed = git('diff', '--name-only', receipt['revision'], revision, '--', *scope).decode().splitlines()
        allowed = {'src/c_api/patch_retired_members.rs', 'tests/patch_5_1_0_behavior.rs',
                   'tests/patch_5_1_0_publication_sweep.rs', 'tests/data/patch_5_1_0_sweep_known_gaps.json'}
        assert set(changed) <= (allowed if name.startswith('master-') else set()), (name, changed)
        if name.startswith(('all-sweeps', 'master-', 'prefork-', 'integration-', 'lib-', 'mists-all')):
            match = re.search(r'test result: ok\. (\d+) passed', (HERE / receipt['log']).read_text())
            assert match and int(match[1]) > 0, ('empty acceptance', name)
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all(line.startswith('warning: iced-wgpu-patched/Cargo.toml:') or
               line == 'warning: `iced_wgpu` (manifest) generated 6 warnings' for line in warnings), warnings
    for scan in read('retirement-scans.json'):
        raw = (HERE / scan['log']).read_bytes()
        assert digest(raw) == scan['sha256'] and len(raw.decode().splitlines()) == scan['matches']
        assert scan['exit'] in (0, 1) and '-w' in scan['command'] and '-F' in scan['command']
        if scan['scope'] == 'cache':
            assert scan['matches'] == 0
    assert read('retirement-review.json')['readds'] == []


def main():
    context = read('context.json')
    for name, expected in context['own_files'].items():
        assert digest((HERE / name).read_bytes()) == expected, ('integrated artifact drift', name)
    for directory, expected in context['directory_trees'].items():
        assert git('rev-parse', context['runtime_revision'] + ':' + directory).decode().strip() == expected
    check_history(context)
    registers, extracts = check_reproduction(context)
    sweeps, gaps = check_accounting(context)
    check_proofs(context)
    assert all(path.stat().st_size < 5_000_000 for path in HISTORY.rglob('*') if path.is_file())
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts,
                      'sweeps': sweeps, 'publication_gaps': gaps,
                      'prior_validators': len(read('prior-validator-inventory.json')['validators'])}))


if __name__ == '__main__':
    main()
