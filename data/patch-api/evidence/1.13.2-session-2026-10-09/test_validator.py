"""Targeted development fixtures for separate receipt validation, not final gates."""
import copy
import unittest
import audit
import validator

class ReceiptValidation(unittest.TestCase):

    def test_derives_original_and_actual_successor_counts(self):
        result = validator.validate()
        self.assertEqual(result.get('original_seals'), len(audit.read_json('seals.json')))
        self.assertEqual(result.get('source_totals'), audit.read_json('ledger.json')['totals'])
        self.assertEqual(result.get('actual_successor_totals'), audit.read_json('successor-application.json')['totals'])
        self.assertEqual(result.get('historical_credit'), {'runtime': 0, 'model': 0, 'native': 0})
        self.assertEqual(result.get('current_model_transitions'), 6)

    def test_derives_portable_replay_and_exact_restore_receipts(self):
        result = validator.validate()
        self.assertEqual(result.get('portable_tests'), 3)
        self.assertEqual(set(result.get('exact_restored_files', [])), {'ledger.json', 'green.log'})
        self.assertEqual(result.get('separate_receipt_seals'), len(audit.read_json('receipt-seals.json')))

    def test_current_model_reads_are_not_factory_defaults(self):
        observations = audit.read_json('current-model-observations.json')
        self.assertEqual(validator.validate_model(observations), 6)
        changed = copy.deepcopy(observations)
        changed['observations'][1]['observed'][1] = '0'
        with self.assertRaises(AssertionError):
            validator.validate_model(changed)
        changed = copy.deepcopy(observations)
        changed['observations'][2]['rust_store'] = '1'
        with self.assertRaises(AssertionError):
            validator.validate_model(changed)
        changed = copy.deepcopy(observations)
        changed['observations'].pop()
        with self.assertRaises(AssertionError):
            validator.validate_model(changed)
        for key in ['native_credit', 'source_default_credit', 'historical_signature_credit', 'cvar_effect_credit', 'source_ledger_mutated']:
            changed = copy.deepcopy(observations)
            changed[key] = True
            with self.assertRaises(AssertionError):
                validator.validate_model(changed)
if __name__ == '__main__':
    unittest.main(verbosity=2)
