"""Serialized 5.0.4 source identities, signatures and separate diff origins."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from extract_patch_non_inventory import extract_text

ROOT = Path(__file__).resolve().parents[1]


class MistsPrepatchTests(unittest.TestCase):
    def register(self, page, diff):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / 'page.wikitext'
            inventory = Path(tmp) / 'diff.wikitext'
            output = Path(tmp) / 'register.json'
            source.write_text(page)
            inventory.write_text(diff)
            result = subprocess.run([
                sys.executable, str(ROOT / 'tools/gen_patch_wikitext_register.py'),
                '5.0.4', str(source), '3706382', str(output),
                '--mists-prepatch', '--mists-diff', str(inventory),
                '--client-line', 'retail',
            ], cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            return json.loads(output.read_text())

    def test_rename_identities_and_context_are_not_removals(self):
        page = ('== Breaking changes ==\n'
                '*: <code>IsRaidLeader, IsPartyLeader</code> &rarr; '
                '{{api|UnitIsGroupLeader}} (use {{api|IsInRaid}} to check)\n'
                '*: <code>RAID_ROSTER_UPDATE, PARTY_MEMBERS_CHANGED</code> '
                '&rarr; {{api|t=e|GROUP_ROSTER_UPDATE}}\n'
                '* {{api|UnitGUID}} responding to {{api|t=e|PARTY_MEMBERS_CHANGED}} returns nil.\n')
        register = self.register(page, '')
        rows = register['entries']
        self.assertEqual([(r['symbol'], r['direction'], r['section']) for r in rows], [
            ('IsRaidLeader', 'removed', 'global-api'),
            ('IsPartyLeader', 'removed', 'global-api'),
            ('UnitIsGroupLeader', 'added', 'global-api'),
            ('IsInRaid', 'changed', 'global-api'),
            ('RAID_ROSTER_UPDATE', 'removed', 'events'),
            ('PARTY_MEMBERS_CHANGED', 'removed', 'events'),
            ('GROUP_ROSTER_UPDATE', 'added', 'events'),
            ('UnitGUID', 'changed', 'global-api'),
            ('PARTY_MEMBERS_CHANGED', 'changed', 'events'),
        ])
        self.assertEqual(rows[0]['id'], 'wt-global-api-IsRaidLeader-2')
        self.assertEqual(rows[0]['annotation'], page.splitlines()[1])
        self.assertEqual(register['source']['revid'], 3706382)
        self.assertEqual(register['client_line'], 'retail')

    def test_code_inventories_have_original_diff_ids_and_counts(self):
        diff = ('{| class="wikitable"\n'
                '|+ Widget API (4.3.4.15595 &rarr; 5.0.4.16016)\n'
                '! style="width: 50%"| 1 new methods\n'
                '! style="width: 50%"| 1 removed methods\n'
                '| valign="top" | <div>\n'
                ': {{api|t=w|Button:SetEnabled}}\n</div>\n'
                '| valign="top" | <div>\n'
                ': <code>Cooldown:SetDrawEdge</code>\n</div>\n|}\n')
        register = self.register('* NEW {{api|t=w|Button:SetEnabled}}(enable) - Enables.\n', diff)
        self.assertEqual([(r['id'], r['direction'], r['wikitext_line'])
                          for r in register['entries']], [
            ('wt-widgets-Button:SetEnabled-1', 'added', 1),
            ('diff-wt-widgets-Button:SetEnabled-6', 'added', 6),
            ('diff-wt-widgets-Cooldown:SetDrawEdge-9', 'removed', 9),
        ])
        self.assertEqual([(c['header_count'], c['parsed_count'])
                          for c in register['header_counts']], [(1, 1), (1, 1)])
        self.assertEqual(register['entries'][0]['signature'], '(enable)')

    def test_signatures_framexml_and_prose_remain_distinct(self):
        page = ('== Breaking changes ==\n'
                '*: <code>GetAchievementCriteriaInfo</code> split into two separate functions:\n'
                '*:* By criteria order: {{api|GetAchievementCriteriaInfo}}(achievementID, criteriaIndex)\n'
                '=== Widget API ===\n'
                '* NEW {{api|t=w|Frame:RegisterUnitEvent}}("event", "unit1"[, "unit2"]) - Triggers OnEvent.\n'
                '== FrameXML ==\n'
                '* <code>OldTemplate</code> is renamed to <code>NewTemplate</code>; the old version of which is removed.\n'
                '* <code>OldFrame</code> is removed; handled by <code>SuccessorFrame</code>.\n')
        rows = self.register(page, '')['entries']
        self.assertEqual([(r['symbol'], r['direction']) for r in rows], [
            ('GetAchievementCriteriaInfo', 'changed'),
            ('GetAchievementCriteriaInfo', 'changed'),
            ('Frame:RegisterUnitEvent', 'added'),
            ('OldTemplate', 'removed'), ('NewTemplate', 'added'),
            ('OldFrame', 'removed'), ('SuccessorFrame', 'changed'),
        ])
        self.assertEqual(rows[1]['signature'], '(achievementID, criteriaIndex)')
        self.assertEqual(rows[2]['signature'], '("event", "unit1"[, "unit2"])')
        self.assertEqual(rows[2]['annotation'], page.splitlines()[4])
        self.assertTrue(all(r['section'] == 'framexml' for r in rows[3:]))
        self.assertEqual(extract_text(page, retain_patch_diff_reference=True),
                         '== Breaking changes ==\n'
                         '*: GetAchievementCriteriaInfo split into two separate functions:\n'
                         '*:* By criteria order: GetAchievementCriteriaInfo(achievementID, criteriaIndex)\n'
                         '=== Widget API ===\n'
                         '* NEW Frame:RegisterUnitEvent("event", "unit1"[, "unit2"]) - Triggers OnEvent.\n'
                         '== FrameXML ==\n'
                         '* OldTemplate is renamed to NewTemplate; the old version of which is removed.\n'
                         '* OldFrame is removed; handled by SuccessorFrame.\n')


if __name__ == '__main__':
    unittest.main()
