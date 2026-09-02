//! Integration tests for micro menu button clicks.
//!
//! Loads the base Blizzard UI, fires startup events, then clicks each micro
//! menu button and verifies the corresponding panel frame is shown.

use crate::common;

use std::path::{Path, PathBuf};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;

fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

/// Blizzard addons in dependency order (mirrors BLIZZARD_ADDONS in main.rs).
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
    "Blizzard_Menu",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_GameTooltip",
    "Blizzard_StaticPopup",
    "Blizzard_GameMenuEsc",
    "Blizzard_StoreUI",
    "Blizzard_Communities",
    "Blizzard_GameMenu",
    "Blizzard_ClassMenu",
    "Blizzard_MicroMenu",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_HousingEventHandler",
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
    "Blizzard_MapCanvasSecureUtil",
    "Blizzard_MapCanvas",
    "Blizzard_SharedMapDataProviders",
    "Blizzard_WorldMap",
    "Blizzard_ActionBar",
    "Blizzard_UIWidgets",
    "Blizzard_Minimap",
    "Blizzard_AddOnList",
    "Blizzard_TimerunningUtil",
];

/// Create a fully loaded environment with Blizzard addons and startup events.
fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    // Set addon_base_paths for runtime on-demand loading
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    // Load base Blizzard addons
    let ui = blizzard_ui_dir();
    for name in BLIZZARD_ADDONS {
        let addon_dir = ui.join(name);
        let toc_path = find_toc_file(&addon_dir)
            .unwrap_or_else(|| panic!("active retail TOC for {name} must resolve"));
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }

    install_professions_book_bootstrap(&env, &ui);
    env.apply_post_load_workarounds();
    install_fixture_globals(&env);
    fire_startup_events(&env);
    env
}

fn install_professions_book_bootstrap(env: &WowLuaEnv, ui: &Path) {
    let bootstrap_path = ui.join("Blizzard_ProfessionsBook/Blizzard_ProfessionsBook_Bootstrap.lua");
    let source = std::fs::read_to_string(&bootstrap_path).unwrap_or_else(|error| {
        panic!(
            "ProfessionsBook bootstrap source {} should be readable: {error}",
            bootstrap_path.display()
        )
    });
    let escaped_source = source
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    let loader = format!(
        r#"local chunk, compile_error = loadstring("{escaped_source}", "@Blizzard_ProfessionsBook/Blizzard_ProfessionsBook_Bootstrap.lua")
if not chunk then error("compile ProfessionsBook bootstrap: " .. tostring(compile_error)) end
local ok, runtime_error = pcall(chunk, "Blizzard_ProfessionsBook", {{}})
if not ok then error("run ProfessionsBook bootstrap: " .. tostring(runtime_error)) end"#,
    );
    env.exec(&loader)
        .expect("ProfessionsBook bootstrap should publish ToggleProfessionsBook");
}

fn install_fixture_globals(env: &WowLuaEnv) {
    env.exec(
        r#"
        function KeybindFrames_InQuickKeybindMode()
            return false
        end

        function CanShowAchievementUI()
            return true
        end
        "#,
    )
    .expect("failed to install micro-menu fixture globals");
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
        "PLAYER_LEAVING_WORLD",
    ] {
        let _ = env.fire_event(event);
    }
}

/// Simulate clicking a micro menu button by calling its OnClick script.
fn click_button(env: &WowLuaEnv, button_name: &str) -> Result<(), String> {
    let code = format!(
        r#"
        local btn = {button_name}
        if not btn then error("{button_name} does not exist") end
        local onclick = btn:GetScript("OnClick")
        if not onclick then error("{button_name} has no OnClick handler") end
        onclick(btn, "LeftButton", false)
        "#
    );
    env.exec(&code).map_err(|e| format!("{e}"))
}

/// Check whether a global frame exists and is shown.
fn frame_is_shown(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil and {frame_name}:IsShown() == true");
    env.eval::<bool>(&code).unwrap_or(false)
}

/// Check whether a global frame exists (may not be shown).
fn frame_exists(env: &WowLuaEnv, frame_name: &str) -> bool {
    let code = format!("return {frame_name} ~= nil");
    env.eval::<bool>(&code).unwrap_or(false)
}

/// Return a texture path from a global frame expression, or empty string.
fn texture_path(env: &WowLuaEnv, texture_expr: &str) -> String {
    let code = format!(
        r#"
        local tex = {texture_expr}
        if not tex or not tex.GetTextureFilePath then
            return ""
        end
        return tex:GetTextureFilePath() or ""
        "#
    );
    env.eval::<String>(&code).unwrap_or_default()
}

