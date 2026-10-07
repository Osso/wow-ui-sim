#!/usr/bin/env python3
"""Validate retained 10.1.7 occurrence accounting and revision-scoped local proof."""
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
PATCHES = [
    '10.1.7', '10.2.5', '10.2.6', '10.2.7', '11.0.0', '11.0.2',
    '11.0.5', '11.0.7', '11.1.0', '11.1.5', '11.1.7', '11.2.0',
    '11.2.5', '11.2.7', '12.0.0', '12.0.1', '12.0.5', '12.0.7', '12.1.0',
]


def read_json(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_source():
    sources = ROOT / 'data/patch-api/sources'
    register = read_json(sources / '10.1.7-wikitext-register.json')
    provenance = read_json(sources / '10.1.7-api-changes.provenance.json')
    coverage = read_json(sources / '10.1.7-page-coverage.json')
    assert provenance['pageid'] == 442982
    assert register['source']['revid'] == provenance['wikitext']['revid'] == 6473483
    assert digest(ROOT / register['source']['path']) == register['source']['sha256']
    assert register['source']['sha256'] == provenance['wikitext']['sha256']
    assert digest(ROOT / coverage['source_register']) == coverage['source_sha256']
    assert len(register['entries']) == 48
    assert Counter(row['direction'] for row in register['entries']) == {
        'added': 45, 'removed': 3}
    drift = [row for row in register['header_counts']
             if row['parsed_count'] != row['header_count']]
    assert [(row['section'], row['header_count'], row['parsed_count']) for row in drift] == [
        ('global-api', 19, 28), ('events', 8, 3)]
    spec = importlib.util.spec_from_file_location(
        'patch_extract', ROOT / 'tools/extract_patch_non_inventory.py')
    extractor = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(extractor)
    text_path = ROOT / coverage['non_inventory_source']['path']
    assert digest(text_path) == coverage['non_inventory_source']['sha256']
    assert extractor.extract_text((ROOT / register['source']['path']).read_text(),
                                  preserve_examples=True) == text_path.read_text()
    supplemental = extractor.seed_rows(text_path.read_text(), '10.1.7')
    assert len(supplemental) == 55
    inventory_ids = {row['id'] for row in register['entries']}
    extra_ids = {row['source_id'] for row in supplemental}
    rows = coverage['source_rows']
    assert len(rows) == len({row['source_id'] for row in rows}) == 103
    assert {row['source_id'] for row in rows} == inventory_ids | extra_ids
    assert Counter(row['status'] for row in rows) == {
        'partial-development-green': 31, 'bounded-coverage': 5,
        'audit-pending': 56, 'metadata-only': 11}
    scout = read_json(EVIDENCE / 'p1017-extract-scout.json')
    assert len(scout) == 55
    assert {row['source_id'] for row in scout} == extra_ids
    assert all(row['reason'] and row['statement'] for row in scout)
    assert all(row['note'] for row in rows)
    return register['entries']


def validate_sweeps(inventory):
    table = read_json(EVIDENCE / 'p1017-sweep-table.json')
    assert [row['patch'] for row in table] == PATCHES
    latest = {}
    for patch in PATCHES[1:]:
        register = read_json(ROOT / f'data/patch-api/sources/{patch}-wikitext-register.json')
        for entry in register['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    for row in table:
        results = read_json(ROOT / row['result'])
        fixture = read_json(ROOT / ('tests/data/patch_' + row['patch'].replace('.', '_') +
                                    '_sweep_known_gaps.json'))
        failures = {key for key, value in results.items() if not value['ok']}
        assert failures == set(fixture)
        assert len(results) == row['rows']
        assert sum(value['ok'] for value in results.values()) == row['ok']
        assert len(failures) == row['gaps'] and row['exit'] == 0
    results = read_json(EVIDENCE / 'p1017-sweep-result.json')
    for entry in inventory:
        later = latest.get(entry['symbol'])
        removed = entry['direction'] == 'removed'
        reverse = later is not None and (later['direction'] == 'removed') != removed
        expected = results[entry['id']]['expected']
        assert expected['publication'] == ('absent' if removed != reverse else 'published')
        assert expected['superseded_by'] == (later['id'] if reverse else None)
    review = read_json(EVIDENCE / 'p1017-gap-review.json')
    before = {key for key, value in results.items() if not value['ok']}
    assert len(review) == 14 and {row['source_id'] for row in review} == before
    assert all(row['reason'] and row['decision'] == 'retained-gap' for row in review)
    control = read_json(EVIDENCE / 'p1017-negative-control.json')
    negative = read_json(EVIDENCE / 'p1017-negative-result.json')
    after = {key for key, value in negative.items() if not value['ok']}
    assert after - before == {control['changed_id']} and not before - after
    assert (len(before), len(after), control['exit_code']) == (14, 15, 101)
    mutated = read_json(EVIDENCE / 'p1017-negative-register.json')['entries']
    differences = [(left['id'], key) for left, right in zip(inventory, mutated)
                   for key in left if left[key] != right[key]]
    assert differences == [(control['changed_id'], 'direction')]


def validate_proof():
    reproduction = read_json(EVIDENCE / 'p1017-register-reproduction.json')
    assert len(reproduction) == 18
    assert all(row['exit_code'] == 0 and row['byte_identical'] for row in reproduction)
    preserved = read_json(EVIDENCE / 'p1017-preserved-inputs.json')['files']
    assert len(preserved) == 116
    assert all(digest(ROOT / row['path']) == row['sha256'] for row in preserved)
    scopes = read_json(EVIDENCE / 'p1017-proof-scopes.json')
    assert all(digest(ROOT / path) == sha for path, sha in scopes['source_hashes'].items())
    warning = read_json(EVIDENCE / 'p1017-warning-boundary.json')
    assert warning['count'] == 0 and not warning['non_vendor_warnings']
    startup = read_json(EVIDENCE / 'p1017-startup-result.json')
    assert startup == {'exit_code': 0, 'json': []}
    assert (EVIDENCE / 'p1017-startup.log').read_text().startswith('[]\n')
    proof = read_json(EVIDENCE / 'p1017-proof.json')
    for row in proof:
        assert row['cwd'] == '/home/osso/.worktrees/wow-ui-sim-p1017-page'
        assert len(row['revision']) == 40
        assert digest(ROOT / row['log']) == row['log_sha256']
        target = row.get('environment', {}).get('CARGO_TARGET_DIR')
        assert target is None or target == row['cwd'] + '/target'
        if not row['invalidated']:
            assert row['exit_code'] == (101 if row['scope'] == 'one-row negative control' else 0)
    active = [row for row in proof if not row['invalidated']]
    assert len([row for row in active if row['scope'].startswith('isolated publication sweep')]) == 19
    assert any(row['scope'] == 'full cached ping template attributes and unit-target DTO'
               for row in active)
    assert any(row['scope'] == 'runtime typed attributes, override/OnLoad and notification boundary'
               for row in active)
    supersessions = read_json(EVIDENCE / 'p1017-possible-1020-supersessions.json')
    assert supersessions['matches'] == []


if __name__ == '__main__':
    entries = validate_source()
    validate_sweeps(entries)
    validate_proof()
    print('PASS: 103 source IDs, 19 isolated sweeps, 14 exact gaps, one-row negative control, '
          'typed template behavior, cached prefork, source-scoped proof and 116 preserved inputs')
