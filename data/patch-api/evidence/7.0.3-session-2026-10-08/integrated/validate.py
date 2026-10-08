"""Read-only 7.0.3 integration gate; original historical proof remains mandatory."""
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
MERGE_REVISION = 'dfede62de1f48ef2af1a4d37a18d274e26df3d81'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args, data=None):
    return subprocess.check_output(['git', *args], cwd=ROOT, input=data)


def blob(revision, path):
    return git('show', revision + ':' + path)


def git_scope(revision, names):
    """Hash recorded Git blobs in one batch, not thousands of separate processes."""
    tree = {}
    for line in git('ls-tree', '-r', revision).decode().splitlines():
        metadata, path = line.split('\t', 1)
        if path in names:
            tree[path] = metadata.split()[2]
    assert set(tree) == set(names), 'missing recorded source at ' + revision
    paths = sorted(tree)
    output = git('cat-file', '--batch', data=('\n'.join(tree[p] for p in paths) + '\n').encode())
    offset = 0
    result = {}
    for path in paths:
        end = output.index(b'\n', offset)
        object_id, kind, size = output[offset:end].split()
        assert object_id.decode() == tree[path] and kind == b'blob'
        start = end + 1
        offset = start + int(size)
        result[path] = hashlib.sha256(output[start:offset]).hexdigest()
        assert output[offset:offset + 1] == b'\n'
        offset += 1
    assert offset == len(output)
    return result


def check_history():
    mapping = read(HERE / 'rebase-mapping.json')
    assert mapping['original_base'] == 'aa57dd8f8' and mapping['rebased_base'] == '25fbde058'
    mapped = {}
    for row in mapping['commits']:
        old, new = row['recorded_revision'], row['rebased_revision']
        for revision, expected in [(old, row['recorded_patch_id']), (new, row['rebased_patch_id'])]:
            assert git('show', '-s', '--format=%s', revision).decode().strip() == row['subject']
            patch = git('show', '--format=', '--binary', revision)
            assert git('patch-id', '--stable', data=patch).decode().split()[0] == expected
        assert row['patch_identical'] == (row['recorded_patch_id'] == row['rebased_patch_id'])
        assert git('diff', '--name-only', old, new, '--', 'src') == b'', 'rebased runtime changed: ' + old
        mapped[old] = new
    context = read(HISTORICAL / 'p703-context.json')
    for name in ('retail_proof_revision', 'final_runtime_revision', 'accounting_revision'):
        old = git('rev-parse', context[name]).decode().strip()
        assert old in mapped, 'unmapped recorded proof revision: ' + old
    for receipt in HISTORICAL.glob('*.proof.json'):
        revision = read(receipt)['revision']
        git('rev-parse', '--verify', revision + '^{commit}')
        assert revision in mapped or revision == git('rev-parse', context['base_revision']).decode().strip()
    for name, expected in read(HERE / 'historical-preservation.json').items():
        assert digest(ROOT / name) == expected, 'historical artifact changed: ' + name
    # This executes the original revision/history, accounting, scan and receipt checks.
    subprocess.check_call([sys.executable, '-B', str(HISTORICAL / 'validate.py')], cwd=ROOT)


def check_sources(context):
    expected = {p.name.removesuffix('-wikitext-register.json') for p in historical_registers(ROOT, context['runtime_revision'])}
    registers = read(HERE / 'p703-register-reproduction.json')
    extracts = read(HERE / 'p703-saved-extract-reproduction.json')
    extension = read(HERE / 'extension/p703-register-reproduction.json')
    inherited = {r['patch']: r for r in read(ROOT / 'data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/p710-saved-extract-reproduction.json')}
    historical_register_rows = {r['patch']: r for r in read(HISTORICAL / 'p703-register-reproduction.json')}
    historical_extract_rows = {r['patch']: r for r in read(HISTORICAL / 'p703-saved-extract-reproduction.json')}
    for rows in (registers, extracts, extension):
        assert len(rows) == len(expected) and {r['patch'] for r in rows} == expected
    for row in registers + extension:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-wikitext-register.json')
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        flags = provenance.get('generator_flags', historical_register_rows.get(row['patch'], {}).get('verified_flags', []))
        assert row['exit'] == 0 and row['byte_identical'] and row['sha256'] == digest(path)
        assert row['verified_flags'] == flags
    added = next(row for row in extension if row['patch'] == '7.1.0')
    assert added['verified_flags'] == ['--top-level-api-bullets', '--legacy-widget-cvar-bullets']
    for row in extracts:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-api-changes.txt')
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        flags = provenance.get('extractor_flags', historical_extract_rows.get(row['patch'], {}).get('verified_flags', []))
        assert row['verified_flags'] == flags and row['saved_sha256'] == digest(path)
        if row['patch'] in inherited:
            before = inherited[row['patch']]
            assert all(row[key] == before[key] for key in ('byte_identical', 'sha256', 'saved_sha256', 'error')), row['patch']
        else:
            assert row['byte_identical'] and row['exit'] == 0 and row['error'] is None
    return len(expected), sum(row['byte_identical'] for row in extracts)


