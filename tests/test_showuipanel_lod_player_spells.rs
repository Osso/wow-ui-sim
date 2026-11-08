//! Integration tests for the Blizzard_PlayerSpells panel: keybind-driven
//! load and PlayerSpells data-flow regressions.
//!
//! Companion to `test_showuipanel_lod.rs` (other panel loaders) and
//! `test_showuipanel_lod_fixtures.rs` (harness/fixture coverage). All
//! three share `tests/common/panel_fixtures.rs`.

// The Mists manifest has no Blizzard_PlayerSpells entries; Mists uses the
// separate SpellBookFrame and TalentUI consumers instead.
retail_ptr_forever_toc_fixture! {
use crate::common;

use common::panel_fixtures::{
    clear_recorded_lua_errors, player_spells_panel_debug_snapshot, recorded_lua_errors, setup_env,
};
use wow_ui_sim::startup::run_extra_update_ticks;
post_wrath_ui_fixture! {
use wow_ui_sim::startup::prewarm_player_spells_spellbook;
}
use wow_ui_sim::traits::{TRAIT_CURRENCY_DB, TRAIT_TREE_DB};

fn open_talents(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        assert(PlayerSpellsUtil and PlayerSpellsUtil.ToggleClassTalentFrame, "ToggleClassTalentFrame should exist")
        PlayerSpellsUtil.ToggleClassTalentFrame()
        assert(PlayerSpellsFrame and PlayerSpellsFrame:IsShown(), "PlayerSpellsFrame should be shown")
        assert(PlayerSpellsFrame.TalentsFrame and PlayerSpellsFrame.TalentsFrame:IsShown(), "TalentsFrame should be shown")
        "#,
    )
    .expect("Failed to open talents");
}

fn max_points_for_currency(currency_id: u32) -> u32 {
    let flags = TRAIT_CURRENCY_DB
        .get(&currency_id)
        .map(|currency| currency.flags)
        .unwrap_or_default();
    match flags {
        4 => 31,
        8 => 30,
        _ => 0,
    }
}

