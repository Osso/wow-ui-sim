#!/usr/bin/env python3
"""Validate retained accounting/proof artifacts, not runtime/native parity."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
SESSION = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
PATCHES = ('11.0.7', '11.1.0', '11.1.5', '11.1.7', '11.2.0', '11.2.5',
           '11.2.7', '12.0.0', '12.0.1', '12.0.5', '12.0.7', '12.1.0')
EXPECTED = ((98, 70, 28), (116, 97, 19), (125, 89, 36), (48, 40, 8),
            (162, 135, 27), (163, 118, 45), (508, 414, 94), (1010, 989, 21),
            (225, 222, 3), (363, 352, 11), (174, 171, 3), (778, 773, 5))


def load(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources(register, coverage):
    raw = SOURCES / '11.0.7-api-changes.wikitext'
    provenance = load(SOURCES / '11.0.7-api-changes.provenance.json')
    assert provenance['pageid'] == 609319
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 6726777
    assert provenance['wikitext']['revision_timestamp'] == '2026-05-25T20:32:18Z'
    assert provenance['wikitext']['retrieved'] == '2026-10-07'
    assert digest(raw) == provenance['wikitext']['sha256'] == register['source']['sha256']
    assert digest(SOURCES / '11.0.7-wikitext-register.json') == coverage['source_sha256']
    assert len(register['header_counts']) == 8
    assert all(r['header_count'] == r['parsed_count'] for r in register['header_counts'])
    assert digest(SOURCES / '11.0.7-api-changes.txt') == coverage['non_inventory_source']['sha256']
    reproduction = load(SESSION / 'p1107-register-reproduction.json')
    assert {r['patch'] for r in reproduction} == set(PATCHES)
    assert all(r['byte_identical'] for r in reproduction)
    for item in load(SESSION / 'p1107-preserved-inputs.json'):
        assert digest(ROOT / item['path']) == item['baseline_sha256']


def validate_inventory(register, coverage, results):
    entries = {r['id']: r for r in register['entries']}
    assert len(entries) == len(register['entries']) == 98
    assert set(entries) == set(results)
    latest = {}
    for patch in PATCHES[1:]:
        for entry in load(SOURCES / f'{patch}-wikitext-register.json')['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    ledger = {r['source_id']: r for r in coverage['source_rows']}
    aliases = 0
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
        capabilities = [] if status in ('audit-pending', 'metadata-only') else ['publication-sweep-11-0-7']
        assert ledger[key]['capabilities'] == capabilities
        if status == 'bounded-coverage':
            aliases += 'deprecated-fallback=' in results[key]['observed']['detail']
    assert aliases == 2
    searches = load(SESSION / 'p1107-retirement-consumers.json')
    assert {row['source_id'] for row in searches} == {key for key, row in entries.items() if row['direction'] == 'removed'}
    retired = ('HideWorldLootObjectCallout', 'SetWorldLootObjectCalloutFromGUID',
               'SwapWorldLootObjectCallout', 'GetCurrentWorldLootObjectSwapInventoryType')
    for row in searches:
        if row['symbol'].split('.')[-1] in retired:
            assert row['exit_code'] == 1 and row['matches'] == [] and row['stderr'] == ''


def validate_extract(register, coverage):
    module_spec = importlib.util.spec_from_file_location('extract', ROOT / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(module_spec)
    module_spec.loader.exec_module(module)
    text = (SOURCES / '11.0.7-api-changes.txt').read_text()
    assert module.extract_text((SOURCES / '11.0.7-api-changes.wikitext').read_text()) == text
    seeded = module.seed_rows(text, '11.0.7')
    scout = load(SESSION / 'p1107-extract-scout.json')
    assert len(seeded) == len(scout) == 83
    expected = {r['id'] for r in register['entries']} | {r['source_id'] for r in seeded}
    ledger = {r['source_id']: r for r in coverage['source_rows']}
    assert len(ledger) == len(coverage['source_rows']) == len(expected) == 181
    assert set(ledger) == expected
    assert len({r['source_id'] for r in scout}) == len(scout)
    assert {r['source_id'] for r in scout} == {r['source_id'] for r in seeded}
    for row in seeded:
        assert ledger[row['source_id']] == row
    lines = text.splitlines()
    for row in scout:
        assert row['source_text'] == lines[row['line'] - 1].strip()
        assert not row['behavioral_credit']
        assert row['status'] == ledger[row['source_id']]['status']
    assert Counter(r['batch'] for r in scout) == {
        'B01-profiler': 9, 'B02-lfg-search': 1, 'B03-enums': 31,
        'B04-structures': 28, 'editorial': 14}
    counts = Counter(r['status'] for r in coverage['source_rows'])
    assert dict(counts) == coverage['summary'] == {
        'partial-development-green': 56, 'bounded-coverage': 11,
        'audit-pending': 97, 'metadata-only': 17}


def validate_results(results):
    for patch, expected in zip(PATCHES, EXPECTED):
        observed = load(SESSION / ('p' + patch.replace('.', '') + '-sweep-result.json'))
        known = load(ROOT / 'tests/data' / ('patch_' + patch.replace('.', '_') + '_sweep_known_gaps.json'))
        gaps = {key for key, row in observed.items() if not row['ok']}
        assert gaps == set(known)
        assert len(known) == len(set(known))
        assert (len(observed), len(observed) - len(gaps), len(gaps)) == expected
    negative = load(SESSION / 'p1107-negative-result.json')
    normal_gaps = {k for k, v in results.items() if not v['ok']}
    negative_gaps = {k for k, v in negative.items() if not v['ok']}
    assert negative_gaps - normal_gaps == {'wt-global-api-C_AccountStore.BeginPurchase-43'}
    assert not normal_gaps - negative_gaps
    original = load(SOURCES / '11.0.7-wikitext-register.json')
    changed = load(SESSION / 'p1107-negative-register.json')
    changed_rows = [(a, b) for a, b in zip(original['entries'], changed['entries']) if a != b]
    assert len(original['entries']) == len(changed['entries']) == 98
    assert len(changed_rows) == 1
    a, b = changed_rows[0]
    assert {**a, 'direction': 'removed'} == b
    review = load(SESSION / 'p1107-gap-review.json')
    assert Counter(r['outcome'] for r in review['outcomes']) == {'retained-gap': 28, 'bounded-closure': 5}
    assert {r['source_id'] for r in review['outcomes'] if r['outcome'] == 'retained-gap'} == normal_gaps
    proof = load(SESSION / 'p1107-proof.json')
    records = {r['name']: r for r in proof['results']}
    required = ['p1107-sweep', 'fixes-green', 'mists-check', 'retail-build', 'startup',
                'fmt-check', 'prefork-cached-raid-targets-green',
                'prefork-deprecated-lfg-alias', 'prefork-deprecated-lfg-wrapper', 'extract-green']
    required += ['p' + p.replace('.', '') + '-sweep' for p in PATCHES[1:]]
    assert all(records[name]['exit_code'] == 0 and not records[name]['invalidated'] for name in required)
    assert records['negative-control']['exit_code'] == 101
    assert records['startup']['stdout'].strip() == '[]'
    for row in proof['results']:
        assert row['cwd'] == str(ROOT)
        assert row['environment']['CARGO_TARGET_DIR'] == str(ROOT / 'target')
    summaries = {row['name']: row for row in load(SESSION / 'p1107-verification-summary.json')}
    for name in required:
        assert summaries[name]['revision'] == records[name]['revision']
        assert summaries[name]['command'] == records[name]['command']
        assert summaries[name]['exit_code'] == records[name]['exit_code']
    for patch in PATCHES:
        name = 'p' + patch.replace('.', '') + '-sweep'
        assert len(summaries[name]['test_summary']) == 1
        assert '1 passed; 0 failed' in summaries[name]['test_summary'][0]
    assert '2 passed; 0 failed' in summaries['fixes-green']['test_summary'][0]
    for name in ('prefork-cached-raid-targets-green', 'prefork-deprecated-lfg-alias',
                 'prefork-deprecated-lfg-wrapper'):
        assert '1 passed; 0 failed; 1 total' in summaries[name]['test_summary'][0]
    assert summaries['startup']['stdout'] == '[]'
    warnings = summaries['mists-check']['warning_lines']
    assert summaries['mists-check']['non_vendor_warning_count'] == 0
    assert len(warnings) == 7
    assert all('iced-wgpu-patched/Cargo.toml:' in line or '`iced_wgpu` (manifest)' in line for line in warnings)


def main():
    register = load(SOURCES / '11.0.7-wikitext-register.json')
    coverage = load(SOURCES / '11.0.7-page-coverage.json')
    results = load(SESSION / 'p1107-sweep-result.json')
    validate_sources(register, coverage)
    validate_inventory(register, coverage, results)
    validate_extract(register, coverage)
    validate_results(results)
    print('PASS: revision/hash, 98 inventory + 83 extract IDs, 28 exact gaps, '
          'negative control, chronological credit and twelve isolated result maps')


if __name__ == '__main__':
    main()
