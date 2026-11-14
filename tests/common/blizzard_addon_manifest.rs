#![allow(dead_code)]

use wow_ui_sim::loader::BlizzardAddonOverride;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BlizzardAddonSmokeShape {
    MostlyFunctional,
    TemplateHeavy,
    LayoutHeavy,
    MultiAddonFlow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlizzardAddonSmokeTarget<'a> {
    pub name: &'a str,
    pub shape: BlizzardAddonSmokeShape,
    pub roots: &'a [&'a str],
    pub overrides: &'a [BlizzardAddonOverride<'a>],
    pub required_addons: &'a [&'a str],
    pub expected_global: &'a str,
    pub expected_frame: &'a str,
    pub behavior_probe_lua: &'a str,
}

/// Shared non-TOC overrides for the world-map voice-button harness.
///
/// `Blizzard_Channels` is not pulled in by the isolated world-map roots
/// themselves, but the combined voice-button render-order checks need it
/// so Blizzard's implicit channel UI path is present.
pub const WORLD_MAP_VOICE_CHAT_OVERRIDES: &[BlizzardAddonOverride<'static>] =
    &[BlizzardAddonOverride {
        addon: "Blizzard_WorldMap",
        extra_roots: &["Blizzard_ChatFrame", "Blizzard_Channels"],
    }];
pub const COMBAT_LOG_SMOKE_ROOTS: &[&str] = &["Blizzard_CombatLog"];
pub const WORLD_MAP_SMOKE_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[BlizzardAddonOverride {
    addon: "Blizzard_WorldMap",
    extra_roots: &["Blizzard_SharedXML"],
}];

pub const MACRO_UI_SMOKE_ROOTS: &[&str] = &["Blizzard_MacroUI"];
pub const SETTINGS_PANEL_SMOKE_ROOTS: &[&str] = &["Blizzard_SettingsDefinitions_Frame"];
pub const WORLD_MAP_SMOKE_ROOTS: &[&str] = &["Blizzard_WorldMap"];

pub const BLIZZARD_ADDON_SMOKE_TARGETS: &[BlizzardAddonSmokeTarget<'static>] = &[
    BlizzardAddonSmokeTarget {
        name: "combat_log",
        shape: BlizzardAddonSmokeShape::MostlyFunctional,
        roots: COMBAT_LOG_SMOKE_ROOTS,
        overrides: &[],
        required_addons: &["Blizzard_CombatLog"],
        expected_global: "Blizzard_CombatLog_GenerateFullEventList",
        expected_frame: "CombatLogQuickButtonFrame",
        behavior_probe_lua: r#"
            if COMBATLOG ~= ChatFrame2 then
                return "combatlog_frame=" .. tostring(COMBATLOG and COMBATLOG:GetName())
            end
            if not CombatLogQuickButtonFrame then
                return "missing_quick_button_frame"
            end
            local eventList = Blizzard_CombatLog_GenerateFullEventList()
            if type(eventList) ~= "table" or eventList.SWING_DAMAGE ~= true then
                return "event_list_missing_swing_damage"
            end
            local entryCount = CombatLogGetNumEntries and CombatLogGetNumEntries()
            if type(entryCount) ~= "number" then
                return "entry_count_type=" .. tostring(type(entryCount))
            end
            return "ok"
        "#,
    },
    BlizzardAddonSmokeTarget {
        name: "macro_ui",
        shape: BlizzardAddonSmokeShape::TemplateHeavy,
        roots: MACRO_UI_SMOKE_ROOTS,
        overrides: &[],
        required_addons: &["Blizzard_MacroUI"],
        expected_global: "ShowMacroFrame",
        expected_frame: "MacroFrame",
        behavior_probe_lua: r#"
            ShowMacroFrame()
            if not MacroFrame or not MacroFrame:IsShown() then
                return "macro_frame_not_shown"
            end
            if PanelTemplates_GetSelectedTab(MacroFrame) ~= 1 then
                return "selected_tab=" .. tostring(PanelTemplates_GetSelectedTab(MacroFrame))
            end
            local accountCount = select(1, GetNumMacros())
            if type(accountCount) ~= "number" or MacroFrame.MacroSelector.numMacros ~= accountCount then
                return "account_count=" .. tostring(accountCount)
            end
            return "ok"
        "#,
    },
    BlizzardAddonSmokeTarget {
        name: "world_map",
        shape: BlizzardAddonSmokeShape::LayoutHeavy,
        roots: WORLD_MAP_SMOKE_ROOTS,
        overrides: WORLD_MAP_SMOKE_OVERRIDES,
        required_addons: &[
            "Blizzard_MapCanvas",
            "Blizzard_SharedMapDataProviders",
            "Blizzard_WorldMap",
        ],
        expected_global: "ToggleWorldMap",
        expected_frame: "WorldMapFrame",
        behavior_probe_lua: r#"
            ToggleWorldMap()
            if not WorldMapFrame or not WorldMapFrame:IsShown() then
                return "world_map_not_shown"
            end
            local mapID = WorldMapFrame:GetMapID()
            if type(mapID) ~= "number" or mapID <= 0 then
                return "map_id=" .. tostring(mapID)
            end
            local titleText = WorldMapFrame.BorderFrame
                and WorldMapFrame.BorderFrame.TitleContainer
                and WorldMapFrame.BorderFrame.TitleContainer.TitleText
            local title = titleText and titleText:GetText()
            if type(title) ~= "string" or title == "" then
                return "world_map_title=" .. tostring(title)
            end
            ToggleWorldMap()
            if WorldMapFrame:IsShown() then
                return "world_map_did_not_close"
            end
            return "ok"
        "#,
    },
    BlizzardAddonSmokeTarget {
        name: "settings_panel",
        shape: BlizzardAddonSmokeShape::MultiAddonFlow,
        roots: SETTINGS_PANEL_SMOKE_ROOTS,
        overrides: &[],
        required_addons: &[
            "Blizzard_Settings_Shared",
            "Blizzard_SettingsDefinitions_Shared",
            "Blizzard_SettingsDefinitions_Frame",
        ],
        expected_global: "Settings.OpenToCategory",
        expected_frame: "SettingsPanel",
        behavior_probe_lua: r#"
            if type(Settings) ~= "table" then
                return "settings_missing"
            end

            if type(Settings.OpenToCategory) ~= "function" then
                return "open_to_category_missing"
            end

            if not SettingsPanel or type(SettingsPanel.Open) ~= "function" or type(SettingsPanel.Close) ~= "function" then
                return "settings_panel_methods_missing"
            end

            SettingsPanel:Open()
            if not SettingsPanel:IsShown() then
                return "settings_panel_not_shown"
            end
            SettingsPanel:Close()
            if SettingsPanel:IsShown() then
                return "settings_panel_still_shown"
            end
            return "ok"
        "#,
    },
];