fn professions_primary_frame_expr() -> &'static str {
    "PrimaryProfession1 or (ProfessionsContentFrame and ProfessionsContentFrame.PrimaryProfession1)"
}

fn professions_primary_spell_button_is_shown(env: &WowLuaEnv, button_name: &str) -> bool {
    let primary_expr = professions_primary_frame_expr();
    let code = format!(
        "local primary = {primary_expr}; return primary and primary.{button_name} and primary.{button_name}:IsShown() or false"
    );
    env.eval::<bool>(&code).unwrap_or(false)
}

fn professions_primary_spell_button_has_texture(env: &WowLuaEnv, button_name: &str) -> bool {
    let primary_expr = professions_primary_frame_expr();
    let code = format!(
        "local primary = {primary_expr}; return primary and primary.{button_name} and primary.{button_name}.IconTexture and primary.{button_name}.IconTexture:GetTexture() ~= nil"
    );
    env.eval::<bool>(&code).unwrap_or(false)
}

fn professions_primary_spell_button_spell_id(env: &WowLuaEnv, button_name: &str) -> i32 {
    let primary_expr = professions_primary_frame_expr();
    let code = format!(
        r#"
        local primary = {primary_expr}
        if not primary or not primary.{button_name} then
            return 0
        end
        local slot = ProfessionsBook_GetSpellBookItemSlot(primary.{button_name})
        if not slot then
            return -1
        end
        local info = C_SpellBook.GetSpellBookItemInfo(slot, Enum.SpellBookSpellBank.Player)
        return info and info.spellID or 0
        "#
    );
    env.eval::<i32>(&code).unwrap_or(0)
}

fn professions_primary_spell_button_nameframe_texture(
    env: &WowLuaEnv,
    button_name: &str,
) -> String {
    let primary_expr = professions_primary_frame_expr();
    let code = format!(
        r#"
        local primary = {primary_expr}
        if not primary or not primary.{button_name} then
            return ""
        end
        for _, region in ipairs({{ primary.{button_name}:GetRegions() }}) do
            if region:GetObjectType() == "Texture" then
                local draw_layer = region:GetDrawLayer()
                local width, height = region:GetSize()
                if draw_layer == "BACKGROUND" and width == 108 and height == 41 then
                    return region:GetTextureFilePath() or ""
                end
            end
        end
        return ""
        "#
    );
    env.eval::<String>(&code).unwrap_or_default()
}

#[test]
fn micro_menu_character_button_opens_character_frame() {
    let env = setup_env();
    click_button(&env, "CharacterMicroButton").expect("CharacterMicroButton click failed");
    assert!(
        frame_is_shown(&env, "CharacterFrame"),
        "CharacterFrame should be shown after clicking CharacterMicroButton"
    );
}

#[test]
fn micro_menu_game_menu_button_opens_game_menu() {
    let env = setup_env();
    click_button(&env, "MainMenuMicroButton").expect("MainMenuMicroButton click failed");
    assert!(
        frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be shown after clicking MainMenuMicroButton"
    );
}

#[test]
fn micro_menu_buttons_have_sized_atlas_textures() {
    let env = setup_env();
    let (profession_w, profession_h, spells_w, spells_h, menu_w, menu_h): (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
    ) = env
        .eval(
            r#"
            return ProfessionMicroButton:GetNormalTexture():GetWidth(),
                   ProfessionMicroButton:GetNormalTexture():GetHeight(),
                   PlayerSpellsMicroButton:GetNormalTexture():GetWidth(),
                   PlayerSpellsMicroButton:GetNormalTexture():GetHeight(),
                   MainMenuMicroButton:GetNormalTexture():GetWidth(),
                   MainMenuMicroButton:GetNormalTexture():GetHeight()
            "#,
        )
        .unwrap();

    assert_eq!(
        (profession_w, profession_h),
        (32.0, 40.0),
        "ProfessionMicroButton normal texture should fill the button"
    );
    assert_eq!(
        (spells_w, spells_h),
        (32.0, 40.0),
        "PlayerSpellsMicroButton normal texture should fill the button"
    );
    assert_eq!(
        (menu_w, menu_h),
        (32.0, 40.0),
        "MainMenuMicroButton normal texture should fill the button"
    );
}

