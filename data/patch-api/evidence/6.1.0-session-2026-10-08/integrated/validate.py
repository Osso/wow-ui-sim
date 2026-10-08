"""Read-only integrated 6.1.0 proof: own receipts live, shared files pinned."""
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


def read(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(*args, input=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=input)


def blob(revision, path):
    return git('show', revision + ':' + path)


def names(revision, directory):
    return git('ls-tree', '-r', '--name-only', revision, directory).decode().splitlines()


def pinned(revision, path):
    return json.loads(blob(revision, path))


def check_mapping(context):
    mapping = read(HERE / 'rebase-mapping.json')
    assert mapping['rebased_base'] == context['master_revision']
    for row in mapping['commits']:
        revision = row['rebased_revision']
        assert git('show', '-s', '--format=%s', revision).decode().strip() == row['subject']
        assert git('rev-parse', revision + '^{tree}').decode().strip() == row['rebased_tree']
        patch = git('show', '--format=', '--binary', revision)
        assert git('patch-id', '--stable', input=patch).decode().split()[0] == row['rebased_patch_id']
        for path, expected in row['blobs'].items():
            exception = row['historical_exceptions'].get(path)
            if exception:
                content = (HERE / exception['file']).read_bytes()
                assert digest(content) == exception['sha256']
                assert git('hash-object', '--stdin', input=content).decode().strip() == expected
            else:
                assert git('rev-parse', revision + ':' + path).decode().strip() == expected
        before = set(row['inventories']['data/patch-api/sources'])
        after = set(names(revision, 'data/patch-api/sources'))
        assert before <= after
        assert all(Path(path).name.startswith(('6.2.0-', '6.2.2-', '6.2.4-')) for path in after - before)
        before = {path for path in row['inventories']['tests'] if path.endswith('_publication_sweep.rs')}
        after = {path for path in names(revision, 'tests') if path.endswith('_publication_sweep.rs')}
        assert before <= after
        assert after - before <= {'tests/patch_6_2_0_publication_sweep.rs',
                                  'tests/patch_6_2_2_publication_sweep.rs',
                                  'tests/patch_6_2_4_publication_sweep.rs'}


def check_preservation(context):
    for path, expected in read(HERE / 'historical-preservation.json').items():
        current = HERE / 'historical-validator.py.txt' if path.endswith('/validate.py') else ROOT / path
        assert current.is_relative_to(HISTORY)
        assert digest(current.read_bytes()) == expected, path
    preserved = read(HERE / 'p610-input-preservation.json')
    assert preserved['base_revision'] == '787b47591'
    assert {row['path'] for row in preserved['rows']} == set(names(context['master_revision'], 'data/patch-api/sources'))
    for row in preserved['rows']:
        assert digest(blob(context['master_revision'], row['path'])) == row['before_sha256']
        assert digest(blob(context['runtime_revision'], row['path'])) == row['after_sha256'] == row['before_sha256']
    assert all(row['before'] == row['after'] for row in read(HERE / 'p610-extract-preservation.json'))
    assert git('rev-parse', context['master_revision'] + ':src') == git('rev-parse', context['runtime_revision'] + ':src')


def check_reproduction(context):
    revision = context['runtime_revision']
    patches = {Path(path).name.removesuffix('-wikitext-register.json')
               for path in names(revision, 'data/patch-api/sources') if path.endswith('-wikitext-register.json')}
    registers = read(HERE / 'p610-register-reproduction.json')
    extracts = read(HERE / 'p610-saved-extract-reproduction.json')
    previous = 'data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/'
    prior_registers = {row['patch']: row for row in pinned(context['master_revision'], previous + 'p620-register-reproduction.json')}
    prior_extracts = {row['patch']: row for row in pinned(context['master_revision'], previous + 'p620-saved-extract-reproduction.json')}
    assert {row['patch'] for row in registers} == {row['patch'] for row in extracts} == patches
    for row in registers:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('generator_flags', prior_registers.get(patch, {}).get('verified_flags', []))
        assert row['exit'] == 0 and row['byte_identical']
        assert row['sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-wikitext-register.json'))
    for row in extracts:
        patch = row['patch']
        provenance = pinned(revision, f'data/patch-api/sources/{patch}-api-changes.provenance.json')
        assert row['verified_flags'] == provenance.get('extractor_flags', prior_extracts.get(patch, {}).get('verified_flags', []))
        assert row['saved_sha256'] == digest(blob(revision, f'data/patch-api/sources/{patch}-api-changes.txt'))
        if patch in prior_extracts:
            assert all(row[key] == prior_extracts[patch][key] for key in ('byte_identical', 'error', 'sha256', 'saved_sha256'))
        else:
            assert row['byte_identical'] and row['error'] is None
    assert sorted(row['patch'] for row in extracts if not row['byte_identical']) == ['12.0.5', '12.0.7', '12.1.0']
    return len(registers), sum(row['byte_identical'] for row in extracts)


