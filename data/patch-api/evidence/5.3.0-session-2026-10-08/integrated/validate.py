"""Read-only integrated retail 5.3.0 receipts, pinned independently of later audits."""
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


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def blob(revision, path):
    return git('show', revision + ':' + path)


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def digest(content):
    return hashlib.sha256(content).hexdigest()


def pinned(revision, path):
    return json.loads(blob(revision, path))


def main():
    context = read('context.json')
    revision, master = context['runtime_revision'], context['master_revision']
    for name, expected in context['own_files'].items():
        assert digest((HERE / name).read_bytes()) == expected, ('integrated artifact drift', name)
    for name, expected in read('historical-preservation.json').items():
        assert digest((HISTORY / name).read_bytes()) == expected, ('historical artifact drift', name)
    for directory, expected in context['directory_trees'].items():
        assert git('rev-parse', revision + ':' + directory).decode().strip() == expected
    assert not git('diff', master, revision, '--', 'src', 'Interface'), 'runtime/vendor changes'
    for path, minimum in [('docs/wiki/index.md', 2767), ('docs/wiki/log.md', 536)]:
        assert len(blob(context['wiki_revision'], path).decode().splitlines()) >= minimum
    mapping = read('rebase-mapping.json')
    for row in mapping['commits']:
        rebased = row['rebased_revision']
        assert git('rev-parse', rebased + '^{tree}').decode().strip() == row['rebased_tree']
        patch = git('show', '--format=', '--binary', rebased)
        assert git('patch-id', '--stable', input=patch).decode().split()[0] == row['rebased_patch_id']
        for directory, pins in row['directories'].items():
            assert git('rev-parse', rebased + ':' + directory).decode().strip() == pins['rebased_tree']
        if row['recorded_patch_id'] != row['rebased_patch_id']:
            assert row['subject'] == 'Audit pinned retail 5.3.0 source and caption inventories'
            old_patch = (HERE / 'historical-tool-commit.patch').read_bytes()
            assert git('patch-id', '--stable', input=old_patch).decode().split()[0] == row['recorded_patch_id']
    registers = read('p530-register-reproduction.json')
    extracts = read('p530-saved-extract-reproduction.json')
    expected = {Path(path).name.removesuffix('-wikitext-register.json')
                for path in names(revision, 'data/patch-api/sources')
                if path.endswith('-wikitext-register.json')}
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    prior_registers = {row['patch']: row for row in pinned(master,
        'data/patch-api/evidence/5.4.0-session-2026-10-08/integrated/p540-register-reproduction.json')}
    for row in registers:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        expected_flags = provenance.get('generator_flags')
        if expected_flags is None:
            expected_flags = prior_registers[patch]['verified_flags']
        assert row['verified_flags'] == expected_flags
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
    prior = {row['patch']: row for row in pinned(master,
        'data/patch-api/evidence/5.4.0-session-2026-10-08/integrated/p540-saved-extract-reproduction.json')}
    for row in extracts:
        patch = row['patch']
        assert row['saved_sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-api-changes.txt'))
        if patch in prior:
            assert all(row[key] == prior[patch][key] for key in
                       ['verified_flags', 'byte_identical', 'error', 'sha256', 'saved_sha256'])
        else:
            assert row['byte_identical'] and row['error'] is None
    assert sorted(row['patch'] for row in extracts if not row['byte_identical']) == ['12.0.5', '12.0.7', '12.1.0']
    preservation = read('p530-input-preservation.json')
    assert {row['path'] for row in preservation['rows']} == set(names(master, 'data/patch-api/sources'))
    for row in preservation['rows']:
        assert row['before_sha256'] == digest(blob(master, row['path']))
        assert row['after_sha256'] == digest(blob(revision, row['path'])) == row['before_sha256']
    assert all(row['before'] == row['after'] for row in read('p530-extract-preservation.json'))
    pages = read('gap-comparison.json')
    expected_sweeps = {Path(path).stem for path in names(revision, 'tests')
                       if re.fullmatch(r'tests/patch_.*_publication_sweep.rs', path)}
    assert expected_sweeps == {'patch_' + row['patch'].replace('.', '_') + '_publication_sweep' for row in pages}
    for row in pages:
        stem = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        observations = read(stem + '-results.json')
        register = pinned(revision, 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json')
        assert set(observations) == {entry['id'] for entry in register['entries']}
        gaps = sorted(key for key, value in observations.items() if not value['ok'])
        known = 'tests/data/' + stem.removesuffix('_publication_sweep') + '_sweep_known_gaps.json'
        assert gaps == row['gaps'] == sorted(pinned(revision, known))
        assert len(observations) == row['observations']
        if row['patch'] != '5.3.0':
            assert row['unchanged_vs_master'] and observations == read('master/' + stem + '-results.json')
            assert blob(revision, known) == blob(master, known)
    positive = read('patch_5_3_0_publication_sweep-results.json')
    historical = json.loads((HISTORY / 'patch_5_3_0_publication_sweep-results.json').read_text())
    assert positive == historical, '5.3.0 changed without attributable supersession'
    negative = read('negative-results.json')
    assert set(negative) == set(positive)
    control = context['negative_id']
    assert positive[control]['ok'] and not negative[control]['ok']
    assert all(negative[key] == value for key, value in positive.items() if key != control)
    assert sum(not row['ok'] for row in negative.values()) == 16
    assert read('mists-line-control-results.json') == read('master/mists-line-control-results.json')
    for name in ('all-sweeps', 'master-all-sweeps', 'prefork-patch_5_3_0',
                 'integration-patch_5_3_0', 'mists-all-sweeps', 'master-mists-all-sweeps', 'own-sweep'):
        receipt = read(name + '.proof.json')
        assert receipt['exit'] == 0 and not receipt['invalidated']
        match = re.search(r'test result: ok\. (\d+) passed', (HERE / receipt['log']).read_text())
        assert match and int(match[1]) > 0, ('empty acceptance', name)
    for name in ('mists-all-sweeps', 'master-mists-all-sweeps'):
        assert 'test patch_5_5_4_publication_sweep::patch_5_5_4_client_line_excludes_retail_and_era ... ok' in (HERE / (name + '.txt')).read_text()
    for name in context['proofs']:
        receipt = read(name + '.proof.json')
        assert receipt['command'] and not receipt['invalidated']
        assert receipt['exit'] in (1, 101) if name == 'negative' else receipt['exit'] == 0
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        changed = git('diff', '--name-only', receipt['revision'], revision, '--',
                      'src', 'tests', 'Cargo.toml', 'Cargo.lock', 'build.rs').decode().splitlines()
        allowed = {'tests/data/patch_5_3_0_sweep_known_gaps.json',
                   'tests/patch_5_3_0_behavior.rs', 'tests/patch_5_3_0_publication_sweep.rs'}
        assert set(changed) <= (allowed if name.startswith('master-') else set()), (name, changed)
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all(line.startswith('warning: iced-wgpu-patched/Cargo.toml:') or
               line == 'warning: `iced_wgpu` (manifest) generated 6 warnings' for line in warnings), warnings
    inventory = read('prior-validator-inventory.json')
    assert inventory['revision'] == master
    assert inventory['validators'] == sorted(path for path in names(master, 'data/patch-api/evidence')
                                            if path.endswith('/validate.py'))
    assert all(path.stat().st_size < 5_000_000 for path in HISTORY.rglob('*') if path.is_file())
    print(json.dumps({'status': 'PASS', 'registers': len(registers),
                      'extracts': sum(row['byte_identical'] for row in extracts),
                      'sweeps': len(pages), 'publication_gaps': 15,
                      'prior_validators': len(inventory['validators'])}))


if __name__ == '__main__':
    main()
