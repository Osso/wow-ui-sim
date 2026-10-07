//! Integration tests for keybinding dispatch against the real Blizzard UI.
//!
//! Loads the full Blizzard addon set, fires startup events, then presses each
//! default keybind and verifies the corresponding panel frame is shown.
//!
//! These tests exercise the real Blizzard toggle functions (ToggleAllBags,
//! ToggleCharacter, etc.) — not stubs. Failures surface real missing APIs,
//! nil widget errors, and broken on-demand addon loads.
//!
//! Panel interaction tests (spellbook, talents, world map, escape menu, etc.)
//! are in `test_keybindings_panels.rs`.
//!
//! Targeting tests (TargetFrame, F2–F6) are in `test_keybindings_targeting.rs`.

use crate::common;
#[path = "common/token_ui_fixtures.rs"]
mod token_ui_fixtures;

use std::path::PathBuf;
use token_ui_fixtures::load_token_ui;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::global_frames;
use wow_ui_sim::startup::{fire_one_on_update_tick, process_pending_timers};

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Blizzard addons in dependency order (same as micro_menu.rs).
const BLIZZARD_ADDONS: &[(&str, &str)] = &[
    ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
    ("Blizzard_Colors", "Blizzard_Colors.toc"),
    ("Blizzard_SharedXML", "Blizzard_SharedXML.toc"),
    ("Blizzard_SharedXMLGame", "Blizzard_SharedXMLGame.toc"),
    (
        "Blizzard_UIPanelTemplates",
        "Blizzard_UIPanelTemplates_Mainline.toc",
    ),
    (
        "Blizzard_FrameXMLBase",
        "Blizzard_FrameXMLBase_Mainline.toc",
    ),
    ("Blizzard_FrameEffects", "Blizzard_FrameEffects.toc"),
    ("Blizzard_LoadLocale", "Blizzard_LoadLocale.toc"),
    ("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc"),
    ("Blizzard_HelpPlate", "Blizzard_HelpPlate.toc"),
    (
        "Blizzard_AccessibilityTemplates",
        "Blizzard_AccessibilityTemplates.toc",
    ),
    ("Blizzard_ObjectAPI", "Blizzard_ObjectAPI_Mainline.toc"),
    ("Blizzard_UIParent", "Blizzard_UIParent.toc"),
    ("Blizzard_TextStatusBar", "Blizzard_TextStatusBar.toc"),
    ("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Mainline.toc"),
    ("Blizzard_POIButton", "Blizzard_POIButton.toc"),
    ("Blizzard_Flyout", "Blizzard_Flyout.toc"),
    ("Blizzard_GameMenuEsc", "Blizzard_GameMenuEsc.toc"),
    ("Blizzard_Communities", "Blizzard_Communities_Mainline.toc"),
    ("Blizzard_StoreUI", "Blizzard_StoreUI.toc"),
    ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
    ("Blizzard_ManagedFrameSystem", "Blizzard_ManagedFrameSystem_Mainline.toc"),
    ("Blizzard_EditMode", "Blizzard_EditMode.toc"),
    ("Blizzard_GarrisonBase", "Blizzard_GarrisonBase.toc"),
    ("Blizzard_GameTooltip", "Blizzard_GameTooltip_Mainline.toc"),
    ("Blizzard_StaticPopup_Game", "Blizzard_StaticPopup_Game.toc"),
    ("Blizzard_TransmogShared", "Blizzard_TransmogShared.toc"),
    ("Blizzard_ActionBar", "Blizzard_ActionBar_Mainline.toc"),
    ("Blizzard_UnitFrame", "Blizzard_UnitFrame_Mainline.toc"),
    (
        "Blizzard_UIParentPanelManager",
        "Blizzard_UIParentPanelManager_Mainline.toc",
    ),
    (
        "Blizzard_Settings_Shared",
        "Blizzard_Settings_Shared.toc",
    ),
    (
        "Blizzard_SettingsDefinitions_Shared",
        "Blizzard_SettingsDefinitions_Shared.toc",
    ),
    (
        "Blizzard_SettingsDefinitions_Frame",
        "Blizzard_SettingsDefinitions_Frame.toc",
    ),
    ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil.toc"),
    ("Blizzard_ItemButton", "Blizzard_ItemButton_Mainline.toc"),
    ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ("Blizzard_FrameXML", "Blizzard_FrameXML.toc"),
    (
        "Blizzard_UIPanels_Game",
        "Blizzard_UIPanels_Game_Mainline.toc",
    ),
    ("Blizzard_TokenUI", "Blizzard_TokenUI.toc"),
    (
        "Blizzard_MainMenuBarBagButtons",
        "Blizzard_MainMenuBarBagButtons_Mainline.toc",
    ),
];

