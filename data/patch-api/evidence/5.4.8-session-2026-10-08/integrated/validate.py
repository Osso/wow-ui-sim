"""Read-only integrated 5.4.8 proof; all shared inputs resolve at pinned Git revisions."""
import hashlib
import json
from pathlib import Path
import re
import runpy
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORY = HERE.parent


def read(name):
    return json.loads((HERE / name).read_text())


def digest(value):
    return hashlib.sha256(value).hexdigest()


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def blob(revision, path):
    return git('show', revision + ':' + path)


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def pinned(revision, path):
    return json.loads(blob(revision, path))


def verify_mapping():
    mapping = read('rebase-mapping.json')
    for row in mapping['commits'] + mapping['external_commits']:
        revision = row['rebased_revision']
        assert git('show', '-s', '--format=%s', revision).decode().strip() == row['subject']
        assert git('rev-parse', revision + '^{tree}').decode().strip() == row['rebased_tree']
        patch = git('show', '--format=', '--binary', revision)
        assert git('patch-id', '--stable', input=patch).decode().split()[0] == row['rebased_patch_id']
    objects = {}
    needed = {row['rebased_blob'] for scope in mapping['pinned_inputs'] for row in scope['inputs'].values()
              if 'historical_file' not in row}
    raw = git('cat-file', '--batch', input=('\n'.join(sorted(needed)) + '\n').encode())
    offset = 0
    for oid in sorted(needed):
        end = raw.index(b'\n', offset)
        header = raw[offset:end].decode().split()
        assert header[:2] == [oid, 'blob']
        size = int(header[2])
        offset = end + 1
        objects[oid] = raw[offset:offset + size]
        offset += size + 1
    for scope in mapping['pinned_inputs']:
        tree = {line.split('\t', 1)[1]: line.split('\t', 1)[0].split()[2]
                for line in git('ls-tree', '-r', scope['rebased_revision']).decode().splitlines()}
        for path, row in scope['inputs'].items():
            assert tree[path] == row['rebased_blob'], path
            content = (HERE / row['historical_file']).read_bytes() if 'historical_file' in row else objects[row['rebased_blob']]
            assert digest(content) == row['sha256'], path
            assert hashlib.sha1(b'blob ' + str(len(content)).encode() + b'\0' + content).hexdigest() == row['recorded_blob']
    for path, expected in read('historical-preservation.json').items():
        current = HERE / 'historical-validator.py.txt' if path.endswith('/validate.py') else ROOT / path
        assert current.is_relative_to(HISTORY)
        assert digest(current.read_bytes()) == expected, path
    return len(mapping['commits']), len(mapping['external_commits'])


def verify_sources(context):
    revision = context['runtime_revision']
    registers = read('p548-register-reproduction.json')
    extracts = read('p548-saved-extract-reproduction.json')
    expected = {Path(p).name.removesuffix('-wikitext-register.json') for p in names(revision, 'data/patch-api/sources') if p.endswith('-wikitext-register.json')}
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    prior_registers = {row['patch']: row for row in pinned(context['master_revision'], 'data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/p602-register-reproduction.json')}
    for row in registers:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('generator_flags', prior_registers.get(patch, {}).get('verified_flags', []))
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
    previous = pinned(context['master_revision'], 'data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/p602-saved-extract-reproduction.json')
    inherited = {row['patch']: row for row in previous}
    for row in extracts:
        provenance = pinned(revision, f'data/patch-api/sources/{row["patch"]}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('extractor_flags', inherited.get(row['patch'], {}).get('verified_flags', []))
        assert row['saved_sha256'] == digest(blob(revision, f'data/patch-api/sources/{row["patch"]}-api-changes.txt'))
        if row['patch'] in inherited:
            assert all(row[key] == inherited[row['patch']][key] for key in ['byte_identical', 'error', 'sha256', 'saved_sha256'])
        else:
            assert row['byte_identical'] and row['error'] is None
    assert sorted(row['patch'] for row in extracts if not row['byte_identical']) == ['12.0.5', '12.0.7', '12.1.0']
    preservation = read('p548-input-preservation.json')
    assert {row['path'] for row in preservation['rows']} == set(names(context['master_revision'], 'data/patch-api/sources'))
    for row in preservation['rows']:
        assert row['before_sha256'] == digest(blob(context['master_revision'], row['path']))
        assert row['after_sha256'] == digest(blob(revision, row['path'])) == row['before_sha256']
    assert all(row['before'] == row['after'] for row in read('p548-extract-preservation.json'))
    assert not git('diff', context['master_revision'], revision, '--', 'Interface')
    return len(registers), sum(row['byte_identical'] for row in extracts)


def verify_sweeps(context):
    pages = read('gap-comparison.json')
    expected = {Path(p).stem for p in names(context['runtime_revision'], 'tests') if p.endswith('_publication_sweep.rs')}
    assert expected == {'patch_' + row['patch'].replace('.', '_') + '_publication_sweep' for row in pages}
    for row in pages:
        stem = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        results = read(stem + '-results.json')
        register = pinned(context['runtime_revision'], f'data/patch-api/sources/{row["patch"]}-wikitext-register.json')
        assert set(results) == {entry['id'] for entry in register['entries']}
        assert len(results) == row['observations']
        assert row['gaps'] == sorted(k for k, value in results.items() if not value['ok'])
        assert row['gaps'] == sorted(pinned(context['runtime_revision'], 'tests/data/' + stem.removesuffix('_publication_sweep') + '_sweep_known_gaps.json'))
        if row['patch'] != '5.4.8':
            assert row['unchanged_vs_master'] and results == read('master/' + stem + '-results.json')
    positive = read('patch_5_4_8_publication_sweep-results.json')
    assert positive == json.loads((HISTORY / 'patch_5_4_8_publication_sweep-results.json').read_text())
    negative = read('negative-results.json')
    assert set(negative) == set(positive)
    changed = {k for k in positive if positive[k] != negative[k]}
    assert changed == {k for k, value in positive.items() if value['expected']['symbol'] == 'SetUIVisibility'}
    assert sum(not value['ok'] for value in negative.values()) == sum(not value['ok'] for value in positive.values()) + 1
    assert read('supersession-review.json') == {'before_gaps': 7, 'after_gaps': 7, 'observation_changes': {}, 'replacements': [], 'integrated_registers': ['6.0.1', '6.0.2']}
    for label in ['all-sweeps', 'master-all-sweeps']:
        passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / (label + '.txt')).read_text(), re.M))
        revision = context['master_revision'] if label.startswith('master-') else context['runtime_revision']
        for path in names(revision, 'tests'):
            if path.endswith('_publication_sweep.rs'):
                functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(revision, path).decode())
                assert all(any(case.endswith('::' + fn) for case in passed) for fn in functions)
    return len(pages)


