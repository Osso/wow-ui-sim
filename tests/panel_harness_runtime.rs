use crate::common;

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

const PANEL_HARNESS_ADDONS: &[(&str, &str)] = &[
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
    ("Blizzard_StoreUI", "Blizzard_StoreUI.toc"),
    ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
    ("Blizzard_EditMode", "Blizzard_EditMode.toc"),
    ("Blizzard_GarrisonBase", "Blizzard_GarrisonBase.toc"),
    ("Blizzard_GameTooltip", "Blizzard_GameTooltip_Mainline.toc"),
    (
        "Blizzard_ManagedFrameSystem",
        "Blizzard_ManagedFrameSystem_Mainline.toc",
    ),
    ("Blizzard_GameMenuEsc", "Blizzard_GameMenuEsc.toc"),
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
    ("Blizzard_Menu", "Blizzard_Menu.toc"),
    ("Blizzard_Minimap", "Blizzard_Minimap_Mainline.toc"),
    ("Blizzard_StaticPopup", "Blizzard_StaticPopup.toc"),
    ("Blizzard_TimeManager", "Blizzard_TimeManager_Mainline.toc"),
    ("Blizzard_TimerunningUtil", "Blizzard_TimerunningUtil.toc"),
    ("Blizzard_ItemButton", "Blizzard_ItemButton_Mainline.toc"),
    ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ("Blizzard_FrameXML", "Blizzard_FrameXML.toc"),
    (
        "Blizzard_UIPanels_Game",
        "Blizzard_UIPanels_Game_Mainline.toc",
    ),
];

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be synced")

}

fn load_blizzard_addon(env: &WowLuaEnv, addon_name: &str, toc_name: &str) {
    let addon_dir = blizzard_ui_dir().join(addon_name);
    let requested = addon_dir.join(toc_name);
    let toc_path = if requested.exists() {
        requested
    } else {
        [
            addon_dir.join(format!("{addon_name}.toc")),
            addon_dir.join(format!("{addon_name}_Mainline.toc")),
        ]
        .into_iter()
        .find(|candidate| candidate.exists())
        .unwrap_or(requested)
    };
    load_addon(&env.loader_env(), &toc_path)
        .unwrap_or_else(|error| panic!("{addon_name} should load in panel harness: {error}"));
    env.apply_runtime_addon_load_workarounds(addon_name);
    common::fire_addon_loaded(env, addon_name);
}

fn load_panel_harness(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for (addon_name, toc_name) in PANEL_HARNESS_ADDONS {
        let toc_path = ui.join(addon_name).join(toc_name);
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("{addon_name} should load in panel harness: {error}"));
    }

    for addon_name in [
        "Blizzard_MapCanvas",
        "Blizzard_SharedTalentUI",
        "Blizzard_PlayerSpells",
    ] {
        env.apply_runtime_addon_load_workarounds(addon_name);
    }
    common::panel_fixtures::install_lua_harness_stubs(env);
    env.apply_post_load_workarounds();
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

#[test]
fn achievement_addon_reports_runtime_load_reason() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    let (loaded, reason, frame_exists): (bool, Option<String>, bool) = env
        .eval(
            r#"
            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_AchievementUI")
            return loaded, reason, AchievementFrame ~= nil
            "#,
        )
        .expect("LoadAddOn should return");

    assert!(
        loaded,
        "Blizzard_AchievementUI should load in the panel harness, reason: {}",
        reason.unwrap_or_else(|| "<nil>".to_string())
    );
    assert!(
        frame_exists,
        "Blizzard_AchievementUI should create AchievementFrame when loaded"
    );
}

