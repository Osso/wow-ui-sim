#!/usr/bin/env python3
"""Replay frozen 3.0.2 accounting from archives only; never Git/runtime/current files."""
import importlib.util
import json
from pathlib import Path
import re

EVIDENCE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('launch_accounting', EVIDENCE / 'accounting.py')
accounting = importlib.util.module_from_spec(spec)
spec.loader.exec_module(accounting)


def check_seals(evidence, name='seals.json'):
    seals = json.loads((evidence / name).read_bytes())
    for path, expected in seals.items():
        assert accounting.digest((evidence / path).read_bytes()) == expected, 'seal: ' + path
    return len(seals)


def validate_ledger(evidence, ledger):
    expected = accounting.build_ledger(evidence)
    assert ledger == expected, 'complete literal ledger / client history / proof limits'
    return accounting.counts(ledger)


def replay(evidence):
    sealed = check_seals(evidence)
    original = evidence / 'original'
    pin = json.loads((original / 'source-pin.json').read_bytes())
    page = json.loads((original / 'response.json').read_bytes())['query']['pages']['482353']
    revision, = page['revisions']
    assert (page['pageid'], page['title'], revision['revid'], revision['timestamp']) == (
        482353, 'Patch 3.0.2/API changes', 4638841, '2020-02-23T21:55:29Z'), 'frozen identity'
    raw = (original / 'source.wikitext').read_bytes()
    assert revision['slots']['main']['*'].encode() == raw, 'returned content'
    assert accounting.digest(raw) == pin['wikitext_sha256'], 'source hash'
    assert len(raw) == pin['wikitext_bytes'], 'source bytes'
    assert accounting.digest((original / 'response.json').read_bytes()) == pin['response_sha256'], 'response hash'
    manifest = json.loads((original / 'manifest.json').read_bytes())
    assert next(p for p in manifest['pages'] if p['version'] == '3.0.2') == pin, 'manifest pin'
    registry_bytes = (original / 'registry.json').read_bytes()
    assert accounting.digest(registry_bytes) == manifest['registry_sha256'], 'registry hash'
    registry = json.loads(registry_bytes)
    assert registry['pages'][-1]['version'] == '1.0.0', 'registry terminus'
    assert next(p for p in registry['pages'] if p['version'] == '3.0.2')['revid'] == 4638841, 'registry identity'
    ledger = json.loads((original / 'ledger.json').read_bytes())
    result = validate_ledger(evidence, ledger)
    generator = accounting.tool(evidence, 'gen_patch_wikitext_register')
    register = json.loads((original / 'register.json').read_bytes())
    assert register['entries'] == generator.parse_wrath_launch_inventory(raw.decode()), 'register reproduction'
    expected_register = {'schema': 'patch-api-wikitext-register/v1', 'patch': '3.0.2',
                         'source': {'path': 'data/patch-api/sources/3.0.2-api-changes.wikitext',
                                    'revid': 4638841, 'sha256': pin['wikitext_sha256']},
                         'header_counts': [], 'entries': generator.parse_wrath_launch_inventory(raw.decode()),
                         'client_line': 'retail'}
    assert (original / 'register.json').read_text() == json.dumps(
        expected_register, indent=2, ensure_ascii=False) + '\n', 'register exact bytes'
    assert register['client_line'] == 'retail' and register['header_counts'] == [], 'register profile'
    extractor = accounting.tool(evidence, 'extract_patch_non_inventory')
    assert (original / 'extract.txt').read_text() == extractor.extract_text(
        raw.decode(), wrath_summary_markup=True), 'extract reproduction'
    gaps = json.loads((original / 'gaps.json').read_bytes())
    assert gaps == {'runtime_publication': 'NOT MEASURED', 'native': 'NOT MEASURED',
                    'unproven_inventory_ids': [r['id'] for r in ledger['inventory_rows']],
                    'unproven_prose_ids': [r['id'] for r in ledger['prose_ledger']],
                    'unproven_signature_ids': [r['id'] for r in ledger['signature_ledger']]}, 'original gaps'
    overlap = {}
    own = {(r['section'], r['symbol']) for r in ledger['inventory_rows']}
    for patch in accounting.ACTUAL:
        successor = json.loads((original / 'successors' / (patch + '.json')).read_bytes())
        assert successor['patch'] == patch and successor.get('client_line', 'retail') == 'retail', 'same-line successor'
        matched = [r for r in successor['entries'] if (r['section'], r['symbol']) in own]
        overlap[patch] = [{'section': r['section'], 'symbol': r['symbol'], 'direction': r['direction']} for r in matched]
    assert overlap == json.loads((original / 'overlaps.json').read_bytes()), 'actual successor overlap'
    queued_overlaps = {}
    for patch in accounting.QUEUE:
        response = json.loads((original / 'queue' / (patch + '-response.json')).read_bytes())
        queued_page, = response['query']['pages'].values()
        queued_revision, = queued_page['revisions']
        queued_pin = next(p for p in manifest['pages'] if p['version'] == patch)
        queued_raw = (original / 'queue' / (patch + '-wikitext.txt')).read_bytes()
        assert queued_page['pageid'] == queued_pin['pageid'], 'queue identity'
        assert queued_revision['revid'] == queued_pin['revid'], 'queue revision'
        assert queued_revision['slots']['main']['*'].encode() == queued_raw, 'queue literal bytes'
        assert accounting.digest(queued_raw) == queued_pin['wikitext_sha256'], 'queue source hash'
        queued_overlaps[patch] = sorted(symbol for _, symbol in own if re.search(
            r'(?<![A-Za-z0-9_])' + re.escape(symbol) + r'(?![A-Za-z0-9_])', queued_raw.decode()))
        queued_overlaps[patch] = sorted(set(queued_overlaps[patch]))
    assert queued_overlaps == json.loads((original / 'queued-literal-overlaps.json').read_bytes()), 'queued overlap accounting'
    closure_seals = check_seals(evidence, 'closure-seals.json')
    closures = json.loads((evidence / 'closures/claims.json').read_bytes())
    assert closures == {'meaningful_closures': [], 'runtime_changes': [], 'retirements': [],
                        'native_claims': [], 'source_accounting_is_not_publication': True}, 'closure separation'
    return dict(result, sealed_inputs=sealed, closure_seals=closure_seals,
                actual_overlaps=overlap, queued_literal_overlaps=queued_overlaps)


if __name__ == '__main__':
    print(json.dumps(replay(EVIDENCE), sort_keys=True))
