#!/usr/bin/env python3
"""Validate sealed historical 2.4.0 accounting from this copied directory only."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent
ARCHIVE = ROOT / 'archives/original'


def read_json(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError) as error:
        raise ValueError(f'{path.relative_to(ROOT)}: {error}') from error


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_seals():
    roots = read_json(ROOT / 'seal-root.json')
    for name, expected in roots.items():
        path = ROOT / name
        if sha(path) != expected:
            raise ValueError(f'{name}: seal manifest changed')
        seals = read_json(path)
        for relative, digest in seals.items():
            target = ROOT / relative
            if not target.resolve().is_relative_to(ROOT) or not target.is_file():
                raise ValueError(f'{relative}: missing or escaped sealed file')
            if sha(target) != digest:
                raise ValueError(f'{relative}: sealed bytes changed')
    return sum(len(read_json(ROOT / name)) for name in roots)


def load_parser(name):
    path = ARCHIVE / 'tools' / name
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def verify_identity():
    pin = read_json(ROOT / 'source-pin.json')
    assert (pin['pageid'], pin['revid'], pin['timestamp']) == (
        73272, 6471380, '2025-09-13T09:55:32Z'), 'wrong historical source identity'
    manifest = read_json(ARCHIVE / 'data/patch-api/source-cache/legacy-2026-10-09/manifest.json')
    assert pin == next(p for p in manifest['pages'] if p['version'] == '2.4.0')
    registry_path = ARCHIVE / manifest['registry_path']
    assert sha(registry_path) == manifest['registry_sha256'], 'registry identity drift'
    registry = read_json(registry_path)
    assert any(p['version'] == '1.0.0' for p in registry['pages']), 'registry cutoff'
    for kind in ('response', 'wikitext'):
        assert sha(ARCHIVE / pin[kind + '_path']) == pin[kind + '_sha256']
    response = read_json(ARCHIVE / pin['response_path'])
    page = response['query']['pages'][str(pin['pageid'])]
    revision = page['revisions'][0]
    assert page['pageid'] == pin['pageid'] and revision['revid'] == pin['revid']
    assert revision['timestamp'] == pin['timestamp']
    raw = (ARCHIVE / pin['wikitext_path']).read_text()
    assert revision['slots']['main']['*'] == raw, 'response/wikitext mismatch'
    return pin, raw, len(registry['pages'])


def verify_accounting(pin, raw):
    generator = load_parser('gen_patch_wikitext_register.py')
    extractor = load_parser('extract_patch_non_inventory.py')
    entries = generator.parse_retail_240_summary(raw)
    register = read_json(ARCHIVE / 'data/patch-api/sources/2.4.0-wikitext-register.json')
    assert register['entries'] == entries and register['header_counts'] == []
    assert register['source']['sha256'] == pin['wikitext_sha256']
    assert register['client_line'] == 'retail'
    text = extractor.extract_text(raw, retail_240_summary=True)
    assert text == (ARCHIVE / 'data/patch-api/sources/2.4.0-api-changes.txt').read_text()
    ledger = read_json(ROOT / 'original/ledger.json')
    expected = [(n, line) for n, line in enumerate(raw.splitlines(), 1) if line.strip()]
    assert [(r['wikitext_line'], r['literal']) for r in ledger['source_rows']] == expected, 'literal row omission/drift'
    for row in ledger['source_rows']:
        literal = row['literal']
        metadata = bool(re.fullmatch(r'==([^=]+)==', literal)) or literal.startswith('{{apichanges|') or literal == '{{Reflist}}'
        assert row['status'] == ('metadata-only' if metadata else 'audit-pending')
    for row, entry in zip(ledger['inventory_rows'], entries, strict=True):
        assert all(row[key] == value for key, value in entry.items())
    functions = [r for r in entries if r['section'] in ('global-api', 'widgets')]
    for row, entry in zip(ledger['signature_rows'], functions, strict=True):
        assert row['inventory_id'] == entry['id'] and row['literal'] == entry['annotation']
        refs = list(re.finditer(r'\[\[(?:API [^]|]+|SetGuildBankText)(?:\|[^]]+)?\]\]', entry['annotation']))
        match = refs[int(entry['id'].rsplit('-', 1)[1]) - 1]
        suffix = entry['annotation'][match.end():].split(' --', 1)[0]
        explicit = suffix.startswith('(')
        assert row['call_fragment'] == (suffix if explicit else None)
        assert row['signature_kind'] == ('explicit-call-fragment' if explicit else 'unspecified')
    rows = ledger['source_rows'] + ledger['inventory_rows'] + ledger['signature_rows']
    assert len({r['source_id'] for r in rows}) == len(rows), 'duplicate ledger IDs'
    assert all(not r['capabilities'] and r['note'] for r in rows), 'invented closure/empty reason'
    pending = [r for r in rows if r['status'] == 'audit-pending']
    gaps = read_json(ROOT / 'original/gaps.json')
    assert gaps == [{'source_id': r['source_id'], 'note': r['note']} for r in pending]
    closures = read_json(ROOT / 'closures/ledger.json')
    closure_gaps = read_json(ROOT / 'closures/gaps.json')
    assert closures['meaningful_closures'] == ledger['meaningful_closures'] == []
    assert closure_gaps['resolved_ids'] == []
    assert closure_gaps['original_gap_ids'] == [r['source_id'] for r in pending]
    return ledger, entries, pending


def verify_successors(entries):
    successors = read_json(ROOT / 'successors.json')
    versions = []
    for item in successors['actual_retail_registers']:
        version = item['patch']
        assert not version.startswith(('2.5.', '3.4.', '1.13.', '1.14.', '1.15.', '4.4.', '5.5.'))
        register = read_json(ARCHIVE / item['path'])
        assert register['patch'] == version
        overlaps = [{'own_id': own['id'], 'later_id': later['id'], 'direction': later['direction']}
                    for own in entries for later in register['entries']
                    if (own['section'], own['symbol']) == (later['section'], later['symbol'])]
        assert overlaps == item['overlaps'], f'{version}: literal overlap drift'
        versions.append(version)
    assert versions[:5] == ['3.2.0', '3.3.0', '3.3.3', '3.3.5', '4.0.1']
    assert versions == sorted(versions, key=lambda v: tuple(map(int, v.split('.'))))
    queued = successors['queued_placeholders']
    assert [q['patch'] for q in queued] == ['2.4.2', '3.0.2', '3.0.3', '3.0.8', '3.1.0']
    for item in queued:
        pin = item['pin']
        assert item['status'] == 'queued-placeholder-main-adds-real-register'
        for kind in ('response', 'wikitext'):
            assert sha(ARCHIVE / pin[kind + '_path']) == pin[kind + '_sha256']
        text = (ARCHIVE / pin['wikitext_path']).read_text()
        overlap = sorted({r['symbol'] for r in entries if re.search(
            r'(?<![A-Za-z0-9_])' + re.escape(r['symbol']) + r'(?![A-Za-z0-9_])', text)})
        assert overlap == item['lexical_symbol_overlap']
    return len(versions), queued


def main():
    seals = verify_seals()
    pin, raw, registry_count = verify_identity()
    ledger, entries, pending = verify_accounting(pin, raw)
    successors, queued = verify_successors(entries)
    receipts = read_json(ROOT / 'original/parser-replay.json')
    assert all(r['base_equal'] for r in receipts['defaults'])
    assert all(r['base_equal'] for r in receipts['recorded'] if r['patch'] != '2.4.0')
    own = next(r for r in receipts['recorded'] if r['patch'] == '2.4.0')
    assert own['register_matches_recorded'] and own['extract_matches_recorded']
    print(json.dumps({'patch': '2.4.0', 'source_rows': len(ledger['source_rows']),
                      'inventory_rows': len(entries), 'signature_rows': len(ledger['signature_rows']),
                      'pending_contract_ids': len(pending), 'registry_pages': registry_count,
                      'actual_retail_registers': successors, 'queued_placeholders': len(queued),
                      'meaningful_closures': 0, 'native_parity': False, 'seals': seals,
                      'proof_boundary': 'fresh archived source accounting; parser regression receipts retained, not rerun'},
                     sort_keys=True))


if __name__ == '__main__':
    try:
        main()
    except (AssertionError, ValueError, OSError, KeyError) as error:
        print(f'REJECT: {error}', file=sys.stderr)
        sys.exit(1)
