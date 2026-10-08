"""Read-only integrated 6.2.4 proof; historical artifacts and scopes remain sealed."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parent
HISTORICAL = HERE.parent
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def blob(revision, path):
    return git('show', revision + ':' + path)


def git_scope(revision, names):
    tree = {}
    for line in git('ls-tree', '-r', revision).decode().splitlines():
        metadata, name = line.split('\t', 1)
        if name in names:
            tree[name] = metadata.split()[2]
    assert set(tree) == set(names), 'missing proof input at ' + revision
    return tree


def blob_oid(path):
    contents = path.read_bytes()
    return hashlib.sha1(b'blob ' + str(len(contents)).encode() + b'\0' + contents).hexdigest()


def check_preservation(context):
    for name, expected in read(HERE / 'historical-preservation.json').items():
        assert digest(ROOT / name) == expected, 'historical artifact changed: ' + name
    preserved = read(HERE / 'p624-input-preservation.json')
    names = git('ls-tree', '-r', '--name-only', context['master_revision'],
                'data/patch-api/sources').decode().splitlines()
    assert {row['path'] for row in preserved['rows']} == set(names)
    for row in preserved['rows']:
        original = hashlib.sha256(blob(context['master_revision'], row['path'])).hexdigest()
        assert original == row['before_sha256'] == row['after_sha256'] == digest(ROOT / row['path'])
    assert all(row['before'] == row['after'] for row in read(HERE / 'p624-extract-preservation.json'))
    assert git_scope(context['runtime_revision'], context['input_scope']) == context['input_scope']
    for name, expected in context['input_scope'].items():
        assert blob_oid(ROOT / name) == expected, 'proof invalidated: ' + name
    # Retain the original 7.0.3 validator and seal table; only its own validator
    # hash changes to admit four exact, committed opt-in tool replacements.
    before = read(HERE / 'p703-before-artifact-hashes.json.txt')
    directory = ROOT / 'data/patch-api/evidence/7.0.3-session-2026-10-08/integrated'
    after = read(directory / 'artifact-hashes.json')
    own = (directory / 'validate.py').relative_to(ROOT).as_posix()
    assert set(before) == set(after)
    assert {name for name in before if before[name] != after[name]} == {own}
    assert after[own] == digest(ROOT / own)
    original = blob(context['master_revision'], own)
    assert original == (HERE / 'p703-before-validate.py.txt').read_bytes()
    assert before[own] == hashlib.sha256(original).hexdigest()
    assert blob(context['master_revision'], (directory / 'artifact-hashes.json').relative_to(ROOT).as_posix()) == (HERE / 'p703-before-artifact-hashes.json.txt').read_bytes()
    replacements = read(HERE / 'tool-replacements.json')
    for name, pair in replacements['replacements'].items():
        assert pair == [hashlib.sha256(blob(replacements[revision], name)).hexdigest()
                        for revision in ('before_revision', 'after_revision')]
        assert pair[1] == digest(ROOT / name)
    assert not git('diff', '--name-only', context['master_revision'], context['runtime_revision'], '--', 'src')
    return len(names)


def check_reproduction(context):
    paths = historical_registers(ROOT, context['runtime_revision'])
    patches = {p.name.removesuffix('-wikitext-register.json') for p in paths}
    registers = read(HERE / 'p624-register-reproduction.json')
    extracts = read(HERE / 'p624-saved-extract-reproduction.json')
    extension = read(HERE / 'extension/p624-register-reproduction.json')
    inherited = {row['patch']: row for row in read(HISTORICAL / 'p624-saved-extract-reproduction.json')}
    prior_registers = {row['patch']: row for row in read(HISTORICAL / 'p624-register-reproduction.json')}
    for rows in (registers, extracts, extension):
        assert len(rows) == len(patches) and {row['patch'] for row in rows} == patches
    for row in registers + extension:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-wikitext-register.json')
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        flags = provenance.get('generator_flags', prior_registers.get(row['patch'], {}).get('verified_flags', []))
        assert row['verified_flags'] == flags
        assert row['exit'] == 0 and row['byte_identical'] and row['sha256'] == digest(path)
    for row in extracts:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-api-changes.txt')
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        flags = provenance.get('extractor_flags', inherited.get(row['patch'], {}).get('verified_flags', []))
        assert row['verified_flags'] == flags and row['saved_sha256'] == digest(path)
        if row['patch'] in inherited:
            previous = inherited[row['patch']]
            assert all(row[key] == previous[key] for key in ('byte_identical', 'sha256', 'saved_sha256', 'error'))
        else:
            assert row['byte_identical'] and row['error'] is None
    review = read(HERE / 'supersession-review.json')
    own = read(ROOT / 'data/patch-api/sources/6.2.4-wikitext-register.json')['entries']
    for patch in ('7.0.1', '7.0.3'):
        later = read(ROOT / 'data/patch-api/sources' / (patch + '-wikitext-register.json'))['entries']
        intersection = [row for row in later if row['symbol'] in {entry['symbol'] for entry in own}]
        assert intersection == review['later_intersections'][patch] == []
    assert review['before_gaps'] == review['after_gaps'] == 0 and review['replacements'] == []
    return len(patches), sum(row['byte_identical'] for row in extracts)


def passed_cases(path):
    return set(re.findall(r'^test (\S+) \.\.\. ok$', path.read_text(), re.M))


def comparison(context):
    paths = historical_sweep_tests(ROOT, context['runtime_revision'])
    master_paths = {p.stem: p for p in historical_sweep_tests(ROOT, context['master_revision'])}
    summaries = {row['patch']: row for row in read(HERE / 'extension/p624-sweep-summary.json')}
    passed = passed_cases(HERE / 'all-sweeps.txt')
    master_passed = passed_cases(HERE / 'master-all-sweeps.txt')
    pages = []
    for path in paths:
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        relative = path.relative_to(ROOT).as_posix()
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(context['runtime_revision'], relative).decode())
        assert functions and all(any(case.endswith('::' + name) for case in passed) for name in functions)
        results = read(HERE / (path.stem + '-results.json'))
        known_path = 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'
        known = json.loads(blob(context['runtime_revision'], known_path))
        register = json.loads(blob(context['runtime_revision'], 'data/patch-api/sources/' + patch + '-wikitext-register.json'))
        gaps = sorted(key for key, row in results.items() if not row['ok'])
        assert gaps == sorted(known) and set(results) == {row['id'] for row in register['entries']}
        assert summaries[patch] == {'patch': patch, 'rows': len(results), 'ok': len(results) - len(gaps), 'gaps': len(gaps), 'result': 'pass'}
        before = read(HERE / 'master' / (path.stem + '-results.json')) if path.stem in master_paths else None
        if before is not None:
            assert json.loads(blob(context['master_revision'], known_path)) == known
            assert before == results, 'master observations changed: ' + patch
            assert all(any(case.endswith('::' + name) for case in master_passed) for name in functions)
        else:
            assert patch == '6.2.4' and gaps == []
        pages.append({'patch': patch, 'rows': len(results), 'master_rows': len(before) if before is not None else None,
                      'master_gaps': sorted(key for key, row in before.items() if not row['ok']) if before is not None else None,
                      'branch_gaps': gaps, 'changed_ok_ids': [], 'changed_observation_ids': [],
                      'status': 'unchanged' if before is not None else 'new audit; historical zero gaps unchanged'})
    assert len(summaries) == len(paths)
    assert len(passed) == len(paths) + 1 and len(master_passed) == len(master_paths) + 1
    return {'master_revision': context['master_revision'], 'runtime_revision': context['runtime_revision'],
            'pages': pages, 'changes': [], 'master_cases': len(master_passed), 'branch_cases': len(passed),
            'observations': sum(row['rows'] for row in pages)}


def check_receipts(context):
    expected = {label: 0 for label in ('own-sweep', 'all-sweeps', 'prefork-patch-6-2-4', 'integration-patch-6-2-4',
                'bnet-model', 'deprecated-bnet', 'extend-receipts', 'master-all-sweeps', 'format', 'mists-check')}
    expected.update({tool + '-fixtures': 0 for tool in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation')})
    expected['negative'] = 1
    assert read(HERE / 'command-results.json') == expected
    for label, code in expected.items():
        receipt = read(HERE / (label + '.proof.json'))
        assert receipt['exit'] == code and digest(HERE / receipt['log']) == receipt['log_sha256'], label
        git('rev-parse', '--verify', receipt['revision'] + '^{commit}')
        if label == 'master-all-sweeps':
            assert receipt['source_revision'] == context['master_revision']
        else:
            assert git_scope(receipt['source_revision'], context['input_scope']) == context['input_scope'], label
        if receipt['command'][:2] == ['cargo', 'test'] and code == 0:
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', (HERE / receipt['log']).read_text())
            assert counts and any(int(count) > 0 for count in counts), 'empty selection: ' + label
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    for tool in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'):
        log = (HERE / (tool + '-fixtures.txt')).read_text()
        assert re.search(r'Ran [1-9]\d* tests', log) and '\nOK\n' in log
    baseline = read(HERE / 'patch_6_2_4_publication_sweep-results.json')
    negative = read(HERE / 'negative-results.json')
    control = read(HERE / 'negative-control.json')
    original = read(ROOT / 'data/patch-api/sources/6.2.4-wikitext-register.json')
    mutated = read(HISTORICAL / 'p624-negative-register.json')
    assert len(original['entries']) == len(mutated['entries'])
    changes = [(before, after) for before, after in zip(original['entries'], mutated['entries']) if before != after]
    assert len(changes) == 1
    before, after = changes[0]
    assert before['id'] == control['source_id'] and after == dict(before, symbol=control['symbol'])
    assert set(baseline) == set(negative)
    assert {key for key, row in negative.items() if not row['ok']} == {control['source_id']}
    assert control['before'] == 0 and control['after'] == 1
    assert 'resolved/stale gaps: []' in (HERE / 'negative.txt').read_text()
    source = read(HERE / 'source-reproduction.proof.json')
    assert source['exit'] == 0 and digest(HERE / source['log']) == source['log_sha256']
    assert git_scope(source['source_revision'], context['input_scope']) == context['input_scope']
    assert read(HERE / source['log']) == {'registers': 49, 'extracts': 46,
                                       'inherited_extract_failures': ['12.0.5', '12.0.7', '12.1.0']}
    return len(expected) + 1


def check_matrix(context):
    names = git('ls-tree', '-r', '--name-only', context['validator_scope_revision'], 'data/patch-api/evidence').decode().splitlines()
    expected = {name for name in names if Path(name).name in ('validate.py', 'validate_integrated.py')}
    matrix = read(HERE / 'prior-validator-matrix.json')
    assert set(matrix) == expected and all(row['exit'] == 0 for row in matrix.values())
    for name, row in matrix.items():
        assert digest(ROOT / name) == row['validator_sha256']
        assert digest(HERE / row['log']) == row['log_sha256']
    return len(matrix)


def main():
    for name, expected in read(HERE / 'artifact-hashes.json').items():
        assert digest(ROOT / name) == expected, 'integrated artifact changed: ' + name
    context = read(HERE / 'context.json')
    preserved = check_preservation(context)
    registers, extracts = check_reproduction(context)
    result = comparison(context)
    assert result == read(HERE / 'gap-comparison.json')
    commands = check_receipts(context)
    validators = check_matrix(context)
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts,
                      'preserved_master_sources': preserved, 'pages': len(result['pages']),
                      'cases': result['branch_cases'], 'observations': result['observations'],
                      'gaps': 0, 'negative_gaps': 1, 'commands': commands, 'prior_validators': validators}, sort_keys=True))


if __name__ == '__main__':
    main()
