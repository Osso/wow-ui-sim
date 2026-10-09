#!/usr/bin/env python3
"""Pinned CLASSIC4.4.0 source proof, never native/runtime compatibility."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
BASE = '0c7a007e91e13c14b80209104dbb4c12533e10da'
PATHS = {
    'raw': 'data/patch-api/sources/4.4.0-api-changes.wikitext',
    'text': 'data/patch-api/sources/4.4.0-api-changes.txt',
    'ledger': 'data/patch-api/sources/4.4.0-page-coverage.json',
    **{key: f'data/patch-api/evidence/4.4.0-session-2026-10-08/{name}.json'
       for key, name in [('response', 'source-response'), ('pin', 'source-pin'),
                         ('receipt', 'source-proof')]},
}
HASHES = {
    'raw': 'f8598e92092793a0621ea26d5620fd910c178dcb1fccd0b0aa313b83a6b56d5e',
    'text': 'e424ca462d923241d977198ad11f910afc9d7a85dcf172068f17bf4cba22de4d',
    'ledger': 'e05c38c1fc5bcab1193c11814cc261dbc757367a623c3f2f885dffe0610b0ef2',
    'response': '3563e32d0abf16e7fa0e71d1d4b5cc4a8b62cba718ac91b5e73debdb32ee2497',
    'pin': 'ffc989d4f235c924f5dd2df92aa7bf5a4406507a29c4c84b1d6e2affeae3eddb',
    'receipt': '56490e71986cf5913ae2847b537ee306024cfd35b9753d75512283c8de5dedbc',
}
TREES = {
    'src': 'f696a92562e3023ba0abec6d7fd14ea7c79455db',
    'tools': 'd9207917bcd0f6de544e87829eed6d57ffdf1bfa',
    'data/patch-api/evidence/4.4.2-session-2026-10-08':
        'f04f948d445a37a42d49bdb8c4df4c8342d44bd2',
}


def validate(payload):
    """Check external response identity, complete serialized rows and owned seals."""
    page = json.loads(payload['response'])['query']['pages']['580953']
    revision, = page['revisions']
    pin = json.loads(payload['pin'])
    ledger = json.loads(payload['ledger'])
    receipt = json.loads(payload['receipt'])
    source = ledger['source']
    for key, expected in [('pageid', 580953), ('title', 'Patch 4.4.0/API changes')]:
        assert page[key] == pin[key] == source[key] == expected, key
    for key, expected in [('revid', 6163701), ('timestamp', '2024-10-29T16:26:26Z')]:
        assert revision[key] == pin[key] == source[key] == expected, key
    assert revision['parentid'] == 6028368, 'parent revision'
    assert revision['slots']['main']['*'].encode() == payload['raw'], 'returned content'
    assert source['toc'] == 40400, 'TOC'
    assert source['retrieved'] == pin['retrieved'] == '2026-10-08', 'retrieved'
    assert (len(payload['raw']), len(payload['response']), len(payload['text'])) == (1348, 1646, 964), 'byte counts'
    assert (ledger['schema'], ledger['patch'], ledger['client_line'], ledger['profile'], ledger['scope']) == (
        'patch-source-accounting/v1', '4.4.0', 'cataclysm-classic', None, 'source-only'), 'scope'
    assert ledger['source_commit'] == receipt['source_commit'] == BASE, 'source commit'
    assert ledger['inventory'] == {
        'enumerated_api_occurrences': 0, 'prose_api_occurrences': 1,
        'header_counts': [], 'register_created': False, 'removals': 0}, 'inventory'
    lines = payload['raw'].decode().splitlines()
    rows = ledger['source_rows']
    substantive = {4, 5, 6, 14, 15, 16}
    expected = [(i, 'UNPROVEN' if i in substantive else 'metadata-only')
                for i, line in enumerate(lines, 1) if line.strip()]
    assert [(row['wikitext_line'], row['status']) for row in rows] == expected, 'row matrix'
    for row in rows:
        line = row['wikitext_line']
        prefix = 'prose-undated' if line in substantive else 'source-context'
        assert row['source_id'] == f'{prefix}-{line:03}', 'source ID'
        assert row['text_line'] == line and row['source_text'] == lines[line - 1], 'literal occurrence'
        assert row['capabilities'] == [], 'no runtime credit'
        assert row.get('reason' if line in substantive else 'note'), 'per-row rationale'
    message = next(row for row in rows if row['wikitext_line'] == 6)
    assert message['named_api'] == 'C_ChatInfo.SendAddonMessage', 'named API'
    assert message['contracts'] == ['per-prefix throttling of all addon traffic',
                                    'enum result rather than boolean when unable to queue transmission'], 'message contracts'
    assert receipt['identity'] == {'pageid': 580953, 'revid': 6163701, 'toc': 40400}, 'receipt identity'
    assert receipt['row_counts'] == {'total': 12, 'metadata-only': 6, 'UNPROVEN': 6,
                                     'contracts_unproven': 7}, 'receipt counts'
    assert receipt['context_tree_pins'] == TREES, 'context trees'
    assert receipt['owned_sha256'] == {PATHS[key]: value for key, value in HASHES.items()
                                      if key != 'receipt'}, 'receipt seals'
    for key, expected_hash in HASHES.items():
        assert hashlib.sha256(payload[key]).hexdigest() == expected_hash, f'{key} seal'
    assert source['wikitext_sha256'] == pin['wikitext_sha256'] == HASHES['raw']
    assert source['response_sha256'] == HASHES['response']
    assert source['wikitext_bytes'] == 1348 and source['response_bytes'] == 1646
    assert source['wikitext_path'] == PATHS['raw'] and source['response_path'] == PATHS['response']
    assert ledger['non_inventory_source'] == {
        'path': PATHS['text'], 'sha256': HASHES['text'],
        'extract_tool': 'tools/extract_patch_non_inventory.py',
        'extractor_flags': ['--text-only', '--canonical-patch-navigation']}, 'extract receipt'


class SourceProof(unittest.TestCase):
    def setUp(self):
        self.payload = {key: (ROOT / path).read_bytes() for key, path in PATHS.items()}

    def mutate_json(self, key, mutation):
        payload = copy.copy(self.payload)
        value = json.loads(payload[key])
        mutation(value)
        payload[key] = json.dumps(value).encode()
        return payload

    def test_identity_rows_receipt_and_tree_pins(self):
        validate(self.payload)
        for tree in TREES.values():
            result = subprocess.check_output(['git', 'cat-file', '-t', tree], cwd=ROOT, text=True)
            self.assertEqual(result.strip(), 'tree')

    def test_plaintext_reproduction(self):
        spec = importlib.util.spec_from_file_location('extract_440', ROOT / 'tools/extract_patch_non_inventory.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertEqual(module.extract_text(self.payload['raw'].decode(),
                                            canonical_patch_navigation=True).encode(), self.payload['text'])

    def test_fabricated_revision_rejected(self):
        payload = self.mutate_json('response', lambda value: value['query']['pages']['580953']['revisions'][0].update(revid=6163702))
        with self.assertRaisesRegex(AssertionError, 'revid'):
            validate(payload)

    def test_source_tamper_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            validate({**self.payload, 'raw': self.payload['raw'] + b'\n'})

    def test_fabricated_credit_rejected(self):
        payload = self.mutate_json('ledger', lambda value: value['source_rows'][2].update(status='bounded-coverage'))
        with self.assertRaisesRegex(AssertionError, 'row matrix'):
            validate(payload)

    def test_missing_literal_occurrences_rejected(self):
        for index in range(12):
            with self.subTest(index=index):
                payload = self.mutate_json('ledger', lambda value: value['source_rows'].pop(index))
                with self.assertRaisesRegex(AssertionError, 'row matrix'):
                    validate(payload)

    def test_rationale_tamper_rejected(self):
        payload = self.mutate_json('ledger', lambda value: value['source_rows'][2].update(reason='Native Cata proven'))
        with self.assertRaisesRegex(AssertionError, 'ledger seal'):
            validate(payload)

    def test_receipt_tamper_rejected(self):
        payload = self.mutate_json('receipt', lambda value: value['identity'].update(toc=50500))
        with self.assertRaisesRegex(AssertionError, 'receipt identity'):
            validate(payload)

    def test_owned_byte_seals_reject_tampering(self):
        for key in ('text', 'pin', 'receipt', 'response'):
            with self.subTest(key=key):
                with self.assertRaises(AssertionError):
                    validate({**self.payload, key: self.payload[key] + b' '})


if __name__ == '__main__':
    unittest.main(verbosity=2)
