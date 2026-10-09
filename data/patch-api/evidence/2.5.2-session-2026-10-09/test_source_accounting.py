"""Only historical SOURCE accounting; no simulator/native claims."""
from pathlib import Path
import unittest
from validate import account_source

EVIDENCE = Path(__file__).resolve().parent


class SourceAccounting(unittest.TestCase):
    def test_full_inventory_and_raw_rows(self):
        raw = (EVIDENCE / '2.5.2-wikitext.txt').read_text()
        result = account_source(raw)
        self.assertEqual(result.get('api_occurrences'), 145)
        self.assertEqual(result.get('source_rows'), sum(bool(s.strip()) for s in raw.splitlines()))
        self.assertEqual(result.get('removals'), 33)
        self.assertEqual(result.get('explicit_signatures'), 0)
        self.assertEqual(result.get('local_summary_contracts'), 1)
        self.assertEqual(result.get('transclusions'), 0)
        self.assertEqual(result.get('header_pairs'), [(80, 1), (2, 0), (18, 29), (12, 3)])


if __name__ == '__main__':
    unittest.main(verbosity=2)
