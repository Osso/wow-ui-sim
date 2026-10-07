#!/usr/bin/env python3
"""Validate retained launch-page accounting and local proof; no native-parity claim."""
import hashlib
import json
import runpy
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCHES = ('11.0.0', '11.0.2', '11.0.5', '11.0.7', '11.1.0', '11.1.5',
           '11.1.7', '11.2.0', '11.2.5', '11.2.7', '12.0.0', '12.0.1',
           '12.0.5', '12.0.7', '12.1.0')


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources():
    register = load(SOURCES / '11.0.0-wikitext-register.json')
    provenance = load(SOURCES / '11.0.0-api-changes.provenance.json')
    coverage = load(SOURCES / '11.0.0-page-coverage.json')
    assert provenance['pageid'] == 585562
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6726780
    assert provenance['wikitext']['revision_timestamp'] == '2026-05-25T20:32:45Z'
    assert digest(SOURCES / '11.0.0-api-changes.wikitext') == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '11.0.0-wikitext-register.json') == coverage['source_sha256']
    assert digest(SOURCES / '11.0.0-api-changes.txt') == coverage['non_inventory_source']['sha256']
    assert len(register['entries']) == 495
    assert len(register['header_counts']) == 8
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    assert Counter(row['direction'] for row in register['entries']) == {'added': 346, 'removed': 103, 'changed': 46}
    extractor = runpy.run_path(str(ROOT / 'tools/extract_patch_non_inventory.py'))
    text = (SOURCES / '11.0.0-api-changes.txt').read_text()
    assert extractor['extract_text']((SOURCES / '11.0.0-api-changes.wikitext').read_text()) == text
    extract = extractor['seed_rows'](text, '11.0.0')
    scout = load(SESSION / 'p1100-extract-scout.json')
    assert len(extract) == len(scout) == 347
    assert {row['source_id'] for row in extract} == {row['source_id'] for row in scout}
    ids = [row['source_id'] for row in coverage['source_rows']]
    assert len(ids) == len(set(ids)) == 842
    assert set(ids) == {row['id'] for row in register['entries']} | {row['source_id'] for row in extract}
    ledger = {row['source_id']: row for row in coverage['source_rows']}
    for row in scout:
        line = int(row['source_id'].rsplit('-', 1)[-1])
        assert row['statement'] == text.splitlines()[line - 1].strip()
        assert row['proof'] == ledger[row['source_id']]['status']
        assert row['decision'] == ledger[row['source_id']]['note']
        assert not ledger[row['source_id']]['capabilities']
    assert Counter(row['status'] for row in coverage['source_rows']) == {
        'partial-development-green': 233, 'bounded-coverage': 96,
        'audit-pending': 494, 'metadata-only': 19}
    reproduction = load(SESSION / 'p1100-register-reproduction.json')
    assert {row['patch'] for row in reproduction} == set(PATCHES)
    assert all(row['byte_identical'] and row['exit_code'] == 0 for row in reproduction)
    preserved = load(SESSION / 'p1100-preserved-inputs.json')
    assert len(preserved) == 92
    assert all(digest(ROOT / row['path']) == row['baseline_sha256'] for row in preserved)
    return register


def validate_sweeps(register):
    results = load(SESSION / 'p1100-sweep-result.json')
    latest = {}
    for patch in PATCHES[1:]:
        for entry in load(SOURCES / f'{patch}-wikitext-register.json')['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    for entry in register['entries']:
        expected = results[entry['id']]['expected']
        newer = latest.get(entry['symbol'])
        removed = (newer or entry)['direction'] == 'removed'
        assert expected['publication'] == ('absent' if removed else 'published')
        reversed_direction = newer and removed != (entry['direction'] == 'removed')
        assert expected['superseded_by'] == (newer['id'] if reversed_direction else None)
    for patch in PATCHES:
        result = load(SESSION / ('p' + patch.replace('.', '') + '-sweep-result.json'))
        known = load(ROOT / 'tests/data' / ('patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        assert {key for key, value in result.items() if not value['ok']} == set(known)
        assert len(known) == len(set(known))
    baseline = {key for key, value in results.items() if not value['ok']}
    negative = {key for key, value in load(SESSION / 'p1100-negative-result.json').items() if not value['ok']}
    assert len(baseline) == 166
    assert negative - baseline == {'wt-global-api-C_AdventureMap.GetAdventureMapTextureKit-150'}
    assert not baseline - negative
    altered = load(SESSION / 'p1100-negative-register.json')
    different = [(a, b) for a, b in zip(register['entries'], altered['entries']) if a != b]
    assert len(register['entries']) == len(altered['entries']) == 495
    assert len(different) == 1
    original, changed = different[0]
    assert {**original, 'direction': 'removed'} == changed
    review = load(SESSION / 'p1100-gap-review.json')
    assert Counter(row['decision'] for row in review) == {'closed': 9, 'retained gap': 166}
    assert {row['source_id'] for row in review if row['decision'] == 'retained gap'} == baseline
    consumers = {row['source_id']: row for row in load(SESSION / 'p1100-retirement-consumers.json')}
    for row in review:
        assert row['reason']
        if row['decision'] == 'closed':
            assert not consumers[row['source_id']]['qualified_matches']


def validate_proof():
    rows = load(SESSION / 'p1100-proof.json')
    proof = {row['scope']: row for row in rows}
    required = ['behavior-green-final', 'p1100-sweep-final', 'prefork-new',
                'prefork-major-factions', 'extract-fixtures', 'register-fixtures',
                'extract-reproduction', 'fmt-check', 'mists-test-check',
                'retail-binary-build', 'bounded-retail-startup']
    required += ['patch_' + patch.replace('.', '_') + '_publication_sweep' for patch in PATCHES[1:]]
    assert all(proof[key]['exit_code'] == 0 and not proof[key]['invalidated'] for key in required)
    assert proof['mists-test-check']['non_vendor_warnings'] == []
    assert proof['bounded-retail-startup']['stdout'].strip() == '[]'
    assert proof['negative-control']['exit_code'] == 101
    assert proof['behavior-red']['exit_code'] == 101
    assert proof['prefork-map']['invalidated']
    for key in ('prefork-new', 'prefork-major-factions'):
        assert any('1 passed; 0 failed; 1 total' in line for line in proof[key]['summary'])
    sweep_scopes = ['p1100-sweep-final'] + required[12:]
    for key in sweep_scopes + ['behavior-green-final']:
        assert any('1 passed; 0 failed' in line for line in proof[key]['summary'])
    for row in rows:
        assert row['cwd'] == str(ROOT)
        assert row['environment']['CARGO_TARGET_DIR'] == str(ROOT / 'target')


def main():
    register = validate_sources()
    validate_sweeps(register)
    validate_proof()
    print('PASS: 842 unique IDs; 495 inventory + 347 extract; 15 exact isolated sweeps; nine closures, 166 gaps, source hashes, 92 preserved inputs, reproduction, negative control and local proof')


if __name__ == '__main__':
    main()
