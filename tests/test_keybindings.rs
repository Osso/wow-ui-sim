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
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::global_frames;
use wow_ui_sim::startup::{fire_one_on_update_tick, process_pending_timers};

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

/// Blizzard addons in dependency order (same as micro_menu.rs).
const BLIZZARD_ADDONS: &[&str] = &[
    "Blizzard_SharedXMLBase",
    "Blizzard_Colors",
    "Blizzard_SharedXML",
    "Blizzard_SharedXMLGame",
    "Blizzard_UIPanelTemplates",
    "Blizzard_FrameXMLBase",
    "Blizzard_FrameEffects",
    "Blizzard_LoadLocale",
    "Blizzard_Fonts_Shared",
    "Blizzard_HelpPlate",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_GameMenuEsc",
    "Blizzard_Communities",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_StaticPopup_Game",
    "Blizzard_TransmogShared",
    "Blizzard_ActionBar",
    "Blizzard_UnitFrame",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_FrameXML",
    "Blizzard_UIPanels_Game",
    "Blizzard_TokenUI",
    "Blizzard_MainMenuBarBagButtons",
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
    if cfg!(feature = "client-wrath") {
        common::panel_fixtures::load_wrath_panel_core(env);
        return;
    }
    let ui = blizzard_ui_dir();
    for name in BLIZZARD_ADDONS {
        common::load_required_blizzard_addon(env, &ui, name);
        env.apply_runtime_addon_load_workarounds(name);
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
    if !cfg!(feature = "client-wrath") {
        load_token_ui(env);
    }
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
            local actualID = frame and frame:GetID()
            if frame and frame.GetBagID then actualID = frame:GetBagID() end
            if frame and frame:IsShown() and actualID == {bag_id} then
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
        if !cfg!(feature = "client-wrath") {
            load_token_ui(&env);
        }
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
        if cfg!(feature = "client-wrath") {
            env.exec(include_str!("fixtures/wrath_f03/character_surface.lua"))
                .expect("native character title and equipped slot textures");
            return;
        }

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
        if cfg!(feature = "client-wrath") {
            // Native slot hover/tooltip lines, not modern TooltipDataInfo mixin state.
            env.exec(include_str!("fixtures/wrath_f03/inventory_tooltip.lua"))
                .expect("native inventory tooltip consumer opens, populates lines and closes");
            let errors = drain_test_errors(&env);
            assert!(errors.is_empty(), "native inventory tooltip errors: {errors:?}");
            return;
        }

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

#[cfg(feature = "client-retail")]
#[test]
fn retail_item_button_corruption_uses_supplied_identity() {
    let env = setup_env();
    env.state().borrow_mut().corrupted_items.insert(900001);
    env.exec(
        r#"
        local button = CreateFrame('Button', nil, UIParent)
        button.IconOverlay = button:CreateTexture(nil, 'OVERLAY')
        SetItemButtonOverlay(button, 900001, Enum.ItemQuality.Common, false)
        assert(button.IconOverlay:GetAtlas() == 'Nzoth-inventory-icon')
        assert(button.IconOverlay:IsShown())
        assert(C_Item.IsCorruptedItem(900001) == true)
        assert(C_Item.IsCorruptedItem(900002) == false)
        "#,
    )
    .expect("original item overlay uses supplied corruption");
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_paper_doll_azerite_overlay_uses_supplied_location() {
    use wow_ui_sim::lua_api::{AzeriteItemState, ItemLocationData};
    let env = setup_env();
    env.state().borrow_mut().azerite_item = Some(AzeriteItemState {
        item_location: ItemLocationData { equipment_slot_index: Some(2), ..Default::default() },
        current_xp: 0,
        max_xp: 100,
        power_level: 42,
        unlimited_power_level: 0,
        unlimited_unlocked: false,
        at_max_level: false,
        enabled: true,
    });
    env.exec(
        r#"
        local slot = CharacterNeckSlot
        local first = ItemLocation:CreateFromEquipmentSlot(2)
        local second = ItemLocation:CreateFromEquipmentSlot(1)
        slot:SetAzeriteItem(first)
        assert(slot.RankFrame:IsShown())
        assert(slot.AzeriteTexture:GetAtlas() == 'AzeriteArmor-CharacterInfo-Neck')
        slot:SetAzeriteItem(second)
        assert(not slot.RankFrame:IsShown())
        assert(not slot.AzeriteTexture:IsShown())
        "#,
    )
    .expect("original paper doll overlay uses supplied Azerite location");
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

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn item_button_azerite_overlay_uses_supplied_item_identity() {
    let env = setup_env();
    env.state()
        .borrow_mut()
        .azerite_empowered
        .empowered_items
        .insert(900001);
    env.exec(
        r#"
        local button = CreateFrame('Button', nil, UIParent)
        button.IconOverlay = button:CreateTexture(nil, 'OVERLAY')
        SetItemButtonOverlay(button, 900001, Enum.ItemQuality.Common, false)
        assert(button.IconOverlay:GetAtlas() == 'AzeriteIconFrame')
        assert(button.IconOverlay:IsShown())
        assert(C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItemByID(900001) == true)
        assert(C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItemByID(900002) == false)
    "#,
    )
    .expect("original item button Azerite overlay and distinct supplied identities");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn item_button_reagent_overlay_uses_supplied_item_qualities() {
    use wow_ui_sim::c_api::c_trade_skill_quality::CraftingQualityInfo;
    let env = setup_env();
    {
        let mut state = env.state().borrow_mut();
        for (id, quality, atlas) in [
            (900001, 1, "professions-icon-quality-tier1-small"),
            (900002, 2, "professions-icon-quality-tier2-small"),
        ] {
            state.recipe_quality_inputs.reagent_item_quality.insert(
                id,
                CraftingQualityInfo {
                    quality,
                    icon_inventory: atlas.to_owned(),
                    ..Default::default()
                },
            );
        }
    }
    env.exec(
        r#"
        local first = CreateFrame('Button', nil, UIParent)
        local second = CreateFrame('Button', nil, UIParent)
        SetItemCraftingQualityOverlay(first, 900001)
        SetItemCraftingQualityOverlay(second, 900002)
        assert(first.isProfessionItem and not first.isCraftedItem)
        assert(second.isProfessionItem and not second.isCraftedItem)
        assert(first.ProfessionQualityOverlay:GetAtlas() == 'professions-icon-quality-tier1-small')
        assert(second.ProfessionQualityOverlay:GetAtlas() == 'professions-icon-quality-tier2-small')
        assert(C_TradeSkillUI.GetItemReagentQualityInfo(900001).quality == 1)
        assert(C_TradeSkillUI.GetItemReagentQualityInfo(900002).quality == 2)
        assert(C_TradeSkillUI.GetItemReagentQualityInfo(900003) == nil)
    "#,
    )
    .expect("original item button reagent overlays use supplied quality");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn item_button_queries_authenticate_secret_arguments_by_caller_taint() {
    use wow_ui_sim::c_api::c_trade_skill_quality::CraftingQualityInfo;

    let env = setup_env();
    {
        let mut state = env.state().borrow_mut();
        state.azerite_empowered.empowered_items.insert(900001);
        state.recipe_quality_inputs.reagent_item_quality.insert(
            900001,
            CraftingQualityInfo {
                quality: 2,
                icon_inventory: "professions-icon-quality-tier2-small".to_owned(),
                ..Default::default()
            },
        );
    }
    env.exec(
        r#"
        local azerite = C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItemByID
        local reagent = C_TradeSkillUI.GetItemReagentQualityInfo
        assert(type(azerite) == 'function', 'Azerite query must exist')
        assert(type(reagent) == 'function', 'reagent query must exist')
        assert(debug.getstacktaint() == nil, 'untainted setup')
        local item = secretwrap(900001)
        assert(issecretvalue(item), 'authentic secret item')
        assert(azerite(item) == true, 'untainted secret Azerite result')
        local info = reagent(item)
        assert(info.quality == 2, 'untainted secret reagent quality')
        assert(info.iconInventory == 'professions-icon-quality-tier2-small', 'untainted secret reagent atlas')
        local function probe()
            assert(debug.getstacktaint() == 'ItemButtonProbe', 'tainted caller context')
            assert(issecretvalue(item), 'secret identity retained')
            assert(not pcall(azerite, item), 'tainted secret Azerite query must fail')
            assert(not pcall(reagent, item), 'tainted secret reagent query must fail')
            assert(azerite(900001) == true, 'tainted public Azerite query')
            assert(reagent(900001).quality == 2, 'tainted public reagent query')
        end
        debug.setobjecttaint(probe, 'ItemButtonProbe')
        probe()
        assert(debug.getstacktaint() == nil, 'caller taint restored')
        "#,
    )
    .expect("item button queries authenticate secret arguments");
}
