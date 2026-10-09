"""Serialized historical source contracts only; no simulator/native claims."""
import json
import unittest

from validate import load_inputs, validate


class SourceAccounting(unittest.TestCase):
    def setUp(self):
        self.inputs = load_inputs()

    def mutate(self, key, change):
        value = json.loads(self.inputs[key])
        change(value)
        return dict(self.inputs, **{key: json.dumps(value).encode()})

    def reject(self, key, change, reason):
        with self.assertRaisesRegex(AssertionError, reason):
            validate(**self.mutate(key, change))

    def test_exact_inventory_and_prose_counts(self):
        result = validate(**self.inputs)
        self.assertEqual(result, {
            'source_rows': 208,
            'statuses': {'metadata-only': 51, 'UNPROVEN': 157},
            'inventory_occurrences': 155, 'added': 109, 'removed': 46,
            'section_counts': {'global-api': 94, 'widgets': 18, 'events': 8, 'cvars': 35},
            'prose_limits': 2, 'signature_occurrences': 0,
            'successor_inventory_occurrences': 0,
            'runtime_observations': 0, 'native_observations': 0,
            'cache_files': 42,
        })

    def test_wrong_revision_and_content_rejected(self):
        self.reject('response', lambda v: v['query']['pages']['355030']['revisions'][0].update(revid=1), 'revid')
        with self.assertRaisesRegex(AssertionError, 'returned content'):
            validate(**dict(self.inputs, raw=self.inputs['raw'] + b'\n'))

    def test_each_missing_literal_row_rejected(self):
        rows = json.loads(self.inputs['ledger'])['source_rows']
        for index in range(len(rows)):
            with self.subTest(index=index):
                self.reject('ledger', lambda v: v['source_rows'].pop(index), 'literal rows')

    def test_each_missing_inventory_occurrence_rejected(self):
        entries = json.loads(self.inputs['inventory'])['entries']
        for index in range(len(entries)):
            with self.subTest(index=index):
                self.reject('inventory', lambda v: v['entries'].pop(index), 'inventory reproduction')

    def test_direction_defaults_and_header_tampering_rejected(self):
        self.reject('inventory', lambda v: v['entries'][0].update(direction='removed'), 'inventory reproduction')
        self.reject('inventory', lambda v: v['header_counts'][0].update(header_count=1), 'inventory reproduction')
        ledger = json.loads(self.inputs['ledger'])
        index = next(i for i, row in enumerate(ledger['source_rows']) if 'source_contract' in row)
        self.reject('ledger', lambda v: v['source_rows'][index]['source_contract'].update(default='999'), 'CVar contract')

    def test_every_unproven_row_rejects_fabricated_credit(self):
        for index, row in enumerate(json.loads(self.inputs['ledger'])['source_rows']):
            if row['status'] != 'UNPROVEN':
                continue
            for field, value in [('status', 'bounded-coverage'), ('capabilities', ['publication'])]:
                with self.subTest(index=index, field=field):
                    self.reject('ledger', lambda v: v['source_rows'][index].update({field: value}), 'proof limit')

    def test_wrong_history_and_foreign_successors_rejected(self):
        for field, value in [('client_line', 'retail'), ('profile', 'mists'),
                             ('later_registers', ['10.1.0']), ('later_registers', ['4.4.0'])]:
            with self.subTest(field=field, value=value):
                self.reject('ledger', lambda v: v.update({field: value}), 'client history')
        self.reject('ledger', lambda v: v['successor'].update(member_supersessions=['StripHyperlinks']), 'successor')
        self.reject('successor_response', lambda v: v['query']['pages']['152751']['revisions'][0].update(revid=1), 'revid')

    def test_wrong_profile_and_native_credit_rejected(self):
        for field, value in [('profile', 'retail'), ('source_toc', 30403),
                             ('runtime_observations', 1), ('native_observations', 1),
                             ('supported_profile', False), ('configured_interface', 30402)]:
            with self.subTest(field=field):
                self.reject('profile', lambda v: v.update({field: value}), 'profile evidence')

    def test_literal_identity_and_reason_required(self):
        self.reject('ledger', lambda v: v['source_rows'][0].update(source_text='changed'), 'literal rows')
        self.reject('ledger', lambda v: v['source_rows'][0].update(note=''), 'reason')
        self.reject('ledger', lambda v: v['source_rows'][20].update(source_id='duplicate'), 'row identity')

    def test_extract_and_pin_tampering_rejected(self):
        with self.assertRaisesRegex(AssertionError, 'plaintext reproduction'):
            validate(**dict(self.inputs, text=self.inputs['text'] + b'fake signature()'))
        self.reject('pin', lambda v: v.update(wikitext_sha256='0' * 64), 'source hash')


if __name__ == '__main__':
    unittest.main(verbosity=2)
