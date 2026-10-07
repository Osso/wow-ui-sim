#!/usr/bin/env python3
"""Validate retained 10.2.5 source accounting and revision-scoped local evidence."""
import argparse
import hashlib
import importlib.util
import json
from collections import Counter
from pathlib import Path


def read_json(root, path):
    return json.loads((root / path).read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_extractor(root):
    spec = importlib.util.spec_from_file_location(
        'patch_extract', root / 'tools/extract_patch_non_inventory.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_source(root, evidence):
    base = Path('data/patch-api/sources')
    register = read_json(root, base / '10.2.5-wikitext-register.json')
    provenance = read_json(root, base / '10.2.5-api-changes.provenance.json')
    coverage = read_json(root, base / '10.2.5-page-coverage.json')
    assert provenance['pageid'] == 564286
    assert provenance['wikitext']['revid'] == register['source']['revid'] == 5993852
    assert digest(root / register['source']['path']) == register['source']['sha256']
    assert register['source']['sha256'] == provenance['wikitext']['sha256']
    assert digest(root / coverage['source_register']) == coverage['source_sha256']
    text_path = root / coverage['non_inventory_source']['path']
    assert digest(text_path) == coverage['non_inventory_source']['sha256']
    extractor = load_extractor(root)
    assert extractor.extract_text((root / register['source']['path']).read_text(),
                                  preserve_examples=True) == text_path.read_text()
    inventory = register['entries']
    assert len(inventory) == 59
    assert Counter(row['direction'] for row in inventory) == {
        'added': 43, 'removed': 13, 'changed': 3}
    assert all(row['header_count'] == row['parsed_count']
               for row in register['header_counts'])
    expected = {row['id'] for row in inventory}
    expected.update(row['source_id'] for row in extractor.seed_rows(text_path.read_text(), '10.2.5'))
    rows = coverage['source_rows']
    assert len(rows) == len(expected) == 415
    assert {row['source_id'] for row in rows} == expected
    assert Counter(row['status'] for row in rows) == {
        'bounded-coverage': 10, 'partial-development-green': 35,
        'audit-pending': 96, 'metadata-only': 274}
    scout = read_json(root, evidence / 'p1025-extract-scout.json')
    assert len(scout) == 356
    assert {row['source_id'] for row in scout} == expected - {row['id'] for row in inventory}
    assert all(row['reason'] and row['statement'] for row in scout)
    return inventory


def validate_sweeps(root, evidence, inventory):
    table = read_json(root, evidence / 'p1025-sweep-table.json')
    patches = [row['patch'] for row in table]
    assert patches == ['10.2.5', '10.2.7', '11.0.0', '11.0.2', '11.0.5',
                       '11.0.7', '11.1.0', '11.1.5', '11.1.7', '11.2.0',
                       '11.2.5', '11.2.7', '12.0.0', '12.0.1', '12.0.5',
                       '12.0.7', '12.1.0']
    latest = {}
    for patch in patches[1:]:
        register = read_json(root, f'data/patch-api/sources/{patch}-wikitext-register.json')
        for entry in register['entries']:
            if entry['direction'] != 'changed':
                latest[entry['symbol']] = entry
    for row in table:
        results = read_json(root, row['result'])
        known = read_json(root, 'tests/data/patch_' + row['patch'].replace('.', '_') +
                          '_sweep_known_gaps.json')
        assert {key for key, value in results.items() if not value['ok']} == set(known)
        assert len(results) == row['rows']
        assert sum(value['ok'] for value in results.values()) == row['ok']
        assert len(known) == row['gaps'] and row['exit'] == 0
    results = read_json(root, evidence / 'p1025-sweep-result.json')
    for entry in inventory:
        later = latest.get(entry['symbol'])
        own_removed = entry['direction'] == 'removed'
        superseded = later is not None and (later['direction'] == 'removed') != own_removed
        expected = results[entry['id']]['expected']
        removed = not own_removed if superseded else own_removed
        assert expected['publication'] == ('absent' if removed else 'published')
        assert expected['superseded_by'] == (later['id'] if superseded else None)
    review = read_json(root, evidence / 'p1025-gap-review.json')
    assert len(review) == 14
    assert {row['source_id'] for row in review} == {
        key for key, value in results.items() if not value['ok']}
    assert all(row['reason'] and row['decision'] == 'retained-gap' for row in review)
    control = read_json(root, evidence / 'p1025-negative-result.json')
    before = {key for key, value in results.items() if not value['ok']}
    after = {key for key, value in control.items() if not value['ok']}
    negative = read_json(root, evidence / 'p1025-negative-control.json')
    assert negative['exit_code'] == 101
    assert after - before == {negative['changed_id']} and not before - after
    assert (len(before), len(after)) == (14, 15)


def validate_proof(root, evidence):
    reproduction = read_json(root, evidence / 'p1025-register-reproduction.json')
    assert len(reproduction) == 17
    assert all(row['exit_code'] == 0 and row['byte_identical'] for row in reproduction)
    preserved = read_json(root, evidence / 'p1025-preserved-inputs.json')['files']
    assert len(preserved) == 101
    assert all(digest(root / row['path']) == row['sha256'] for row in preserved)
    warning = read_json(root, evidence / 'p1025-warning-boundary.json')
    assert warning['count'] == 0 and warning['non_vendor_warnings'] == []
    proof = read_json(root, evidence / 'p1025-proof.json')
    assert all(Path(row['cwd']).name == 'wow-ui-sim-p1025-page' for row in proof)
    for row in proof:
        assert (root / row['log']).exists(), row['log']
        target = row.get('environment', {}).get('CARGO_TARGET_DIR')
        if target:
            assert target == row['cwd'] + '/target'
        assert len(row['revision']) == 40
        if not row['invalidated']:
            expected = 101 if 'negative' in row['log'] else 0
            assert row['exit_code'] == expected, row['scope']
    assert len([row for row in proof if row['scope'].startswith('isolated publication sweep')]) == 17
    assert (root / evidence / 'p1025-startup.log').read_text().startswith('[]\n')
    scopes = read_json(root, evidence / 'p1025-proof-scopes.json')
    for path, sha in scopes['source_hashes'].items():
        assert digest(root / path) == sha, path
    supersessions = read_json(root, evidence / 'p1025-possible-1026-supersessions.json')
    assert supersessions['matches'] == []


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[4])
    args = parser.parse_args()
    evidence = Path('data/patch-api/evidence/10.2.5-session-2026-10-07')
    inventory = validate_source(args.root, evidence)
    validate_sweeps(args.root, evidence, inventory)
    validate_proof(args.root, evidence)
    print(json.dumps({'source_ids': 415, 'inventory': 59, 'extract': 356,
                      'isolated_sweeps': 17, 'retained_gaps': 14,
                      'preserved_inputs': 101, 'result': 'PASS'}))


if __name__ == '__main__':
    main()