#[test]
fn achievement_error_handling_load_matches_raw_load_addon() {
    let error_handling_env = WowLuaEnv::new().expect("Failed to create Lua environment");
    error_handling_env.set_screen_size(1024.0, 768.0);
    error_handling_env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&error_handling_env);

    let (error_handling_loaded, error_handling_is_loaded): (bool, bool) = error_handling_env
        .eval(
            r#"
            return LoadAddOnWithErrorHandling("Blizzard_AchievementUI"), C_AddOns.IsAddOnLoaded("Blizzard_AchievementUI")
            "#,
        )
        .expect("load comparison should return");

    assert!(
        error_handling_loaded,
        "LoadAddOnWithErrorHandling should succeed in a fresh panel harness"
    );
    assert!(
        error_handling_is_loaded,
        "LoadAddOnWithErrorHandling should mark the addon loaded"
    );

    let raw_env = WowLuaEnv::new().expect("Failed to create Lua environment");
    raw_env.set_screen_size(1024.0, 768.0);
    raw_env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&raw_env);

    let (raw_loaded, raw_is_loaded): (bool, bool) = raw_env
        .eval(
            r#"
            local loaded = C_AddOns.LoadAddOn("Blizzard_AchievementUI")
            return loaded, C_AddOns.IsAddOnLoaded("Blizzard_AchievementUI")
            "#,
        )
        .expect("raw load comparison should return");

    assert!(raw_loaded, "raw C_AddOns.LoadAddOn should succeed");
    assert!(raw_is_loaded, "raw load should mark the addon loaded");
}

#[test]
fn collections_addon_load_creates_collections_journal() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    let (loaded, reason, frame_exists): (bool, Option<String>, bool) = env
        .eval(
            r#"
            local loaded, reason = C_AddOns.LoadAddOn("Blizzard_Collections")
            return loaded, reason, CollectionsJournal ~= nil
            "#,
        )
        .expect("collections load should return");

    assert!(
        loaded,
        "Blizzard_Collections should load in the panel harness, reason: {}",
        reason.unwrap_or_else(|| "<nil>".to_string())
    );
    assert!(
        frame_exists,
        "Blizzard_Collections should create CollectionsJournal when loaded"
    );
}

#[test]
fn encounter_journal_addon_load_creates_frame() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    load_blizzard_addon(
        &env,
        "Blizzard_EncounterJournal",
        "Blizzard_EncounterJournal.toc",
    );

    let frame_exists: bool = env
        .eval(
            r#"
            return EncounterJournal ~= nil
            "#,
        )
        .expect("encounter journal load probe should return");

    assert!(
        frame_exists,
        "Blizzard_EncounterJournal should create EncounterJournal when loaded"
    );
}

#[test]
fn compact_unit_frame_runtime_template_keeps_over_heal_absorb_glow_child() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);
    load_blizzard_addon(&env, "Blizzard_BuffFrame", "Blizzard_BuffFrame.toc");
    load_blizzard_addon(
        &env,
        "Blizzard_UnitFrame",
        "Blizzard_UnitFrame_Mainline.toc",
    );

    let has_glow: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Button", "CompactUnitRuntimeProbe", UIParent, "CompactUnitFrameTemplate")
            return frame.overHealAbsorbGlow ~= nil
                and CompactUnitRuntimeProbeOverHealAbsorbGlow == frame.overHealAbsorbGlow
            "#,
        )
        .expect("compact unit frame probe should return");

    assert!(
        has_glow,
        "CompactUnitFrameTemplate should attach overHealAbsorbGlow as a parentKey child"
    );
}

#[test]
fn deprecated_arena_match_frame_keeps_over_heal_absorb_glow_named_from_ancestor() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);
    load_blizzard_addon(&env, "Blizzard_BuffFrame", "Blizzard_BuffFrame.toc");
    load_blizzard_addon(
        &env,
        "Blizzard_UnitFrame",
        "Blizzard_UnitFrame_Mainline.toc",
    );
    load_blizzard_addon(
        &env,
        "Blizzard_Deprecated_ArenaUI",
        "Blizzard_Deprecated_ArenaUI.toc",
    );

    let has_glow: bool = env
        .eval(
            r#"
            local frame = ArenaEnemyMatchFrame1
            local glow = ArenaEnemyMatchFrame1OverHealAbsorbGlow
            return frame ~= nil
                and glow ~= nil
                and frame.overHealAbsorbGlow == glow
                and glow:GetParent() ~= nil
            "#,
        )
        .expect("deprecated arena frame probe should return");

    assert!(
        has_glow,
        "ArenaEnemyMatchFrame1 should keep its named overHealAbsorbGlow through anonymous XML wrapper frames"
    );
}

