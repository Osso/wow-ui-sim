"""Occurrence-level accounting of the frozen retail page."""
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent


class AccountingTests(unittest.TestCase):
    def fixture(self):
        from patch_2_4_2_accounting import account_source
        base = ROOT / 'data/patch-api/sources'
        raw = (base / '2.4.2-api-changes.wikitext').read_text()
        register = json.loads((base / '2.4.2-wikitext-register.json').read_text())
        observations = json.loads((ROOT / 'data/patch-api/evidence/2.4.2-session-2026-10-09/factory-green-results.json').read_text())
        return account_source, raw, register, observations

    def test_every_nonblank_line_and_separate_contract_is_retained(self):
        account, raw, register, observations = self.fixture()
        rows = account(raw, register, observations)
        literal = [r for r in rows if r['source_id'].startswith('raw-')]
        self.assertEqual([(r['wikitext_line'], r['literal']) for r in literal],
                         [(i, line) for i, line in enumerate(raw.splitlines(), 1) if line.strip()])
        self.assertEqual(len(rows), 53)
        self.assertEqual(len({r['source_id'] for r in rows}), 53)
        self.assertEqual(sum(r['status'] == 'bounded-coverage' for r in rows), 2)
        self.assertEqual(sum(r['status'] == 'metadata-only' for r in rows), 7)
        self.assertEqual(sum(r['source_id'].startswith('signature-') for r in rows), 11)
        self.assertEqual(sum(r['source_id'].startswith('constant-') for r in rows), 6)
        self.assertEqual(sum(r['source_id'].startswith('context-api-') for r in rows), 3)
        self.assertTrue(all(r['note'] for r in rows))
        self.assertTrue(all(not r['capabilities'] for r in rows if r['source_id'].startswith('signature-')))

    def test_missing_inventory_observation_or_raw_line_rejects(self):
        account, raw, register, observations = self.fixture()
        for entry in register['entries']:
            omitted = dict(register, entries=[r for r in register['entries'] if r != entry])
            with self.assertRaisesRegex(AssertionError, 'literal inventory'):
                account(raw, omitted, observations)
            omitted_observation = {k: v for k, v in observations.items() if k != entry['id']}
            with self.assertRaisesRegex(AssertionError, 'observation set'):
                account(raw, register, omitted_observation)
        with self.assertRaisesRegex(AssertionError, 'literal inventory'):
            account(raw.replace('* [[API strreplace|strreplace]]', ''), register, observations)


if __name__ == '__main__':
    unittest.main()
