"""Read-only integrated 5.4.1 proof; all shared inputs resolve at pinned Git revisions."""
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
        if row['recorded_patch_id'] != row['rebased_patch_id']:
            patch = (HERE / row['historical_patch']).read_bytes()
            assert git('patch-id', '--stable', input=patch).decode().split()[0] == row['recorded_patch_id']
            assert row['conflict_reason'] and row['subject'] in ('Account 5.4.1 gaps and preserve bounded realm and retirement evidence', 'Pin retail 5.4.1 with opt-in Mists extraction and discovery')
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
            assert tree.get(path) == row['rebased_blob'], path
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
    registers = read('p541-register-reproduction.json')
    extracts = read('p541-saved-extract-reproduction.json')
    expected = {Path(p).name.removesuffix('-wikitext-register.json') for p in names(revision, 'data/patch-api/sources') if p.endswith('-wikitext-register.json')}
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == expected
    prior = pinned(context['master_revision'], 'data/patch-api/evidence/5.5.3-session-2026-10-08/p553-reproduction.json')['records']
    prior_registers = {row['patch']: {'verified_flags': row['generator_flags']} for row in prior}
    for row in registers:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('generator_flags', prior_registers.get(patch, {}).get('verified_flags', []))
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
    inherited = {row['patch']: {'verified_flags': row['extractor_flags'], 'byte_identical': row['extract_byte_identical'], 'error': row['generated_extract_error'], 'sha256': row['generated_extract_sha256'], 'saved_sha256': row['extract_sha256']} for row in prior}
    for row in extracts:
        provenance = pinned(revision, f'data/patch-api/sources/{row["patch"]}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('extractor_flags', inherited.get(row['patch'], {}).get('verified_flags', []))
        assert row['saved_sha256'] == digest(blob(revision, f'data/patch-api/sources/{row["patch"]}-api-changes.txt'))
        if row['patch'] in inherited:
            assert all(row[key] == inherited[row['patch']][key] for key in ['byte_identical', 'error', 'sha256', 'saved_sha256'])
        else:
            assert row['byte_identical'] and row['error'] is None
    assert sorted(row['patch'] for row in extracts if not row['byte_identical']) == ['12.0.5', '12.0.7', '12.1.0']
    preservation = read('p541-input-preservation.json')
    assert {row['path'] for row in preservation['rows']} == set(names(context['master_revision'], 'data/patch-api/sources'))
    for row in preservation['rows']:
        assert row['before_sha256'] == digest(blob(context['master_revision'], row['path']))
        assert row['after_sha256'] == digest(blob(revision, row['path'])) == row['before_sha256']
    assert all(row['before'] == row['after'] for row in read('p541-extract-preservation.json'))
    assert not git('diff', context['master_revision'], revision, '--', 'Interface')
    return len(registers), sum(row['byte_identical'] for row in extracts)


def verify_sweeps(context):
    revision = context['runtime_revision']
    pages = read('gap-comparison.json')
    expected = {Path(path).stem for path in names(revision, 'tests') if path.endswith('_publication_sweep.rs')}
    assert expected == {'patch_' + row['patch'].replace('.', '_') + '_publication_sweep' for row in pages}
    master_expected = {Path(path).stem for path in names(context['master_revision'], 'tests') if path.endswith('_publication_sweep.rs')}
    assert master_expected == expected - {'patch_5_4_1_publication_sweep'}
    for row in pages:
        stem = 'patch_' + row['patch'].replace('.', '_') + '_publication_sweep'
        results = read(stem + '-results.json')
        register = pinned(revision, 'data/patch-api/sources/' + row['patch'] + '-wikitext-register.json')
        assert set(results) == {entry['id'] for entry in register['entries']}
        assert len(results) == row['observations']
        assert row['gaps'] == sorted(key for key, value in results.items() if not value['ok'])
        known = 'tests/data/' + stem.removesuffix('_publication_sweep') + '_sweep_known_gaps.json'
        assert row['gaps'] == sorted(pinned(revision, known))
        if row['patch'] != '5.4.1':
            assert row['unchanged_vs_master'] and results == read('master/' + stem + '-results.json')
            assert blob(revision, known) == blob(context['master_revision'], known)
    positive = read('patch_5_4_1_publication_sweep-results.json')
    historical = json.loads((HISTORY / 'patch_5_4_1_publication_sweep-results.json').read_text())
    negative = read('negative-results.json')
    assert set(negative) == set(positive)
    changed = {key for key in positive if positive[key] != negative[key]}
    assert changed == {'wt-global-api-C_ProductChoice.GetChoices-17'}
    assert positive[next(iter(changed))]['ok'] and not negative[next(iter(changed))]['ok']
    assert sum(not value['ok'] for value in negative.values()) == sum(not value['ok'] for value in positive.values()) + 1
    changes = {key: {'before': historical[key], 'after': positive[key]} for key in positive if positive[key] != historical[key]}
    review = read('supersession-review.json')
    assert review['observation_changes'] == changes
    assert review['before_gaps'] == sum(not value['ok'] for value in historical.values())
    assert review['after_gaps'] == sum(not value['ok'] for value in positive.values())
    # Later retail audits resolve neither the event producer nor selected-realm state gap.
    assert review['before_gaps'] == review['after_gaps'] == 2
    assert review['replacements'] == [] and review['integrated_registers'] == ['5.4.2', '5.4.7', '5.4.8']
    for label, pin in [('all-sweeps', revision), ('master-all-sweeps', context['master_revision']), ('mists-all-sweeps', revision), ('master-mists-all-sweeps', context['master_revision'])]:
        passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / (label + '.txt')).read_text(), re.M))
        for path in names(pin, 'tests'):
            if path.endswith('_publication_sweep.rs'):
                patch = Path(path).stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
                classic = pinned(pin, f'data/patch-api/sources/{patch}-wikitext-register.json').get('client_line') == 'mists-classic'
                if classic != ('mists-' in label):
                    continue
                functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(pin, path).decode())
                assert functions and all(any(case.endswith('::' + fn) for case in passed) for fn in functions)
    return len(pages)


