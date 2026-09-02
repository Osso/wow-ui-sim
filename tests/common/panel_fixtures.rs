//! Shared fixtures for the Load-on-Demand panel-loading test binaries.
//!
//! Originally lived inline in `tests/test_showuipanel_lod.rs`; pulled out so
//! the panel test surface can be split across multiple test binaries (panel
//! loaders, Player Spells diagnostics, harness-coverage) without
//! copy-pasting setup. `setup_env`, the harness installers, and the
//! `player_spells_panel_debug_snapshot` helpers are all preserved exactly
//! as they ran from the single-file form.

use std::path::PathBuf;
use wow_ui_sim::lua_api::WowLuaEnv;

pub fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

/// Blizzard addons needed for the panel system (dependency order).
pub const PANEL_ADDONS: &[&str] = &[
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
    "Blizzard_GuildControlUI",
    "Blizzard_TimerunningUtil",
    "Blizzard_GameMenuEsc",
    "Blizzard_Menu",
    "Blizzard_ChatFrameBase",
    "Blizzard_ChatFrame",
    "Blizzard_FrameXMLUtil",
    "Blizzard_Communities",
    "Blizzard_AccessibilityTemplates",
    "Blizzard_ObjectAPI",
    "Blizzard_UIParent",
    "Blizzard_TextStatusBar",
    "Blizzard_MoneyFrame",
    "Blizzard_POIButton",
    "Blizzard_Flyout",
    "Blizzard_StoreUI",
    "Blizzard_MicroMenu",
    "Blizzard_ManagedFrameSystem",
    "Blizzard_UIParentUtil",
    "Blizzard_EditMode",
    "Blizzard_GarrisonBase",
    "Blizzard_GameTooltip",
    "Blizzard_UIParentPanelManager",
    "Blizzard_Settings_Shared",
    "Blizzard_SettingsDefinitions_Shared",
    "Blizzard_SettingsDefinitions_Frame",
    // Blizzard_FrameXML.toc (the bare TOC the loader now prefers) declares
    // `## Dep: Blizzard_UIErrorsFrame`; the legacy _Mainline.toc carried
    // UIErrorsFrame.xml itself.
    "Blizzard_UIErrorsFrame",
    "Blizzard_FrameXML",
    "Blizzard_StaticPopup",
    "Blizzard_TimeManager",
    "Blizzard_ItemButton",
    "Blizzard_QuickKeybind",
    "Blizzard_UIPanels_Game",
    "Blizzard_PingUI",
    "Blizzard_ActionBar",
    "Blizzard_ColorPickerFrame",
    "Blizzard_UnitFrame",
    "Blizzard_TokenUI",
    "Blizzard_Minimap",
];

pub fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    seed_addon_search_paths(&env);
    load_panel_addons(&env);
    install_lua_harness_stubs(&env);

    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

pub fn seed_addon_search_paths(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.addon_base_paths = vec![blizzard_ui_dir()];
}

pub fn load_panel_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for addon_name in PANEL_ADDONS {
        crate::common::load_required_blizzard_addon(env, &ui, addon_name);
    }
}

pub fn install_lua_harness_stubs(env: &WowLuaEnv) {
    install_uiparent_load_addon_seam(env);
    install_action_button_util_stub(env);
    install_multi_action_bar_grid_stubs(env);
}

