"""Pinned 2012 retail syntax: code removals and owner-specific handlers."""
import unittest

import gen_patch_wikitext_register as generator


class Patch51Tests(unittest.TestCase):
    def test_code_removals_preserve_identity_direction_owner_and_line(self):
        raw = ('{| class="wikitable"\n|+ Global API (5.0.4.16016 &rarr; 5.1.0.16309)\n'
               '! style="width: 50%;"| 1 new functions\n'
               '! style="width: 50%;"| 1 removed functions\n'
               '| valign="top" | <div>\n: {{api|C_PetJournal.SummonPetByGUID}}\n</div>\n'
               '| valign="top" | <div>\n: <code>C_PetJournal.SummonPetByID</code>\n</div>\n|}\n')
        rows, counts = generator.parse_mists_automated_diff(
            generator.normalize_mists_code_removals(raw))
        self.assertEqual([(r['symbol'], r['direction'], r['wikitext_line']) for r in rows],
                         [('C_PetJournal.SummonPetByGUID', 'added', 6),
                          ('C_PetJournal.SummonPetByID', 'removed', 9)])
        self.assertEqual([(c['header_count'], c['parsed_count']) for c in counts],
                         [(1, 1), (1, 1)])

    def test_removed_animation_handlers_keep_owners(self):
        raw = ('{| class="wikitable"\n|+ Widget Handlers (old &rarr; new)\n'
               '! style="width: 50%;"| 0 new handlers\n'
               '! style="width: 50%;"| 2 removed handlers\n'
               '| valign="top" | <div>\n</div>\n'
               '| valign="top" | <div>\n: <code>Animation OnEvent</code>\n'
               ': <code>AnimationGroup OnEvent</code>\n</div>\n|}\n')
        rows, counts = generator.parse_mists_widget_handlers(
            generator.normalize_mists_code_removals(raw))
        self.assertEqual([(r['symbol'], r['kind'], r['direction']) for r in rows],
                         [('Animation OnEvent', 'widget-script', 'removed'),
                          ('AnimationGroup OnEvent', 'widget-script', 'removed')])
        self.assertEqual([(c['header_count'], c['parsed_count']) for c in counts],
                         [(0, 0), (2, 2)])

    def test_summary_retains_restricted_environment_and_cooldown_annotations(self):
        rows = generator.parse_mists_summary(
            '== New features ==\n** {{api|CanExitVehicle}}\n'
            '* The {{api|t=w|Cooldown:SetCooldown}} function shows/hides.\n')
        self.assertEqual([(r['symbol'], r['section']) for r in rows],
                         [('CanExitVehicle', 'global-api'), ('Cooldown:SetCooldown', 'widgets')])
        self.assertIn('shows/hides', rows[1]['annotation'])


if __name__ == '__main__':
    unittest.main()
