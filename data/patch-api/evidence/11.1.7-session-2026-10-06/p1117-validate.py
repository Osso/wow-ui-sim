#!/usr/bin/env python3
"""Validate retained 11.1.7 source accounting and revision-scoped local proof."""
import collections
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
LATER = ('11.2.0', '11.2.5', '11.2.7', '12.0.0', '12.0.1', '12.0.5', '12.0.7', '12.1.0')
EXPECTED_SWEEPS = ((48, 40, 8), (162, 136, 26), (163, 118, 45), (508, 414, 94),
                   (1010, 987, 23), (225, 222, 3), (363, 352, 11), (174, 171, 3), (778, 773, 5))


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources(register, coverage):
    provenance = load(SOURCES / '11.1.7-api-changes.provenance.json')
    assert provenance['pageid'] == 628473
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6726774
    raw = SOURCES / '11.1.7-api-changes.wikitext'
    assert digest(raw) == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '11.1.7-wikitext-register.json') == coverage['source_sha256']
    assert len(register['header_counts']) == 8
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    source = coverage['non_inventory_source']
    assert digest(ROOT / source['path']) == source['sha256']
    assert source['wikitext_revid'] == 6726774
    assert source['wikitext_sha256'] == digest(raw)
    regeneration = load(SESSION / 'p1117-register-regeneration.json')
    assert {row['patch'] for row in regeneration} == set(LATER)
    assert all(row['exit_code'] == 0 and row['byte_identical'] for row in regeneration)