def check_sweeps(context):
    revision = context['runtime_revision']
    passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / 'all-sweeps.txt').read_text(), re.M))
    master_passed = set(re.findall(r'^test (\S+) \.\.\. ok$', (HERE / 'master-all-sweeps.txt').read_text(), re.M))
    pages = []
    for path in names(revision, 'tests'):
        if not path.endswith('_publication_sweep.rs'):
            continue
        stem = Path(path).stem
        patch = stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(revision, path).decode())
        assert functions and all(any(case.endswith('::' + function) for case in passed) for function in functions)
        results = read(HERE / (stem + '-results.json'))
        register = pinned(revision, f'data/patch-api/sources/{patch}-wikitext-register.json')
        known = pinned(revision, f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        gaps = sorted(key for key, value in results.items() if not value['ok'])
        assert gaps == sorted(known)
        assert set(results) == {row['id'] for row in register['entries']}
        if patch != '6.1.0':
            assert all(any(case.endswith('::' + function) for case in master_passed) for function in functions)
            assert results == read(HERE / 'master' / (stem + '-results.json')), patch
        pages.append({'patch': patch, 'observations': len(results), 'gaps': gaps,
                      'unchanged_vs_master': patch != '6.1.0'})
    assert read(HERE / 'gap-comparison.json') == pages
    positive = read(HERE / 'patch_6_1_0_publication_sweep-results.json')
    historical = read(HISTORY / 'patch_6_1_0_publication_sweep-results.json')
    assert positive == historical
    assert read(HERE / 'supersession-review.json') == {
        'before_gaps': ['wt-global-api-SendChatMessage-14'],
        'after_gaps': ['wt-global-api-SendChatMessage-14'], 'replacements': [],
        'later_intersections': {'6.2.0': [], '6.2.2': [], '6.2.4': []},
        'pending_extract_contracts': 8}
    negative = read(HERE / 'negative-results.json')
    assert {key for key, value in negative.items() if not value['ok']} == {
        'p610-negative-control', 'wt-global-api-SendChatMessage-14'}
    assert {key: value for key, value in negative.items() if key != 'p610-negative-control'} == {
        key: value for key, value in positive.items() if key != 'wt-global-api-DeathRecap_HasEvents-7'}
    assert '__P610MissingAPI' in (HERE / 'negative-register.json').read_text()
    return len(pages), len(passed)


def check_commands(context):
    required = {'own-sweep', 'all-sweeps', 'prefork-patch', 'prefork-recap', 'integration-recap',
                'integration-absence', 'format', 'mists-check', 'negative', 'master-all-sweeps'}
    required |= {Path(path).stem for path in names(context['runtime_revision'], 'tools')
                 if Path(path).parent == Path('tools')
                 and Path(path).name.startswith('test_') and path.endswith('.py')}
    assert required <= set(context['receipts'])
    for label, expected in context['receipts'].items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt == expected
        assert digest((HERE / receipt['log']).read_bytes()) == receipt['log_sha256']
        assert receipt['exit'] == (1 if label == 'negative' else 0), label
        revision = context['master_revision'] if label == 'master-all-sweeps' else context['runtime_revision']
        for directory in ('src', 'tests', 'tools'):
            assert git('rev-parse', receipt['revision'] + ':' + directory) == git('rev-parse', revision + ':' + directory)
        if receipt['command'][:2] == ['cargo', 'test'] and label != 'negative':
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', (HERE / receipt['log']).read_text())
            assert counts and sum(map(int, counts)) > 0, label
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    matrix = read(HERE / 'prior-validator-matrix.json')
    paths = {path for path in names(context['master_revision'], 'data/patch-api/evidence')
             if Path(path).name in ('validate.py', 'validate_integrated.py')}
    assert {row['path'] for row in matrix} == paths
    for row in matrix:
        assert row['exit'] == 0
        assert digest((HERE / row['log']).read_bytes()) == row['log_sha256']
        assert digest(blob(context['master_revision'], row['path'])) == row['code_sha256']
    return len(matrix)


def main():
    context = read(HERE / 'context.json')
    for path, expected in read(HERE / 'artifact-hashes.json').items():
        assert digest((HERE / path).read_bytes()) == expected, path
    check_mapping(context)
    check_preservation(context)
    registers, extracts = check_reproduction(context)
    pages, cases = check_sweeps(context)
    prior = check_commands(context)
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts,
                      'pages': pages, 'cases': cases, 'prior_validators': prior}, sort_keys=True))


if __name__ == '__main__':
    main()