/// Wrap `UIParentLoadAddOn` so the LoD panel tests don't drag in
/// `Blizzard_CooldownBroadcaster` by default (the addon's `ADDON_LOADED` /
/// `PLAYER_ENTERING_WORLD` handlers reach for `C_ChatInfo` / `C_Spell`
/// surface the panel tests don't bring up). The previous version returned
/// `false` straight from the wrapper, which silently bypassed the failure
/// bookkeeping the real Blizzard impl performs (`FailedAddOnLoad[name] =
/// true` plus the dialog message), and removed any dedicated seam for
/// future tests that *do* want to exercise `CooldownBroadcaster_LoadUI`
/// end-to-end.
///
/// The seam:
///
/// * keeps the default skip behaviour for the broadcaster, so every
///   currently-passing test stays passing;
/// * routes ALL non-broadcaster `UIParentLoadAddOn` calls through the
///   original implementation untouched (no other addon is affected);
/// * mirrors the failure bookkeeping the real `UIParentLoadAddOn`
///   performs by populating both a global `FailedAddOnLoad` table and a
///   harness-visible `__test_uiparent_load_addon_failures` log so tests
///   can assert on the failure record;
/// * exposes a `__test_skip_cooldown_broadcaster_load` opt-out flag so a
///   dedicated coverage test can flip it to `false` and run
///   `CooldownBroadcaster_LoadUI()` against the real addon load path.
pub fn install_uiparent_load_addon_seam(env: &WowLuaEnv) {
    env.exec(
        r#"
        __test_uiparent_load_addon_failures = __test_uiparent_load_addon_failures or {}
        if __test_skip_cooldown_broadcaster_load == nil then
            __test_skip_cooldown_broadcaster_load = true
        end

        if type(UIParentLoadAddOn) == "function" and not __test_original_uiparent_load_addon then
            __test_original_uiparent_load_addon = UIParentLoadAddOn
            UIParentLoadAddOn = function(name)
                if name == "Blizzard_CooldownBroadcaster" and __test_skip_cooldown_broadcaster_load then
                    -- Mirror the real Blizzard_UIParent failure bookkeeping:
                    -- record the failure into FailedAddOnLoad (the global
                    -- name the test seam can observe — Blizzard's local-of-
                    -- the-same-name is not reachable from outside) and log
                    -- the {name, reason} into a harness-visible array so
                    -- tests can assert the broadcaster was indeed skipped.
                    FailedAddOnLoad = FailedAddOnLoad or {}
                    if not FailedAddOnLoad[name] then
                        FailedAddOnLoad[name] = true
                        __test_uiparent_load_addon_failures[
                            #__test_uiparent_load_addon_failures + 1
                        ] = { name = name, reason = "DISABLED_FOR_TESTS" }
                    end
                    return false
                end
                return __test_original_uiparent_load_addon(name)
            end
        end
        "#,
    )
    .expect("failed to install UIParentLoadAddOn harness seam");
}

/// Install the `ActionButtonUtil` namespace used by `Blizzard_SpellSearch`
/// (`SpellSearchUtil.GetActionBarStatusFor*`) and the talent / spellbook
/// item templates. The fixture loads `Blizzard_ActionBar` and
/// `Blizzard_UnitFrame` for CharacterFrame dependencies, while this harness
/// keeps the three ActionButtonUtil status probes deterministic and overridable.
/// The fixture:
///
/// * publishes the `ActionBarActionStatus` enum literal Blizzard addons
///   key off (NotMissing / MissingFromAllBars / OnInactiveBonusBar /
///   OnDisabledActionBar);
/// * defaults the three `GetActionBarStatusFor{Spell,PetAction,Flyout}`
///   probes to `NotMissing` (matches the previous stub — keeps every
///   currently-passing test passing);
/// * exposes a per-id override table per probe
///   (`__test_action_bar_status_for_spell` / `_pet_action` / `_flyout`)
///   so tests that *want* to drive the `MissingFromAllBars` /
///   `OnInactiveBonusBar` / `OnDisabledActionBar` branches in
///   `SpellSearchUtil` can do so by writing a single Lua line before
///   invoking the panel.
pub fn install_action_button_util_stub(env: &WowLuaEnv) {
    env.exec(
        r#"
        ActionButtonUtil = ActionButtonUtil or {}
        ActionButtonUtil.ActionBarActionStatus = ActionButtonUtil.ActionBarActionStatus or {
            NotMissing = 1,
            MissingFromAllBars = 2,
            OnInactiveBonusBar = 3,
            OnDisabledActionBar = 4,
        }

        __test_action_bar_status_for_spell = {}
        __test_action_bar_status_for_pet_action = {}
        __test_action_bar_status_for_flyout = {}

        function ActionButtonUtil.GetActionBarStatusForSpell(spellID)
            local override = spellID and __test_action_bar_status_for_spell[spellID]
            return override or ActionButtonUtil.ActionBarActionStatus.NotMissing
        end

        function ActionButtonUtil.GetActionBarStatusForPetAction(petActionID)
            local override = petActionID and __test_action_bar_status_for_pet_action[petActionID]
            return override or ActionButtonUtil.ActionBarActionStatus.NotMissing
        end

        function ActionButtonUtil.GetActionBarStatusForFlyout(flyoutActionID)
            local override = flyoutActionID and __test_action_bar_status_for_flyout[flyoutActionID]
            return override or ActionButtonUtil.ActionBarActionStatus.NotMissing
        end
        "#,
    )
    .expect("failed to install ActionButtonUtil harness stub");
}

fn install_multi_action_bar_grid_stubs(env: &WowLuaEnv) {
    env.exec(
        r#"
        if type(MultiActionBar_ShowAllGrids) ~= "function" then
            function MultiActionBar_ShowAllGrids()
            end
        end
        if type(MultiActionBar_HideAllGrids) ~= "function" then
            function MultiActionBar_HideAllGrids()
            end
        end
        "#,
    )
    .expect("failed to install multi action bar grid harness stubs");
}

