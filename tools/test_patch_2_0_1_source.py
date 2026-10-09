"""Own historical retail source fixtures; no publication/native measurement."""
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
RAW = ROOT / 'data/patch-api/source-cache/legacy-2026-10-09/2.0.1-wikitext.txt'


class SourceTests(unittest.TestCase):
    def audit(self, raw):
        path = ROOT / 'tools/audit_patch_2_0_1_source.py'
        self.assertTrue(path.exists(), 'own literal source auditor missing')
        spec = importlib.util.spec_from_file_location('p201', path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module.account(raw)

    def test_literal_signatures_duplicates_and_unexpanded_references(self):
        raw = ('==API Changes: Bindings==\n'
               '* NEW SetOverrideBindingClick(owner, isPriority, "KEY", "ButtoName"[,"mouseButton"])\n'
               '* UPDATED Button:RegisterForClicks("button"[,"button"...])\n'
               '* NEW SetOverrideBindingClick(owner)\n'
               '* RENAMED isPassive = IsPassiveSpell(spell) -- Formerly IsSpellPassive(spell)\n'
               '[[API Elsewhere]] {{:LinkedContract}}\n')
        audit = self.audit(raw)
        calls = [r for r in audit['occurrences'] if r['kind'] in ('global-api', 'widget-method')]
        self.assertEqual([r['symbol'] for r in calls], [
            'SetOverrideBindingClick', 'Button:RegisterForClicks',
            'SetOverrideBindingClick', 'IsPassiveSpell', 'IsSpellPassive'])
        self.assertEqual(calls[0]['signature'],
                         'SetOverrideBindingClick(owner, isPriority, "KEY", "ButtoName"[,"mouseButton"])')
        self.assertEqual(calls[-1]['direction'], 'former-name')
        self.assertEqual(len({r['id'] for r in calls}), 5)
        self.assertEqual([r['literal'] for r in audit['references']],
                         ['[[API Elsewhere]]', '{{:LinkedContract}}'])
        self.assertTrue(all(r['status'] == 'UNPROVEN' for r in audit['references']))
        self.assertTrue(all(not r['capabilities'] for r in audit['source_rows']))

    def test_frozen_full_rows_handlers_commands_and_late_qualifiers(self):
        raw = RAW.read_text()
        audit = self.audit(raw)
        self.assertEqual([(r['line'], r['literal']) for r in audit['source_rows']],
                         [(n, line) for n, line in enumerate(raw.splitlines(), 1) if line.strip()])
        rows = audit['occurrences']
        self.assertTrue(any(r['symbol'] == 'OnEvent' and r['signature'] == 'function(self,event,...)'
                            for r in rows))
        self.assertTrue(any(r['symbol'] == '/randomcast' for r in rows))
        self.assertTrue(any(r['symbol'] == '/castrandom' for r in rows))
        self.assertTrue(any(r['symbol'] == 'SPELLCAST_*' and r['kind'] == 'event-family'
                            for r in rows))
        self.assertTrue(any(r['symbol'] == 'SecureAnchorUpDownTemplate' for r in rows))
        self.assertTrue(any(r['symbol'] == 'PLAYER_FOCUS_CHANGED' for r in rows))
        self.assertEqual(audit['counts']['cvars'], 0)
        self.assertEqual(audit['counts']['meaningful_closures'], 0)
        self.assertEqual(audit['counts']['runtime_observations'], 0)
        self.assertTrue(any(r['patch_qualifier'] == '2.0.6' for r in audit['source_rows']))
        self.assertEqual(audit['headers'][0]['title'], 'Interface AddOn Kit')
        self.assertEqual(audit['headers'][-1]['title'], 'Bug Fixes 2.0.6')

    def test_malformed_signature_is_preserved_not_completed(self):
        audit = self.audit('* NEW ExampleCall("unterminated)\n')
        row = audit['occurrences'][0]
        self.assertEqual(row['signature'], 'ExampleCall("unterminated)')
        self.assertEqual(row['signature_status'], 'malformed-unclosed')
        self.assertEqual(row['status'], 'UNPROVEN')


if __name__ == '__main__':
    unittest.main(verbosity=2)