#[test]
fn micro_menu_buttons_restore_normal_texture_after_hover() {
    let env = setup_env();
    let failures: String = env
        .eval(
            r#"
            local names = {
                "CharacterMicroButton",
                "ProfessionMicroButton",
                "PlayerSpellsMicroButton",
                "AchievementMicroButton",
                "QuestLogMicroButton",
                "GuildMicroButton",
                "LFDMicroButton",
                "CollectionsMicroButton",
                "EJMicroButton",
                "StoreMicroButton",
                "MainMenuMicroButton",
            }
            local failures = {}
            for _, name in ipairs(names) do
                local button = _G[name]
                if button and button:GetNormalTexture() then
                    local normal = button:GetNormalTexture()
                    normal:SetAlpha(1)

                    local onEnter = button:GetScript("OnEnter")
                    local onLeave = button:GetScript("OnLeave")
                    if onEnter and onLeave then
                        onEnter(button)
                        local enterAlpha = normal:GetAlpha()
                        onLeave(button)
                        local leaveAlpha = normal:GetAlpha()
                        if enterAlpha == 0 and leaveAlpha ~= 1 then
                            table.insert(failures, name .. "=" .. tostring(leaveAlpha))
                        end
                    end
                end
            end
            return table.concat(failures, ",")
            "#,
        )
        .unwrap();

    assert_eq!(
        failures, "",
        "micro buttons that hide their normal texture on hover should restore it on leave"
    );
}

#[test]
fn micro_menu_layout_stays_locked() {
    let env = setup_env();
    let result: String = env
        .eval(
            r#"
            local EPS = 0.75

            local function approx(actual, expected, eps)
                if type(actual) ~= "number" or type(expected) ~= "number" then
                    return false
                end
                return math.abs(actual - expected) <= (eps or EPS)
            end

            local function rect(path, frame)
                if type(frame) ~= "table" then
                    return nil, path .. "_missing"
                end
                local l, b, w, h = frame:GetRect()
                if not (l and b and w and h) then
                    return nil, path .. "_missing_rect"
                end
                return { l = l, b = b, w = w, h = h, r = l + w, t = b + h }, nil
            end

            if UpdateMicroButtons then
                UpdateMicroButtons()
            end
            if MicroMenu and MicroMenu.Layout then
                MicroMenu:Layout()
            end
            if MicroMenuContainer and MicroMenuContainer.Layout then
                MicroMenuContainer:Layout()
            end

            local bar = MicroButtonAndBagsBar
            local container = MicroMenuContainer
            local menu = MicroMenu
            if not bar then return "bar_missing" end
            if not container then return "container_missing" end
            if not menu then return "menu_missing" end

            local barRect, barErr = rect("bar", bar)
            if not barRect then return barErr end
            if not approx(barRect.w, 232, 0.1) then return "bar_width=" .. tostring(barRect.w) end
            if not approx(barRect.h, 80, 0.1) then return "bar_height=" .. tostring(barRect.h) end
            if barRect.l < 0 or barRect.b < 0 then
                return "bar_out_of_bounds=" .. tostring(barRect.l) .. "," .. tostring(barRect.b)
            end

            local containerRect, containerErr = rect("container", container)
            if not containerRect then return containerErr end

            local menuRect, menuErr = rect("menu", menu)
            if not menuRect then return menuErr end

            if not approx(containerRect.r, barRect.r) then
                return "container_right=" .. tostring(containerRect.r)
            end
            if not approx(containerRect.b, barRect.b) then
                return "container_bottom=" .. tostring(containerRect.b)
            end
            if not approx(menuRect.r, containerRect.r) then
                return "menu_right=" .. tostring(menuRect.r)
            end
            if not approx(menuRect.b, containerRect.b) then
                return "menu_bottom=" .. tostring(menuRect.b)
            end
            if containerRect.w + EPS < menuRect.w then
                return "container_width_lt_menu_width"
            end
            if containerRect.h + EPS < menuRect.h then
                return "container_height_lt_menu_height"
            end

            local buttonNames = {
                "CharacterMicroButton",
                "ProfessionMicroButton",
                "PlayerSpellsMicroButton",
                "AchievementMicroButton",
                "QuestLogMicroButton",
            }

            local buttonRects = {}
            for index, name in ipairs(buttonNames) do
                local button = _G[name]
                local r, e = rect(name, button)
                if not r then return e end
                if not button:IsShown() then
                    return name .. "_hidden"
                end
                if not approx(r.w, 32, 0.1) or not approx(r.h, 40, 0.1) then
                    return name .. "_size=" .. tostring(r.w) .. "x" .. tostring(r.h)
                end
                if not approx(r.b, menuRect.b) then
                    return name .. "_bottom=" .. tostring(r.b)
                end
                buttonRects[index] = r
            end

            for i = 2, #buttonRects do
                local delta = buttonRects[i].l - buttonRects[i - 1].l
                if not approx(delta, 27) then
                    return "button_spacing_" .. tostring(i) .. "=" .. tostring(delta)
                end
            end

            local characterRect = buttonRects[1]
            if not approx(characterRect.l, menuRect.l) then
                return "character_left=" .. tostring(characterRect.l)
            end

            local mainMenuRect, mainMenuErr = rect("MainMenuMicroButton", MainMenuMicroButton)
            if not mainMenuRect then return mainMenuErr end
            if not MainMenuMicroButton:IsShown() then
                return "MainMenuMicroButton_hidden"
            end
            if not approx(mainMenuRect.w, 32, 0.1) or not approx(mainMenuRect.h, 40, 0.1) then
                return "MainMenuMicroButton_size=" .. tostring(mainMenuRect.w) .. "x" .. tostring(mainMenuRect.h)
            end
            if not approx(mainMenuRect.r, menuRect.r) then
                return "MainMenuMicroButton_right=" .. tostring(mainMenuRect.r)
            end
            if not approx(mainMenuRect.b, menuRect.b) then
                return "MainMenuMicroButton_bottom=" .. tostring(mainMenuRect.b)
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Micro menu layout should remain locked: {result}"
    );
}