pub fn fire_startup_events(env: &WowLuaEnv) {
    super::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    super::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

pub fn clear_recorded_lua_errors(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.lua_errors.clear();
    state.lua_error_records.clear();
    state.lua_error_counts.clear();
}

pub fn recorded_lua_errors(env: &WowLuaEnv) -> Vec<String> {
    env.state().borrow().lua_errors.clone()
}

pub fn player_spells_panel_debug_snapshot(env: &WowLuaEnv) -> String {
    if let Some(missing) = player_spells_frame_missing_marker(env) {
        return missing;
    }
    [
        player_spells_panel_settings_section(env),
        player_spells_subframe_names_section(env),
        player_spells_missing_onload_methods_section(env),
        player_spells_callback_invocation_section(env),
    ]
    .join("\n")
}

/// `Some("player_spells_frame=nil")` if the frame doesn't exist (matches
/// the original snapshot's nil-marker line); `None` otherwise so the
/// caller proceeds with the per-aspect probes.
fn player_spells_frame_missing_marker(env: &WowLuaEnv) -> Option<String> {
    let exists: bool = env.eval("return PlayerSpellsFrame ~= nil").unwrap_or(false);
    (!exists).then(|| "player_spells_frame=nil".to_string())
}

fn player_spells_panel_settings_section(env: &WowLuaEnv) -> String {
    eval_snapshot_section(env, PANEL_SETTINGS_SECTION_LUA)
}

fn player_spells_subframe_names_section(env: &WowLuaEnv) -> String {
    eval_snapshot_section(env, SUBFRAME_NAMES_SECTION_LUA)
}

fn player_spells_missing_onload_methods_section(env: &WowLuaEnv) -> String {
    eval_snapshot_section(env, MISSING_ONLOAD_METHODS_SECTION_LUA)
}

fn player_spells_callback_invocation_section(env: &WowLuaEnv) -> String {
    eval_snapshot_section(env, CALLBACK_INVOCATION_SECTION_LUA)
}

fn eval_snapshot_section(env: &WowLuaEnv, lua: &str) -> String {
    env.eval(lua)
        .unwrap_or_else(|error| format!("snapshot_error={error:?}"))
}

const PANEL_SETTINGS_SECTION_LUA: &str = r#"
    local panelSettings = UIPanelWindows and UIPanelWindows[PlayerSpellsFrame:GetName()]
    local storedAutoMinimize = panelSettings and panelSettings.autoMinimizeOnCondition
    local storedSetMinimized = panelSettings and panelSettings.setMinimizedFunc
    local frameAutoMinimize = PlayerSpellsFrame:GetAttribute("UIPanelLayout-autoMinimizeOnCondition")
    local frameSetMinimized = PlayerSpellsFrame:GetAttribute("UIPanelLayout-setMinimizedFunc")
    local onLoadScript = PlayerSpellsFrame:GetScript("OnLoad")
    local playerGetTabOk, playerGetTabResult = pcall(function()
        return PlayerSpellsFrame:GetTab()
    end)
    return table.concat({
        "PlayerSpellsFrame=" .. tostring(type(PlayerSpellsFrame)),
        "ShouldAutoMinimize=" .. tostring(type(PlayerSpellsFrame.ShouldAutoMinimize)),
        "SetMinimized=" .. tostring(type(PlayerSpellsFrame.SetMinimized)),
        "UIPanelWindows.PlayerSpellsFrame=" .. tostring(type(panelSettings)),
        "stored.autoMinimizeOnCondition=" .. tostring(type(storedAutoMinimize)),
        "stored.setMinimizedFunc=" .. tostring(type(storedSetMinimized)),
        "frame.autoMinimizeOnCondition=" .. tostring(type(frameAutoMinimize)),
        "frame.setMinimizedFunc=" .. tostring(type(frameSetMinimized)),
        "PlayerSpellsFrame.OnLoadScript=" .. tostring(type(onLoadScript)),
        "PlayerSpellsFrame.internalTabTracker=" .. tostring(type(PlayerSpellsFrame.internalTabTracker)),
        "PlayerSpellsFrame.minimizedWidth=" .. tostring(PlayerSpellsFrame.minimizedWidth),
        "PlayerSpellsFrame.maximizedWidth=" .. tostring(PlayerSpellsFrame.maximizedWidth),
        "PlayerSpellsFrame.GetTab()=" .. tostring(playerGetTabOk) .. ":" .. tostring(playerGetTabResult),
    }, "\n")
"#;

const SUBFRAME_NAMES_SECTION_LUA: &str = r#"
    local spellbookFrame = PlayerSpellsFrame.SpellBookFrame
    local spellbookGetTabOk, spellbookGetTabResult = pcall(function()
        if not spellbookFrame then
            return "missing"
        end
        return spellbookFrame:GetTab()
    end)
    local function frameName(frame) return frame and frame:GetName() or "missing" end
    return table.concat({
        "SpecFrame.name=" .. frameName(PlayerSpellsFrame.SpecFrame),
        "TalentsFrame.name=" .. frameName(PlayerSpellsFrame.TalentsFrame),
        "SpellBookFrame.name=" .. frameName(spellbookFrame),
        "SpellBookFrame.internalTabTracker=" .. (spellbookFrame and tostring(type(spellbookFrame.internalTabTracker)) or "missing"),
        "SpellBookFrame.minimizedWidth=" .. tostring(spellbookFrame and spellbookFrame.minimizedWidth or nil),
        "SpellBookFrame.maximizedWidth=" .. tostring(spellbookFrame and spellbookFrame.maximizedWidth or nil),
        "SpellBookFrame.GetTab()=" .. tostring(spellbookGetTabOk) .. ":" .. tostring(spellbookGetTabResult),
    }, "\n")
"#;

const MISSING_ONLOAD_METHODS_SECTION_LUA: &str = r#"
    local missingOnLoadMethods = {}
    local function childAliases(parent, child)
        if not debug or not debug.getfenv then
            return ""
        end
        local env = debug.getfenv(parent)
        local fields = env and env[1]
        if type(fields) ~= "table" then
            return ""
        end
        local aliases = {}
        for key, value in pairs(fields) do
            if value == child and type(key) == "string" then
                table.insert(aliases, key)
            end
        end
        table.sort(aliases)
        return table.concat(aliases, ",")
    end
    local function childSegment(index, child, alias)
        local childName = child.GetName and child:GetName() or nil
        if childName then
            if alias ~= "" then
                return childName .. ":" .. alias
            end
            return childName
        end
        return 'child_' .. tostring(index) .. (alias ~= "" and ':' .. alias or '')
    end
    local function appendMissing(frame, path)
        local frameOnLoadScript = frame.GetScript and frame:GetScript("OnLoad")
        if frameOnLoadScript and type(frame.OnLoad) ~= "function" then
            local objectType = frame.GetObjectType and frame:GetObjectType() or "?"
            table.insert(missingOnLoadMethods, path .. " type=" .. tostring(objectType) .. " OnLoad=" .. tostring(type(frame.OnLoad)))
        end
        local children = { frame:GetChildren() }
        for index, child in ipairs(children) do
            local alias = childAliases(frame, child)
            local segment = childSegment(index, child, alias)
            appendMissing(child, path .. "." .. segment)
        end
    end
    appendMissing(PlayerSpellsFrame, "PlayerSpellsFrame")
    return "missing.OnLoad.methods=" .. table.concat(missingOnLoadMethods, " | ")
"#;

const CALLBACK_INVOCATION_SECTION_LUA: &str = r#"
    local panelSettings = UIPanelWindows and UIPanelWindows[PlayerSpellsFrame:GetName()]
    local storedAutoMinimize = panelSettings and panelSettings.autoMinimizeOnCondition
    local storedSetMinimized = panelSettings and panelSettings.setMinimizedFunc
    local autoCallOk, autoCallResult = pcall(function()
        if type(storedAutoMinimize) ~= "function" then
            return "skip"
        end
        return storedAutoMinimize(PlayerSpellsFrame)
    end)
    local setCallOk, setCallResult = pcall(function()
        if type(storedSetMinimized) ~= "function" then
            return "skip"
        end
        return storedSetMinimized(PlayerSpellsFrame, false)
    end)
    return table.concat({
        "call.autoMinimizeOnCondition=" .. tostring(autoCallOk) .. ":" .. tostring(autoCallResult),
        "call.setMinimizedFunc=" .. tostring(setCallOk) .. ":" .. tostring(setCallResult),
    }, "\n")
"#;

type EnvFn = fn(&WowLuaEnv);

// This fixture module is compiled independently by many integration tests.
// Keep the shared helper API referenced even when one target uses only a slice
// of the fixture surface.
const _: () = {
    let _ = blizzard_ui_dir as fn() -> PathBuf;
    let _ = PANEL_ADDONS;
    let _ = setup_env as fn() -> WowLuaEnv;
    let _ = seed_addon_search_paths as EnvFn;
    let _ = load_panel_addons as EnvFn;
    let _ = install_lua_harness_stubs as EnvFn;
    let _ = install_uiparent_load_addon_seam as EnvFn;
    let _ = install_action_button_util_stub as EnvFn;
    let _ = fire_startup_events as EnvFn;
    let _ = clear_recorded_lua_errors as EnvFn;
    let _ = recorded_lua_errors as fn(&WowLuaEnv) -> Vec<String>;
    let _ = player_spells_panel_debug_snapshot as fn(&WowLuaEnv) -> String;
};
