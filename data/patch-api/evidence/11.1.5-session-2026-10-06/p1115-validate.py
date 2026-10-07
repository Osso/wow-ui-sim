#!/usr/bin/env python3
"""Validate retained 11.1.5 page accounting; no runtime/native parity claim."""
import collections
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
LATER = ('11.1.7', '11.2.0', '11.2.5', '11.2.7', '12.0.0',
         '12.0.1', '12.0.5', '12.0.7', '12.1.0')
EXPECTED = ((125, 89, 36), (48, 40, 8), (162, 135, 27), (163, 118, 45),
            (508, 414, 94), (1010, 989, 21), (225, 222, 3), (363, 352, 11),
            (174, 171, 3), (778, 773, 5))


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources(register, coverage):
    provenance = load(SOURCES / '11.1.5-api-changes.provenance.json')
    raw = SOURCES / '11.1.5-api-changes.wikitext'
    assert provenance['pageid'] == 621744
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6726775
    assert digest(raw) == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '11.1.5-wikitext-register.json') == coverage['source_sha256']
    assert len(register['header_counts']) == 8
    assert all(r['header_count'] == r['parsed_count'] for r in register['header_counts'])
    extract = coverage['non_inventory_source']
    assert extract['wikitext_revid'] == 6726775
    assert extract['wikitext_sha256'] == digest(raw)
    assert extract['sha256'] == digest(ROOT / extract['path'])
    regenerated = load(SESSION / 'p1115-register-regeneration.json')
    assert {r['patch'] for r in regenerated} == {'11.1.5', *LATER}
    assert all(r['exit_code'] == 0 and r['byte_identical'] for r in regenerated)


def validate_inventory(register, coverage, results):
    entries = {r['id']: r for r in register['entries']}
    assert len(entries) == len(register['entries']) == 125
    assert set(entries) == set(results)
    known = load(ROOT / 'tests/data/patch_11_1_5_sweep_known_gaps.json')
    assert len(known) == len(set(known)) == 36
    assert set(known) == {key for key, r in results.items() if not r['ok']}
    latest = {}
    for patch in LATER:
        for row in load(SOURCES / f'{patch}-wikitext-register.json')['entries']:
            if row['direction'] != 'changed':
                latest[row['symbol']] = row
    ledger = {r['source_id']: r for r in coverage['source_rows']}
    for key, entry in entries.items():
        validate_inventory_row(entry, results[key], ledger[key], latest)


def validate_inventory_row(entry, observed, credit, latest):
    expected = observed['expected']
    assert expected['direction'] == entry['direction']
    assert expected['symbol'] == entry['symbol']
    removed = entry['direction'] == 'removed'
    newer = latest.get(entry['symbol'])
    reversed_direction = newer is not None and (newer['direction'] == 'removed') != removed
    assert expected['superseded_by'] == (newer['id'] if reversed_direction else None)
    effective_removed = not removed if reversed_direction else removed
    assert expected['publication'] == ('absent' if effective_removed else 'published')
    assert not observed['observed']['default_mismatch']
    if not observed['ok']:
        assert credit['status'] == 'audit-pending' and credit['capabilities'] == []
        return
    assert credit['capabilities'] == ['publication-sweep-11-1-5']
    if reversed_direction:
        assert credit['status'] == 'metadata-only'
    elif removed:
        assert credit['status'] == 'bounded-coverage'
        assert 'deprecated-fallback=' not in observed['observed']['detail']
    else:
        assert credit['status'] == 'partial-development-green'


def validate_review_and_control(results):
    initial = load(SESSION / 'p1115-initial-sweep.json')
    review = load(SESSION / 'p1115-gap-review.json')['rows']
    assert len(review) == len({r['source_id'] for r in review}) == 44
    assert {r['source_id'] for r in review} == {k for k, r in initial.items() if not r['ok']}
    for row in review:
        key = row['source_id']
        assert row['initial'] == initial[key] and row['final'] == results[key]
        assert (row['decision'] == 'fixed') == results[key]['ok']
        assert row['reason'] and row['investigate']
    assert sum(r['decision'] == 'fixed' for r in review) == 8
    retained = next(r for r in review if r['symbol'] == 'UpdateUIParentPosition')
    assert retained['decision'] == 'retained-gap'
    assert 'Blizzard_UIParentUtil/UIParentUtil.lua:13' in retained['investigate']
    control = load(SESSION / 'p1115-negative-control.json')
    negative = load(SESSION / 'p1115-negative-result.json')
    key = 'wt-global-api-C_ChatInfo.DropCautionaryChatMessage-70'
    assert set(negative) == set(results)
    assert [k for k in results if results[k] != negative[k]] == [key]
    assert results[key]['ok'] and not negative[key]['ok']
    assert sum(not r['ok'] for r in negative.values()) == 37
    assert control['actual_exit_code'] == control['expected_exit_code'] == 101
    assert control['new_ids'] == control['changed_ids'] == [key]
    assert control['resolved_ids'] == []
    original = load(SOURCES / '11.1.5-wikitext-register.json')['entries']
    altered = load(SESSION / 'p1115-negative-register.json')['entries']
    assert len(original) == len(altered) == 125
    changes = [(a, b) for a, b in zip(original, altered) if a != b]
    assert len(changes) == 1 and changes[0][1] == dict(changes[0][0], direction='removed')