/// Create a fully loaded environment with Blizzard addons and startup events.
fn setup_env() -> common::LockedEnv {
    setup_locked_env(false)
}

fn setup_locked_env(settle_after_startup: bool) -> common::LockedEnv {
    common::lock_env(|| {
        let env = setup_unsettled_env();
        if settle_after_startup {
            settle_env(&env);
        }
        env
    })
}

fn setup_unsettled_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    seed_addon_search_paths(&env);
    load_base_blizzard_addons(&env);
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

fn seed_addon_search_paths(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.addon_base_paths = vec![blizzard_ui_dir()];
}

fn load_base_blizzard_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for (name, toc) in BLIZZARD_ADDONS {
        let toc_path = ui.join(name).join(toc);
        if !toc_path.exists() {
            continue;
        }
        if let Err(e) = load_addon(&env.loader_env(), &toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        } else {
            env.apply_runtime_addon_load_workarounds(name);
        }
    }
}

fn settle_env(env: &WowLuaEnv) {
    env.apply_post_event_workarounds();
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    let _ = global_frames::hide_runtime_hidden_frames(&*env.rilua());
}

fn prepare_bag_env(env: &WowLuaEnv) {
    settle_env(env);
    load_token_ui(env);
    settle_env(env);
}

/// Create a fully loaded environment with Blizzard addons and startup events.
fn setup_settled_env() -> common::LockedEnv {
    setup_locked_env(true)
}

/// Fire startup events (same sequence as main.rs).
fn fire_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::call_global_if_present(env, "RequestTimePlayed");
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
        "UPDATE_CHAT_WINDOWS",
    ] {
        let _ = env.fire_event(event);
    }
}

/// Check whether a global frame exists and is shown.
fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil and {frame_name}:IsShown() == true");
    env.eval::<bool>(&code).unwrap_or(false)
}

fn visible_bag_frames_debug(env: &WowLuaEnv) -> String {
    env.eval::<String>(
        r#"
        local parts = {}
        local arrayParts = {}
        local function add(name)
            local frame = _G[name]
            if not frame then
                return
            end
            local shown = frame.IsShown and frame:IsShown()
            local bagID = frame.GetBagID and frame:GetBagID()
            table.insert(parts, string.format("%s(shown=%s, bagID=%s)", name, tostring(shown), tostring(bagID)))
        end

        add("ContainerFrameCombinedBags")
        for i = 1, 6 do
            add("ContainerFrame" .. i)
        end
        local container = _G.ContainerFrameContainer
        local frames = container and container.ContainerFrames
        if type(frames) == "table" then
            for i, frame in ipairs(frames) do
                local name = frame and frame.GetName and frame:GetName() or tostring(frame)
                table.insert(arrayParts, string.format("%s=%s", tostring(i), tostring(name)))
            end
        end
        return string.format("%s | array=[%s]", table.concat(parts, ", "), table.concat(arrayParts, ", "))
    "#,
    )
    .unwrap_or_else(|_| "<bag frame introspection failed>".to_string())
}

fn bag_id_is_shown(env: &WowLuaEnv, bag_id: i32) -> bool {
    env.eval::<bool>(&format!(
        r#"
        if ContainerFrameCombinedBags and ContainerFrameCombinedBags:IsShown() then
            return true
        end
        for i = 1, 6 do
            local frame = _G["ContainerFrame" .. i]
            if frame and frame:IsShown() and frame.GetBagID and frame:GetBagID() == {bag_id} then
                return true
            end
        end
        return false
    "#
    ))
    .unwrap_or(false)
}