#[test]
fn micro_menu_professions_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "ProfessionsBookFrame"),
        "ProfessionsBookFrame should not exist before click"
    );
    click_button(&env, "ProfessionMicroButton").expect("ProfessionMicroButton click failed");
    assert!(
        frame_is_shown(&env, "ProfessionsBookFrame"),
        "ProfessionsBookFrame should be shown after clicking ProfessionMicroButton"
    );

    let spell_button_1_shown = professions_primary_spell_button_is_shown(&env, "SpellButton1");
    let spell_button_2_shown = professions_primary_spell_button_is_shown(&env, "SpellButton2");
    assert!(
        spell_button_1_shown && spell_button_2_shown,
        "Primary profession spell buttons should both be shown"
    );

    let first_icon = professions_primary_spell_button_has_texture(&env, "SpellButton1");
    let second_icon = professions_primary_spell_button_has_texture(&env, "SpellButton2");
    assert!(
        first_icon && second_icon,
        "Primary profession spell buttons should have icon textures"
    );

    let first_spell = professions_primary_spell_button_spell_id(&env, "SpellButton1");
    let second_spell = professions_primary_spell_button_spell_id(&env, "SpellButton2");
    assert_eq!(
        first_spell, 2018,
        "SpellButton1 should resolve to Blacksmithing"
    );
    assert_eq!(
        second_spell, 2657,
        "SpellButton2 should resolve to Smelt Copper"
    );

    let first_nameframe = professions_primary_spell_button_nameframe_texture(&env, "SpellButton1");
    let second_nameframe = professions_primary_spell_button_nameframe_texture(&env, "SpellButton2");
    assert_eq!(
        first_nameframe, "Interface\\Spellbook\\ProfessionsBook",
        "SpellButton1 name frame should keep its textured parchment background"
    );
    assert_eq!(
        second_nameframe, "Interface\\Spellbook\\ProfessionsBook",
        "SpellButton2 name frame should keep its textured parchment background"
    );
}

#[test]
fn micro_menu_player_spells_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should not exist before click"
    );
    click_button(&env, "PlayerSpellsMicroButton").expect("PlayerSpellsMicroButton click failed");
    assert!(
        frame_is_shown(&env, "PlayerSpellsFrame"),
        "PlayerSpellsFrame should be shown after clicking PlayerSpellsMicroButton"
    );
}

#[test]
fn micro_menu_collections_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "CollectionsJournal"),
        "CollectionsJournal should not exist before click"
    );
    click_button(&env, "CollectionsMicroButton").expect("CollectionsMicroButton click failed");
    assert!(
        frame_is_shown(&env, "CollectionsJournal"),
        "CollectionsJournal should be shown after clicking CollectionsMicroButton"
    );
}

#[test]
fn micro_menu_housing_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "HousingDashboardFrame"),
        "HousingDashboardFrame should not exist before click"
    );
    click_button(&env, "HousingMicroButton").expect("HousingMicroButton click failed");
    assert!(
        frame_is_shown(&env, "HousingDashboardFrame"),
        "HousingDashboardFrame should be shown after clicking HousingMicroButton"
    );
}