def validate_extract(coverage):
    spec = importlib.util.spec_from_file_location('extract', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text = extractor.extract_text((SOURCES / '11.1.5-api-changes.wikitext').read_text())
    assert text == (SOURCES / '11.1.5-api-changes.txt').read_text()
    expected = extractor.seed_rows(text, '11.1.5')
    actual = [r for r in coverage['source_rows'] if not r['source_id'].startswith('wt-')]
    assert len(actual) == 99 and actual == expected
    allocation = load(SESSION / 'p1115-extract-assignments.json')
    pending = [key for ids in allocation['batches'].values() for key in ids]
    metadata = allocation['metadata']
    assert len(pending) == len(set(pending)) == 85
    assert len(metadata) == len(set(metadata)) == 14
    assert set(pending).isdisjoint(metadata)
    assert set(pending + metadata) == {r['source_id'] for r in actual}
    assert set(pending) == {r['source_id'] for r in actual if r['status'] == 'audit-pending'}
    scout = (SESSION / 'p1115-extract-scout.md').read_text()
    assert all(scout.count(f'`{key}`') == 1 for key in pending + metadata)


def validate_preservation():
    preserved = load(SESSION / 'p1115-preserved-inputs.json')['files']
    assert len(preserved) == 36
    assert all(r['unchanged'] and digest(ROOT / r['path']) == r['sha256'] for r in preserved)
    comparisons = load(SESSION / 'p1115-baseline-preservation.json')
    assert {r['patch'] for r in comparisons} == set(LATER)
    permitted = {'11.2.0': ['wt-widgets-Browser:NavigateTo-173'],
                 '12.0.0': ['wt-global-api-dropsecretaccess-470', 'wt-global-api-issecrettable-472']}
    for row in comparisons:
        short = 'p' + row['patch'].replace('.', '')
        current = load(SESSION / f'{short}-sweep-result.json')
        old = load(ROOT / row['baseline_path'])
        changes = [key for key in current if current[key] != old[key]]
        assert changes == row['changed_ids'] == permitted.get(row['patch'], [])
        assert row['identical_observations'] == (not changes)


def validate_proof(coverage):
    proof = load(SESSION / 'p1115-proof.json')
    sweeps = [r for r in proof['results'] if 'rows' in r]
    assert len(sweeps) == 10
    for patch, counts, row in zip(('11.1.5',) + LATER, EXPECTED, sweeps):
        results = load(ROOT / row['result_path'])
        actual = (len(results), sum(r['ok'] for r in results.values()), sum(not r['ok'] for r in results.values()))
        assert actual == counts == (row['rows'], row['ok'], row['gaps'])
        assert row['exit_code'] == 0 and row['revision'].startswith(proof['runtime_revision'])
        known = load(ROOT / f'tests/data/patch_{patch.replace(".", "_")}_sweep_known_gaps.json')
        assert set(known) == {k for k, r in results.items() if not r['ok']}
        assert row['command'][-2:] == ['--nocapture', '--test-threads=1']
    assert coverage['capabilities'][0]['compiled_revision'].startswith(proof['runtime_revision'])
    successful = {r['name'] for r in proof['results'] if r['exit_code'] == 0}
    assert {'fixes-native-green', 'extract-green', 'prefork-edit-mode', 'environment-regression',
            'fmt-final', 'mists-check', 'build-final', 'startup'} <= successful
    summary = load(SESSION / 'p1115-verification-summary.json')
    assert all(not r['errors'] for r in summary.values())
    assert all('iced-wgpu-patched/Cargo.toml' in line or '`iced_wgpu` (manifest)' in line
               for r in summary.values() for line in r['warnings'])
    assert any('2 passed' in line for line in summary['behavior']['test_summary'])
    assert any('1 passed' in line for line in summary['prefork']['test_summary'])
    assert any('1 passed' in line for line in summary['environment']['test_summary'])
    assert any('Ran 13 tests' in line for line in summary['extract']['test_summary'])
    startup = load(SESSION / 'p1115-startup-result.json')
    assert startup['exit_code'] == 0 and startup['json'] == []
    assert load(ROOT / startup['stdout_path']) == []


def main():
    register = load(SOURCES / '11.1.5-wikitext-register.json')
    coverage = load(SOURCES / '11.1.5-page-coverage.json')
    results = load(SESSION / 'p1115-sweep-result.json')
    validate_sources(register, coverage)
    validate_inventory(register, coverage, results)
    validate_review_and_control(results)
    validate_extract(coverage)
    validate_preservation()
    validate_proof(coverage)
    rows = coverage['source_rows']
    assert len(rows) == len({r['source_id'] for r in rows}) == 224
    counts = collections.Counter(r['status'] for r in rows)
    assert counts == {'partial-development-green': 73, 'bounded-coverage': 15,
                      'audit-pending': 121, 'metadata-only': 15}
    print(json.dumps({'rows': len(rows), 'statuses': counts, 'sweeps': 10,
                      'initial_gaps': 44, 'closed': 8, 'retained': 36,
                      'negative_control_gaps': 37, 'result': 'PASS'}, indent=2))


if __name__ == '__main__':
    main()
