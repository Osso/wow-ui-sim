#!/usr/bin/env python3
"""Pinned external source/serialized ledger proof only; no runtime assertions."""
import copy
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCE_HASH = '484b49adb4301ad901914389cf37f9849c21d62eea20797d0ad2fefbaa2b8669'
RESPONSE_HASH = 'c0604a21b721a67b5fde2d429b72e7f10f5e272758718c1343c94c9bb5839eb4'
TEXT = (
    'Patch 4.4.2 API changes\n\n== Summary ==\n'
    '* The modern auction house and its associated C_AuctionHouse APIs has been enabled for Cataclysm Classic.\n'
    '\n== Resources ==\n* TOC: 40402\n'
    '* Diffs: wow-ui-source, BlizzardInterfaceResources\n'
)


def validate(response_bytes, raw, pin, ledger, text):
    """Check returned bytes, exact source rows and zero runtime credit."""
    page = json.loads(response_bytes)['query']['pages']['619773']
    revision, = page['revisions']
    source = ledger['source']
    for key, expected in [('pageid', 619773), ('title', 'Patch 4.4.2/API changes')]:
        assert page[key] == pin[key] == source[key] == expected, key
    for key, expected in [('revid', 6303388), ('timestamp', '2025-04-25T00:45:23Z')]:
        assert revision[key] == pin[key] == source[key] == expected, key
    assert revision['slots']['main']['*'].encode() == raw, 'returned content bytes'
    assert hashlib.sha256(raw).hexdigest() == pin['wikitext_sha256'] == source['wikitext_sha256'] == SOURCE_HASH, 'source hash'
    assert hashlib.sha256(response_bytes).hexdigest() == source['response_sha256'] == RESPONSE_HASH, 'response hash'
    assert len(raw) == source['wikitext_bytes'] == 390
    assert len(response_bytes) == source['response_bytes'] == 674
    assert source['toc'] == 40402
    assert pin['retrieved'] == source['retrieved'] == '2026-10-08'
    assert text == TEXT.encode(), 'external plaintext contract'
    assert hashlib.sha256(text).hexdigest() == ledger['non_inventory_source']['sha256']
    assert ledger['non_inventory_source']['extractor_flags'] == ['--text-only', '--canonical-patch-navigation']
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'], ledger['scope']) == (
        'patch-source-accounting/v1', '4.4.2', 'cataclysm-classic', None, 'source-only')
    assert ledger['source_commit'] == '4c6c87e8ffe9c201ad0b62c56571aa376e151671'
    assert ledger['inventory'] == {
        'enumerated_api_occurrences': 0, 'header_counts': [], 'register_created': False, 'removals': 0}
    lines = raw.decode().splitlines()
    expected_rows = [
        ('source-context-001', 1, 'metadata-only'),
        ('source-context-003', 3, 'metadata-only'),
        ('prose-undated-004', 4, 'UNPROVEN'),
        ('source-context-006', 6, 'metadata-only'),
        ('source-context-007', 7, 'metadata-only'),
        ('source-context-008', 8, 'metadata-only'),
    ]
    rows = ledger['source_rows']
    assert [(r['source_id'], r['wikitext_line'], r['status']) for r in rows] == expected_rows, 'exact row matrix'
    assert [r['wikitext_line'] for r in rows] == [i for i, line in enumerate(lines, 1) if line.strip()]
    for row in rows:
        assert row['source_text'] == lines[row['wikitext_line'] - 1], 'literal source row'
        assert row['text_line'] == row['wikitext_line']
        assert row['capabilities'] == [], 'no runtime credit'
    assert rows[2]['named_namespace'] == 'C_AuctionHouse', 'named namespace'
    assert rows[2]['contract'] == 'Modern auction house and associated C_AuctionHouse APIs enabled in Cataclysm Classic'


class SourceProof(unittest.TestCase):
    def setUp(self):
        self.response = (EVIDENCE / 'source-response.json').read_bytes()
        self.raw = (ROOT / 'data/patch-api/sources/4.4.2-api-changes.wikitext').read_bytes()
        self.pin = json.loads((EVIDENCE / 'source-pin.json').read_bytes())
        self.ledger = json.loads((ROOT / 'data/patch-api/sources/4.4.2-page-coverage.json').read_bytes())
        self.text = (ROOT / 'data/patch-api/sources/4.4.2-api-changes.txt').read_bytes()

    def check(self, response=None, raw=None, ledger=None):
        validate(self.response if response is None else response,
                 self.raw if raw is None else raw, self.pin,
                 self.ledger if ledger is None else ledger, self.text)

    def test_pinned_source_and_exact_serialized_ledger(self):
        self.check()

    def test_changed_returned_revision_rejected(self):
        response = json.loads(self.response)
        response['query']['pages']['619773']['revisions'][0]['revid'] = 6303389
        with self.assertRaisesRegex(AssertionError, 'revid'):
            self.check(response=json.dumps(response).encode())

    def test_changed_source_bytes_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'returned content bytes'):
            self.check(raw=self.raw + b'\n')

    def test_namespace_credit_rejected(self):
        ledger = copy.deepcopy(self.ledger)
        ledger['source_rows'][2]['status'] = 'bounded-coverage'
        with self.assertRaisesRegex(AssertionError, 'exact row matrix'):
            self.check(ledger=ledger)

    def test_missing_external_boundary_row_rejected(self):
        ledger = copy.deepcopy(self.ledger)
        ledger['source_rows'].pop()
        with self.assertRaisesRegex(AssertionError, 'exact row matrix'):
            self.check(ledger=ledger)


if __name__ == '__main__':
    unittest.main(verbosity=2)