/// Install a Lua error handler that collects errors into `__test_errors`.
fn install_test_error_handler(env: &WowLuaEnv) {
    common::install_error_collector(env, "__test_errors");
}

/// Read collected errors from `__test_errors` and clear it.
fn drain_test_errors(env: &WowLuaEnv) -> Vec<String> {
    common::drain_string_table(env, "__test_errors")
}

fn clear_recorded_lua_errors(env: &WowLuaEnv) {
    common::panel_fixtures::clear_recorded_lua_errors(env);
}

fn assert_no_backpack_open_errors(env: &WowLuaEnv, context: &str) {
    let recorded_errors = common::panel_fixtures::recorded_lua_errors(env);
    let handler_errors = drain_test_errors(env);
    assert!(
        recorded_errors.is_empty(),
        "{context} produced {} recorded Lua error(s):\n{}\nhandler errors:\n{}",
        recorded_errors.len(),
        recorded_errors.join("\n"),
        handler_errors.join("\n"),
    );
    assert!(
        handler_errors.is_empty(),
        "{context} produced {} Lua error(s):\n{}",
        handler_errors.len(),
        handler_errors.join("\n"),
    );
}

// ── B → ToggleAllBags() ─────────────────────────────────────────────────

#[test]
fn keybind_b_opens_bags() {
    test_timeout! {
        let env = setup_settled_env();
        load_token_ui(&env);
        settle_env(&env);
        env.send_key_press("B", None).expect("B keybind failed");
        assert!(
            env.eval::<bool>(
                r#"
                if ContainerFrameCombinedBags and ContainerFrameCombinedBags:IsShown() then
                    return true
                end
                for i = 1, 6 do
                    local frame = _G["ContainerFrame" .. i]
                    if frame and frame:IsShown() then
                        return true
                    end
                end
                return false
            "#,
            )
            .unwrap_or(false),
            "A bag frame should be visible after pressing B; {}",
            visible_bag_frames_debug(&env)
        );
    }
}

// ── BACKSPACE → ToggleBackpack() ────────────────────────────────────────

#[test]
fn keybind_backspace_opens_backpack() {
    test_timeout! {
        let env = setup_settled_env();
        prepare_bag_env(&env);
        install_test_error_handler(&env);
        clear_recorded_lua_errors(&env);
        env.send_key_press("BACKSPACE", None).expect("BACKSPACE keybind failed");
        assert_no_backpack_open_errors(&env, "BACKSPACE backpack open");
        assert!(
            bag_id_is_shown(&env, 0),
            "Backpack should be visible after pressing BACKSPACE; {}",
            visible_bag_frames_debug(&env)
        );
    }
}

#[test]
fn backpack_button_click_opens_backpack() {
    test_timeout! {
        let env = setup_settled_env();
        prepare_bag_env(&env);
        let backpack_id = env
            .state()
            .borrow()
            .widgets
            .get_id_by_name("MainMenuBarBackpackButton")
            .expect("MainMenuBarBackpackButton should exist");
        env.send_click(backpack_id)
            .expect("MainMenuBarBackpackButton click failed");
        assert!(
            bag_id_is_shown(&env, 0),
            "Backpack should be visible after clicking MainMenuBarBackpackButton; {}",
            visible_bag_frames_debug(&env)
        );
    }
}

// ── F8 → ToggleBag(4) ──────────────────────────────────────────────

#[test]
fn keybind_f8_opens_bag4() {
    test_timeout! {
        let env = setup_settled_env();
        prepare_bag_env(&env);
        env.send_key_press("F8", None).expect("F8 keybind failed");
        assert!(
            bag_id_is_shown(&env, 4),
            "A bag frame should be visible after pressing F8; {}",
            visible_bag_frames_debug(&env)
        );
    }
}

// ── F9 → ToggleBag(3) ──────────────────────────────────────────────

