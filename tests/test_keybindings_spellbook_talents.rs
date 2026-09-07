//! Integration tests for spellbook and talent panel keybinding details.

use crate::common;
#[path = "common/keybindings_panels_detail.rs"]
mod keybindings_panels_detail;
#[path = "common/token_ui_fixtures.rs"]
mod token_ui_fixtures;

use keybindings_panels_detail::{
    drain_test_errors, frame_is_shown, install_test_error_handler, setup_env,
};

// These regressions use the ordered startup loader, not setup_env's injected
// ClickBinding bootstrap, so Escape runs against production panel bookkeeping.
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

#[test]
fn specialization_escape_after_direct_open_closes_panel() {
    test_timeout! {
        let env = setup_specialization_escape_env();
        env.exec("PlayerSpellsUtil.OpenToClassSpecializationsTab()")
            .expect("open specialization through the public helper");
        assert_specialization_escape_closes_panel(&env);
    }
}

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

// ── S → Spellbook panel opens without errors ─────────────────────────────

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