def validate_inventory(register, coverage, results):
    entries = {row['id']: row for row in register['entries']}
    assert len(entries) == 48 and set(entries) == set(results)
    known = load(ROOT / 'tests/data/patch_11_1_7_sweep_known_gaps.json')
    gaps = {key for key, row in results.items() if not row['ok']}
    assert len(known) == len(set(known)) == 8 and set(known) == gaps
    latest = {}
    for patch in LATER:
        for entry in load(SOURCES / f'{patch}-wikitext-register.json')['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    ledger = {row['source_id']: row for row in coverage['source_rows']}
    counts = collections.Counter()
    for key, entry in entries.items():
        observed = results[key]
        expected = observed['expected']
        assert expected['direction'] == entry['direction'] and expected['symbol'] == entry['symbol']
        removed = entry['direction'] == 'removed'
        newer = latest.get(entry['symbol'])
        reversed_direction = newer is not None and (newer['direction'] == 'removed') != removed
        effective_removed = not removed if reversed_direction else removed
        assert expected['publication'] == ('absent' if effective_removed else 'published')
        assert expected['superseded_by'] == (newer['id'] if reversed_direction else None)
        assert not observed['observed']['default_mismatch']
        credit = ledger[key]
        if not observed['ok']:
            assert credit['status'] == 'audit-pending' and not credit['capabilities']
            continue
        assert credit['capabilities'] == ['publication-sweep-11-1-7']
        if reversed_direction:
            counts['superseded_ok'] += 1
            assert credit['status'] == 'metadata-only'
        elif removed:
            counts['strict_removals'] += 1
            assert credit['status'] == 'bounded-coverage'
            assert 'deprecated-fallback=' not in observed['observed']['detail']
        else:
            counts['published'] += 1
            assert credit['status'] == 'partial-development-green'
    assert counts == {'published': 28, 'strict_removals': 5, 'superseded_ok': 7}
    return dict(counts)


def validate_review_and_control(results):
    initial = load(SESSION / 'p1117-initial-sweep.json')
    review = load(SESSION / 'p1117-gap-review.json')['rows']
    assert len(review) == len({row['source_id'] for row in review}) == 12
    assert {row['source_id'] for row in review} == {key for key, row in initial.items() if not row['ok']}
    for row in review:
        key = row['source_id']
        assert row['initial'] == initial[key] and row['final'] == results[key]
        assert (row['decision'] == 'fixed') == results[key]['ok']
        assert row['reason'] and row['investigate']
    assert sum(row['decision'] == 'fixed' for row in review) == 4
    control = load(SESSION / 'p1117-negative-control.json')
    negative = load(SESSION / 'p1117-negative-result.json')
    key = 'wt-global-api-C_ActionBar.ForceUpdateAction-24'
    assert set(results) == set(negative)
    assert [name for name in results if results[name] != negative[name]] == [key]
    assert results[key]['ok'] and not negative[key]['ok']
    assert sum(not row['ok'] for row in negative.values()) == 9
    assert control['actual_exit_code'] == control['expected_exit_code'] == 101
    assert control['changed_ids'] == control['new_ids'] == [key] and not control['resolved_ids']
    original = load(SOURCES / '11.1.7-wikitext-register.json')['entries']
    modified = load(SESSION / 'p1117-negative-register.json')['entries']
    assert len(original) == len(modified) == 48
    changes = [(a, b) for a, b in zip(original, modified) if a != b]
    assert len(changes) == 1
    a, b = changes[0]
    assert a['id'] == key and b == dict(a, direction='removed')


def validate_extract(coverage):
    spec = importlib.util.spec_from_file_location('extract', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text = extractor.extract_text((SOURCES / '11.1.7-api-changes.wikitext').read_text())
    assert text == (SOURCES / '11.1.7-api-changes.txt').read_text()
    expected = extractor.seed_rows(text, '11.1.7')
    actual = [row for row in coverage['source_rows'] if not row['source_id'].startswith('wt-')]
    assert actual == expected and len(actual) == 27
    assignment = load(SESSION / 'p1117-extract-assignments.json')
    pending = [key for keys in assignment['batches'].values() for key in keys]
    metadata = assignment['metadata']
    assert len(pending) == len(set(pending)) == 16
    assert len(metadata) == len(set(metadata)) == 11
    assert not set(pending) & set(metadata)
    assert set(pending + metadata) == {row['source_id'] for row in actual}
    assert set(pending) == {row['source_id'] for row in actual if row['status'] == 'audit-pending'}
    scout = (SESSION / 'p1117-extract-scout.md').read_text()
    assert all(scout.count(f'`{key}`') == 1 for key in pending + metadata)
    return {name: len(ids) for name, ids in assignment['batches'].items()}


def validate_preservation():
    preserved = load(SESSION / 'p1117-preserved-inputs.json')['files']
    assert len(preserved) == 32
    assert all(row['unchanged'] and digest(ROOT / row['path']) == row['sha256'] for row in preserved)
    comparisons = load(SESSION / 'p1117-baseline-preservation.json')
    assert {row['patch'] for row in comparisons} == set(LATER)
    for row in comparisons:
        short = 'p' + row['patch'].replace('.', '')
        current = load(SESSION / f'{short}-sweep-result.json')
        baseline = load(ROOT / row['baseline_path'])
        changes = [key for key in current if current[key] != baseline[key]]
        assert changes == row['changed_ids']
        if row['patch'] == '11.2.7':
            outcomes = load(ROOT / 'data/patch-api/evidence/11.2.7-session-2026-10-06/p1127-gap-outcomes.json')
            closed = {item['source_id'] for item in outcomes['rows'] if item['outcome'] != 'still-gap'}
            assert set(changes) == closed and len(closed) == 27
            assert row['all_changes_are_preexisting_closures']
        else:
            assert not changes and row['identical_observations']


def validate_proof(coverage):
    proof = load(SESSION / 'p1117-proof.json')
    current = [row for row in proof['results'] if row.get('invalidated_by') is None]
    sweeps = [row for row in current if 'rows' in row]
    assert len(sweeps) == 9
    assert coverage['capabilities'][0]['compiled_revision'] == proof['runtime_revision']
    for patch, counts, row in zip(('11.1.7',) + LATER, EXPECTED_SWEEPS, sweeps):
        results = load(ROOT / row['result_path'])
        assert len(results) == row['rows'] == counts[0]
        assert sum(value['ok'] for value in results.values()) == row['ok'] == counts[1]
        assert sum(not value['ok'] for value in results.values()) == row['gaps'] == counts[2]
        known = load(ROOT / f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        assert set(known) == {key for key, value in results.items() if not value['ok']}
        assert row['exit_code'] == 0 and row['revision'] == proof['runtime_revision']
        assert row['command'][-2:] == ['--nocapture', '--test-threads=1']
        assert row['command'][7] == 'patch_' + patch.replace('.', '_') + '_publication_sweep'
    for name in ('fixes-green', 'extract-green', 'extract-reproduce', 'fmt-final', 'mists-check', 'build-final'):
        matched = [row for row in current if row['name'] == name]
        assert len(matched) == 1 and matched[0]['exit_code'] == 0
    mists = next(row for row in current if row['name'] == 'mists-check')
    log = (ROOT / mists['log_path']).read_text()
    warnings = [line for line in log.splitlines() if line.startswith('warning:')]
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line for line in warnings)
    assert not any(line.startswith('error') for line in log.splitlines())
    startup = load(SESSION / 'p1117-startup-result.json')
    assert startup['exit_code'] == 0 and startup['json'] == []
    assert json.loads((ROOT / startup['stdout_path']).read_text()) == []
    return [{key: row[key] for key in ('name', 'rows', 'ok', 'gaps', 'exit_code')} for row in sweeps]


def main():
    register = load(SOURCES / '11.1.7-wikitext-register.json')
    coverage = load(SOURCES / '11.1.7-page-coverage.json')
    results = load(SESSION / 'p1117-sweep-result.json')
    validate_sources(register, coverage)
    inventory = validate_inventory(register, coverage, results)
    validate_review_and_control(results)
    batches = validate_extract(coverage)
    validate_preservation()
    sweeps = validate_proof(coverage)
    rows = coverage['source_rows']
    assert len(rows) == len({row['source_id'] for row in rows}) == 75
    counts = dict(collections.Counter(row['status'] for row in rows))
    assert counts == {'audit-pending': 24, 'partial-development-green': 28,
                      'bounded-coverage': 5, 'metadata-only': 18}
    capability = coverage['capabilities'][0]
    for path in capability['tests'] + [capability['spec'], capability['ledger']]:
        assert (ROOT / path).is_file()
    output = {'result': 'PASS', 'source_rows': len(rows), 'inventory': inventory,
              'statuses': counts, 'extract_batches': batches, 'sweeps': sweeps}
    (SESSION / 'p1117-page-validation.json').write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps(output, indent=2))


if __name__ == '__main__':
    main()