def verify_receipts(context):
    receipts = read('receipts.json')
    assert set(context['required_receipts']) == set(receipts)
    for label, receipt in receipts.items():
        assert receipt == read(label + '.proof.json')
        assert not receipt['invalidated'] and receipt['scope'] == label
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256'], label
        assert receipt['exit'] == context['expected_exits'].get(label, 0), (label, receipt['exit'])
        revision = context['master_revision'] if label.startswith('master-') else context['runtime_revision']
        if receipt['command'][0] == 'cargo':
            directories = ['src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'Interface']
        elif label.startswith('test_') or label == 'reproduction':
            directories = ['tools', 'data/patch-api/sources']
        else:
            directories = []
        for directory in directories:
            assert git('rev-parse', receipt['revision'] + ':' + directory) == git('rev-parse', revision + ':' + directory), label
        if receipt['command'][:2] == ['cargo', 'test'] and label != 'negative':
            log = (HERE / receipt['log']).read_text()
            if label == 'integration-patch_5_4_1':
                assert 'running 0 tests' in log and 'test result: ok. 0 passed;' in log
            else:
                assert re.search(r'test result: ok\. [1-9][0-9]* passed;', log), label
    warnings = (HERE / 'mists-check.txt').read_text().splitlines()
    assert not [line for line in warnings if line.startswith('warning:') and 'iced-wgpu-patched/Cargo.toml:' not in line and '`iced_wgpu` (manifest)' not in line]
    inventory = read('prior-validator-inventory.json')
    assert inventory['revision'] == context['master_revision']
    assert inventory['validators'] == [path for path in names(context['master_revision'], 'data/patch-api/evidence') if path.endswith('/validate.py')]
    prior = read('prior-results.json')
    assert [row['validator'] for row in prior] == inventory['validators']
    for row in prior:
        assert row['exit'] == 0 and row['log_sha256'] == digest((HERE / row['log']).read_bytes())
    assert not git('diff', '--name-only', context['master_revision'], context['runtime_revision'], '--', 'src', 'Cargo.toml', 'Cargo.lock', 'Interface')
    assert set(git('diff', '--name-only', context['master_revision'], context['runtime_revision'], '--', 'tools').decode().splitlines()) == {'tools/extract_patch_non_inventory.py', 'tools/test_patch_mists_register.py'}
    return len(receipts), len(prior)


CONTEXT_SHA256 = 'pending-seal'


def main():
    assert digest((HERE / 'context.json').read_bytes()) == CONTEXT_SHA256, 'integrated context tamper'
    context = read('context.json')
    assert digest((HISTORY / 'validate.py').read_bytes()) == context['wrapper_sha256']
    tracked = set(names(context['seal_revision'], 'data/patch-api/evidence/5.4.1-session-2026-10-08'))
    for path, expected in context['seals'].items():
        target = HERE / path
        assert target.is_relative_to(HERE) and target.relative_to(ROOT).as_posix() in tracked
        assert digest(target.read_bytes()) == expected, 'own evidence tamper: ' + path
    commits, external = verify_mapping()
    registers, extracts = verify_sources(context)
    pages = verify_sweeps(context)
    receipts, prior = verify_receipts(context)
    print(json.dumps({'status': 'PASS', 'mapped_commits': commits, 'external_commits': external,
                      'registers': registers, 'extracts': extracts,
                      'inherited_extract_failures': ['12.0.5', '12.0.7', '12.1.0'],
                      'sweep_pages': pages, 'publication_gaps': 2, 'prior_validators': prior,
                      'receipts': receipts}, sort_keys=True))


if __name__ == '__main__':
    main()
