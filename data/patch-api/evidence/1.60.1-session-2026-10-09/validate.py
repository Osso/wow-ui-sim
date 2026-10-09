#!/usr/bin/env python3
"""Offline frozen source replay; uses only this portable evidence directory."""
import importlib.util
import json
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('forever_accounting', EVIDENCE / 'accounting.py')
accounting = importlib.util.module_from_spec(spec)
spec.loader.exec_module(accounting)


def check_seals(evidence):
    seals = json.loads((evidence / 'seals.json').read_bytes())
    for path, expected in seals.items():
        data = (evidence / path).read_bytes()
        assert len(data) < 5_000_000, 'compact file: ' + path
        assert accounting.digest(data) == expected, 'seal: ' + path
    return len(seals)


def validate_ledger(ledger, expected):
    assert ledger == expected, 'complete literal source/client/proof ledger'
    return accounting.counts(ledger)


def replay(evidence):
    sealed = check_seals(evidence)
    original = evidence / 'original'
    pin = json.loads((original / 'source-pin.json').read_bytes())
    response = (original / 'response.json').read_bytes()
    page = json.loads(response)['query']['pages']['707613']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        707613, 'Patch 1.60.1/API changes', 6902509, '2026-10-07T05:30:51Z'), 'frozen identity'
    raw = (original / 'source.wikitext').read_bytes()
    assert revision['slots']['main']['*'].encode() == raw, 'returned body exact bytes'
    assert accounting.digest(raw) == pin['wikitext_sha256'] == 'f0e3dc86be50c02dbbc2a5962adda9cf96aa8f3a3959995a4cf7a6a9c8b43ce7', 'source hash'
    assert len(raw) == pin['wikitext_bytes'] == 125873, 'source byte count'
    assert accounting.digest(response) == pin['response_sha256'] == '687133019ca6e39cfe2c2902fd398ac2c6f161cab5aed52e7809ca562c6ea5e5', 'response hash'
    manifest = json.loads((original / 'manifest.json').read_bytes())
    assert next(row for row in manifest['pages'] if row['version'] == '1.60.1') == pin, 'manifest pin'
    registry_bytes = (original / 'registry.json').read_bytes()
    assert accounting.digest(registry_bytes) == manifest['registry_sha256'] == 'e357f60af2c745b7797ab8f9e7ac151345cddb6bf43de7786ee25785ee92e91c', 'registry hash'
    registry = json.loads(registry_bytes)['pages']
    assert len(registry) == 101 and registry[-1]['version'] == '1.0.0', 'registry 101/endpoint'
    assert len({row['version'] for row in registry}) == 101, 'registry unique versions'
    indexed = {row['version']: row for row in registry}
    for row in manifest['pages']:
        registered = indexed[row['version']]
        assert all(row[key] == registered[key] for key in ('title', 'pageid', 'revid', 'timestamp')), 'manifest registry identity: ' + row['version']
    ledger = json.loads((original / 'ledger.json').read_bytes())
    result = validate_ledger(ledger, accounting.build_ledger(evidence))
    register = accounting.parse_register(evidence, raw)
    assert (original / 'register.json').read_bytes() == accounting.json_bytes(register), 'default register exact bytes'
    assert (original / 'literal-extract.wikitext').read_text() == accounting.literal_extract(ledger), 'literal non-inventory mirror'
    gaps = dict(runtime_publication='NOT MEASURED', native='NOT MEASURED',
                unproven_inventory_ids=[row['id'] for row in ledger['inventory_rows']],
                unproven_signature_ids=[row['id'] for row in ledger['signature_ledger']],
                unproven_prose_ids=[row['id'] for row in ledger['prose_ledger']])
    assert json.loads((original / 'gaps.json').read_bytes()) == gaps, 'unproven contracts'
    result.update(registry_pages=len(registry), registry_endpoint=registry[-1]['version'],
                  manifest_pages=len(manifest['pages']), sealed_inputs=sealed,
                  header_mismatches=[row for row in register['header_counts'] if row['header_count'] != row['parsed_count']])
    return result


if __name__ == '__main__':
    print(json.dumps(replay(EVIDENCE), sort_keys=True))
