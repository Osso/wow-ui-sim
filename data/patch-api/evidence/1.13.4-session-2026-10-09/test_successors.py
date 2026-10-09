"""Actual same-Era precedence, distinct from immutable original source queue."""
import copy
from pathlib import Path
import unittest
import audit
import apply_successors
E = Path(__file__).resolve().parent

class SuccessorAccounting(unittest.TestCase):

    def test_serialized_exact_precedence(self):
        self.assertTrue((E / 'successor-application.json').is_file(), 'separate application receipt absent')
        result = audit.read_json('successor-application.json')
        apply_successors.validate(result)
        self.assertEqual(result['totals']['actual_registers'], 18)
        self.assertEqual(result['totals']['source_inventory'], 23)
        self.assertEqual([(r['patch'], r['symbol'], r['later_direction']) for r in result['overlaps']], [('1.14.0', 'C_Commentator.GetUnitTeamIndex', 'removed')])
        self.assertEqual(result['totals']['meaningful_historical_closures'], 0)
        self.assertEqual(result['totals']['native_closures'], 0)
        self.assertTrue(all((r['semantic_status'] == 'UNPROVEN' and (not r['model_credit']) and (not r['native_credit']) for r in result['occurrence_status'])))

    def test_omission_reordering_and_foreign_credit_rejected(self):
        self.assertTrue((E / 'successor-application.json').is_file(), 'separate application receipt absent')
        original = audit.read_json('successor-application.json')
        for field in ['inputs', 'successor_summaries', 'overlaps', 'occurrence_status']:
            for index in range(len(original[field])):
                changed = copy.deepcopy(original)
                changed[field].pop(index)
                with self.assertRaises(AssertionError):
                    apply_successors.validate(changed)
        for key, value in [('history', 'retail'), ('native_credit', True), ('patch', '2.5.6')]:
            changed = copy.deepcopy(original['inputs'])
            changed[0][key] = value
            with self.assertRaises(AssertionError):
                apply_successors.build(changed)
        changed = copy.deepcopy(original['inputs'])
        changed.reverse()
        with self.assertRaises(AssertionError):
            apply_successors.build(changed)

    def test_frozen_queue_and_source_ledger_unchanged(self):
        frozen = audit.read_json('ledger.json')
        audit.validate_ledger(frozen)
        self.assertEqual(frozen['later_registers'], [])
        self.assertEqual(frozen['measurements'], dict(runtime=0, model=0, native=0))
        self.assertTrue(all((not r['applied'] and (not r['native_proof']) for r in frozen['successors'])))
if __name__ == '__main__':
    unittest.main(verbosity=2)
