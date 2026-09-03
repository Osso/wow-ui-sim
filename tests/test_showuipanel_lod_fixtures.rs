//! Coverage tests for the LoD panel test harness itself.
//!
//! Verifies that the shared fixtures in `tests/common/panel_fixtures.rs`
//! behave as documented:
//!
//! * The `ActionButtonUtil` fixture defaults to `NotMissing` AND can drive
//!   every `ActionBarActionStatus` through `SpellSearchUtil` via the
//!   per-id override tables.
//! * The `LoadAddOnWithErrorHandling` seam mirrors broadcaster failure
//!   bookkeeping, dedupes per name, and passes unrelated names through.
//! * The `__test_skip_cooldown_broadcaster_load` opt-out flag wires up a
//!   live path to exercise the real `CooldownBroadcaster_LoadUI` (kept
//!   `#[ignore]`d because the broadcaster's runtime deps aren't brought
//!   up by these panel tests).

use crate::common;

use common::panel_fixtures::setup_env;

/// Verifies the `ActionButtonUtil` test fixture: defaults remain `NotMissing`
/// (so existing panel tests don't change behaviour) but per-id override tables
/// can drive `MissingFromAllBars` / `OnInactiveBonusBar` / `OnDisabledActionBar`
/// through `SpellSearchUtil.GetActionbarStatusForSpell`. Without these branches
/// the spell-book / talent search-filter UI cannot be exercised end-to-end.
#[test]
fn action_button_util_fixture_drives_all_action_bar_statuses() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            local enum = ActionButtonUtil and ActionButtonUtil.ActionBarActionStatus
            if not enum then
                return "missing_action_bar_action_status_enum"
            end
            if enum.NotMissing ~= 1 or enum.MissingFromAllBars ~= 2
               or enum.OnInactiveBonusBar ~= 3 or enum.OnDisabledActionBar ~= 4 then
                return "wrong_enum_values"
            end

            -- Defaults: nothing configured → NotMissing for every probe.
            if ActionButtonUtil.GetActionBarStatusForSpell(101) ~= enum.NotMissing then
                return "default_spell_not_NotMissing"
            end
            if ActionButtonUtil.GetActionBarStatusForPetAction(202) ~= enum.NotMissing then
                return "default_pet_not_NotMissing"
            end
            if ActionButtonUtil.GetActionBarStatusForFlyout(303) ~= enum.NotMissing then
                return "default_flyout_not_NotMissing"
            end

            -- Overrides drive each non-default branch SpellSearchUtil cares about.
            __test_action_bar_status_for_spell[1001] = enum.MissingFromAllBars
            __test_action_bar_status_for_spell[1002] = enum.OnInactiveBonusBar
            __test_action_bar_status_for_spell[1003] = enum.OnDisabledActionBar
            __test_action_bar_status_for_pet_action[2001] = enum.OnDisabledActionBar
            __test_action_bar_status_for_flyout[3001] = enum.OnInactiveBonusBar

            if ActionButtonUtil.GetActionBarStatusForSpell(1001) ~= enum.MissingFromAllBars then
                return "spell_override_MissingFromAllBars_failed"
            end
            if ActionButtonUtil.GetActionBarStatusForSpell(1002) ~= enum.OnInactiveBonusBar then
                return "spell_override_OnInactiveBonusBar_failed"
            end
            if ActionButtonUtil.GetActionBarStatusForSpell(1003) ~= enum.OnDisabledActionBar then
                return "spell_override_OnDisabledActionBar_failed"
            end
            if ActionButtonUtil.GetActionBarStatusForPetAction(2001) ~= enum.OnDisabledActionBar then
                return "pet_override_failed"
            end
            if ActionButtonUtil.GetActionBarStatusForFlyout(3001) ~= enum.OnInactiveBonusBar then
                return "flyout_override_failed"
            end

            -- The overrides must flow through SpellSearchUtil's wrappers
            -- (this is the path real Blizzard search-filter UI exercises).
            if SpellSearchUtil then
                if SpellSearchUtil.GetActionbarStatusForSpell(1001) ~= enum.MissingFromAllBars then
                    return "spellsearch_spell_override_failed"
                end

                local talentNode = { entryIDsWithCommittedRanks = { 1 } }
                if SpellSearchUtil.GetActionBarStatusForTraitNode(talentNode, 1002) ~= enum.OnInactiveBonusBar then
                    return "spellsearch_trait_override_failed"
                end
                if SpellSearchUtil.GetActionBarStatusForTraitNodeEntry(1, talentNode, 1003) ~= enum.OnDisabledActionBar then
                    return "spellsearch_trait_entry_override_failed"
                end

                -- And the tooltip lookup table on SpellSearchUtil should index
                -- non-`NotMissing` statuses (sanity: confirms the enum keys
                -- match the ones SpellSearchUtil built its lookup tables with).
                if SpellSearchUtil.ActionBarStatusTooltips[enum.MissingFromAllBars] == nil
                   or SpellSearchUtil.ActionBarStatusMatchTypes[enum.OnDisabledActionBar] == nil then
                    return "spellsearch_lookup_table_missing_entries"
                end
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result, "ok",
            "ActionButtonUtil fixture should drive every ActionBarActionStatus through SpellSearchUtil: {result}"
        );
    }
}