def verify_receipts(context):
    receipts = read('receipts.json')
    assert set(context['required_receipts']) <= receipts.keys()
    for label, receipt in receipts.items():
        assert receipt == read(label + '.proof.json')
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256'], label
        if receipt['command'][:2] == ['cargo', 'test']:
            revision = context['master_revision'] if label.startswith('master-') else context['runtime_revision']
            for directory in ['src', 'tests', 'tools']:
                assert git('rev-parse', receipt['revision'] + ':' + directory) == git('rev-parse', revision + ':' + directory), label
    for label in context['required_receipts']:
        receipt = receipts[label]
        expected = context['expected_failures'].get(label, 0)
        assert receipt['exit'] == expected, (label, receipt['exit'], expected)
    summarizer = runpy.run_path(str(HERE / 'summarize_proofs.py'))
    for row in read('regression-comparison.json'):
        checks = row.get('checks', [row])
        for check in checks:
            for side in ['branch', 'master']:
                label = check[side]['receipt'].removesuffix('.proof.json')
                assert check[side] == summarizer['result'](label)
            assert check['branch']['failures'] == check['master']['failures']
            assert check['branch']['panics'] == check['master']['panics']
    failures = read('integration-failure-comparison.json')
    assert failures['identical_assertion_messages'] and failures['branch'] == failures['master']
    assert all(value['errors'] == [] and value['addons_enabled'] for value in read('startup-comparison.json').values())
    for label in ['branch-startup', 'master-startup']:
        assert '--no-addons' not in receipts[label]['command']
        assert 'WOW_SIM_NO_ADDONS' not in receipts[label]['environment']
        assert (HERE / receipts[label]['log']).read_text().rstrip().endswith('[]')
    warnings = (HERE / 'mists-check.txt').read_text().splitlines()
    assert not [line for line in warnings if line.startswith('warning:') and 'iced-wgpu-patched/Cargo.toml:' not in line and '`iced_wgpu` (manifest)' not in line]
    inventory = read('prior-validator-inventory.json')
    assert inventory['revision'] == context['master_revision']
    assert inventory['validators'] == [p for p in names(context['master_revision'], 'data/patch-api/evidence') if p.endswith('/validate.py')]
    prior = read('prior-results.json')
    assert [row['validator'] for row in prior] == inventory['validators']
    assert all(row['exit'] == 0 and receipts[row['receipt'].removesuffix('.proof.json')]['exit'] == 0 for row in prior)
    for row in read('caller-scans.json'):
        content = (HERE / row['output']).read_bytes()
        assert row['command'][0] == '/usr/bin/grep' and row['exit'] == 0 and not row['stderr']
        assert digest(content) == row['sha256'] and len(content.decode().splitlines()) == row['lines']
    for row in read('combat-fixtures.json'):
        assert row['sha256'] == digest(blob(context['runtime_revision'], row['path']))
    return len(receipts), len(prior)


if __name__ == '__main__':
    context = read('context.json')
    wrapper = HISTORY / 'validate.py'
    git('ls-files', '--error-unmatch', wrapper.relative_to(ROOT).as_posix())
    assert digest(wrapper.read_bytes()) == context['wrapper_sha256'], 'own historical replay wrapper tamper'
    for path, expected in context['seals'].items():
        target = HERE / path
        assert target.is_relative_to(HERE)
        git('ls-files', '--error-unmatch', target.relative_to(ROOT).as_posix())
        assert digest(target.read_bytes()) == expected, 'own evidence tamper: ' + path
    commits, external = verify_mapping()
    registers, extracts = verify_sources(context)
    pages = verify_sweeps(context)
    receipts, prior = verify_receipts(context)
    print(json.dumps({'status': 'PASS', 'mapped_commits': commits, 'external_commits': external,
                      'registers': registers, 'extracts': extracts, 'inherited_extract_failures': ['12.0.5', '12.0.7', '12.1.0'],
                      'sweep_pages': pages, 'publication_gaps': 7, 'prior_validators': prior, 'receipts': receipts}, sort_keys=True))
