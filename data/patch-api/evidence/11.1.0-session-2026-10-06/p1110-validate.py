#!/usr/bin/env python3
"""Validate retained page accounting, not runtime/native parity."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCHES = ('11.1.0', '11.1.5', '11.1.7', '11.2.0', '11.2.5', '11.2.7',
           '12.0.0', '12.0.1', '12.0.5', '12.0.7', '12.1.0')
EXPECTED = ((116, 97, 19), (125, 89, 36), (48, 40, 8), (162, 135, 27),
            (163, 118, 45), (508, 414, 94), (1010, 989, 21), (225, 222, 3),
            (363, 352, 11), (174, 171, 3), (778, 773, 5))


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources(register, coverage):
    raw = SOURCES / '11.1.0-api-changes.wikitext'
    provenance = load(SOURCES / '11.1.0-api-changes.provenance.json')
    assert provenance['pageid'] == 616105
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6726776
    assert digest(raw) == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '11.1.0-wikitext-register.json') == coverage['source_sha256']
    assert len(register['header_counts']) == 8
    assert all(r['header_count'] == r['parsed_count'] for r in register['header_counts'])
    assert digest(SOURCES / '11.1.0-api-changes.txt') == coverage['non_inventory_source']['sha256']
    reproduction = load(SESSION / 'p1110-register-reproduction.json')
    assert {r['patch'] for r in reproduction} == set(PATCHES[1:])
    assert all(r['byte_identical'] for r in reproduction)
    for item in load(SESSION / 'p1110-preserved-inputs.json'):
        assert digest(ROOT / item['path']) == item['baseline_sha256']


def validate_inventory(register, coverage, results):
    entries = {r['id']: r for r in register['entries']}
    assert len(entries) == len(register['entries']) == 116
    assert set(entries) == set(results)
    latest = {}
    for patch in PATCHES[1:]:
        for entry in load(SOURCES / f'{patch}-wikitext-register.json')['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    ledger = {r['source_id']: r for r in coverage['source_rows']}
    for key, entry in entries.items():
        expected = results[key]['expected']
        assert expected['symbol'] == entry['symbol']
        assert expected['direction'] == entry['direction']
        removed = entry['direction'] == 'removed'
        newer = latest.get(entry['symbol'])
        reversal = newer is not None and (newer['direction'] == 'removed') != removed
        assert expected['superseded_by'] == (newer['id'] if reversal else None)
        effective_removed = not removed if reversal else removed
        assert expected['publication'] == ('absent' if effective_removed else 'published')
        assert not results[key]['observed']['default_mismatch']
        status = ('audit-pending' if not results[key]['ok'] else
                  'metadata-only' if reversal else
                  'bounded-coverage' if removed else 'partial-development-green')
        assert ledger[key]['status'] == status
        capabilities = [] if status in ('audit-pending', 'metadata-only') else ['publication-sweep-11-1-0']
        assert ledger[key]['capabilities'] == capabilities
        if status == 'bounded-coverage':
            assert 'deprecated-fallback=' not in results[key]['observed']['detail']


def validate_extract(register, coverage):
    module_spec = importlib.util.spec_from_file_location('extract', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(module_spec)
    module_spec.loader.exec_module(module)
    text = (SOURCES / '11.1.0-api-changes.txt').read_text()
    assert module.extract_text((SOURCES / '11.1.0-api-changes.wikitext').read_text()) == text
    seeded = module.seed_rows(text, '11.1.0')
    scout = load(SESSION / 'p1110-extract-scout.json')
    assert len(seeded) == len(scout) == 100
    expected = {r['id'] for r in register['entries']} | {r['source_id'] for r in seeded}
    ledger = {r['source_id']: r for r in coverage['source_rows']}
    assert len(ledger) == len(coverage['source_rows']) == len(expected) == 216
    assert set(ledger) == expected
    assert {r['source_id'] for r in scout} == {r['source_id'] for r in seeded}
    for row in seeded:
        assert ledger[row['source_id']]['status'] == row['status']
        assert ledger[row['source_id']]['capabilities'] == []
    counts = Counter(r['status'] for r in coverage['source_rows'])
    assert dict(counts) == coverage['summary'] == {
        'partial-development-green': 71, 'bounded-coverage': 18,
        'audit-pending': 106, 'metadata-only': 21}


def validate_results(results):
    for patch, expected in zip(PATCHES, EXPECTED):
        observed = load(SESSION / ('p' + patch.replace('.', '') + '-sweep-result.json'))
        known = load(ROOT / 'tests/data' / ('patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        gaps = {key for key, row in observed.items() if not row['ok']}
        assert gaps == set(known)
        assert len(known) == len(set(known))
        assert (len(observed), len(observed) - len(gaps), len(gaps)) == expected
    negative = load(SESSION / 'p1110-negative-result.json')
    normal_gaps = {k for k, v in results.items() if not v['ok']}
    negative_gaps = {k for k, v in negative.items() if not v['ok']}
    assert negative_gaps - normal_gaps == {'wt-global-api-C_WarbandScene.SetFavorite-91'}
    assert not normal_gaps - negative_gaps
    original = load(SOURCES / '11.1.0-wikitext-register.json')
    changed = load(SESSION / 'p1110-negative-register.json')
    changed_rows = [(a, b) for a, b in zip(original['entries'], changed['entries']) if a != b]
    assert len(changed_rows) == 1
    a, b = changed_rows[0]
    assert {**a, 'direction': 'removed'} == b
    review = load(SESSION / 'p1110-gap-review.json')
    assert Counter(r['outcome'] for r in review['outcomes']) == {'retained-gap': 19, 'bounded-closure': 3}
    assert {r['source_id'] for r in review['outcomes'] if r['outcome'] == 'retained-gap'} == normal_gaps
    proof = load(SESSION / 'p1110-proof.json')
    records = {r['name']: r for r in proof['results']}
    required = ['p1110-final-sweep', 'fixes-green', 'mists-check', 'retail-build', 'startup',
                'fmt-check', 'prefork-blizzard_collections_loads',
                'prefork-blizzard_deprecated_specialization_loads']
    required += ['p' + p.replace('.', '') + '-sweep' for p in PATCHES[1:]]
    assert all(records[name]['exit_code'] == 0 for name in required)
    assert records['negative-control']['exit_code'] == 101
    assert records['startup']['stdout'].strip() == '[]'


def main():
    register = load(SOURCES / '11.1.0-wikitext-register.json')
    coverage = load(SOURCES / '11.1.0-page-coverage.json')
    results = load(SESSION / 'p1110-sweep-result.json')
    validate_sources(register, coverage)
    validate_inventory(register, coverage, results)
    validate_extract(register, coverage)
    validate_results(results)
    print('PASS: revision/hash, 116 inventory + 100 extract IDs, 19 exact gaps, '
          'negative control, chronological credit and eleven isolated result maps')


if __name__ == '__main__':
    main()
