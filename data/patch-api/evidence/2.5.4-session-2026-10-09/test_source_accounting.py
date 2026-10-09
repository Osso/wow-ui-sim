"""Literal serialized SOURCE contracts, not simulator/native measurements."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

EVIDENCE = Path(__file__).resolve().parent
ROOT = EVIDENCE.parents[3]
LEDGER = ROOT / 'data/patch-api/sources/2.5.4-page-coverage.json'


class SourceAccounting(unittest.TestCase):
    def test_complete_literal_inventory_and_rows(self):
        ledger = json.loads(LEDGER.read_bytes())
        self.assertEqual(len(ledger['source_rows']), 461)
        self.assertEqual(len(ledger['contracts']), 414)
        self.assertEqual(ledger['inventory']['counts'], {
            'global-api': {'added': 187, 'removed': 0},
            'widgets': {'added': 1, 'removed': 0},
            'events': {'added': 200, 'removed': 0},
            'cvars': {'added': 15, 'removed': 9},
        })


if __name__ == '__main__':
    unittest.main(verbosity=2)