post_wrath_ui_fixture! {
#[test]
fn startup_prewarm_loads_blizzard_player_spells_and_keeps_it_hidden() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__spellbook_prewarm_errors");
        clear_recorded_lua_errors(&env);

        let unloaded_before: bool = env
            .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells")"#)
            .expect("initial addon load probe should return");
        assert!(
            !unloaded_before,
            "Test harness should start with Blizzard_PlayerSpells unloaded"
        );

        let warmed = prewarm_player_spells_spellbook(&env);
        assert!(warmed, "SpellBook prewarm should run on the game screen");

        let loaded_after: bool = env
            .eval(r#"return C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells")"#)
            .expect("addon load probe after prewarm should return");
        assert!(loaded_after, "SpellBook prewarm should demand-load Blizzard_PlayerSpells");

        let hidden_after: bool = env
            .eval(r#"return PlayerSpellsFrame ~= nil and not PlayerSpellsFrame:IsShown()"#)
            .expect("panel visibility probe after prewarm should return");
        assert!(
            hidden_after,
            "SpellBook prewarm should leave PlayerSpellsFrame hidden"
        );

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__spellbook_prewarm_errors");
        assert!(
            recorded_errors.is_empty(),
            "SpellBook prewarm produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );
        assert!(
            handler_errors.is_empty(),
            "SpellBook prewarm produced {} Lua error(s):\n{}",
            handler_errors.len(),
            handler_errors.join("\n")
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn raw_toggle_spellbook_frame_loads_blizzard_player_spells_and_shows_spellbook() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__spellbook_raw_toggle_errors");
        clear_recorded_lua_errors(&env);

        let result: String = env.eval(r#"
            if C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells") then
                return "addon_preloaded"
            end

            if not PlayerSpellsUtil or type(PlayerSpellsUtil.ToggleSpellBookFrame) ~= "function" then
                return "missing_toggle_spellbook_frame"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Test harness should start with Blizzard_PlayerSpells unloaded and the raw toggle helper available: {result}"
        );

        env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
            .expect("raw ToggleSpellBookFrame call failed");

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__spellbook_raw_toggle_errors");
        assert!(
            recorded_errors.is_empty(),
            "Opening spellbook through raw ToggleSpellBookFrame produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );
        assert!(
            handler_errors.is_empty(),
            "Opening spellbook through raw ToggleSpellBookFrame produced {} Lua error(s):\n{}",
            handler_errors.len(),
            handler_errors.join("\n")
        );

        let result: String = env.eval(r#"
            if not C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells") then
                return "addon_not_loaded"
            end
            if not PlayerSpellsFrame or not PlayerSpellsFrame:IsShown() then
                return "player_spells_not_shown"
            end
            if not PlayerSpellsFrame.SpellBookFrame or not PlayerSpellsFrame.SpellBookFrame:IsShown() then
                return "spellbook_tab_not_shown"
            end
            if not (PlayerSpellsFrame:IsFrameTabActive(PlayerSpellsUtil.FrameTabs.SpellBook)) then
                return "spellbook_tab_not_active"
            end
            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Raw PlayerSpellsUtil.ToggleSpellBookFrame() should demand-load Blizzard_PlayerSpells and show the SpellBook tab: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn keybind_s_loads_blizzard_player_spells_and_shows_spellbook() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__spellbook_keybind_errors");
        clear_recorded_lua_errors(&env);

        let result: String = env.eval(r#"
            local loadedBefore = C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells")
            if loadedBefore then
                return "addon_preloaded"
            end

            if not PlayerSpellsUtil or type(PlayerSpellsUtil.ToggleSpellBookFrame) ~= "function" then
                return "missing_toggle_spellbook_frame"
            end

            if GetBindingAction("S") ~= "" then
                return "unexpected_binding_store_seed"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Test harness should start with Blizzard_PlayerSpells unloaded and the keybinding store unseeded: {result}"
        );

        env.send_key_press("S", None).expect("S keybind failed");

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__spellbook_keybind_errors");
        assert!(
            recorded_errors.is_empty(),
            "Opening spellbook through S produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );

        assert!(handler_errors.is_empty(), "Opening spellbook through S produced {} Lua error(s):\n{}", handler_errors.len(), handler_errors.join("\n"));

        let result: String = env.eval(r#"
            if not C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells") then
                return "addon_not_loaded"
            end
            if not PlayerSpellsFrame or not PlayerSpellsFrame:IsShown() then
                return "player_spells_not_shown"
            end
            if not PlayerSpellsFrame.SpellBookFrame or not PlayerSpellsFrame.SpellBookFrame:IsShown() then
                return "spellbook_tab_not_shown"
            end
            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Pressing S should demand-load Blizzard_PlayerSpells and show the SpellBook tab: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn raw_spellbook_toggle_hands_off_to_blizzard_playerspells_util_on_first_load() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__spellbook_raw_toggle_errors");
        clear_recorded_lua_errors(&env);

        let (before_source, after_source, result): (String, String, String) = env
            .eval(
                r#"
                local function fn_source(fn)
                    local info = debug.getinfo(fn, "S")
                    return info and info.source or "missing"
                end

                if C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells") then
                    return "addon_preloaded", "addon_preloaded", "addon_preloaded"
                end

                if not PlayerSpellsUtil or type(PlayerSpellsUtil.ToggleSpellBookFrame) ~= "function" then
                    return "missing_toggle_spellbook_frame", "missing_toggle_spellbook_frame", "missing_toggle_spellbook_frame"
                end

                local beforeSource = fn_source(PlayerSpellsUtil.ToggleSpellBookFrame)
                PlayerSpellsUtil.ToggleSpellBookFrame()
                local afterSource = fn_source(PlayerSpellsUtil.ToggleSpellBookFrame)

                if not C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells") then
                    return beforeSource, afterSource, "addon_not_loaded"
                end
                if not PlayerSpellsFrame or not PlayerSpellsFrame:IsShown() then
                    return beforeSource, afterSource, "player_spells_not_shown"
                end
                if not PlayerSpellsFrame.SpellBookFrame or not PlayerSpellsFrame.SpellBookFrame:IsShown() then
                    return beforeSource, afterSource, "spellbook_tab_not_shown"
                end

                return beforeSource, afterSource, "ok"
                "#,
            )
            .expect("raw spellbook toggle handoff probe should return");

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__spellbook_raw_toggle_errors");
        assert!(
            recorded_errors.is_empty(),
            "Raw spellbook toggle produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );
        assert!(
            handler_errors.is_empty(),
            "Raw spellbook toggle produced {} Lua error(s):\n{}",
            handler_errors.len(),
            handler_errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Raw PlayerSpellsUtil.ToggleSpellBookFrame() should demand-load Blizzard_PlayerSpells and show the spellbook: {result}"
        );
        assert!(
            before_source.contains("Blizzard_FrameXMLUtil/Mainline/PlayerSpellsUtil.lua"),
            "Before the first spellbook open, ToggleSpellBookFrame should already come from Blizzard_FrameXMLUtil: {before_source}"
        );
        assert!(
            after_source.contains("Blizzard_FrameXMLUtil/Mainline/PlayerSpellsUtil.lua"),
            "After load, ToggleSpellBookFrame should still be Blizzard-owned: {after_source}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn keybind_n_loads_blizzard_player_spells_and_shows_talents() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__talents_keybind_errors");
        clear_recorded_lua_errors(&env);

        let result: String = env.eval(r#"
            local loadedBefore = C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells")
            if loadedBefore then
                return "addon_preloaded"
            end

            if not PlayerSpellsUtil or type(PlayerSpellsUtil.ToggleClassTalentFrame) ~= "function" then
                return "missing_toggle_class_talent_frame"
            end

            if GetBindingAction("N") ~= "" then
                return "unexpected_binding_store_seed"
            end

            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Test harness should start with Blizzard_PlayerSpells unloaded and the keybinding store unseeded: {result}"
        );

        env.send_key_press("N", None).expect("N keybind failed");

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__talents_keybind_errors");
        assert!(
            recorded_errors.is_empty(),
            "Opening talents through N produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );

        assert!(handler_errors.is_empty(), "Opening talents through N produced {} Lua error(s):\n{}", handler_errors.len(), handler_errors.join("\n"));

        let result: String = env.eval(r#"
            if not C_AddOns.IsAddOnLoaded("Blizzard_PlayerSpells") then
                return "addon_not_loaded"
            end
            if not PlayerSpellsFrame or not PlayerSpellsFrame:IsShown() then
                return "player_spells_not_shown"
            end
            if not PlayerSpellsFrame.TalentsFrame or not PlayerSpellsFrame.TalentsFrame:IsShown() then
                return "talents_tab_not_shown"
            end
            if not (PlayerSpellsUtil and PlayerSpellsUtil.FrameTabs and PlayerSpellsFrame:IsFrameTabActive(PlayerSpellsUtil.FrameTabs.ClassTalents)) then
                return "talents_tab_not_active"
            end
            return "ok"
        "#).unwrap();
        assert_eq!(
            result,
            "ok",
            "Pressing N should demand-load Blizzard_PlayerSpells and show the talents tab: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn starter_build_highlight_uses_player_spells_talent_ui_call_path() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__starter_build_highlight_errors");
        clear_recorded_lua_errors(&env);

        {
            let mut state = env.state().borrow_mut();
            state.talents.has_starter_build = true;
            state.talents.is_starter_build_active = true;
        }

        env.send_key_press("N", None).expect("N keybind failed");
        run_extra_update_ticks(&env, 3);

        let result: String = env.eval(r#"
            local talents = PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame
            assert(talents and talents:IsShown(), "talents frame should be visible")

            talents:UpdateStarterBuildHighlights()
            local highlight = talents.activeStarterBuildHighlight
            local nodeID, entryID = C_ClassTalents.GetNextStarterBuildPurchase()
            assert(nodeID and entryID, "starter build API should provide a purchase candidate")
            if highlight then
                assert(nodeID == highlight.nodeID, "highlight should use the starter build node")
                assert(entryID == highlight.entryID, "highlight should use the starter build entry")
                assert(not talents:WillDeviateFromStarterBuild(nodeID, entryID), "highlighted purchase should not count as deviation")
            end

            local function findExportClipboardSentinel(loadSystem)
                for _, sentinelInfo in pairs(loadSystem.sentinelInfos) do
                    if sentinelInfo.sentinelInfos then
                        for _, child in ipairs(sentinelInfo.sentinelInfos) do
                            if child.text == TALENT_FRAME_DROP_DOWN_EXPORT_CLIPBOARD then
                                return child
                            end
                        end
                    end
                end
            end

            local clipboard = assert(findExportClipboardSentinel(talents.LoadSystem), "export clipboard sentinel should exist")
            local disabled, _, _, _ = clipboard.disabledCallback()
            assert(type(disabled) == "boolean", "disabled callback should return a boolean")

            return table.concat({
                tostring(nodeID),
                tostring(entryID),
                tostring(disabled),
            }, ",")
        "#).unwrap();

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__starter_build_highlight_errors");
        assert!(
            recorded_errors.is_empty(),
            "Starter build highlight regression produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );
        assert!(
            handler_errors.is_empty(),
            "Starter build highlight regression produced {} Lua error(s):\n{}",
            handler_errors.len(),
            handler_errors.join("\n")
        );
        assert!(
            result.contains(','),
            "starter build highlight regression should return node and entry ids: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn player_spells_export_disabled_callback_tracks_unspent_hero_points() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__hero_export_gate_errors");
        clear_recorded_lua_errors(&env);

        open_talents(&env);

        let hero_currency_id = {
            let mut state = env.state().borrow_mut();
            let hero_currency_id = match state.talents.active_hero_subtree() {
                Some(48) => 2986,
                Some(49) => 2987,
                Some(50) => 2988,
                other => panic!("unexpected active hero subtree: {other:?}"),
            };
            let class_currency_ids = TRAIT_TREE_DB
                .get(&790)
                .expect("Paladin class tree should exist")
                .currency_ids;
            for currency_id in class_currency_ids {
                let currency_id = *currency_id;
                let max_points = max_points_for_currency(currency_id);
                state.talents.currency_spent.insert(currency_id, max_points);
            }
            state.talents.currency_spent.insert(hero_currency_id, 10);
            hero_currency_id
        };

        let disabled_with_points: bool = env
            .eval(
                r#"
                local talents = assert(PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame, "talents frame should exist")

                local function find_export_clipboard(loadSystem)
                    for _, sentinelInfo in pairs(loadSystem.sentinelInfos) do
                        if sentinelInfo.sentinelInfos then
                            for _, child in ipairs(sentinelInfo.sentinelInfos) do
                                if child.text == TALENT_FRAME_DROP_DOWN_EXPORT_CLIPBOARD then
                                    return child
                                end
                            end
                        end
                    end
                end

                local clipboard = assert(find_export_clipboard(talents.LoadSystem), "export clipboard sentinel should exist")
                local disabledWithPoints = select(1, clipboard.disabledCallback())
                assert(disabledWithPoints == true, "hero points should disable export")

                local hadPoints, numPoints = C_ClassTalents.HasUnspentHeroTalentPoints()
                assert(hadPoints and numPoints == 1, "expected one remaining hero point before final spend")
                return disabledWithPoints
                "#,
            )
            .unwrap();

        env.state()
            .borrow_mut()
            .talents
            .currency_spent
            .insert(hero_currency_id, 11);

        let disabled_without_points: bool = env
            .eval(
                r#"
                local talents = assert(PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame, "talents frame should exist")

                local function find_export_clipboard(loadSystem)
                    for _, sentinelInfo in pairs(loadSystem.sentinelInfos) do
                        if sentinelInfo.sentinelInfos then
                            for _, child in ipairs(sentinelInfo.sentinelInfos) do
                                if child.text == TALENT_FRAME_DROP_DOWN_EXPORT_CLIPBOARD then
                                    return child
                                end
                            end
                        end
                    end
                end

                local clipboard = assert(find_export_clipboard(talents.LoadSystem), "export clipboard sentinel should exist")
                local disabledWithoutPoints = select(1, clipboard.disabledCallback())
                local hasPointsAfter, numPointsAfter = C_ClassTalents.HasUnspentHeroTalentPoints()
                assert(not hasPointsAfter and numPointsAfter == 0, "hero points should be exhausted after final spend")

                return disabledWithoutPoints
                "#,
            )
            .unwrap();

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__hero_export_gate_errors");
        assert!(
            recorded_errors.is_empty(),
            "Hero export gate regression produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );
        assert!(
            handler_errors.is_empty(),
            "Hero export gate regression produced {} Lua error(s):\n{}",
            handler_errors.len(),
            handler_errors.join("\n")
        );
        assert!(disabled_with_points, "hero points should disable export");
        assert!(
            !disabled_without_points,
            "export should re-enable once hero points are exhausted and class points are capped"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn player_spells_tiered_button_costs_aggregate_real_trait_data() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__tiered_button_cost_errors");
        clear_recorded_lua_errors(&env);

        open_talents(&env);
        run_extra_update_ticks(&env, 3);

        let result: String = env
            .eval(
                r#"
                local talents = assert(PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame, "talents frame should exist")
                local configID = assert(talents:GetConfigID(), "talents frame should have a config id")

                local tieredButton
                for talentButton in talents:EnumerateAllTalentButtons() do
                    local nodeInfo = talentButton:GetNodeInfo()
                    if nodeInfo and nodeInfo.type == Enum.TraitNodeType.Tiered then
                        tieredButton = talentButton
                        break
                    end
                end

                assert(tieredButton, "expected a tiered talent button")

                local function cost_map(costs)
                    local map = {}
                    if type(costs) ~= "table" then
                        return map
                    end
                    for _, cost in ipairs(costs) do
                        map[cost.ID] = (map[cost.ID] or 0) + cost.amount
                    end
                    return map
                end

                local nodeID = tieredButton:GetNodeID()
                local nodeCost = assert(C_Traits.GetNodeCost(configID, nodeID), "tiered button should have node cost data")
                local entryInfo = assert(tieredButton:GetEntryInfo(), "tiered button should have entry info")
                local combinedCost = assert(tieredButton:GetTraitCurrenciesCost(), "tiered button should have combined cost data")

                local expected = cost_map(nodeCost)
                for _, cost in ipairs(entryInfo.entryCost or {}) do
                    expected[cost.ID] = (expected[cost.ID] or 0) + cost.amount
                end

                local actual = cost_map(combinedCost)
                for id, amount in pairs(expected) do
                    assert(actual[id] == amount, string.format("combined cost mismatch for %s: expected %s got %s", id, amount, tostring(actual[id])))
                end
                for id, amount in pairs(actual) do
                    assert(expected[id] == amount, string.format("combined cost has unexpected entry %s=%s", id, amount))
                end

                return string.format("%d:%d:%d", nodeID, #nodeCost, #combinedCost)
                "#,
            )
            .unwrap();

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors = common::drain_string_table(&env, "__tiered_button_cost_errors");
        assert!(
            recorded_errors.is_empty(),
            "Tiered talent cost regression produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );
        assert!(
            handler_errors.is_empty(),
            "Tiered talent cost regression produced {} Lua error(s):\n{}",
            handler_errors.len(),
            handler_errors.join("\n")
        );
        assert!(
            result.split(':').count() == 3,
            "tiered cost regression should return node and cost counts: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn player_spells_view_loadout_uses_trait_tree_for_imported_spec() {
    test_timeout! {
        let env = setup_env();
        common::install_error_collector(&env, "__view_loadout_trait_tree_errors");
        clear_recorded_lua_errors(&env);

        open_talents(&env);

        let result: String = env
            .eval(
                r#"
                local talents = assert(PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame, "talents frame should exist")
                local exportString = talents:GetLoadoutExportString()
                assert(type(exportString) == "string" and exportString ~= "", "loadout export should be a non-empty string")

                local captured = {}
                local original = C_ClassTalents.ViewLoadout
                C_ClassTalents.ViewLoadout = function(loadoutEntryInfo, importText)
                    captured.entryCount = #loadoutEntryInfo
                    captured.importMatches = importText == exportString
                    return true
                end

                local ok, specID = talents:ViewLoadout(exportString, UnitLevel("player"))
                C_ClassTalents.ViewLoadout = original

                assert(ok == true, "ViewLoadout should succeed for the exported loadout")
                assert(specID == PlayerUtil.GetCurrentSpecID(), "ViewLoadout should return the imported spec id")
                assert(captured.importMatches, "ViewLoadout should pass the same import string through to C_ClassTalents.ViewLoadout")
                assert(captured.entryCount and captured.entryCount >= 0, "ViewLoadout should decode loadout entries using GetTraitTreeForSpec")

                return string.format("%d:%d", specID, captured.entryCount)
                "#,
            )
            .unwrap();

        let recorded_errors = recorded_lua_errors(&env);
        let handler_errors =
            common::drain_string_table(&env, "__view_loadout_trait_tree_errors");
        assert!(
            recorded_errors.is_empty(),
            "ViewLoadout trait-tree regression produced {} recorded Lua error(s):\n{:#?}\nhandler_errors:\n{}\n{}",
            recorded_errors.len(),
            recorded_errors,
            handler_errors.join("\n"),
            player_spells_panel_debug_snapshot(&env),
        );
        assert!(
            handler_errors.is_empty(),
            "ViewLoadout trait-tree regression produced {} Lua error(s):\n{}",
            handler_errors.len(),
            handler_errors.join("\n")
        );
        assert!(
            result.starts_with("66:"),
            "Protection export should round-trip through ViewLoadout using spec 66: {result}"
        );
    }
}
}
}