/// Verifies the `LoadAddOnWithErrorHandling` harness seam: the broadcaster
/// publisher reaches the scoped skip path, records one observable failure, and
/// unrelated names still reach Blizzard's real loader helper.
#[test]
fn load_addon_with_error_handling_seam_records_broadcaster_failure() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            if type(CooldownBroadcaster_LoadUI) ~= "function" then
                return "missing_cooldown_broadcaster_loadui"
            end
            CooldownBroadcaster_LoadUI()

            if type(__test_load_addon_with_error_handling_failures) ~= "table"
               or #__test_load_addon_with_error_handling_failures < 1 then
                return "harness_log_empty"
            end
            local last = __test_load_addon_with_error_handling_failures[
                #__test_load_addon_with_error_handling_failures
            ]
            if last.name ~= "Blizzard_CooldownBroadcaster" or last.reason ~= "DISABLED_FOR_TESTS" then
                return "harness_log_wrong_entry:" .. tostring(last.name) .. "/" .. tostring(last.reason)
            end

            local before = #__test_load_addon_with_error_handling_failures
            CooldownBroadcaster_LoadUI()
            if #__test_load_addon_with_error_handling_failures ~= before then
                return "harness_log_duplicated_entry"
            end

            local already_loaded_addon = "Blizzard_SharedXMLBase"
            local loaded = LoadAddOnWithErrorHandling(already_loaded_addon)
            if loaded == nil then
                return "wrapper_swallowed_unrelated_addon"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result, "ok",
            "LoadAddOnWithErrorHandling seam should preserve broadcaster bookkeeping and pass-through: {result}"
        );
    }
}

/// Dedicated coverage path for `CooldownBroadcaster_LoadUI` against the
/// real `Blizzard_CooldownBroadcaster` addon load. Off by default
/// (`#[ignore]`) because its runtime dependencies are outside this panel
/// fixture. Set `__test_skip_cooldown_broadcaster_load` to `false` before
/// invoking its current bootstrap publisher to exercise the real path.
#[test]
#[ignore = "opt-in coverage for real CooldownBroadcaster load — needs C_ChatInfo/C_Spell surface"]
fn cooldown_broadcaster_loadui_can_be_exercised_when_opt_in() {
    test_timeout! {
        let env = setup_env();
        let result: String = env.eval(r#"
            -- Disable the broadcaster skip and clear any prior failure
            -- bookkeeping so we observe a fresh run.
            __test_skip_cooldown_broadcaster_load = false
            if FailedAddOnLoad then
                FailedAddOnLoad["Blizzard_CooldownBroadcaster"] = nil
            end

            if type(CooldownBroadcaster_LoadUI) ~= "function" then
                return "missing_cooldown_broadcaster_loadui"
            end

            CooldownBroadcaster_LoadUI()

            -- Either the addon loaded (CooldownBroadcasterFrame is set) OR
            -- the real load failed. The scoped skip marker must be absent.
            for _, entry in ipairs(__test_load_addon_with_error_handling_failures or {}) do
                if entry.name == "Blizzard_CooldownBroadcaster"
                   and entry.reason == "DISABLED_FOR_TESTS" then
                    return "seam_failed_to_deactivate"
                end
            end

            if CooldownBroadcasterFrame then
                return "loaded"
            end
            return "real_load_failed_but_seam_deactivated"
        "#).unwrap();
        // Either outcome is acceptable for the seam itself — the assertion
        // proves the opt-in path is live, not that CB fully starts up.
        assert!(
            result == "loaded" || result == "real_load_failed_but_seam_deactivated",
            "broadcaster opt-in seam should run the real load path (got {result})"
        );
    }
}