#[test]
fn keybind_f9_opens_bag3() {
    test_timeout! {
        let env = setup_settled_env();
        prepare_bag_env(&env);
        env.send_key_press("F9", None).expect("F9 keybind failed");
        assert!(
            bag_id_is_shown(&env, 3),
            "A bag frame should be visible after pressing F9; {}",
            visible_bag_frames_debug(&env)
        );
    }
}

// ── F10 → ToggleBag(2) ─────────────────────────────────────────────

#[test]
fn keybind_f10_opens_bag2() {
    test_timeout! {
        let env = setup_settled_env();
        prepare_bag_env(&env);
        env.send_key_press("F10", None).expect("F10 keybind failed");
        assert!(
            bag_id_is_shown(&env, 2),
            "A bag frame should be visible after pressing F10; {}",
            visible_bag_frames_debug(&env)
        );
    }
}

// ── F11 → ToggleBag(1) ─────────────────────────────────────────────

#[test]
fn keybind_f11_opens_bag1() {
    test_timeout! {
        let env = setup_settled_env();
        prepare_bag_env(&env);
        env.send_key_press("F11", None).expect("F11 keybind failed");
        assert!(
            bag_id_is_shown(&env, 1),
            "A bag frame should be visible after pressing F11; {}",
            visible_bag_frames_debug(&env)
        );
    }
}

// ── C → ToggleCharacter("PaperDollFrame") ───────────────────────────────

#[test]
fn keybind_c_opens_character() {
    test_timeout! {
        let env = setup_env();
        env.send_key_press("C", None).expect("C keybind failed");
        assert!(
            frame_is_shown(&env, "CharacterFrame"),
            "CharacterFrame should be shown after pressing C"
        );
    }
}

#[test]
fn keybind_c_populates_character_panel_surface() {
    test_timeout! {
        let env = setup_env();
        env.send_key_press("C", None).expect("C keybind failed");

        let result: String = env
            .eval(
                r#"
                if not (CharacterFrame and CharacterFrame:IsShown()) then
                    return "character_not_open"
                end

                if not CharacterFrame.TitleContainer then
                    return "missing_title_container"
                end

                if not CharacterFrame.TitleContainer.TitleText then
                    return "missing_title_text"
                end

                local expectedTitle = UnitPVPName("player")
                local actualTitle = CharacterFrame.TitleContainer.TitleText:GetText()
                if actualTitle ~= expectedTitle then
                    return string.format(
                        "title_mismatch_expected_%s_actual_%s",
                        tostring(expectedTitle),
                        tostring(actualTitle)
                    )
                end

                local slotNames = {
                    "CharacterHeadSlot",
                    "CharacterChestSlot",
                    "CharacterMainHandSlot",
                }

                for _, frameName in ipairs(slotNames) do
                    local slot = _G[frameName]
                    if not slot then
                        return "missing_slot_" .. frameName
                    end
                    if not slot.icon then
                        return "missing_icon_" .. frameName
                    end

                    local expectedTexture = GetInventoryItemTexture("player", slot:GetID())
                    if expectedTexture == nil then
                        expectedTexture = slot.backgroundTextureName
                    end
                    local actualTexture = slot.icon:GetTexture()

                    if actualTexture ~= expectedTexture then
                        return string.format(
                            "slot_texture_mismatch_%s_expected_%s_actual_%s",
                            frameName,
                            tostring(expectedTexture),
                            tostring(actualTexture)
                        )
                    end
                end

                return "ok"
            "#,
            )
            .unwrap();

        assert_eq!(
            result,
            "ok",
            "Character panel should populate title text and representative slot icons on first open: {result}"
        );
    }
}