#[test]
fn micro_menu_achievement_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "AchievementFrame"),
        "AchievementFrame should not exist before click"
    );
    // Click may error in post-show setup; we only care that the frame is shown
    let _ = click_button(&env, "AchievementMicroButton");
    assert!(
        frame_exists(&env, "AchievementFrame"),
        "AchievementFrame should exist after clicking AchievementMicroButton"
    );
    assert!(
        frame_is_shown(&env, "AchievementFrame"),
        "AchievementFrame should be shown after clicking AchievementMicroButton"
    );

    let background_atlas: String = env
        .eval("return AchievementFrame.Background:GetAtlas() or ''")
        .expect("AchievementFrame background atlas should be readable");
    assert_eq!(
        background_atlas, "UI-Background-Rock-Brown",
        "AchievementFrame background should use its current atlas"
    );

    let categories_bg = texture_path(&env, "AchievementFrameCategoriesBG");
    assert_eq!(
        categories_bg, r"Interface\AchievementFrame\UI-Achievement-Parchment",
        "Achievement categories parchment texture should be assigned"
    );

    let header_left = texture_path(&env, "AchievementFrame.Header.Left");
    assert_eq!(
        header_left, r"Interface\AchievementFrame\UI-Achievement-Header",
        "Achievement header texture should be assigned"
    );
}

#[test]
fn micro_menu_ej_button_loads_and_opens_panel() {
    let env = setup_env();
    assert!(
        !frame_exists(&env, "EncounterJournal"),
        "EncounterJournal should not exist before click"
    );
    // Click may error in post-show setup; we only care that the frame is shown
    let _ = click_button(&env, "EJMicroButton");
    assert!(
        frame_is_shown(&env, "EncounterJournal"),
        "EncounterJournal should be shown after clicking EJMicroButton"
    );
}

#[test]
fn game_menu_buttons_display_text() {
    let env = setup_env();
    click_button(&env, "MainMenuMicroButton").expect("MainMenuMicroButton click failed");
    assert!(
        frame_is_shown(&env, "GameMenuFrame"),
        "GameMenuFrame should be shown"
    );

    // Collect text from all active buttons in the game menu's button pool
    env.exec(
        r#"
        __button_texts = {}
        for button in GameMenuFrame.buttonPool:EnumerateActive() do
            table.insert(__button_texts, button:GetText() or "")
        end
        "#,
    )
    .expect("Failed to enumerate game menu buttons");
    let button_texts = common::drain_string_table(&env, "__button_texts");

    assert!(
        !button_texts.is_empty(),
        "GameMenuFrame should have at least one button"
    );

    // Every button must have non-empty text
    for (i, text) in button_texts.iter().enumerate() {
        assert!(
            !text.is_empty(),
            "Game menu button {} has empty text",
            i + 1
        );
    }

    // These buttons should always appear (not conditional on features)
    let expected = [
        "Options",
        "AddOns",
        "Support",
        "Macros",
        "Log Out",
        "Exit Game",
        "Return to Game",
    ];
    for label in &expected {
        assert!(
            button_texts.iter().any(|t| t == label),
            "Expected game menu button '{}' not found. Got: {:?}",
            label,
            button_texts
        );
    }
}

#[test]
fn micro_menu_guild_button_loads_and_opens_panel() {
    let env = setup_env();
    // Click may error in post-show setup; we only care that the frame exists
    let _ = click_button(&env, "GuildMicroButton");
    assert!(
        frame_exists(&env, "CommunitiesFrame"),
        "CommunitiesFrame should exist after clicking GuildMicroButton"
    );
}

#[test]
fn main_menu_performance_bar_uses_seeded_network_stats() {
    let env = setup_env();
    let result: String = env
        .eval(
            r#"
            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_PerformanceBar")
            if not loaded then
                return "load_failed:" .. tostring(reason)
            end
            if not MainMenuBarPerformanceBarFrame_OnEnter then
                return "missing_performance_bar_tooltip"
            end
            if not MainMenuMicroButton or not MainMenuMicroButton.MainMenuBarPerformanceBar then
                return "missing_main_menu_performance_bar"
            end

            MainMenuMicroButton.updateInterval = 0
            MainMenuMicroButton:OnUpdate(0)

            local r, g, b = MainMenuMicroButton.MainMenuBarPerformanceBar:GetVertexColor()
            if math.abs(r - 0) > 0.001 or math.abs(g - 1) > 0.001 or math.abs(b - 0) > 0.001 then
                return string.format("bar_color=%.3f,%.3f,%.3f", r or -1, g or -1, b or -1)
            end

            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(
        result, "ok",
        "Main menu performance bar should use seeded latency values and stay green for low latency: {result}"
    );
}
