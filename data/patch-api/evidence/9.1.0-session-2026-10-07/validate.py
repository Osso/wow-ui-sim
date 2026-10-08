#!/usr/bin/env python3
"""Validate retained publication artifacts; never rerun runtime tests."""
import hashlib
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_accounting():
    register = read_json(SOURCES / '9.1.0-wikitext-register.json')
    ledger = read_json(SOURCES / '9.1.0-page-coverage.json')
    result = read_json(EVIDENCE / 'p910_sweep_out-results.json')
    gaps = read_json(ROOT / 'tests/data/patch_9_1_0_sweep_known_gaps.json')
    review = read_json(EVIDENCE / 'p910-gap-review.json')
    extract = read_json(EVIDENCE / 'p910-extract-scout.json')
    context = read_json(EVIDENCE / 'p910-inventory-context.json')
    ids = [row['source_id'] for row in ledger['source_rows']]
    expected = {entry['id'] for entry in register['entries']}
    expected.update(row['source_id'] for row in extract + context)
    assert len(ids) == len(set(ids)) == 192
    assert set(ids) == expected
    assert len(register['entries']) == len(result) == 179
    assert set(result) == {entry['id'] for entry in register['entries']}
    assert set(gaps) == {key for key, value in result.items() if not value['ok']}
    assert len(gaps) == len(review) == 53
    assert {row['source_id'] for row in review} == set(gaps)
    assert all(row['reason'] and row['literal'] for row in review)
    assert Counter(row['status'] for row in ledger['source_rows']) == {
        'bounded-coverage': 61, 'partial-development-green': 65,
        'audit-pending': 56, 'metadata-only': 10,
    }
    assert all(not row['capabilities'] for row in ledger['source_rows']
               if row['status'] in ('audit-pending', 'metadata-only'))
    assert ledger['source_sha256'] == sha256(SOURCES / '9.1.0-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(SOURCES / '9.1.0-api-changes.txt')
    assert register['source']['sha256'] == sha256(SOURCES / '9.1.0-api-changes.wikitext')
    provenance = read_json(SOURCES / '9.1.0-api-changes.provenance.json')
    assert provenance['generator_flags'] == ['--expand-shared-changes', '--skip-plain-scripts-label']
    assert provenance['revid'] == register['source']['revid'] == 167980
    fetch = read_json(EVIDENCE / 'p910-fetch.json')
    page = fetch['query']['pages']['17181']
    assert page['revisions'][0]['revid'] == 167980
    raw = (SOURCES / '9.1.0-api-changes.wikitext').read_text().splitlines()
    assert page['revisions'][0]['slots']['main']['*'].splitlines() == raw
    for entry in register['entries']:
        assert entry['symbol'] in raw[entry['wikitext_line'] - 1]
    for row in review + context:
        assert row['literal'] == raw[row['wikitext_line'] - 1]
    text = (SOURCES / '9.1.0-api-changes.txt').read_text().splitlines()
    assert len(extract) == 12 and len(context) == 1
    assert {row['extract_line'] for row in extract} == {
        number for number, line in enumerate(text, 1) if line.strip()
    }
    assert all(row['wikitext_lines'] for row in extract)
    assert all(text[row['extract_line'] - 1] == row['literal'] for row in extract)
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])


def check_retirements():
    initial = read_json(EVIDENCE / 'p910-initial-results.json')
    final = read_json(EVIDENCE / 'p910_sweep_out-results.json')
    closed = {key for key in initial if not initial[key]['ok'] and final[key]['ok']}
    assert len(closed) == 21
    assert sum(not value['ok'] for value in initial.values()) == 74
    assert not any(initial[key]['ok'] and not final[key]['ok'] for key in initial)
    scans = read_json(EVIDENCE / 'p910-removal-consumers.json')
    assert len(scans) == 51
    by_id = {row['id']: row for row in scans}
    assert all(not by_id[key]['qualified'] and not by_id[key]['bare'] for key in closed)
    assert all(initial[key]['expected']['direction'] == 'removed' for key in closed)
    assert 'C_BarberShop.OldBarberShopLoaded' in (EVIDENCE / 'p910-retirement-red.txt').read_text()
    assert '1 passed; 0 failed' in (EVIDENCE / 'p910-retirement-green.txt').read_text()
    assert '2 passed; 0 failed; 2 total' in (EVIDENCE / 'p910-cached-green.txt').read_text()
    assert '1 passed; 0 failed' in (EVIDENCE / 'p910-mists-regression.txt').read_text()
    assert '1 passed; 0 failed' in (EVIDENCE / 'p910-mists-legacy-caller.txt').read_text()