#[test]
fn group_members_pin_acquire_keeps_data_provider_on_pin() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);
    load_blizzard_addon(
        &env,
        "Blizzard_MapCanvasSecureUtil",
        "Blizzard_MapCanvasSecureUtil.toc",
    );
    load_blizzard_addon(&env, "Blizzard_MapCanvas", "Blizzard_MapCanvas.toc");
    load_blizzard_addon(
        &env,
        "Blizzard_SharedMapDataProviders",
        "Blizzard_SharedMapDataProviders_Mainline.toc",
    );
    load_blizzard_addon(
        &env,
        "Blizzard_BattlefieldMap",
        "Blizzard_BattlefieldMap.toc",
    );

    let probe: String = env
        .eval(
            r#"
            local map = BattlefieldMapFrame
            if not (map and map.ScrollContainer and map.ScrollContainer.Child) then
                return "missing_map"
            end
            map:SetMapID(C_Map.GetCurrentMapID())

            local provider = CreateFromMixins(GroupMembersDataProviderMixin)
            map:AddDataProvider(provider)

            local pin = provider.pin
            local has_pin = pin ~= nil
            local has_backref = false
            local backref_type = "nil"
            if has_pin then
                has_backref = pin.dataProvider == provider
                backref_type = type(pin.dataProvider)
            end
            return string.format("pin=%s pin_type=%s backref=%s backref_type=%s",
                tostring(has_pin), type(pin), tostring(has_backref), backref_type)
            "#,
        )
        .expect("group members pin probe should return");

    assert!(
        probe.contains("pin=true") && probe.contains("backref=true"),
        "GroupMembersDataProvider should keep the acquired pin wired to its data provider ({probe})"
    );
}

