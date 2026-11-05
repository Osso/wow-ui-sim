//! Integration tests for spellbook and talent panel keybinding details.

use crate::common;
#[path = "common/keybindings_panels_detail.rs"]
mod keybindings_panels_detail;
#[path = "common/token_ui_fixtures.rs"]
mod token_ui_fixtures;

use keybindings_panels_detail::{
    drain_test_errors, frame_is_shown, install_test_error_handler, setup_env,
};

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_trait_currency_headers_match_groups_and_refresh_spent_values() {
    use wow_ui_sim::c_api::trait_group_currency::{ConfigCurrencyInputs, CurrencyAmounts, RankedNode};
    use wow_ui_sim::c_api::trait_group_display::{GroupDisplayRow, SkillLineDisplay};
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);
        env.exec(r#"
            PlayerSpellsUtil.ToggleSpellBookFrame()
            assert(PlayerSpellsFrame:TrySetTab(PlayerSpellsUtil.FrameTabs.ClassTalents))
            local frame = PlayerSpellsFrame.TalentsFrame
            local source = debug.getinfo(frame.RefreshTreeHeaders, 'S').source:gsub('\\\\', '/')
            assert(source:find('Camelot/ClassTalents/Blizzard_ClassTalentsFrame.lua', 1, true), source)
        "#).unwrap();
        let (config_id, tree_id): (i32, i32) = env.eval("local f=PlayerSpellsFrame.TalentsFrame; return f:GetConfigID(), f:GetTalentTreeID()").unwrap();
        {
            let state = env.state();
            let mut sim = state.borrow_mut();
            let talents = &mut sim.talents.forever;
            talents.group_display.rows_by_tree.insert(tree_id, vec![
                GroupDisplayRow { group_id: 10, skill_line_id: 50, order_index: 1 },
                GroupDisplayRow { group_id: 20, skill_line_id: 51, order_index: 2 },
            ]);
            talents.group_display.skill_lines.insert(50, SkillLineDisplay { name: "First supplied".into(), icon: 135920 });
            talents.group_display.skill_lines.insert(51, SkillLineDisplay { name: "Second supplied".into(), icon: 135873 });
            let currency = &mut talents.group_currency;
            currency.group_trees.extend([(10, tree_id), (20, tree_id)]);
            currency.node_groups.insert(100, vec![10,20]);
            currency.tree_currencies.insert(tree_id, vec![80]);
            let mut config = ConfigCurrencyInputs {
                nodes: vec![RankedNode { node_id: 100, rank: 1 }],
                accumulator_order: vec![20,10],
                ..Default::default()
            };
            for (group, spent) in [(10,2), (20,14)] {
                config.amounts.insert((group,80), CurrencyAmounts { quantity: 30-spent, max_quantity: Some(30), spent, spent_in_tree: None });
            }
            currency.configs.insert(config_id, config);
        }
        env.exec(r#"
            local frame=PlayerSpellsFrame.TalentsFrame
            frame:RefreshTreeHeaders()
            assert(#frame.treeHeaders == 2)
            assert(frame.treeHeaders[1].Name:GetText() == 'First supplied')
            assert(frame.treeHeaders[2].Name:GetText() == 'Second supplied')
            assert(frame.treeHeaders[1].Text:GetText() == '2')
            assert(frame.treeHeaders[2].Text:GetText() == '14')
        "#).expect("original headers match currency groups, not outer array positions");
        {
            let state = env.state();
            let mut sim = state.borrow_mut();
            let config = sim.talents.forever.group_currency.configs.get_mut(&config_id).unwrap();
            for (group, spent) in [(10, 11), (20, 7)] {
                let amounts = config.amounts.get_mut(&(group, 80)).unwrap();
                amounts.spent = spent;
                amounts.quantity = 30 - spent;
            }
        }
        env.exec(r#"
            local frame=PlayerSpellsFrame.TalentsFrame
            frame:RefreshTreeHeaders()
            assert(frame.treeHeaders[1].Text:GetText() == '11')
            assert(frame.treeHeaders[2].Text:GetText() == '7')
        "#).expect("original headers must refresh changed supplied spent values");
        let errors=drain_test_errors(&env);
        assert!(errors.is_empty(), "original header flow errors: {}", errors.join("\n"));
    }
}

#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
#[test]
fn supplied_spellbook_rank_is_observed() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("create environment");
    env.exec(r#"
        A_Admin.SetSpellBookItemLowRank(6603, true)
        A_Admin.SetSpellBookItemLowRank(8690, false)
        assert(C_SpellBook.IsSpellBookItemLowRank(1, 0) == true)
        assert(C_SpellBook.IsSpellBookItemLowRank(2, 0) == false)
    "#).expect("supplied rank observations");
}

#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
#[test]
fn supplied_spec_selection_is_observed() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("create environment");
    env.exec(r#"
        A_Admin.SetSpecSelectionEnabled(true)
        assert(C_SpecializationInfo.IsSpecSelectionEnabled(2) == true)
        A_Admin.SetSpecSelectionEnabled(false)
        assert(C_SpecializationInfo.IsSpecSelectionEnabled(2) == false)
    "#).expect("supplied specialization observations");
}

#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
#[test]
fn spellbook_rank_accepts_untainted_secret_arguments() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("create environment");
    env.exec(r#"
        A_Admin.SetSpellBookItemLowRank(6603, true)
        local slot, bank = secretwrap(1), secretwrap(0)
        assert(C_SpellBook.IsSpellBookItemLowRank(slot, 0) == true)
        assert(C_SpellBook.IsSpellBookItemLowRank(1, bank) == true)
        local function addon_slot() return C_SpellBook.IsSpellBookItemLowRank(slot, 0) end
        local function addon_bank() return C_SpellBook.IsSpellBookItemLowRank(1, bank) end
        debug.setobjecttaint(addon_slot, 'RankFixture')
        debug.setobjecttaint(addon_bank, 'RankFixture')
        assert(not pcall(addon_slot))
        assert(not pcall(addon_bank))
    "#).expect("AllowedWhenUntainted spellbook arguments");
}

#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
#[test]
fn spec_selection_accepts_untainted_secret_argument() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("create environment");
    env.exec(r#"
        A_Admin.SetSpecSelectionEnabled(true)
        local class = secretwrap(2)
        assert(C_SpecializationInfo.IsSpecSelectionEnabled(class) == true)
        local function addon() return C_SpecializationInfo.IsSpecSelectionEnabled(class) end
        debug.setobjecttaint(addon, 'SpecFixture')
        assert(not pcall(addon))
    "#).expect("AllowedWhenUntainted specialization argument");
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_spellbook_filter_uses_supplied_rank() {
    let env = setup_env();
    env.send_key_press("S", None).expect("load spellbook through key binding");
    env.exec(r#"
        A_Admin.SetSpellBookItemLowRank(6603, true)
        assert(SpellBookFrameMixin.ShouldDisplaySpellBookItem(nil, false, false, true, true, 1, 0) == false)
        A_Admin.SetSpellBookItemLowRank(6603, false)
        local ok, result = pcall(SpellBookFrameMixin.ShouldDisplaySpellBookItem, nil, false, false, true, true, 1, 0)
        assert((ok and result == true) or (not ok and tostring(result):find("IsSpellBookItemLooseFlyoutMember", 1, true)),
            "non-low-rank item must reach the later flyout query")
    "#).expect("original spellbook filter consumes supplied rank");
}

#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
#[test]
fn flyout_membership_accepts_untainted_secret_arguments() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("create environment");
    env.state().borrow_mut().spellbook_loose_flyout_members.insert(6603);
    env.exec(r#"
        slot, bank = secretwrap(1), secretwrap(0)
        assert(C_SpellBook.IsSpellBookItemLooseFlyoutMember(slot, 0) == true)
        assert(C_SpellBook.IsSpellBookItemLooseFlyoutMember(1, bank) == true)
        function addon_slot() return C_SpellBook.IsSpellBookItemLooseFlyoutMember(slot, 0) end
        function addon_bank() return C_SpellBook.IsSpellBookItemLooseFlyoutMember(1, bank) end
        debug.setobjecttaint(addon_slot, 'FlyoutFixture')
        debug.setobjecttaint(addon_bank, 'FlyoutFixture')
    "#).expect("untainted secret arguments");
    assert!(env.exec("addon_slot()").is_err(), "tainted secret slot must fail");
    assert!(env.exec("addon_bank()").is_err(), "tainted secret bank must fail");
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_spellbook_filter_uses_supplied_flyout_membership() {
    let env = setup_env();
    env.send_key_press("S", None).expect("load spellbook");
    env.state().borrow_mut().spellbook_loose_flyout_members.insert(6603);
    env.exec(r#"
        assert(SpellBookFrameMixin.ShouldDisplaySpellBookItem(nil, false, false, false, false, 1, 0) == false)
        assert(SpellBookFrameMixin.ShouldDisplaySpellBookItem(nil, false, false, false, false, 2, 0) == true)
        assert(SpellBookFrameMixin.ShouldDisplaySpellBookItem(nil, false, false, false, true, 1, 0) == true)
    "#).expect("original spellbook filter consumes supplied flyout membership");
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_item_context_uses_supplied_target_spell() {
    let env = setup_env();
    env.state().borrow_mut().forever_startup.target_spell_id = Some(6991);
    env.exec("assert(ItemButtonUtil.GetItemContext() == ItemButtonUtil.ItemContextEnum.FeedPet)")
        .expect("feed pet context");
    env.state().borrow_mut().forever_startup.target_spell_id = Some(19750);
    env.exec("assert(ItemButtonUtil.GetItemContext() ~= ItemButtonUtil.ItemContextEnum.FeedPet)")
        .expect("other target context");
    env.state().borrow_mut().forever_startup.target_spell_id = None;
    env.exec("assert(C_Spell.GetTargetSpellID() == nil)")
        .expect("no supplied target");
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_spec_tab_uses_supplied_spec_selection() {
    let env = setup_env();
    env.send_key_press("S", None).expect("load player spells through key binding");
    env.exec(r#"
        A_Admin.SetSpecSelectionEnabled(true)
        assert(PlayerSpellsFrame:IsTabAvailable(PlayerSpellsFrame.specTabID) == true)
        A_Admin.SetSpecSelectionEnabled(false)
        assert(PlayerSpellsFrame:IsTabAvailable(PlayerSpellsFrame.specTabID) == false)
    "#).expect("original specialization tab consumes supplied state");
}

// These regressions use the ordered startup loader, not setup_env's injected
// ClickBinding bootstrap, so Escape runs against production panel bookkeeping.
post_wrath_ui_fixture! {
#[test]
fn specialization_escape_after_micro_button_and_tab_click_closes_panel() {
    test_timeout! {
        let env = setup_specialization_escape_env();
        env.exec(r#"
            local button = PlayerSpellsMicroButton
            assert(button and button:IsVisible(), "player-spells microbutton is visible")
            button:GetScript("OnClick")(button, "LeftButton", false)
            assert(PlayerSpellsFrame:IsShown(), "microbutton opens PlayerSpells")
            local tab = PlayerSpellsFrame:GetTabButton(PlayerSpellsFrame.specTabID)
            assert(tab and tab:IsVisible(), "specialization tab is visible")
            tab:GetScript("OnClick")(tab, "LeftButton", false)
        "#).expect("open specialization through actual button scripts");
        assert_specialization_escape_closes_panel(&env);
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn specialization_escape_after_direct_open_closes_panel() {
    test_timeout! {
        let env = setup_specialization_escape_env();
        env.exec("PlayerSpellsUtil.OpenToClassSpecializationsTab()")
            .expect("open specialization through the public helper");
        assert_specialization_escape_closes_panel(&env);
    }
}
}

post_wrath_ui_fixture! {
fn setup_specialization_escape_env() -> common::LockedEnv {
    common::lock_env(|| {
        use wow_ui_sim::screen::ScreenKind;
        let env = wow_ui_sim::lua_api::WowLuaEnv::new().expect("create specialization environment");
        env.set_screen_size(1600.0, 1200.0);
        let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("synced UI source");
        env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
        env.gc_stop();
        load_ordered_specialization_addons(&env, &ui);
        env.sync_string_metatable_to_global_string();
        env.sync_addon_names_to_lua();
        env.apply_post_load_workarounds();
        env.gc_restart_after_bootstrap()
            .expect("restart runtime GC");
        wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
        wow_ui_sim::startup::run_extra_update_ticks(&env, 10);
        env
    })
}

fn load_ordered_specialization_addons(env: &wow_ui_sim::lua_api::WowLuaEnv, ui: &std::path::Path) {
    use wow_ui_sim::loader::{
        StartupAddonLoadKind, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    };
    for addon in
        discover_blizzard_startup_addons_for_screen(ui, wow_ui_sim::screen::ScreenKind::Game)
    {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .unwrap_or_else(|error| panic!("load {}: {error}", addon.name));
        if addon.kind == StartupAddonLoadKind::Full {
            env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(&addon.name)])
                .expect("dispatch addon completion");
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
}

fn assert_specialization_escape_closes_panel(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    wow_ui_sim::startup::run_extra_update_ticks(env, 10);
    env.exec(
        r#"
        assert(PlayerSpellsFrame:IsShown(), "PlayerSpells remains open after update ticks")
        assert(PlayerSpellsFrame.SpecFrame:IsVisible(), "specialization content is visible")
        assert(not GameMenuFrame:IsShown(), "menu is initially closed")
        assert(GetUIPanel("left") == PlayerSpellsFrame or GetUIPanel("center") == PlayerSpellsFrame,
            "PlayerSpells is tracked as an active UIPanel")
    "#,
    )
    .expect("specialization state before keyboard input");
    env.send_key_press("ESCAPE", None)
        .expect("dispatch real Escape input");
    wow_ui_sim::startup::run_extra_update_ticks(env, 10);
    env.exec(
        r#"
        assert(not PlayerSpellsFrame:IsShown(), "Escape closes PlayerSpells")
        assert(not PlayerSpellsFrame.SpecFrame:IsVisible(), "specialization content is hidden")
        assert(not GameMenuFrame:IsShown(), "Escape must not also open GameMenu")
    "#,
    )
    .expect("specialization state after keyboard input");
    let state = env.state().borrow();
    assert!(
        state.lua_errors.is_empty(),
        "Lua errors: {:?}",
        state.lua_errors
    );
}
}

// ── S → Spellbook panel opens without errors ─────────────────────────────

post_wrath_ui_fixture! {
#[test]
fn keybind_s_opens_spellbook_no_errors() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("S", None).expect("S keybind dispatch failed");

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Opening spellbook produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert!(
            frame_is_shown(&env, "PlayerSpellsFrame"),
            "PlayerSpellsFrame should be shown after pressing S"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn keybind_s_opens_spellbook_tab_on_first_press() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("S", None).expect("S keybind dispatch failed");

        let result: String = env
            .eval(
                r#"
                if not (PlayerSpellsFrame and PlayerSpellsFrame:IsShown()) then
                    return "player_spells_not_shown"
                end
                if not (PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame:IsShown()) then
                    return "spellbook_tab_not_shown"
                end
                return "ok"
                "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Opening spellbook through S produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Pressing S should show the spellbook tab on the first open: {result}"
        );
    }
}
}

#[test]
fn keybind_s_is_a_single_thin_dispatch_to_toggle_spellbook_frame() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.exec(
            r#"
            if not (PlayerSpellsUtil and type(PlayerSpellsUtil.ToggleSpellBookFrame) == "function") then
                error("missing_toggle_spellbook")
            end

            original_toggle_spellbook_frame = PlayerSpellsUtil.ToggleSpellBookFrame
            spellbook_toggle_calls = 0

            PlayerSpellsUtil.ToggleSpellBookFrame = function(...)
                spellbook_toggle_calls = spellbook_toggle_calls + 1
                return false
            end
            "#,
        )
        .unwrap();

        env.send_key_press("S", None)
            .expect("S keybind dispatch failed");

        let result: (i32, bool, bool) = env
            .eval(
                r#"
                return
                    spellbook_toggle_calls or 0,
                    PlayerSpellsFrame and PlayerSpellsFrame:IsShown() == true or false,
                    PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame:IsShown() == true or false
                "#,
            )
            .unwrap();
        env.exec(
            r#"
            if original_toggle_spellbook_frame ~= nil then
                PlayerSpellsUtil.ToggleSpellBookFrame = original_toggle_spellbook_frame
            end
            "#,
        )
        .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Spellbook keybind fallback regression produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            (1, false, false),
            "Spellbook keybind should be a single dispatch into ToggleSpellBookFrame without force-show fallback"
        );
    }
}

#[test]
fn keybind_s_dispatches_directly_to_playerspellsutil_toggle_spellbook_frame() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.exec(
            r#"
            if not (PlayerSpellsUtil and type(PlayerSpellsUtil.ToggleSpellBookFrame) == "function") then
                error("missing_toggle_spellbook")
            end
            if type(ToggleSpellBook) ~= "function" then
                error("missing_legacy_toggle_spellbook")
            end

            original_toggle_spellbook_frame = PlayerSpellsUtil.ToggleSpellBookFrame
            original_toggle_spellbook = ToggleSpellBook
            spellbook_toggle_frame_calls = 0
            legacy_toggle_spellbook_calls = 0

            PlayerSpellsUtil.ToggleSpellBookFrame = function(...)
                spellbook_toggle_frame_calls = spellbook_toggle_frame_calls + 1
                return false
            end

            ToggleSpellBook = function(...)
                legacy_toggle_spellbook_calls = legacy_toggle_spellbook_calls + 1
                return false
            end
            "#,
        )
        .unwrap();

        env.send_key_press("S", None)
            .expect("S keybind dispatch failed");

        let result: (i32, i32) = env
            .eval(
                r#"
                return
                    spellbook_toggle_frame_calls or 0,
                    legacy_toggle_spellbook_calls or 0
                "#,
            )
            .unwrap();
        env.exec(
            r#"
            if original_toggle_spellbook_frame ~= nil then
                PlayerSpellsUtil.ToggleSpellBookFrame = original_toggle_spellbook_frame
            end
            if original_toggle_spellbook ~= nil then
                ToggleSpellBook = original_toggle_spellbook
            end
            "#,
        )
        .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Direct spellbook keybind dispatch produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            (1, 0),
            "Spellbook keybind should dispatch directly into PlayerSpellsUtil.ToggleSpellBookFrame, not legacy ToggleSpellBook"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn keybind_s_toggles_spellbook_closed_on_second_press() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("S", None).expect("first S keybind dispatch failed");
        assert!(
            frame_is_shown(&env, "PlayerSpellsFrame"),
            "PlayerSpellsFrame should be shown after the first S press"
        );

        env.send_key_press("S", None).expect("second S keybind dispatch failed");

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Toggling spellbook through S produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert!(
            !frame_is_shown(&env, "PlayerSpellsFrame"),
            "PlayerSpellsFrame should be hidden after pressing S twice"
        );
    }
}
}

poi_button_panel_or_wrath_fixture! {
#[test]
fn spellbook_panel_spell_tooltip_has_lines_after_tab_switch_and_closes_without_errors() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        let result: String = env
            .eval(
                r#"
                if not (PlayerSpellsUtil and type(PlayerSpellsUtil.ToggleSpellBookFrame) == "function") then
                    return "missing_toggle_spellbook"
                end

                if not (PlayerSpellsUtil.FrameTabs and PlayerSpellsUtil.FrameTabs.ClassTalents and PlayerSpellsUtil.FrameTabs.SpellBook) then
                    return "missing_frame_tabs"
                end

                PlayerSpellsUtil.ToggleSpellBookFrame()

                if not (PlayerSpellsFrame and PlayerSpellsFrame:IsShown()) then
                    return "spellbook_not_open"
                end

                if not PlayerSpellsFrame:TrySetTab(PlayerSpellsUtil.FrameTabs.ClassTalents) then
                    return "talent_tab_unavailable"
                end

                if not PlayerSpellsFrame:IsFrameTabActive(PlayerSpellsUtil.FrameTabs.ClassTalents) then
                    return "talent_tab_not_selected"
                end

                if not PlayerSpellsFrame:TrySetTab(PlayerSpellsUtil.FrameTabs.SpellBook) then
                    return "spellbook_tab_unavailable"
                end

                if not PlayerSpellsFrame:IsFrameTabActive(PlayerSpellsUtil.FrameTabs.SpellBook) then
                    return "spellbook_tab_not_selected"
                end

                local hasSpell = GameTooltip:SetSpellBookItem(1)
                if not hasSpell then
                    return "no_spellbook_item"
                end

                local info = GameTooltip:GetPrimaryTooltipInfo()
                local tooltipData = GameTooltip:GetPrimaryTooltipData()
                if not info
                    or not tooltipData
                    or not tooltipData.lines
                    or not tooltipData.lines[1]
                then
                    return "tooltip_has_no_lines"
                end

                PlayerSpellsUtil.ToggleSpellBookFrame()

                if PlayerSpellsFrame:IsShown() then
                    return "spellbook_not_closed"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Spellbook tooltip flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Spellbook panel tooltip flow should open, switch tabs, populate tooltip lines, and close: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn spellbook_first_visible_item_icon_matches_spellbook_texture() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("S", None).expect("S keybind dispatch failed");

        let result: String = env
            .eval(
                r#"
                local paged = PlayerSpellsFrame and PlayerSpellsFrame.SpellBookFrame and PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame
                if not paged then
                    return "missing_paged_spells_frame"
                end

                for _, frame in paged:EnumerateFrames() do
                    if frame
                        and frame:IsShown()
                        and frame.HasValidData
                        and frame:HasValidData()
                        and frame.slotIndex
                        and frame.spellBank
                        and frame.Button
                        and frame.Button.Icon
                    then
                        local expected = C_SpellBook.GetSpellBookItemTexture(frame.slotIndex, frame.spellBank)
                        local actual = frame.Button.Icon:GetTexture()
                        if actual ~= expected then
                            return string.format(
                                "icon_mismatch_slot_%s_expected_%s_actual_%s",
                                tostring(frame.slotIndex),
                                tostring(expected),
                                tostring(actual)
                            )
                        end
                        return "ok"
                    end
                end

                return "no_visible_spellbook_item"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Spellbook icon regression produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "The first visible spellbook item icon should match C_SpellBook.GetSpellBookItemTexture for its slot: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn spellbook_paging_label_is_formatted_on_first_open() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("S", None).expect("S keybind dispatch failed");

        let result: String = env
            .eval(
                r#"
                local pagingControls = PlayerSpellsFrame
                    and PlayerSpellsFrame.SpellBookFrame
                    and PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame
                    and PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame.PagingControls
                if not pagingControls then
                    return "missing_paging_controls"
                end

                local text = pagingControls.PageText and pagingControls.PageText:GetText()
                if not text then
                    return "missing_page_text"
                end
                if text:find("%%d") then
                    return "unformatted_page_text_" .. text
                end
                if not text:match("^Page %d+/%d+$") then
                    return "unexpected_page_text_" .. text
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Spellbook paging label regression produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Spellbook paging controls should render a formatted page label, not a literal format string: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn talent_panel_switches_spec_tabs_and_closes_without_errors() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        let result: String = env
            .eval(
                r#"
                if not (PlayerSpellsUtil and type(PlayerSpellsUtil.ToggleClassTalentFrame) == "function") then
                    return "missing_toggle_class_talent_frame"
                end

                if not (PlayerSpellsUtil.FrameTabs and PlayerSpellsUtil.FrameTabs.ClassSpecializations and PlayerSpellsUtil.FrameTabs.ClassTalents) then
                    return "missing_frame_tabs"
                end

                PlayerSpellsUtil.ToggleClassTalentFrame()

                if not (PlayerSpellsFrame and PlayerSpellsFrame:IsShown()) then
                    return "talent_panel_not_open"
                end

                if not PlayerSpellsFrame:IsFrameTabActive(PlayerSpellsUtil.FrameTabs.ClassTalents) then
                    return "talent_tab_not_initial"
                end

                if not PlayerSpellsFrame:TrySetTab(PlayerSpellsUtil.FrameTabs.ClassSpecializations) then
                    return "spec_tab_unavailable"
                end

                if not PlayerSpellsFrame:IsFrameTabActive(PlayerSpellsUtil.FrameTabs.ClassSpecializations) then
                    return "spec_tab_not_selected"
                end

                if not PlayerSpellsFrame:TrySetTab(PlayerSpellsUtil.FrameTabs.ClassTalents) then
                    return "talent_tab_unavailable"
                end

                if not PlayerSpellsFrame:IsFrameTabActive(PlayerSpellsUtil.FrameTabs.ClassTalents) then
                    return "talent_tab_not_reselected"
                end

                PlayerSpellsUtil.ToggleClassTalentFrame()

                if PlayerSpellsFrame:IsShown() then
                    return "talent_panel_not_closed"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Talent panel tab-switch flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Talent panel flow should open, switch to spec tab, switch back, and close: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn talent_panel_has_at_least_one_visible_talent_node_frame() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        let result: String = env
            .eval(
                r#"
                if not (PlayerSpellsUtil and type(PlayerSpellsUtil.ToggleClassTalentFrame) == "function") then
                    return "missing_toggle_class_talent_frame"
                end

                PlayerSpellsUtil.ToggleClassTalentFrame()

                if not (PlayerSpellsFrame and PlayerSpellsFrame:IsShown()) then
                    return "talent_panel_not_open"
                end
                if not (PlayerSpellsFrame.TalentsFrame and PlayerSpellsFrame.TalentsFrame:IsShown()) then
                    return "talents_frame_not_shown"
                end

                local totalButtons = 0
                local visibleButtons = 0
                for talentButton in PlayerSpellsFrame.TalentsFrame:EnumerateAllTalentButtons() do
                    totalButtons = totalButtons + 1
                    if talentButton and talentButton:IsShown() then
                        visibleButtons = visibleButtons + 1
                    end
                end

                if totalButtons == 0 then
                    return "no_talent_buttons"
                end
                if visibleButtons == 0 then
                    return "no_visible_talent_buttons"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Talent panel flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Talent panel should expose at least one visible active talent button frame: {result}"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn talent_panel_hero_spec_button_uses_protection_hero_icon_atlases() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        let result: String = env
            .eval(
                r#"
                if not (PlayerSpellsUtil and type(PlayerSpellsUtil.ToggleClassTalentFrame) == "function") then
                    return "missing_toggle_class_talent_frame"
                end

                PlayerSpellsUtil.ToggleClassTalentFrame()

                local talentsFrame = PlayerSpellsFrame and PlayerSpellsFrame.TalentsFrame
                if not (talentsFrame and talentsFrame:IsShown()) then
                    return "talents_frame_not_shown"
                end

                local heroButton = talentsFrame.HeroTalentsContainer and talentsFrame.HeroTalentsContainer.HeroSpecButton
                if not (heroButton and heroButton:IsShown()) then
                    return "hero_button_not_shown"
                end

                local icon1Atlas = heroButton.Icon1 and heroButton.Icon1:GetAtlas()
                if icon1Atlas ~= "talents-heroclass-paladin-lightsmith" then
                    return "bad_icon1:" .. tostring(icon1Atlas)
                end

                heroButton:Click()
                if not (HeroTalentsSelectionDialog and HeroTalentsSelectionDialog:IsShown()) then
                    return "selection_dialog_not_shown"
                end

                local seen = {}
                for _, specFrame in pairs(HeroTalentsSelectionDialog.specFramesBySubTreeID or {}) do
                    local atlas = specFrame.SpecImage and specFrame.SpecImage:GetAtlas()
                    if atlas then
                        seen[atlas] = true
                    end
                end

                if not seen["talents-heroclass-paladin-lightsmith"] then
                    return "missing_lightsmith_dialog_icon"
                end
                if not seen["talents-heroclass-paladin-templar"] then
                    return "missing_templar_dialog_icon"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Talent panel hero icon flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Protection hero talent picker should render atlas-backed hero icons: {result}"
        );
    }
}
}