#[test]
fn game_screen_runtime_loadaddon_rejects_glueparent() {
    test_timeout! {
        let env = setup_env();
        let result: String = env
            .eval(
                r#"
                local before = UIParent and UIParent.GetName and UIParent:GetName() or tostring(UIParent)
                local loaded, reason = C_AddOns.LoadAddOn("Blizzard_GlueParent")
                local after = UIParent and UIParent.GetName and UIParent:GetName() or tostring(UIParent)
                return string.format(
                    "loaded=%s reason=%s before=%s after=%s isLoaded=%s",
                    tostring(loaded),
                    tostring(reason),
                    tostring(before),
                    tostring(after),
                    tostring(C_AddOns and C_AddOns.IsAddOnLoaded and select(2, C_AddOns.IsAddOnLoaded("Blizzard_GlueParent")))
                )
            "#,
            )
            .unwrap();

        assert!(
            result.contains("loaded=false"),
            "GlueParent should not runtime-load on the game screen: {result}"
        );
        assert!(
            result.contains("isLoaded=false"),
            "GlueParent must remain unloaded on the game screen: {result}"
        );
        assert!(
            result.contains("before=UIParent") && result.contains("after=UIParent"),
            "UIParent should stay bound to UIParent when GlueParent load is rejected: {result}"
        );
    }
}

#[test]
fn keybind_c_toggles_character_without_errors() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        env.send_key_press("C", None).expect("first C keybind failed");

        let open_errors = drain_test_errors(&env);
        assert!(
            open_errors.is_empty(),
            "Opening character panel produced {} Lua error(s):\n{}",
            open_errors.len(),
            open_errors.join("\n"),
        );
        assert!(
            frame_is_shown(&env, "CharacterFrame"),
            "CharacterFrame should be shown after first C press"
        );

        env.send_key_press("C", None).expect("second C keybind failed");

        let close_errors = drain_test_errors(&env);
        assert!(
            close_errors.is_empty(),
            "Closing character panel produced {} Lua error(s):\n{}",
            close_errors.len(),
            close_errors.join("\n"),
        );
        assert!(
            !frame_is_shown(&env, "CharacterFrame"),
            "CharacterFrame should be hidden after second C press"
        );
    }
}

#[test]
fn character_panel_inventory_tooltip_has_lines_and_closes_without_errors() {
    test_timeout! {
        let env = setup_env();
        install_test_error_handler(&env);

        let result: String = env
            .eval(
                r#"
                if type(ToggleCharacter) ~= "function" then
                    return "missing_toggle_character"
                end

                ToggleCharacter("PaperDollFrame")

                if not (CharacterFrame and CharacterFrame:IsShown()) then
                    return "character_not_open"
                end

                local hasItem = GameTooltip:SetInventoryItem("player", 1)
                if not hasItem then
                    return "no_inventory_item"
                end

                local info = GameTooltip:GetPrimaryTooltipInfo()
                local tooltipData = GameTooltip:GetPrimaryTooltipData()
                if not info
                    or not tooltipData
                    or tooltipData.type ~= Enum.TooltipDataType.Item
                    or not tooltipData.lines
                    or not tooltipData.lines[1]
                then
                    return "tooltip_has_no_lines"
                end

                ToggleCharacter("PaperDollFrame")

                if CharacterFrame and CharacterFrame:IsShown() then
                    return "character_not_closed"
                end

                return "ok"
            "#,
            )
            .unwrap();

        let errors = drain_test_errors(&env);
        assert!(
            errors.is_empty(),
            "Character panel inventory tooltip flow produced {} Lua error(s):\n{}",
            errors.len(),
            errors.join("\n"),
        );
        assert_eq!(
            result,
            "ok",
            "Character panel inventory tooltip flow should open, populate tooltip lines, and close: {result}"
        );
    }
}

// ── U → ToggleCharacter("ReputationFrame") ──────────────────────────────

#[test]
fn keybind_u_opens_reputation() {
    test_timeout! {
        let env = setup_env();
        env.send_key_press("U", None).expect("U keybind failed");
        assert!(
            frame_is_shown(&env, "CharacterFrame"),
            "CharacterFrame should be shown after pressing U (reputation tab)"
        );
    }
}

#[test]
fn reputation_surface_exists_for_keybind_setup() {
    test_timeout! {
        let env = WowLuaEnv::new().expect("Failed to create Lua environment");

        let result: String = env
            .eval(
                r#"
                if not (C_Reputation and C_Reputation.GetFactionDataByIndex) then
                    return "missing_c_reputation_surface"
                end
                return "ok"
            "#,
            )
            .unwrap();
        assert_eq!(
            result,
            "ok",
            "Reputation data should expose the C_Reputation surface: {result}"
        );
    }
}