#[test]
fn achievement_toggle_loads_and_shows_frame() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    let (pre_toggle_source, pre_toggle_line): (Option<String>, i64) = env
        .eval(
            r#"
            local info = debug.getinfo(ToggleAchievementFrame)
            return info and info.short_src or nil, info and info.linedefined or 0
            "#,
        )
        .expect("pre-wrap toggle info should return");

    env.exec(
        r#"
        __achievement_toggle_called_load_ui = false
        __achievement_toggle_called_uiparent_load_addon = nil
        __achievement_toggle_called_toggle = false
        __achievement_toggle_called_kiosk = false
        __achievement_toggle_called_completed_any = false
        __achievement_toggle_called_can_show = false
        __achievement_toggle_called_c_addons_load_addon = nil
        __achievement_toggle_c_addons_load_result = false

        local originalCAddOnsLoadAddOn = C_AddOns.LoadAddOn
        C_AddOns.LoadAddOn = function(name, ...)
            __achievement_toggle_called_c_addons_load_addon = name
            local loaded = originalCAddOnsLoadAddOn(name, ...)
            __achievement_toggle_c_addons_load_result = loaded == true
            return loaded
        end

        local originalToggleAchievementFrame = ToggleAchievementFrame
        ToggleAchievementFrame = function(...)
            __achievement_toggle_called_toggle = true
            return originalToggleAchievementFrame(...)
        end

        local originalKioskIsEnabled = Kiosk.IsEnabled
        Kiosk.IsEnabled = function(...)
            __achievement_toggle_called_kiosk = true
            return originalKioskIsEnabled(...)
        end

        local originalHasCompletedAnyAchievement = HasCompletedAnyAchievement
        HasCompletedAnyAchievement = function(...)
            __achievement_toggle_called_completed_any = true
            return originalHasCompletedAnyAchievement(...)
        end

        local originalCanShowAchievementUI = CanShowAchievementUI
        CanShowAchievementUI = function(...)
            __achievement_toggle_called_can_show = true
            return originalCanShowAchievementUI(...)
        end

        local originalAchievementFrameLoadUI = AchievementFrame_LoadUI
        AchievementFrame_LoadUI = function(...)
            __achievement_toggle_called_load_ui = true
            return originalAchievementFrameLoadUI(...)
        end

        local originalUIParentLoadAddOn = UIParentLoadAddOn
        UIParentLoadAddOn = function(name, ...)
            __achievement_toggle_called_uiparent_load_addon = name
            return originalUIParentLoadAddOn(name, ...)
        end
        "#,
    )
    .expect("achievement toggle instrumentation should install");

    env.exec("ToggleAchievementFrame()")
        .expect("ToggleAchievementFrame should execute");

    let probe = format!(
        r#"
            local toggleInfo = debug.getinfo(ToggleAchievementFrame)
            local loadUIInfo = debug.getinfo(AchievementFrame_LoadUI)
            local toggleEnv = debug.getfenv(ToggleAchievementFrame)
            local loaded = C_AddOns.IsAddOnLoaded("Blizzard_AchievementUI")
            local shown = AchievementFrame and AchievementFrame:IsShown() or false
            local summary = string.format(
                "pre=%s:%d toggle=%s:%d load_ui=%s called_toggle=%s called_kiosk=%s called_completed=%s called_can_show=%s c_addons=%s c_addons_result=%s patch=%s toggle_env_is_global=%s toggle_env_completed=%s toggle_env_can_show=%s called_load_ui=%s called_uiparent=%s kiosk=%s disallow=%s completed=%s in_guild=%s can_show=%s",
                tostring({pre_toggle_source}),
                {pre_toggle_line},
                tostring(toggleInfo and toggleInfo.short_src or nil),
                toggleInfo and toggleInfo.linedefined or 0,
                tostring(loadUIInfo and loadUIInfo.short_src or nil),
                tostring(__achievement_toggle_called_toggle),
                tostring(__achievement_toggle_called_kiosk),
                tostring(__achievement_toggle_called_completed_any),
                tostring(__achievement_toggle_called_can_show),
                tostring(__achievement_toggle_called_c_addons_load_addon),
                tostring(__achievement_toggle_c_addons_load_result),
                tostring(__wow_toggle_achievement_patch_applied == true),
                tostring(toggleEnv == _G),
                tostring(toggleEnv and toggleEnv.HasCompletedAnyAchievement and toggleEnv.HasCompletedAnyAchievement() or false),
                tostring(toggleEnv and toggleEnv.CanShowAchievementUI and toggleEnv.CanShowAchievementUI() or false),
                tostring(__achievement_toggle_called_load_ui),
                tostring(__achievement_toggle_called_uiparent_load_addon),
                tostring(Kiosk.IsEnabled()),
                tostring(DISALLOW_FRAME_TOGGLING == true),
                tostring(HasCompletedAnyAchievement()),
                tostring(IsInGuild()),
                tostring(CanShowAchievementUI())
            )
            return loaded, shown, summary
            "#,
        pre_toggle_source = format!("{:?}", pre_toggle_source.as_deref().unwrap_or("<nil>")),
        pre_toggle_line = pre_toggle_line,
    );
    let (loaded, shown, summary): (bool, bool, String) = env
        .eval(&probe)
        .expect("achievement toggle probe should return");

    assert!(
        loaded,
        "achievement toggle should load the addon ({summary})"
    );
    assert!(
        shown,
        "achievement toggle should show the frame ({summary})"
    );
}

#[test]
fn collections_toggle_loads_and_shows_frame() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    let toggle_type: String = env
        .eval(r#"return type(ToggleCollectionsJournal)"#)
        .expect("collections toggle type probe should return");
    assert_eq!(
        toggle_type, "function",
        "ToggleCollectionsJournal should be registered before the toggle test runs"
    );

    env.exec("ToggleCollectionsJournal(COLLECTIONS_JOURNAL_TAB_INDEX_MOUNTS)")
        .expect("ToggleCollectionsJournal should execute");

    let (loaded, shown): (bool, bool) = env
        .eval(
            r#"return C_AddOns.IsAddOnLoaded("Blizzard_Collections"), CollectionsJournal and CollectionsJournal:IsShown() or false"#,
        )
        .expect("collections toggle probe should return");

    assert!(loaded, "collections toggle should load the addon");
    assert!(shown, "collections toggle should show the frame");
}

