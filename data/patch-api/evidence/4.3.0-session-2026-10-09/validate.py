#!/usr/bin/env python3
"""Bounded 4.3.0 source/accounting replay. Not a current-head acceptance gate."""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
SOURCES = ROOT / 'data/patch-api/sources'
sys.path.insert(0, str(ROOT / 'tools'))
import gen_patch_wikitext_register as generator
import extract_patch_non_inventory as extractor
from patch_audit_pin_trees import tree_id


def read_json(path):
    return json.loads(path.read_text())


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def unique_ids(rows, field, label):
    ids = [row[field] for row in rows]
    assert len(ids) == len(set(ids)), f'duplicate {label} IDs'
    return set(ids)


def validate_accounting(register, ledger, results, known):
    ids = unique_ids(register['entries'], 'id', 'register')
    assert ids == set(results), 'observation IDs differ from inventory'
    gaps = {key for key, row in results.items() if not row['ok']}
    assert len(known) == len(set(known)), 'duplicate gap IDs'
    assert gaps == set(known), 'gap IDs differ from reviewed observations'
    rows = ledger['source_rows']
    ledger_ids = unique_ids(rows, 'source_id', 'ledger')
    assert ledger_ids == ids | {'source-context-001'}, 'ledger IDs differ'
    for row in rows:
        key = row['source_id']
        assert row['note'].strip(), f'missing reason: {key}'
        if key == 'source-context-001':
            assert row['status'] == 'metadata-only' and not row['capabilities']
        elif key in gaps:
            assert row['status'] == 'audit-pending' and not row['capabilities']
        else:
            assert row['status'] == 'bounded-coverage'
            assert row['capabilities'] == ['current-retail-publication-or-absence'], \
                f'publication-only credit: {key}'
    for entry in register['entries']:
        expected = results[entry['id']]['expected']
        for field in ('section', 'direction', 'symbol'):
            assert entry[field] == expected[field], f'observation identity: {entry["id"]}'
    for header in register['header_counts']:
        count = sum(entry['section'] == header['section'] and
                    entry['direction'] == header['direction']
                    for entry in register['entries'])
        assert count == header['parsed_count'] == header['header_count'], 'header accounting'
    return {'inventory': len(ids), 'gaps': len(gaps),
            'publication_or_absence': len(ids - gaps),
            'statuses': dict(Counter(row['status'] for row in rows))}


def validate_sources():
    provenance = read_json(SOURCES / '4.3.0-api-changes.provenance.json')
    pin = read_json(HERE / 'source-pin.json')
    raw = SOURCES / '4.3.0-api-changes.wikitext'
    assert digest(raw) == pin['wikitext_sha256'], 'pinned source digest'
    assert digest(HERE / 'source-response.json') == pin['response_sha256'], 'response digest'
    response = read_json(HERE / 'source-response.json')
    page = response['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert revision['revid'] == pin['revid'] and revision['timestamp'] == pin['timestamp']
    assert revision['slots']['main']['*'] == raw.read_text(), 'response/source mismatch'
    assert provenance['generator_flags'] == [], 'unexpected generator flags'
    with tempfile.TemporaryDirectory(dir=HERE) as directory:
        output = Path(directory) / 'register.json'
        old_argv = sys.argv
        try:
            sys.argv = ['gen_patch_wikitext_register.py', '4.3.0', str(raw),
                        str(pin['revid']), str(output), *provenance['generator_flags']]
            generator.main()
        finally:
            sys.argv = old_argv
        assert output.read_bytes() == (SOURCES / '4.3.0-wikitext-register.json').read_bytes(), \
            'register reproduction'
    flags = {flag.removeprefix('--').replace('-', '_'): True
             for flag in provenance['extractor_flags']}
    text = extractor.extract_text(raw.read_text(), **flags)
    assert text == (SOURCES / '4.3.0-api-changes.txt').read_text(), 'extract reproduction'
    assert [row['source_id'] for row in extractor.seed_rows(text, '4.3.0')] == \
        ['source-context-001'], 'unaccounted non-inventory source'
    ledger = read_json(SOURCES / '4.3.0-page-coverage.json')
    assert ledger['source_sha256'] == digest(SOURCES / '4.3.0-wikitext-register.json')
    assert ledger['non_inventory_source']['sha256'] == digest(SOURCES / '4.3.0-api-changes.txt')


def validate_receipts():
    for path in sorted(HERE.glob('*.proof.json')):
        receipt = read_json(path)
        assert digest(HERE / receipt['log']) == receipt['log_sha256'], f'log seal: {path.name}'
        if 'results' in receipt:
            assert digest(HERE / receipt['results']) == receipt['results_sha256'], \
                f'result seal: {path.name}'
    for path in HERE.rglob('*'):
        assert not path.is_file() or path.stat().st_size <= 5_000_000, \
            f'evidence artifact exceeds 5 MB: {path.name}'


def validate_tree_pins():
    scopes = read_json(HERE / 'historical-tree-pins.json')['scopes']
    assert scopes, 'missing historical scopes'
    for scope in scopes:
        archive = HERE / scope['manifest']
        assert digest(archive) == scope['manifest_sha256'], 'tree manifest seal'
        entries = json.loads(gzip.decompress(archive.read_bytes()))
        assert tree_id(entries) == scope['tree'], 'historical tree identity'
    return len(scopes)


def main():
    validate_sources()
    validate_receipts()
    pinned_scopes = validate_tree_pins()
    summary = validate_accounting(
        read_json(SOURCES / '4.3.0-wikitext-register.json'),
        read_json(SOURCES / '4.3.0-page-coverage.json'),
        read_json(HERE / 'reviewed-results.json'),
        read_json(ROOT / 'tests/data/patch_4_3_0_sweep_known_gaps.json'))
    print(json.dumps({'scope': 'recorded bounded development evidence only',
                      'historical_tree_scopes': pinned_scopes, **summary}, indent=2))


if __name__ == '__main__':
    main()
