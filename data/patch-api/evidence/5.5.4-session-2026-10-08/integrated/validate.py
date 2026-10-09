"""Validate integrated Mists proof using own receipts and Git-pinned shared inputs."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent

sys.path.insert(0, str(ROOT / 'tools'))
assert hashlib.sha256((ROOT / 'tools/patch_audit_pin_trees.py').read_bytes()).hexdigest() == 'd0405c9e3fb49ced8bfe73ce0f42a566e1f1a9d21ec74a12e26d24bcd2b62e6a', 'compact helper tamper'
from patch_audit_pin_trees import ProofGit, expand_pinned_inputs
PROOF_GIT = ProofGit(ROOT, HERE, json.loads((HERE / 'rebase-mapping.json').read_text()))


def git(*args, input=None):
    return PROOF_GIT(*args, input=input)


def digest(content):
    return hashlib.sha256(content).hexdigest()


def read(name):
    return json.loads((HERE / name).read_text())


def blob(revision, path):
    return git('show', revision + ':' + path)


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def verify_history():
    mapping = read('rebase-mapping.json')
    mapping['pinned_inputs'] = expand_pinned_inputs(ROOT, HERE, mapping['pinned_inputs'])
    for row in mapping['commits'] + mapping['external_commits']:
        revision = row['rebased_revision']
        assert git('show', '-s', '--format=%s', revision).decode().strip() == row['subject']
        assert git('rev-parse', revision + '^{tree}').decode().strip() == row['rebased_tree']
        patch_id = git('patch-id', '--stable', input=git('show', '--format=', '--binary', revision)).decode().split()[0]
        assert patch_id == row['rebased_patch_id']
    needed = sorted({row['rebased_blob'] for scope in mapping['pinned_inputs']
                     for row in scope['inputs'].values() if 'historical_file' not in row})
    raw = git('cat-file', '--batch', input=('\n'.join(needed) + '\n').encode())
    objects, offset = {}, 0
    for oid in needed:
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
        for directory, inventory in scope['inventories'].items():
            assert inventory == sorted(path for path in scope['inputs'] if path.startswith(directory + '/'))
        for path, row in scope['inputs'].items():
            assert tree[path] == row['rebased_blob'], path
            content = ((HERE / row['historical_file']).read_bytes() if 'historical_file' in row
                       else objects[row['rebased_blob']])
            assert digest(content) == row['sha256'], path
            assert hashlib.sha1(b'blob ' + str(len(content)).encode() + b'\0' + content).hexdigest() == row['recorded_blob'], path
    for path, expected in read('historical-preservation.json').items():
        current = HERE / 'historical-validator.py.txt' if path.endswith('/validate.py') else ROOT / path
        assert current.is_relative_to(HERE.parent)
        assert digest(current.read_bytes()) == expected, path
    return len(mapping['commits'])


def verify_sources(context):
    revision, base = context['runtime_revision'], context['master_revision']
    report = read('p554-reproduction.json')
    assert report['revision'] == revision and report['historical_flags_revision'] == base
    expected = {Path(path).name.removesuffix('-wikitext-register.json') for path in names(revision, 'data/patch-api/sources') if path.endswith('-wikitext-register.json')}
    assert {row['patch'] for row in report['records']} == expected
    prefix = 'data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/'
    prior_registers = {row['patch']: row for row in json.loads(blob(base, prefix + 'p548-register-reproduction.json'))}
    prior_extracts = {row['patch']: row for row in json.loads(blob(base, prefix + 'p548-saved-extract-reproduction.json'))}
    for row in report['records']:
        patch = row['patch']
        source = f'data/patch-api/sources/{patch}-'
        provenance = json.loads(blob(revision, source + 'api-changes.provenance.json'))
        assert row['generator_flags'] == provenance.get('generator_flags', prior_registers.get(patch, {}).get('verified_flags', []))
        assert row['extractor_flags'] == provenance.get('extractor_flags', prior_extracts.get(patch, {}).get('verified_flags', []))
        assert row['register_exit'] == 0 and row['register_byte_identical'], patch
        assert row['register_sha256'] == digest(blob(revision, source + 'wikitext-register.json'))
        assert row['extract_sha256'] == digest(blob(revision, source + 'api-changes.txt'))
        if patch in prior_extracts:
            before = prior_extracts[patch]
            assert (row['generated_extract_sha256'], row['generated_extract_error'], row['extract_byte_identical']) == (before['sha256'], before['error'], before['byte_identical']), patch
        else:
            assert row['extract_byte_identical'] and row['generated_extract_error'] is None
    for path in names(base, 'data/patch-api/sources'):
        assert blob(base, path) == blob(revision, path), path
    assert sorted(row['patch'] for row in report['records'] if not row['extract_byte_identical']) == ['12.0.5', '12.0.7', '12.1.0']
    assert not git('diff', base, revision, '--', 'Interface')
    return {'registers': len(expected), 'extracts': sum(row['extract_byte_identical'] for row in report['records'])}


def verify_sweeps(context):
    revision, base = context['runtime_revision'], context['master_revision']
    pages = [Path(path).stem for path in names(base, 'tests') if path.endswith('_publication_sweep.rs')]
    comparison = []
    for stem in pages:
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        before, after = read('master/' + stem + '-results.json'), read('results/' + stem + '-results.json')
        assert before == after, patch
        register = json.loads(blob(base, f'data/patch-api/sources/{patch}-wikitext-register.json'))
        assert 'client_line' not in register, patch
        assert set(after) == {row['id'] for row in register['entries']}, patch
        gaps = sorted(key for key, row in after.items() if not row['ok'])
        assert gaps == sorted(json.loads(blob(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')))
        comparison.append({'patch': patch, 'observations': len(after), 'gaps': gaps, 'identical': True})
    for label in ['all-sweeps', 'master-all-sweeps']:
        passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / (label + '.txt')).read_text(), re.M))
        for stem in pages:
            assert any(case.endswith('::' + stem) for case in passed), (label, stem)
    expected = {'master_revision': base, 'runtime_revision': revision, 'pages': comparison,
                'page_count': len(pages), 'observations': sum(row['observations'] for row in comparison), 'differences': []}
    assert read('gap-comparison.json') == expected
    assert read('results/patch_5_5_4_publication_sweep-results.json') == {}
    control = read('results/patch_5_5_4_publication_sweep-MISTS_LINE_CONTROL_OUT-results.json')
    assert control['own']['expected']['publication'] == 'absent' and not control['own']['ok']
    negative = (HERE / 'negative.txt').read_text()
    assert 'register row count changed' in negative and 'left: 1' in negative and 'right: 0' in negative
    return {'pages': len(pages), 'observations': expected['observations'], 'differences': 0}


def verify_receipts(context):
    for label, policy in context['proofs'].items():
        receipt = read(label + '.proof.json')
        assert receipt['exit'] == policy['exit'] == receipt['expected_exit'], label
        assert receipt['command'] == policy['command'], label
        content = (HERE / receipt['log']).read_bytes()
        assert digest(content) == receipt['log_sha256'], label
        expected_revision = context['master_revision'] if label.startswith('master-') else context['runtime_revision']
        assert not git('diff', receipt['revision'], expected_revision, '--',
                       'src', 'tests', 'tools', 'Cargo.toml', 'Cargo.lock', 'build.rs'), label
        if 'passed' in policy:
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', content.decode())
            assert counts and int(counts[-1]) == policy['passed'], (label, counts)
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    tools = (HERE / 'tools-tests.txt').read_text()
    assert 'Ran 87 tests' in tools and tools.rstrip().endswith('OK')
    inventory = read('prior-validator-inventory.json')
    assert inventory['revision'] == context['master_revision']
    expected = [path for path in names(inventory['revision'], 'data/patch-api/evidence') if path.endswith('/validate.py')]
    assert inventory['paths'] == expected
    results = read('prior-results.json')
    assert [row['path'] for row in results] == expected
    for row in results:
        assert row['exit'] == 0 and digest((HERE / row['log']).read_bytes()) == row['log_sha256']
    return len(expected)


def main():
    context = read('context.json')
    for name, expected in context['session_sha256'].items():
        path = HERE / name
        assert path.is_relative_to(HERE) and '..' not in Path(name).parts
        assert digest(path.read_bytes()) == expected, name
    for path, expected in context['shared_sha256'].items():
        assert digest(blob(context['runtime_revision'], path)) == expected, path
    commits = verify_history()
    sources = verify_sources(context)
    retail = verify_sweeps(context)
    prior = verify_receipts(context)
    print(json.dumps({'status': 'PASS', 'rebase_commits': commits, 'reproduction': sources,
                      'retail': retail, 'prior_validators': prior}, sort_keys=True))


if __name__ == '__main__':
    main()