#[test]
fn encounter_toggle_loads_and_shows_frame() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    env.exec("ToggleEncounterJournal()")
        .expect("ToggleEncounterJournal should execute");

    let (loaded, shown): (bool, bool) = env
        .eval(
            r#"return C_AddOns.IsAddOnLoaded("Blizzard_EncounterJournal"), EncounterJournal and EncounterJournal:IsShown() or false"#,
        )
        .expect("encounter journal toggle probe should return");

    assert!(loaded, "encounter journal toggle should load the addon");
    assert!(shown, "encounter journal toggle should show the frame");
}

#[test]
fn adventure_journal_dungeon_action_opens_lfd_without_recursing() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);
    load_blizzard_addon(&env, "Blizzard_RaidWarning", "Blizzard_RaidWarning.toc");
    load_blizzard_addon(&env, "Blizzard_LFGUtil", "Blizzard_LFGUtil_Mainline.toc");
    load_blizzard_addon(
        &env,
        "Blizzard_GroupFinder",
        "Blizzard_GroupFinder_Mainline.toc",
    );

    env.fire_event_with_args("AJ_DUNGEON_ACTION", &[rilua::Val::Num(1201.0)])
        .expect("AJ_DUNGEON_ACTION should not overflow or error");

    let (shown, selected_lfd, dungeon_type): (bool, bool, i32) = env
        .eval(
            r#"
            return PVEFrame and PVEFrame:IsShown() or false,
                   GroupFinderFrame and GroupFinderFrame.selection == LFDParentFrame or false,
                   type(LFDQueueFrame.type) == "number" and LFDQueueFrame.type or 0
            "#,
        )
        .expect("LFD handoff probe should return");

    assert!(shown, "AJ_DUNGEON_ACTION should show the PVE frame");
    assert!(
        selected_lfd,
        "AJ_DUNGEON_ACTION should select the LFD parent frame"
    );
    assert_eq!(dungeon_type, 1201, "linked dungeon should become LFD type");
}

#[test]
fn encounter_journal_display_dungeon_instance_does_not_recurse() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    env.exec("ToggleEncounterJournal()")
        .expect("ToggleEncounterJournal should execute");
    env.exec("EncounterJournal_DisplayInstance(1271)")
        .expect("displaying a linked dungeon instance should not overflow or error");

    let (instance_id, encounter_shown): (i32, bool) = env
        .eval(
            r#"
            return EncounterJournal.instanceID or 0,
                   EncounterJournal.encounter and EncounterJournal.encounter:IsShown() or false
            "#,
        )
        .expect("display instance probe should return");

    assert_eq!(instance_id, 1271);
    assert!(
        encounter_shown,
        "dungeon display should show the encounter panel"
    );
}

#[test]
fn encounter_journal_journeys_frame_seeds_current_major_factions_without_tww_leak() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    load_panel_harness(&env);

    env.exec("ToggleEncounterJournal()")
        .expect("ToggleEncounterJournal should execute");

    let (count, names, expansion): (i32, String, i32) = env
        .eval(
            r#"
            local frame = EncounterJournal and EncounterJournal.JourneysFrame
            if not frame then
                return -1, "", -1
            end
            frame:Refresh()
            local out = {}
            for _, entry in ipairs(frame.renownJourneyData) do
                out[#out + 1] = entry.name
            end
            return #frame.renownJourneyData, table.concat(out, "|"), frame.expansionFilter or LE_EXPANSION_LEVEL_CURRENT
            "#,
        )
        .expect("Journeys frame probe should return");

    assert_eq!(expansion, 11, "current Journeys tier should be Midnight");
    assert_eq!(
        count, 4,
        "current Midnight Journeys should show the four major-faction rows: {names}"
    );
    assert!(
        names.contains("Silvermoon Court")
            && names.contains("Amani Tribe")
            && names.contains("Hara'ti")
            && names.contains("The Singularity"),
        "current Midnight Journeys should show current faction rows: {names}"
    );
    assert_eq!(
        names.contains("Hallowfall Arathi")
            || names.contains("Council of Dornogal")
            || names.contains("The Assembly of the Deeps")
            || names.contains("The Severed Threads"),
        false,
        "current Midnight Journeys must not show War Within rows; count={count}, names={names}"
    );
}