def check_preservation():
    hashes = read_json(EVIDENCE / 'p910-preserved-inputs.json')
    assert len(hashes) == 153
    assert all(sha256(ROOT / path) == digest for path, digest in hashes.items())
    before = read_json(EVIDENCE / 'p910-extract-before.json')
    after = read_json(EVIDENCE / 'p910-extract-after.json')
    previous = {(row['patch'], row['preserve_examples']): (row['exit'], row['stdout']) for row in before}
    current = {(row['patch'], row['preserve_examples']): (row['exit'], row['stdout']) for row in after}
    assert len(previous) == 58 and len(current) == 60
    assert all(current[key] == value for key, value in previous.items())
    assert current[('9.1.0', False)][0] == current[('9.1.0', True)][0] == 0
    assert sum(value[0] != 0 for value in previous.values()) == 16
    registers = read_json(EVIDENCE / 'p910-register-reproduction.json')
    assert len(registers) == 30 and all(row['byte_identical'] for row in registers)
    assert all(row['matching_flags'] is not None for row in registers)


def check_sweeps_and_negative():
    table = read_json(EVIDENCE / 'p910-sweep-table.json')
    assert len(table) == 30 and sum(row['rows'] for row in table) == 6722
    for row in table:
        result = read_json(EVIDENCE / row['output'])
        register = read_json(SOURCES / f"{row['patch']}-wikitext-register.json")
        fixture = read_json(ROOT / 'tests/data' / f"patch_{row['patch'].replace('.', '_')}_sweep_known_gaps.json")
        assert set(result) == {entry['id'] for entry in register['entries']}
        assert {key for key, value in result.items() if not value['ok']} == set(fixture)
        assert row['rows'] == row['ok'] + row['gaps'] == len(result)
    own = read_json(SOURCES / '9.1.0-wikitext-register.json')
    mutated = read_json(EVIDENCE / 'p910-negative-register.json')
    changes = [(a, b) for a, b in zip(own['entries'], mutated['entries']) if a != b]
    assert len(changes) == 1
    before, after = changes[0]
    assert before['symbol'] == 'DISPLAY_EVENT_TOASTS'
    assert before['direction'] == 'added' and {**before, 'direction': 'removed'} == after
    negative = read_json(EVIDENCE / 'p910-negative-result.json')
    assert negative == {'before': 53, 'after': 54, 'new': [before['id']], 'resolved': [], 'same_ids': True}
    result = read_json(EVIDENCE / 'p910_sweep_out-results.json')
    negative_result = read_json(EVIDENCE / 'p910-negative-results.json')
    assert set(result) == set(negative_result)
    added = {key for key in result if result[key]['ok'] and not negative_result[key]['ok']}
    assert added == {before['id']}
    assert not any(not result[key]['ok'] and negative_result[key]['ok'] for key in result)


def check_proof_and_supersession():
    proof = read_json(EVIDENCE / 'p910-proof.json')
    passing = {row['scope'] for row in proof if row['exit'] == 0 and not row['invalidated']}
    assert {'format-check', 'default-check', 'mists-check', 'mists-regression',
            'mists-legacy-caller', 'retail-build', 'retail-startup'} <= passing
    aggregate = next(row for row in proof if row.get('passing_cases'))
    assert aggregate['exit'] == 1 and len(aggregate['passing_cases']) == 30
    assert '30 passed; 1 failed; 31 total' in (EVIDENCE / 'p910-all-sweeps.txt').read_text()
    assert 'Ran 15 tests' in (EVIDENCE / 'test_gen_patch_wikitext_register-green.txt').read_text()
    assert 'Ran 25 tests' in (EVIDENCE / 'test_extract_patch_non_inventory-green.txt').read_text()
    assert read_json(EVIDENCE / 'p910-startup.stdout') == []
    warnings = read_json(EVIDENCE / 'p910-warning-boundary.json')
    assert all(not row['non_vendor_warnings'] for row in warnings.values())
    for row in proof:
        if row.get('output_sha256'):
            assert sha256(EVIDENCE / row['output']) == row['output_sha256']
    intersections = read_json(EVIDENCE / 'p910-p915-intersections.json')['intersections']
    assert {row['gap_id'] for row in intersections} == {
        'wt-global-api-AcknowledgeAADCAlert-23',
        'wt-global-api-C_ItemUpgrade.GetItemLevelIncrement-42',
    }
    assert all(row['later_direction'] == 'removed' for row in intersections)


def main():
    check_accounting()
    check_retirements()
    check_preservation()
    check_sweeps_and_negative()
    check_proof_and_supersession()
    print('PASS: 192 IDs, 21 retirements, 53 exact gaps, 30 sweep scopes, 153 preserved inputs, 58 prior modes, negative control and targeted proof')


if __name__ == '__main__':
    main()
