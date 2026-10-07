#!/usr/bin/env python3
"""Validate retained source/proof contracts; no native-parity claim."""
import hashlib
import json
import runpy
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCHES = ('11.0.2', '11.0.5', '11.0.7', '11.1.0', '11.1.5', '11.1.7',
           '11.2.0', '11.2.5', '11.2.7', '12.0.0', '12.0.1', '12.0.5',
           '12.0.7', '12.1.0')


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    register = load(SOURCES / '11.0.2-wikitext-register.json')
    provenance = load(SOURCES / '11.0.2-api-changes.provenance.json')
    coverage = load(SOURCES / '11.0.2-page-coverage.json')
    assert provenance['pageid'] == 595347
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6862982
    assert provenance['wikitext']['revision_timestamp'] == '2026-09-06T02:25:37Z'
    assert digest(SOURCES / '11.0.2-api-changes.wikitext') == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '11.0.2-wikitext-register.json') == coverage['source_sha256']
    assert digest(SOURCES / '11.0.2-api-changes.txt') == coverage['non_inventory_source']['sha256']
    assert len(register['entries']) == 34
    assert len(register['header_counts']) == 6
    assert all(r['header_count'] == r['parsed_count'] for r in register['header_counts'])
    extractor = runpy.run_path(str(ROOT / 'tools/extract_patch_non_inventory.py'))
    text = (SOURCES / '11.0.2-api-changes.txt').read_text()
    assert extractor['extract_text']((SOURCES / '11.0.2-api-changes.wikitext').read_text()) == text
    extract = extractor['seed_rows'](text, '11.0.2')
    scout = load(SESSION / 'p1102-extract-scout.json')
    assert len(extract) == len(scout) == 229
    assert {r['source_id'] for r in extract} == {r['source_id'] for r in scout}
    ids = [r['source_id'] for r in coverage['source_rows']]
    assert len(ids) == len(set(ids)) == 263
    assert set(ids) == {r['id'] for r in register['entries']} | {r['source_id'] for r in extract}
    ledger = {r['source_id']: r for r in coverage['source_rows']}
    for row in scout:
        line = int(row['source_id'].rsplit('-', 1)[-1])
        assert row['statement'] == text.splitlines()[line - 1].strip()
        assert row['proof'] == ledger[row['source_id']]['status']
        assert row['decision']
    assert dict(Counter(r['status'] for r in coverage['source_rows'])) == {
        'partial-development-green': 15, 'bounded-coverage': 7,
        'audit-pending': 213, 'metadata-only': 28}
    results = load(SESSION / 'p1102-sweep-result.json')
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
        assert {k for k, v in result.items() if not v['ok']} == set(known)
        assert len(known) == len(set(known))
    baseline = {k for k, v in results.items() if not v['ok']}
    negative = {k for k, v in load(SESSION / 'p1102-negative-result.json').items() if not v['ok']}
    assert negative - baseline == {'wt-global-api-C_Item.IsItemBindToAccountUntilEquip-43'}
    assert not baseline - negative
    altered = load(SESSION / 'p1102-negative-register.json')
    different = [(a, b) for a, b in zip(register['entries'], altered['entries']) if a != b]
    assert len(register['entries']) == len(altered['entries']) == 34
    assert len(different) == 1
    original, changed = different[0]
    assert {**original, 'direction': 'removed'} == changed
    review = load(SESSION / 'p1102-gap-review.json')
    assert Counter(r['decision'] for r in review) == {'closed': 4, 'retained gap': 12}
    assert {r['source_id'] for r in review if r['decision'] == 'retained gap'} == baseline
    reproduction = load(SESSION / 'p1102-register-reproduction.json')
    assert {r['patch'] for r in reproduction} == set(PATCHES)
    assert all(r['byte_identical'] and r['exit_code'] == 0 for r in reproduction)
    preserved = load(SESSION / 'p1102-preserved-inputs.json')
    assert all(digest(ROOT / r['path']) == r['baseline_sha256'] for r in preserved)
    proof = {r['scope']: r for r in load(SESSION / 'p1102-proof.json')}
    required = ['behavior-green', 'p1102-sweep', 'prefork-new', 'prefork-weekly',
                'extract-fixtures', 'register-fixtures', 'fmt-check',
                'mists-test-check', 'retail-binary-build', 'bounded-retail-startup']
    required += ['patch_' + p.replace('.', '_') + '_publication_sweep' for p in PATCHES[1:]]
    assert all(proof[k]['exit_code'] == 0 and not proof[k]['invalidated'] for k in required)
    assert proof['mists-test-check']['non_vendor_warnings'] == []
    assert proof['bounded-retail-startup']['stdout'].strip() == '[]'
    assert proof['negative-control']['exit_code'] == 101
    for key in ('prefork-new', 'prefork-weekly'):
        assert any('1 passed; 0 failed; 1 total' in line for line in proof[key]['summary'])
    assert any('2 passed; 0 failed' in line for line in proof['behavior-green']['summary'])
    for row in proof.values():
        assert row['cwd'] == str(ROOT)
        if row['command'][0] == 'cargo' and 'environment' in row:
            assert row['environment']['CARGO_TARGET_DIR'] == str(ROOT / 'target')
    print('PASS: 263 unique IDs; 34 inventory + 229 extract; 14 exact isolated sweeps; source hashes, preserved inputs, reproduction, negative control and portable proof')


if __name__ == '__main__':
    main()
