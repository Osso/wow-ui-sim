#!/usr/bin/env python3
"""Portable pinned external source and serialized ledger proof, not runtime proof."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCE_HASH = '076080f53193f5c8808f255e5630b9e83db8e9e284c53eff7a7ed7b84623284a'
RESPONSE_HASH = '6f022bc0d02bb1c67f4f531ae52d18309682ae9bb96f58f42f4f4641b1205579'
LEDGER_HASH = '4469d9bc2d8583cc145709bdaad8d60bc582b5295e329ca876b3272f1ce55dd5'
SOURCE_COMMIT = '5960d3b73c48895dc216c4ecf1afccc8b51aa074'
EXTRACTOR_BLOB = '6fcc1724827c7588b91d23e3c815324d86a33b97'
TEXT = ('Patch 4.4.1 API changes\n\n== Resources ==\n* TOC: 40401\n'
        '* Diffs: wow-ui-source, BlizzardInterfaceResources\n\n== Consolidated diffs ==\n')


def digest(value):
    return hashlib.sha256(value).hexdigest()


def validate(response_bytes, raw, pin, ledger_bytes, text):
    page = json.loads(response_bytes)['query']['pages']['608915']
    revision, = page['revisions']
    ledger = json.loads(ledger_bytes)
    source = ledger['source']
    for key, expected in [('pageid', 608915), ('title', 'Patch 4.4.1/API changes')]:
        assert page[key] == pin[key] == source[key] == expected, key
    for key, expected in [('revid', 6234252), ('timestamp', '2025-02-08T13:53:50Z')]:
        assert revision[key] == pin[key] == source[key] == expected, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content bytes'
    assert digest(raw) == pin['wikitext_sha256'] == source['wikitext_sha256'] == SOURCE_HASH, 'source hash'
    assert digest(response_bytes) == source['response_sha256'] == RESPONSE_HASH, 'response hash'
    assert len(raw) == source['wikitext_bytes'] == 883
    assert len(response_bytes) == source['response_bytes'] == 1206
    assert source['toc'] == 40401
    assert pin['retrieved'] == source['retrieved'] == '2026-10-08'
    assert text == TEXT.encode(), 'plaintext contract'
    assert digest(text) == ledger['non_inventory_source']['sha256']
    assert ledger['source_commit'] == SOURCE_COMMIT
    assert ledger['shared_inputs'] == {'revision': SOURCE_COMMIT, 'extractor_git_blob': EXTRACTOR_BLOB}
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'], ledger['scope']) == (
        'patch-source-accounting/v1', '4.4.1', 'cataclysm-classic', None, 'source-only')
    rows = ledger['source_rows']
    lines = raw.decode().splitlines()
    assert [r['wikitext_line'] for r in rows] == [n for n, line in enumerate(lines, 1) if line.strip()], 'exact row matrix'
    for row in rows:
        n = row['wikitext_line']
        assert row['source_text'] == lines[n - 1], 'literal row'
        assert row['capabilities'] == [], 'no runtime credit'
        assert row['status'] == ('UNPROVEN' if n in (16, 19) else 'metadata-only'), 'runtime status'
        if n not in (16, 19):
            assert row['source_id'] == f'source-context-{n:03}', 'metadata ID'
    occurrences = [r for r in rows if r['status'] == 'UNPROVEN']
    assert [(r['source_id'], r['identity'], r['direction'], r['old_patch'], r['new_patch']) for r in occurrences] == [
        ('global-api-added-016', 'C_SpecializationInfo.GetNumSpecializationsForClassID', 'added', '4.4.0', '4.4.1'),
        ('global-api-removed-019', 'GetNumSpecializationsForClassID', 'removed', '4.4.0', '4.4.1')], 'occurrence direction'
    inventory = ledger['inventory']
    assert inventory['enumerated_api_occurrences'] == 2
    assert inventory['added_occurrences'] == inventory['removed_occurrences'] == 1
    assert inventory['register_created'] is False, 'no register'
    assert [inventory[k] for k in ('old_count', 'new_count', 'added_header_count', 'removed_header_count')] == ['x'] * 4, 'literal placeholder counts'
    assert inventory['date_literal'] == 'xxx xx 2024', 'literal date'
    assert digest(ledger_bytes) == LEDGER_HASH, 'serialized ledger hash'


class SourceProof(unittest.TestCase):
    def setUp(self):
        self.response = (EVIDENCE / 'source-response.json').read_bytes()
        self.raw = (ROOT / 'data/patch-api/sources/4.4.1-api-changes.wikitext').read_bytes()
        self.pin = json.loads((EVIDENCE / 'source-pin.json').read_bytes())
        self.ledger = (ROOT / 'data/patch-api/sources/4.4.1-page-coverage.json').read_bytes()
        self.text = (ROOT / 'data/patch-api/sources/4.4.1-api-changes.txt').read_bytes()

    def check(self, **changes):
        args = dict(response_bytes=self.response, raw=self.raw, pin=self.pin,
                    ledger_bytes=self.ledger, text=self.text)
        args.update(changes)
        validate(**args)

    def test_pinned_source_and_serialized_ledger(self):
        self.check()

    def test_source_response_plaintext_and_pin_mutations_rejected(self):
        response = json.loads(self.response)
        response['query']['pages']['608915']['revisions'][0]['revid'] += 1
        pin = dict(self.pin, revid=6234253)
        for changes in [dict(response_bytes=json.dumps(response).encode()),
                        dict(response_bytes=self.response + b' '),
                        dict(raw=self.raw + b'\n'), dict(text=self.text + b' '), dict(pin=pin)]:
            with self.subTest(changes=list(changes)), self.assertRaises(AssertionError):
                self.check(**changes)

    def test_occurrence_metadata_direction_placeholder_and_credit_mutations_rejected(self):
        mutations = {
            'missing-metadata': lambda d: d['source_rows'].pop(3),
            'missing-added': lambda d: d['source_rows'].pop(13),
            'missing-removed': lambda d: d['source_rows'].pop(16),
            'added-credit': lambda d: d['source_rows'][13].update(status='bounded-coverage'),
            'removed-credit': lambda d: d['source_rows'][16].update(status='retired'),
            'capability': lambda d: d['source_rows'][13].update(capabilities=['publication']),
            'reverse-direction': lambda d: d['source_rows'][13].update(old_patch='4.4.1', new_patch='4.4.0'),
            'numeric-count': lambda d: d['inventory'].update(added_header_count=1),
            'invented-date': lambda d: d['inventory'].update(date_literal='2024-10-29'),
            'retail-profile': lambda d: d.update(profile='retail'),
            'register': lambda d: d['inventory'].update(register_created=True),
            'supersession': lambda d: d.update(later_registers=['4.4.2']),
            'shared-pin': lambda d: d['shared_inputs'].update(revision='HEAD'),
        }
        for name, mutate in mutations.items():
            ledger = json.loads(self.ledger)
            mutate(ledger)
            with self.subTest(name=name), self.assertRaises(AssertionError):
                self.check(ledger_bytes=(json.dumps(ledger, indent=2) + '\n').encode())

    def test_serialized_ledger_hash_rejects_equivalent_reformat(self):
        with self.assertRaisesRegex(AssertionError, 'serialized ledger hash'):
            self.check(ledger_bytes=self.ledger + b' ')


if __name__ == '__main__':
    unittest.main(verbosity=2)
