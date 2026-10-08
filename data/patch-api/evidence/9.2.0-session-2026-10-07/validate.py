#!/usr/bin/env python3
"""Validate retained audit artifacts without rerunning runtime tests."""
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
    register = read_json(SOURCES / '9.2.0-wikitext-register.json')
    ledger = read_json(SOURCES / '9.2.0-page-coverage.json')
    results = read_json(EVIDENCE / 'p920_sweep_out-results.json')
    gaps = read_json(ROOT / 'tests/data/patch_9_2_0_sweep_known_gaps.json')
    extract = read_json(EVIDENCE / 'p920-extract-scout.json')
    context = read_json(EVIDENCE / 'p920-inventory-context.json')
    ids = [row['source_id'] for row in ledger['source_rows']]
    expected = {row['id'] for row in register['entries']}
    expected.update(row['source_id'] for row in extract + context)
    assert len(ids) == len(set(ids)) == 95
    assert set(ids) == expected
    assert len(register['entries']) == len(results) == 80
    assert set(results) == {row['id'] for row in register['entries']}
    assert set(gaps) == {key for key, value in results.items() if not value['ok']}
    assert len(gaps) == 26
    review = read_json(EVIDENCE / 'p920-gap-review.json')
    assert {row['source_id'] for row in review} == set(gaps)
    assert all(row['reason'] and row['literal'] for row in review)
    assert Counter(row['status'] for row in ledger['source_rows']) == {
        'bounded-coverage': 12, 'partial-development-green': 42,
        'audit-pending': 32, 'metadata-only': 9,
    }
    assert all(row['capabilities'] == [] for row in ledger['source_rows']
               if row['status'] in ('audit-pending', 'metadata-only'))
    assert ledger['source_sha256'] == sha256(SOURCES / '9.2.0-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == sha256(SOURCES / '9.2.0-api-changes.txt')
    assert register['source']['sha256'] == sha256(SOURCES / '9.2.0-api-changes.wikitext')
    raw = (SOURCES / '9.2.0-api-changes.wikitext').read_text().splitlines()
    for entry in register['entries']:
        assert entry['symbol'] in raw[entry['wikitext_line'] - 1]
    for row in review + context:
        assert raw[row['wikitext_line'] - 1] == row['literal']
    lines = (SOURCES / '9.2.0-api-changes.txt').read_text().splitlines()
    assert {row['extract_line'] for row in extract} == {
        number for number, line in enumerate(lines, 1) if line.strip()
    }
    assert all(row['wikitext_lines'] for row in extract)
    assert all(lines[row['extract_line'] - 1] == row['literal'] for row in extract)
    assert all(row['header_count'] == row['parsed_count'] for row in register['header_counts'])


def check_preservation():
    hashes = read_json(EVIDENCE / 'p920-preserved-inputs.json')
    assert len(hashes) == 143
    assert all(sha256(ROOT / path) == digest for path, digest in hashes.items())
    before = read_json(EVIDENCE / 'p920-extraction-before.json')
    after = read_json(EVIDENCE / 'p920-extraction-after.json')
    previous = {(row['patch'], row['preserve_examples']): row['exit'] for row in before}
    current = {(row['patch'], row['preserve_examples']): row['exit'] for row in after}
    assert len(previous) == 54 and len(current) == 56
    assert all(current[key] == exit_code for key, exit_code in previous.items())
    assert current[('9.2.0', False)] == current[('9.2.0', True)] == 0
    assert sum(exit_code != 0 for exit_code in previous.values()) == 15


def check_proof():
    table = read_json(EVIDENCE / 'p920-sweep-table.json')
    assert len(table) == 28 and sum(row['rows'] for row in table) == 6459
    for row in table:
        result = read_json(EVIDENCE / row['output'])
        register = read_json(SOURCES / f"{row['patch']}-wikitext-register.json")
        fixture = read_json(ROOT / 'tests/data' /
                            f"patch_{row['patch'].replace('.', '_')}_sweep_known_gaps.json")
        assert set(result) == {entry['id'] for entry in register['entries']}
        assert {key for key, value in result.items() if not value['ok']} == set(fixture)
        assert row['rows'] == row['ok'] + row['gaps'] == len(result)
    negative = read_json(EVIDENCE / 'p920-negative-result.json')
    assert negative == {'new_gaps': ['wt-events-FIRST_FRAME_RENDERED-115'],
                        'resolved_gaps': [], 'same_ids': True, 'exit': 1}
    original = read_json(SOURCES / '9.2.0-wikitext-register.json')
    mutated = read_json(EVIDENCE / 'p920-negative-register.json')
    changed = [(before, after) for before, after in
               zip(original['entries'], mutated['entries']) if before != after]
    assert len(changed) == 1
    before, after = changed[0]
    assert {**before, 'direction': 'removed'} == after
    assert before['direction'] == 'added'
    assert read_json(EVIDENCE / 'p920-startup.stdout') == []
    proof = read_json(EVIDENCE / 'p920-proof.json')
    required = {'retirement-green', 'all-sweeps', 'cached-retirements',
                'format-check', 'default-check', 'mists-check', 'mists-regression',
                'Bounded separately rebuilt retail startup'}
    passing = {row['scope'] for row in proof if row['exit'] == 0 and not row['invalidated']}
    assert required <= passing
    assert '29 passed; 0 failed; 29 total' in (EVIDENCE / 'p920-all-sweeps.txt').read_text()
    assert 'Ran 12 tests' in (EVIDENCE / 'p920-generator-green.txt').read_text()
    assert 'Ran 23 tests' in (EVIDENCE / 'p920-extract-fixtures.txt').read_text()
    warnings = read_json(EVIDENCE / 'p920-warning-boundary.json')
    assert all(not report['non_vendor_warnings'] for report in warnings.values())


def main():
    check_accounting()
    check_preservation()
    check_proof()
    print('PASS: 95 IDs, 28 exact sweeps, 143 preserved inputs, 54 prior extract outcomes, negative control and targeted proof')


if __name__ == '__main__':
    main()
