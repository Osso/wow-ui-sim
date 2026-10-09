"""Owned frozen-source accounting fixtures; serialized output is the contract."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

EVIDENCE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('owned_audit', EVIDENCE / 'audit.py')
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class SourceAccounting(unittest.TestCase):
    def test_exact_identity_and_response_body(self):
        raw, pin = audit.pin_source(EVIDENCE)
        self.assertEqual((pin.get('pageid'), pin.get('revid'), pin.get('timestamp')),
                         (480026, 4615755, '2023-07-10T10:27:33Z'))
        self.assertEqual(len(raw), 23291)
        with tempfile.TemporaryDirectory() as tmp:
            copied = Path(tmp)
            shutil.copytree(EVIDENCE / 'original', copied / 'original')
            path = copied / 'original/response.json'
            response = json.loads(path.read_bytes())
            response['query']['pages']['480026']['revisions'][0]['timestamp'] = '2023-07-11T10:27:33Z'
            path.write_text(json.dumps(response))
            with self.assertRaises(AssertionError):
                audit.pin_source(copied)

    def test_every_row_and_inventory_occurrence(self):
        ledger = audit.build(EVIDENCE)
        self.assertEqual(len(ledger.get('source_rows', [])), 463)
        self.assertEqual(len(ledger['inventory_rows']), 412)
        counts = {}
        for row in ledger['inventory_rows']:
            key = (row['section'], row['direction'])
            counts[key] = counts.get(key, 0) + 1
        self.assertEqual(counts, {('global-api', 'added'): 187, ('widgets', 'added'): 1,
                                 ('events', 'added'): 200, ('cvars', 'added'): 15,
                                 ('cvars', 'removed'): 9})
        self.assertEqual(ledger['source_caption']['literal'],
                         '|+ 1.14.2 (42214) &rarr; 1.14.3 (43639) May 10 2022')
        self.assertEqual(ledger['source_interface'], 11403)
        self.assertEqual(ledger['configured_interface'], 11507)

    def test_signatures_headers_and_literal_cvar_effects(self):
        ledger = audit.build(EVIDENCE)
        self.assertEqual(len(ledger.get('signature_ledger', [])), 188)
        self.assertTrue(all(r['arguments'] is None and r['returns'] is None
                            for r in ledger['signature_ledger']))
        self.assertEqual(len(ledger['header_ledger']), 14)
        self.assertEqual(len(ledger['prose_ledger']), 14)
        cvar = next(r for r in ledger['inventory_rows'] if r['symbol'] == 'GamePadEmulateTapWindowMs')
        self.assertEqual(cvar['literal_fields']['default'], '350')
        self.assertEqual(cvar['literal_fields']['description'],
                         "GamePad buttons emulating Ctrl/Alt/Shift will be 'tapped' if released withing this time in MS")
        scoped = next(r for r in ledger['inventory_rows'] if r['symbol'] == 'calendarShowHolidays')
        self.assertEqual(scoped['literal_fields']['scope'], 'Character')

    def test_links_summary_and_successors_do_not_import_behavior(self):
        ledger = audit.build(EVIDENCE)
        self.assertEqual(len(ledger.get('reference_ledger', [])), 27)
        self.assertTrue(all(not r['expanded'] for r in ledger['reference_ledger']))
        self.assertEqual(ledger['prose_ledger'][0]['literal'],
                         'Patch 1.14.3 appears to be in sync with [[Patch_2.5.4/API_changes|patch 2.5.4]].')
        self.assertEqual(ledger['later_registers'], [])
        self.assertEqual(len(ledger['successors']), 11)
        self.assertEqual(ledger['successors'][0]['state'], 'in-flight-pending-main-integration')
        self.assertTrue(all(not r['applied'] and not r['native_proof'] for r in ledger['successors']))

    def test_default_extract_and_raw_rows_are_both_accounted(self):
        ledger = audit.build(EVIDENCE)
        self.assertTrue(ledger.get('extracted_rows'))
        expected = (EVIDENCE / 'original/source.wikitext').read_text().splitlines()
        self.assertEqual([(r['line'], r['literal']) for r in ledger['source_rows']],
                         [(n, line) for n, line in enumerate(expected, 1) if line.strip()])
        self.assertEqual([r['literal'] for r in ledger['extracted_rows']],
                         [line for line in audit.default_extract(EVIDENCE).splitlines() if line.strip()])

    def test_every_accounting_omission_rejected(self):
        expected = audit.build(EVIDENCE)
        self.assertTrue(expected.get('inventory_rows'))
        for key in ('source_rows', 'inventory_rows', 'signature_ledger', 'prose_ledger',
                    'header_ledger', 'reference_ledger', 'extracted_rows', 'successors'):
            for index in range(len(expected[key])):
                changed = dict(expected, **{key: expected[key][:index] + expected[key][index + 1:]})
                with self.subTest(key=key, index=index), self.assertRaises(AssertionError):
                    audit.validate_ledger(changed, expected)

    def test_false_credit_and_foreign_history_rejected(self):
        expected = audit.build(EVIDENCE)
        self.assertEqual(expected.get('measurements'), {'runtime': 0, 'model': 0, 'native': 0})
        for field, value in [('source_interface', 11507), ('configured_interface', 11403),
                             ('measurements', {'runtime': 412, 'model': 412, 'native': 412}),
                             ('later_registers', ['2.5.4']), ('client_line', 'retail')]:
            with self.subTest(field=field), self.assertRaises(AssertionError):
                audit.validate_ledger(dict(expected, **{field: value}), expected)
        changed = copy.deepcopy(expected)
        changed['signature_ledger'][0]['arguments'] = 'itemID'
        with self.assertRaises(AssertionError):
            audit.validate_ledger(changed, expected)

    def test_repeated_literal_occurrences_are_not_deduplicated(self):
        raw = ('==Global API==\n{|\n! <small>(2)</small>\n! <small>(0)</small>\n'
               '| valign="top"\n: {{api|t=a|ExactMisspelling}}\n'
               ': {{api|t=a|ExactMisspelling}}\n</div>\n|}\n')
        register = audit.parse_register(EVIDENCE, raw.encode())
        self.assertEqual(len(register.get('entries', [])), 2)
        self.assertEqual([r['symbol'] for r in register['entries']], ['ExactMisspelling'] * 2)
        self.assertNotEqual(register['entries'][0]['id'], register['entries'][1]['id'])


if __name__ == '__main__':
    unittest.main(verbosity=2)