def passed_cases(path):
    return set(re.findall(r'^test (\S+) \.\.\. ok$', path.read_text(), re.M))


def comparison(context):
    passed = passed_cases(HERE / 'all-sweeps.txt')
    master_passed = passed_cases(HERE / 'master-all-sweeps.txt')
    baseline = {p.stem: p for p in historical_sweep_tests(ROOT, context['master_revision'])}
    paths = historical_sweep_tests(ROOT, context['runtime_revision'])
    summaries = {r['patch']: r for r in read(HERE / 'extension/p703-sweep-summary.json')}
    assert set(baseline) < {p.stem for p in paths}
    pages = []
    for path in paths:
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        relative = path.relative_to(ROOT).as_posix()
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(context['runtime_revision'], relative).decode())
        assert functions and all(any(case.endswith('::' + name) for case in passed) for name in functions)
        results = read(HERE / (path.stem + '-results.json'))
        register = json.loads(blob(context['runtime_revision'], 'data/patch-api/sources/' + patch + '-wikitext-register.json'))
        known_path = 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'
        known = json.loads(blob(context['runtime_revision'], known_path))
        gaps = sorted(key for key, row in results.items() if not row['ok'])
        assert gaps == sorted(known) and set(results) == {r['id'] for r in register['entries']}
        assert summaries[patch] == {'patch': patch, 'rows': len(results), 'ok': len(results) - len(gaps), 'gaps': len(gaps), 'result': 'pass'}
        before = read(HERE / 'master' / (path.stem + '-results.json')) if path.stem in baseline else None
        if before is not None:
            assert json.loads(blob(context['master_revision'], known_path)) == known
            assert set(before) == set(results) and all(before[key]['ok'] == results[key]['ok'] for key in before), patch
            assert sorted(key for key, row in before.items() if not row['ok']) == sorted(known)
            assert all(any(case.endswith('::' + name) for case in master_passed) for name in functions)
        else:
            assert patch == '7.0.3' and len(gaps) == 52
        pages.append({'patch': patch, 'rows': len(results), 'master_rows': len(before) if before is not None else None, 'master_gaps': sorted(key for key, row in before.items() if not row['ok']) if before is not None else None, 'branch_gaps': gaps, 'known_gaps_unchanged': before is not None, 'changed_ok_ids': [], 'status': 'unchanged' if before is not None else 'new audit; historical 52 gaps unchanged'})
    assert len(summaries) == len(paths)
    assert len(passed) == len(paths) + 1 and len(master_passed) == len(baseline) + 1
    return {'master_revision': context['master_revision'], 'runtime_revision': context['runtime_revision'], 'pages': pages, 'changes': [], 'master_cases': len(master_passed), 'branch_cases': len(passed), 'observations': sum(row['rows'] for row in pages)}


