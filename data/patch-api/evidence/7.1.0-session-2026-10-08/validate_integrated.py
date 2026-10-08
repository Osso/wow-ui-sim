"""Validate sealed integrated scope and exact-master per-row sweep comparison."""
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
FRESH = HERE / 'integrated'
sys.path.insert(0, str(ROOT / 'tools'))
from patch_audit_validation import historical_registers, historical_sweep_tests, historical_json


def read(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


@lru_cache(maxsize=None)
def blob(revision, path):
    return subprocess.check_output(['git', 'show', f'{revision}:{path}'], cwd=ROOT)


def check_sources(revision):
    expected = {p.name.removesuffix('-wikitext-register.json') for p in historical_registers(ROOT, revision)}
    registers = read(FRESH / 'p710-register-reproduction.json')
    extracts = read(FRESH / 'p710-saved-extract-reproduction.json')
    extension = read(FRESH / 'extension/p710-register-reproduction.json')
    inherited = {r['patch']: r for r in read(HERE / 'p710-saved-extract-reproduction.json')}
    for rows in (registers, extracts, extension):
        assert len(rows) == len(expected) and {r['patch'] for r in rows} == expected
    for row in registers + extension:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-wikitext-register.json')
        assert row['exit'] == 0 and row['byte_identical'] and digest(path) == row['sha256']
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        if provenance.get('generator_flags') is not None:
            assert row['verified_flags'] == provenance['generator_flags']
    added = next(row for row in extension if row['patch'] == '7.2.0')
    assert added['verified_flags'] == ['--legacy-widget-summaries', '--prose-namespace-migrations']
    assert added['revision'] == read(FRESH / 'extend-receipts.proof.json')['revision']
    for row in extracts:
        path = ROOT / 'data/patch-api/sources' / (row['patch'] + '-api-changes.txt')
        assert digest(path) == row['saved_sha256']
        provenance = read(path.with_name(row['patch'] + '-api-changes.provenance.json'))
        if provenance.get('extractor_flags') is not None:
            assert row['verified_flags'] == provenance['extractor_flags']
        if row['patch'] in inherited:
            before = inherited[row['patch']]
            assert (row['byte_identical'], row['sha256'], row['error']) == (before['byte_identical'], before['sha256'], before['error'])
        else:
            assert row['byte_identical'] and row['error'] is None
    return len(expected), sum(row['byte_identical'] for row in extracts)


def passed_cases(path):
    return set(re.findall(r'^test (\S+) \.\.\. ok$', path.read_text(), re.M))


def comparison(context):
    summaries = {r['patch']: r for r in read(FRESH / 'extension/p710-sweep-summary.json')}
    passed = passed_cases(FRESH / 'all-sweeps.txt')
    master_passed = passed_cases(FRESH / 'master-all-sweeps.txt')
    baseline_paths = {p.stem: p for p in historical_sweep_tests(ROOT, context['master_revision'])}
    paths = historical_sweep_tests(ROOT, context['runtime_revision'])
    assert set(baseline_paths) <= {p.stem for p in paths}
    comparisons = []
    for path in paths:
        patch = path.stem.removeprefix('patch_').removesuffix('_publication_sweep').replace('_', '.')
        relative = path.relative_to(ROOT).as_posix()
        functions = re.findall(r'fn (patch_\w+_publication_sweep)\(', blob(context['runtime_revision'], relative).decode())
        assert functions and all(any(case.endswith('::' + name) for case in passed) for name in functions)
        results = read(FRESH / (path.stem + '-results.json'))
        register_path = 'data/patch-api/sources/' + patch + '-wikitext-register.json'
        known_path = 'tests/data/patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'
        register = historical_json(ROOT, register_path, context['runtime_revision'])
        known = historical_json(ROOT, known_path, context['runtime_revision'])
        gaps = sorted(key for key, row in results.items() if not row['ok'])
        assert set(results) == {r['id'] for r in register['entries']}
        assert gaps == sorted(known), patch
        assert summaries[patch] == {'patch': patch, 'rows': len(results), 'ok': len(results) - len(gaps),
                                    'gaps': len(gaps), 'result': 'pass'}
        before = read(FRESH / 'master' / (path.stem + '-results.json')) if path.stem in baseline_paths else None
        if before is not None:
            before_known = json.loads(blob(context['master_revision'], known_path))
            assert set(before) == set(results), patch
            assert sorted(key for key, row in before.items() if not row['ok']) == sorted(before_known), patch
            assert before_known == known, patch
            assert all(before[key]['ok'] == results[key]['ok'] for key in before), patch
            assert all(any(case.endswith('::' + name) for case in master_passed) for name in functions), patch
        else:
            assert patch == '7.1.0' and gaps == []
        comparisons.append({'patch': patch, 'rows': len(results),
                            'master_rows': len(before) if before is not None else None,
                            'master_gaps': sorted(key for key, row in before.items() if not row['ok']) if before is not None else None,
                            'branch_gaps': gaps, 'known_gaps_unchanged': before is not None,
                            'changed_ok_ids': [], 'status': 'unchanged' if before is not None else 'new audit; historical gaps remain zero'})
    assert len(summaries) == len(paths)
    assert len(passed) == len(paths) + 1 and len(master_passed) == len(baseline_paths) + 1
    return {'master_revision': context['master_revision'], 'runtime_revision': context['runtime_revision'],
            'pages': comparisons, 'changes': [], 'master_cases': len(master_passed),
            'branch_cases': len(passed), 'observations': sum(r['rows'] for r in comparisons)}


def check_receipts(context):
    labels = {'own-sweep', 'extend-receipts', 'master-all-sweeps', 'all-sweeps', 'integration-p710',
              'prefork-p710', 'screen-regression', 'items-regression', 'intrinsic-regression', 'intrinsic-discovery',
              'gen_patch_wikitext_register-fixtures', 'extract_patch_non_inventory-fixtures',
              'patch_audit_validation-fixtures', 'format', 'mists-check', 'negative'}
    expected = {label: (1 if label in ('negative', 'intrinsic-discovery') else 0) for label in labels}
    assert read(FRESH / 'command-results.json') == context['expected_exits'] == expected
    for label, code in expected.items():
        receipt = read(FRESH / (label + '.proof.json'))
        log = FRESH / receipt['log']
        assert receipt['exit'] == code and digest(log) == receipt['log_sha256'], label
        source = context['master_revision'] if label == 'master-all-sweeps' else context['runtime_revision']
        scope = context['master_scope'] if label == 'master-all-sweeps' else context['runtime_scope']
        assert receipt['source_revision'] == source, label
        if label == 'intrinsic-discovery':
            scope = dict(scope, **{'tests/patch_7_1_0_behavior.rs': digest(HERE / 'p710-intrinsic-discovery.rs')})
        assert receipt['scope'] == scope, label
        for name, expected_digest in scope.items():
            if label == 'intrinsic-discovery' and name == 'tests/patch_7_1_0_behavior.rs':
                assert expected_digest == digest(HERE / 'p710-intrinsic-discovery.rs')
                continue
            assert hashlib.sha256(blob(source, name)).hexdigest() == expected_digest, (label, name)
            if label != 'master-all-sweeps':
                assert hashlib.sha256(blob(receipt['revision'], name)).hexdigest() == expected_digest, (label, name)
        if receipt['command'][:2] == ['cargo', 'test'] and code == 0:
            counts = re.findall(r'test result: ok\. (\d+) passed; 0 failed;', log.read_text())
            if label == 'integration-p710':
                # This page defines cached cases only; the requested bare filter is empty.
                # Scoped screen/item/intrinsic integration proof is checked separately.
                assert counts == ['0'], 'update the explicitly documented empty selection'
            else:
                assert counts and any(int(count) > 0 for count in counts), 'empty selection: ' + label
    warnings = [line for line in (FRESH / 'mists-check.txt').read_text().splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings), warnings
    for label in ('gen_patch_wikitext_register', 'extract_patch_non_inventory', 'patch_audit_validation'):
        log = (FRESH / (label + '-fixtures.txt')).read_text()
        assert re.search(r'Ran [1-9]\d* tests', log) and '\nOK\n' in log
    baseline = read(FRESH / 'patch_7_1_0_publication_sweep-results.json')
    negative = read(FRESH / 'negative-results.json')
    register = read(ROOT / 'data/patch-api/sources/7.1.0-wikitext-register.json')
    mutated = read(HERE / 'p710-negative-register.json')
    changed = [(a, b) for a, b in zip(register['entries'], mutated['entries']) if a != b]
    assert len(changed) == 1 and len(register['entries']) == len(mutated['entries'])
    before, after = changed[0]
    assert before['id'] == 'wt-widgets-Frame:SetClipsChildren-6'
    assert before['direction'] == 'added' and after == dict(before, direction='removed')
    assert set(negative) == set(baseline) and all(row['ok'] for row in baseline.values())
    assert {key for key, row in negative.items() if not row['ok']} == {before['id']}
    assert "unknown frame type 'P710ClipIntrinsic'" in (FRESH / 'intrinsic-discovery.txt').read_text()
    assert 'resolved/stale gaps: []' in (FRESH / 'negative.txt').read_text()


def main():
    for name, expected in read(FRESH / 'artifact-hashes.json').items():
        assert digest(ROOT / name) == expected, 'changed integrated artifact: ' + name
    context = read(FRESH / 'context.json')
    registers, extracts = check_sources(context['runtime_revision'])
    assert comparison(context) == read(FRESH / 'gap-comparison.json')
    check_receipts(context)
    print(json.dumps({'status': 'PASS', 'registers': registers, 'extracts': extracts,
                      'pages': len(read(FRESH / 'gap-comparison.json')['pages']),
                      'cases': read(FRESH / 'gap-comparison.json')['branch_cases'],
                      'gaps': 0, 'negative_gaps': 1}, sort_keys=True))


if __name__ == '__main__':
    main()
