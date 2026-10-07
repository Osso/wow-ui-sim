#!/usr/bin/env python3
"""Validate retained accounting/proof contracts, not runtime/native parity."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCHES = ('11.0.5', '11.0.7', '11.1.0', '11.1.5', '11.1.7', '11.2.0',
           '11.2.5', '11.2.7', '12.0.0', '12.0.1', '12.0.5', '12.0.7', '12.1.0')
EXPECTED = ((48, 38, 10), (98, 70, 28), (116, 97, 19), (125, 89, 36),
            (48, 40, 8), (162, 135, 27), (163, 118, 45), (508, 414, 94),
            (1010, 989, 21), (225, 222, 3), (363, 352, 11), (174, 171, 3),
            (778, 773, 5))


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources(register, coverage):
    provenance = load(SOURCES / '11.0.5-api-changes.provenance.json')
    assert provenance['pageid'] == 601519
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6726778
    assert provenance['wikitext']['revision_timestamp'] == '2026-05-25T20:32:26Z'
    assert provenance['wikitext']['retrieved'] == '2026-10-07'
    assert digest(SOURCES / '11.0.5-api-changes.wikitext') == register['source']['sha256'] == provenance['wikitext']['sha256']
    assert digest(SOURCES / '11.0.5-wikitext-register.json') == coverage['source_sha256']
    assert digest(SOURCES / '11.0.5-api-changes.txt') == coverage['non_inventory_source']['sha256']
    assert len(register['header_counts']) == 8
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])
    reproduction = load(SESSION / 'p1105-register-reproduction.json')
    assert {row['patch'] for row in reproduction} == set(PATCHES)
    assert all(row['byte_identical'] and row['exit_code'] == 0 for row in reproduction)
    for row in load(SESSION / 'p1105-preserved-inputs.json'):
        assert digest(ROOT / row['path']) == row['baseline_sha256'], row['path']


def validate_inventory(register, coverage, results):
    entries = {row['id']: row for row in register['entries']}
    assert len(entries) == len(register['entries']) == 48
    assert set(entries) == set(results)
    latest = {}
    for patch in PATCHES[1:]:
        for row in load(SOURCES / f'{patch}-wikitext-register.json')['entries']:
            if row['direction'] != 'changed':
                latest[row['symbol']] = row
    ledger = {row['source_id']: row for row in coverage['source_rows']}
    aliases = 0
    for key, entry in entries.items():
        expected = results[key]['expected']
        assert expected['symbol'] == entry['symbol']
        assert expected['direction'] == entry['direction']
        removed = entry['direction'] == 'removed'
        newer = latest.get(entry['symbol'])
        reversal = newer is not None and (newer['direction'] == 'removed') != removed
        assert not reversal
        assert expected['superseded_by'] is None
        assert expected['publication'] == ('absent' if removed else 'published')
        assert not results[key]['observed']['default_mismatch']
        status = ('audit-pending' if not results[key]['ok'] else
                  'bounded-coverage' if removed else 'partial-development-green')
        assert ledger[key]['status'] == status
        assert ledger[key]['capabilities'] == ([] if status == 'audit-pending' else ['publication-sweep-11-0-5'])
        if status == 'bounded-coverage':
            aliases += 'deprecated-fallback=' in results[key]['observed']['detail']
    assert aliases == 1
    searches = load(SESSION / 'p1105-retirement-consumers.json')
    for symbol in ('C_AuctionHouse.RequestFavorites', 'C_MajorFactions.GetCovenantIDForMajorFaction'):
        matches = [row for row in searches if row['symbol'] == symbol]
        assert len(matches) == 2
        assert all(row['exit_code'] == 1 and not row['matches'] for row in matches)
    glue = next(row for row in searches if row['symbol'] == 'IsOnGlueScreen')
    assert any('Blizzard_DeprecatedGlue/Deprecated_Glue.lua:9:' in text for text in glue['matches'])


def validate_extract(register, coverage):
    spec = importlib.util.spec_from_file_location('extract', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    text = (SOURCES / '11.0.5-api-changes.txt').read_text()
    assert module.extract_text((SOURCES / '11.0.5-api-changes.wikitext').read_text()) == text
    seeded = module.seed_rows(text, '11.0.5')
    scout = load(SESSION / 'p1105-extract-scout.json')['assignments']
    assert len(seeded) == len(scout) == 34
    expected = {row['id'] for row in register['entries']} | {row['source_id'] for row in seeded}
    ledger = {row['source_id']: row for row in coverage['source_rows']}
    assert len(ledger) == len(coverage['source_rows']) == len(expected) == 82
    assert set(ledger) == expected
    assert len({row['source_id'] for row in scout}) == len(scout)
    assert {row['source_id'] for row in scout} == {row['source_id'] for row in seeded}
    for row in seeded:
        assert ledger[row['source_id']] == row
    for row in scout:
        number = int(row['source_id'].rsplit('-', 1)[1])
        assert row['statement'] == text.splitlines()[number - 1].strip()
        assert row['status'] == ledger[row['source_id']]['status']
        assert row['boundary']
    assert Counter(row['batch'] for row in scout) == {'B01-enums': 18, 'B02-structures': 6, 'editorial': 10}
    assert dict(Counter(row['status'] for row in coverage['source_rows'])) == coverage['summary'] == {
        'partial-development-green': 30, 'bounded-coverage': 8,
        'audit-pending': 34, 'metadata-only': 10}


def validate_results(results):
    for patch, expected in zip(PATCHES, EXPECTED):
        observed = load(SESSION / ('p' + patch.replace('.', '') + '-sweep-result.json'))
        known = load(ROOT / 'tests/data' / ('patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        gaps = {key for key, row in observed.items() if not row['ok']}
        assert gaps == set(known)
        assert len(known) == len(set(known))
        assert (len(observed), len(observed) - len(gaps), len(gaps)) == expected
    normal_gaps = {key for key, row in results.items() if not row['ok']}
    negative = load(SESSION / 'p1105-negative-result.json')
    negative_gaps = {key for key, row in negative.items() if not row['ok']}
    assert negative_gaps - normal_gaps == {'wt-global-api-C_BarberShop.HasAlteredForm-19'}
    assert not normal_gaps - negative_gaps
    original = load(SOURCES / '11.0.5-wikitext-register.json')
    changed = load(SESSION / 'p1105-negative-register.json')
    different = [(a, b) for a, b in zip(original['entries'], changed['entries']) if a != b]
    assert len(original['entries']) == len(changed['entries']) == 48
    assert len(different) == 1
    a, b = different[0]
    assert {**a, 'direction': 'removed'} == b
    review = load(SESSION / 'p1105-gap-review.json')
    assert Counter(row['outcome'] for row in review['outcomes']) == {'retained-gap': 10, 'closed-bounded': 5}
    assert {row['source_id'] for row in review['outcomes'] if row['outcome'] == 'retained-gap'} == normal_gaps
    initial = load(SESSION / 'p1105-initial-result.json')
    assert {row['source_id'] for row in review['outcomes']} == {key for key, row in initial.items() if not row['ok']}


def validate_proof():
    records = {row['name']: row for row in load(SESSION / 'p1105-proof.json')['results']}
    required = ['fixes-final', 'prefork-1', 'prefork-2', 'prefork-3', 'mists-check',
                'retail-build', 'startup', 'fmt-check', 'extract-register-fixtures']
    required += ['sweep-' + patch for patch in PATCHES]
    assert all(records[name]['exit_code'] == 0 and not records[name]['invalidated'] for name in required)
    assert records['negative-control']['exit_code'] == 101
    assert records['startup']['summary']['stdout'].strip() == '[]'
    assert records['mists-check']['summary']['non_vendor_warnings'] == []
    assert len(records['mists-check']['summary']['warnings']) == 7
    assert '3 passed; 0 failed' in records['fixes-final']['summary']['test_summary'][0]
    for name in ('prefork-1', 'prefork-2', 'prefork-3'):
        assert any('1 passed; 0 failed; 1 total' in line for line in records[name]['summary']['outcome_lines'])
    for patch in PATCHES:
        assert '1 passed; 0 failed' in records['sweep-' + patch]['summary']['test_summary'][0]
    for row in records.values():
        assert row['cwd'] == str(ROOT)
        if row['command'][0] == 'cargo':
            assert row['environment']['CARGO_TARGET_DIR'] == str(ROOT / 'target')
    assert records['initial-sweep']['exit_code'] == records['fixes-red']['exit_code'] == records['chroma-red']['exit_code'] == 101
    assert records['extract-red']['exit_code'] == 1


def main():
    register = load(SOURCES / '11.0.5-wikitext-register.json')
    coverage = load(SOURCES / '11.0.5-page-coverage.json')
    results = load(SESSION / 'p1105-sweep-result.json')
    validate_sources(register, coverage)
    validate_inventory(register, coverage, results)
    validate_extract(register, coverage)
    validate_results(results)
    validate_proof()
    print('PASS: 82 unique source IDs; 48 inventory + 34 extract; 13 exact sweeps; hashes, reproduction, preserved inputs, negative control and portable proof')


if __name__ == '__main__':
    main()