def expected_commands():
    labels = {'own-sweep', 'all-sweeps', 'negative', 'build', 'startup', 'format', 'extend-receipts', 'master-all-sweeps', 'master-build', 'master-startup', 'mists-check'}
    labels.update(tool + '-fixtures' for tool in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'))
    labels.update('integration-' + name for name in ('patch-7-0-3', 'professions-api', 'test-crafting', 'trade-info', 'c-collection-api', 'c-function-diff-coverage', 'test-showuipanel-professions-crafting'))
    labels.update('prefork_full_ui-' + name for name in ('patch-7-0-3', 'professions', 'crafting', 'trade-skill', 'mount'))
    labels.update('mists-' + name for name in ('patch-7-0-3-recipe-name-filter-changes-catalog-results', 'mists-trade-skill-api', 'professions-api', 'c-collection-api::test-mount-journal'))
    return {label: 1 if label == 'negative' else 0 for label in labels}


def check_receipts(context):
    expected = expected_commands()
    assert read(HERE / 'finished.json') == read(HERE / 'command-results.json') == expected
    for key in ('runtime', 'master'):
        recorded = context[key + '_scope']
        assert git_scope(context[key + '_revision'], recorded) == recorded, key
    for name, expected_digest in context['runtime_scope'].items():
        assert digest(ROOT / name) == expected_digest, 'proof invalidated: ' + name
    for label, code in expected.items():
        receipt = read(HERE / (label + '.proof.json'))
        source = 'master' if label.startswith('master-') else 'runtime'
        log = HERE / receipt['log']
        assert receipt['exit'] == code and digest(log) == receipt['log_sha256'], label
        assert receipt['source_revision'] == context[source + '_revision'] and receipt['scope'] == context[source + '_scope'], label
        git('rev-parse', '--verify', receipt['revision'] + '^{commit}')
        if receipt['command'][:2] == ['cargo', 'test'] and code == 0:
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text())
            if label in ('prefork_full_ui-crafting', 'prefork_full_ui-trade-skill'):
                assert counts == ['0'], 'update explicit empty selection: ' + label
            else:
                assert counts and any(int(count) > 0 for count in counts), 'empty test selection: ' + label
    warnings = [line for line in (HERE / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    for tool in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'):
        log = (HERE / (tool + '-fixtures.txt')).read_text()
        assert re.search(r'Ran [1-9]\d* tests', log) and '\nOK\n' in log
    for label in ('startup', 'master-startup'):
        receipt = read(HERE / (label + '.proof.json'))
        assert '--no-addons' not in receipt['command'] and '--no-saved-vars' in receipt['command']
        log = (HERE / receipt['log']).read_text()
        assert 'Lua errors: 0 unique, 0 occurrence(s)' in log and log.rstrip().endswith('[]')
    register = read(ROOT / 'data/patch-api/sources/7.0.3-wikitext-register.json')
    mutated = read(HISTORICAL / 'p703-negative-register.json')
    changes = [(a, b) for a, b in zip(register['entries'], mutated['entries']) if a != b]
    assert len(changes) == 1 and len(register['entries']) == len(mutated['entries'])
    before, after = changes[0]
    assert after == dict(before, symbol='P703_NONEXISTENT_NAMESPACE')
    baseline = read(HERE / 'patch_7_0_3_publication_sweep-results.json')
    negative = read(HERE / 'negative-results.json')
    assert set(baseline) == set(negative)
    assert {key for key, row in negative.items() if not row['ok']} == {key for key, row in baseline.items() if not row['ok']} | {before['id']}
    assert 'resolved/stale gaps: []' in (HERE / 'negative.txt').read_text()
    return len(expected)


def main():
    for name, expected in read(HERE / 'artifact-hashes.json').items():
        assert digest(ROOT / name) == expected, 'integrated artifact changed: ' + name
    check_history()
    context = read(HERE / 'context.json')
    registers, extracts = check_sources(context)
    assert comparison(context) == read(HERE / 'gap-comparison.json')
    commands = check_receipts(context)
    matrix = read(HERE / 'prior-validator-matrix.json')
    # Validators present at the 7.0.3 merge; later audits add their own validators.
    own = Path(__file__).resolve().relative_to(ROOT).as_posix()
    listed = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', MERGE_REVISION,
                                      'data/patch-api/evidence'], cwd=ROOT, text=True).splitlines()
    expected = {name for name in listed
                if Path(name).name.startswith('validate') and name.endswith('.py') and name != own}
    assert set(matrix) == expected and all(row['exit'] == 0 for row in matrix.values())
    for path, row in matrix.items():
        assert digest(ROOT / path) == row['validator_sha256'], 'validator proof invalidated: ' + path
        assert digest(HERE / row['log']) == row['log_sha256'], 'validator log changed: ' + path
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts, 'pages': len(read(HERE / 'gap-comparison.json')['pages']), 'cases': read(HERE / 'gap-comparison.json')['branch_cases'], 'gaps': 52, 'negative_gaps': 53, 'commands': commands, 'prior_validators': len(matrix)}, sort_keys=True))


if __name__ == '__main__':
    main()
