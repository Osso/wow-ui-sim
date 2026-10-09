"""Separate literal same-Era precedence; never historical semantic credit."""
import copy
from pathlib import Path
import unittest
import audit
E = Path(__file__).resolve().parent

class SuccessorAccounting(unittest.TestCase):

    def application(self):
        self.assertTrue((E / 'successor-application.json').is_file(), 'separate actual successor receipt absent')
        import apply_successors
        data = audit.read_json('successor-application.json')
        apply_successors.validate(data)
        return (apply_successors, data)

    def test_exact_order_and_literal_overlaps(self):
        module, d = self.application()
        self.assertEqual([i['patch'] for i in d['inputs']], [f'1.13.{n}' for n in range(3, 8)] + [f'1.14.{n}' for n in range(5)] + [f'1.15.{n}' for n in range(10)])
        self.assertEqual(d['totals']['actual_registers'], 20)
        self.assertEqual(d['totals']['source_inventory'], 2758)
        self.assertEqual(d['totals']['overlap_occurrences'], 581)
        self.assertEqual([(i['patch'], i['exact_overlap_occurrences']) for i in d['successor_summaries'] if i['exact_overlap_occurrences']], [('1.13.3', 14), ('1.13.4', 15), ('1.13.5', 5), ('1.14.0', 263), ('1.14.1', 1), ('1.14.3', 283)])
        self.assertTrue(all((i['semantic_status'] == 'UNPROVEN' and (not i['native_credit']) and (not i['model_credit']) for i in d['occurrence_status'])))
        self.assertEqual(d['totals']['meaningful_historical_closures'], 0)
        self.assertEqual(d['totals']['native_closures'], 0)

    def test_every_omission_reordering_and_foreign_credit_rejected(self):
        module, original = self.application()
        for f in ['inputs', 'successor_summaries', 'overlaps', 'occurrence_status']:
            for n in range(len(original[f])):
                changed = dict(original)
                changed[f] = original[f][:n] + original[f][n + 1:]
                with self.assertRaises(AssertionError):
                    module.validate(changed, expected=original)
        for key, value in [('history', 'retail'), ('patch', '1.12.0'), ('native_credit', True), ('model_credit', True)]:
            changed = copy.deepcopy(original['inputs'])
            changed[0][key] = value
            with self.assertRaises(AssertionError):
                module.build(changed)
        changed = copy.deepcopy(original['inputs'])
        changed.reverse()
        with self.assertRaises(AssertionError):
            module.build(changed)

    def test_frozen_queue_ledger_and_original_seals_unchanged(self):
        self.application()
        self.assertEqual(audit.validate_ledger(audit.read_json('ledger.json'))['inventory'], 2758)
        audit.check_seals()
        frozen = audit.read_json('ledger.json')
        self.assertEqual(frozen['later_registers'], [])
        self.assertEqual(frozen['measurements'], dict(runtime=0, model=0, native=0))
        self.assertTrue(all((not i['applied'] and (not i['native_proof']) for i in frozen['successors'])))
if __name__ == '__main__':
    unittest.main(verbosity=2)
