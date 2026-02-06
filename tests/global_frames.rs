//! Tests for global frame registrations (global_frames.rs).
//!
//! Verifies that all expected global frame objects, tables, and utility
//! structures are properly registered in the Lua environment.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// Core Frame Globals
// ============================================================================

#[test]
fn test_uiparent_exists_and_is_frame() {
    let env = env();
    let obj_type: String = env.eval("return UIParent:GetObjectType()").unwrap();
    assert_eq!(obj_type, "Frame");
}

#[test]
fn test_world_frame_exists() {
    let env = env();
    let exists: bool = env.eval("return WorldFrame ~= nil").unwrap();
    assert!(exists, "WorldFrame should exist as a global");
    let obj_type: String = env.eval("return WorldFrame:GetObjectType()").unwrap();
    assert_eq!(obj_type, "Frame");
}

#[test]
fn test_minimap_exists() {
    let env = env();
    let exists: bool = env.eval("return Minimap ~= nil").unwrap();
    assert!(exists);
    // Without Blizzard_Minimap loaded, Minimap is a stub Frame.
    // The proper Minimap widget type comes from addon XML loading.
    let is_frame: bool = env.eval("return Minimap:IsObjectType('Frame')").unwrap();
    assert!(is_frame, "Minimap should be a Frame subtype");
}

#[test]
fn test_garrison_type_enum() {
    let env = env();
    let exists: bool = env.eval("return Enum ~= nil").unwrap();
    assert!(exists, "Enum should exist");
    let gt_exists: bool = env.eval("return Enum.GarrisonType ~= nil").unwrap();
    assert!(gt_exists, "Enum.GarrisonType should exist");
    let val: i32 = env.eval("return Enum.GarrisonType.Type_9_0_Garrison").unwrap();
    assert_eq!(val, 111);
}

#[test]
fn test_default_chat_frame_exists() {
    let env = env();
    let exists: bool = env.eval("return DEFAULT_CHAT_FRAME ~= nil").unwrap();
    assert!(exists, "DEFAULT_CHAT_FRAME should exist");
    let obj_type: String = env
        .eval("return DEFAULT_CHAT_FRAME:GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "MessageFrame");
}

#[test]
fn test_chat_frame1_exists() {
    let env = env();
    let exists: bool = env.eval("return ChatFrame1 ~= nil").unwrap();
    assert!(exists, "ChatFrame1 should exist");
}

// ============================================================================
// Unit Frame Globals
// ============================================================================

#[test]
fn test_player_frame_exists() {
    let env = env();
    let exists: bool = env.eval("return PlayerFrame ~= nil").unwrap();
    assert!(exists);
}

#[test]
fn test_target_frame_exists() {
    let env = env();
    let exists: bool = env.eval("return TargetFrame ~= nil").unwrap();
    assert!(exists);
}

#[test]
fn test_focus_frame_exists() {
    let env = env();
    let exists: bool = env.eval("return FocusFrame ~= nil").unwrap();
    assert!(exists);
}

#[test]
fn test_focus_frame_spell_bar_exists() {
    let env = env();
    let exists: bool = env.eval("return FocusFrameSpellBar ~= nil").unwrap();
    assert!(exists);
}

#[test]
fn test_pet_frame_exists() {
    let env = env();
    let exists: bool = env.eval("return PetFrame ~= nil").unwrap();
    assert!(exists);
}

#[test]
fn test_party_frame_exists() {
    let env = env();
    let exists: bool = env.eval("return PartyFrame ~= nil").unwrap();
    assert!(exists);
}

// ============================================================================
// Chat System Globals
// ============================================================================

#[test]
fn test_chat_type_group_structure() {
    let env = env();

    // System group has 5 entries
    let count: i32 = env.eval("return #ChatTypeGroup.SYSTEM").unwrap();
    assert_eq!(count, 5);
    let first: String = env.eval("return ChatTypeGroup.SYSTEM[1]").unwrap();
    assert_eq!(first, "SYSTEM");
    let last: String = env.eval("return ChatTypeGroup.SYSTEM[5]").unwrap();
    assert_eq!(last, "CHANNEL_NOTICE_USER");

    // Whisper group has 2 entries
    let count: i32 = env.eval("return #ChatTypeGroup.WHISPER").unwrap();
    assert_eq!(count, 2);

    // Raid group has 3 entries
    let count: i32 = env.eval("return #ChatTypeGroup.RAID").unwrap();
    assert_eq!(count, 3);
    let rw: String = env.eval("return ChatTypeGroup.RAID[3]").unwrap();
    assert_eq!(rw, "RAID_WARNING");

    // Guild group has 2 entries
    let count: i32 = env.eval("return #ChatTypeGroup.GUILD").unwrap();
    assert_eq!(count, 2);

    // BN_WHISPER group has 3 entries
    let count: i32 = env.eval("return #ChatTypeGroup.BN_WHISPER").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_chat_type_group_all_keys() {
    let env = env();
    let keys: String = env
        .eval(
            r#"
        local keys = {}
        for k, _ in pairs(ChatTypeGroup) do
            table.insert(keys, k)
        end
        table.sort(keys)
        return table.concat(keys, ",")
    "#,
        )
        .unwrap();
    assert_eq!(
        keys,
        "BN_WHISPER,CHANNEL,EMOTE,GUILD,INSTANCE_CHAT,PARTY,RAID,SAY,SYSTEM,WHISPER,YELL"
    );
}

#[test]
fn test_chat_frame_util_process_message() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local filtered, event, args = ChatFrameUtil.ProcessMessageEventFilters(nil, "CHAT_MSG_SAY", "hello")
        return tostring(filtered) .. "," .. event
    "#,
        )
        .unwrap();
    assert_eq!(result, "false,CHAT_MSG_SAY");
}

#[test]
fn test_chat_frame_util_get_window_name() {
    let env = env();
    let name: String = env
        .eval("return ChatFrameUtil.GetChatWindowName(1)")
        .unwrap();
    assert_eq!(name, "Chat Window 1");
}

// ============================================================================
// UI Panel Globals
// ============================================================================

#[test]
fn test_settings_panel_container_structure() {
    let env = env();
    let exists: bool = env
        .eval("return SettingsPanel.Container.SettingsList.ScrollBox.ScrollTarget ~= nil")
        .unwrap();
    assert!(exists, "SettingsPanel.Container deep structure should exist");

    let text: String = env
        .eval("return SettingsPanel.Container.SettingsList.Header.Title:GetText()")
        .unwrap();
    assert_eq!(text, "");
}

#[test]
fn test_objective_tracker_frame_header() {
    let env = env();
    let exists: bool = env
        .eval("return ObjectiveTrackerFrame.Header ~= nil")
        .unwrap();
    assert!(exists, "ObjectiveTrackerFrame.Header should exist");
    let btn_exists: bool = env
        .eval("return ObjectiveTrackerFrame.Header.MinimizeButton ~= nil")
        .unwrap();
    assert!(
        btn_exists,
        "ObjectiveTrackerFrame.Header.MinimizeButton should exist"
    );
}

#[test]
fn test_objective_tracker_manager_methods() {
    let env = env();
    // Verify key methods exist and are callable
    env.exec("ObjectiveTrackerManager:AssignModulesOrder({})")
        .unwrap();
    let has_modules: bool = env
        .eval("return ObjectiveTrackerManager:HasAnyModules()")
        .unwrap();
    assert!(!has_modules);
    env.exec("ObjectiveTrackerManager:UpdateAll()").unwrap();
    let container = env
        .eval::<mlua::Value>("return ObjectiveTrackerManager:GetContainerForModule(nil)")
        .unwrap();
    assert!(matches!(container, mlua::Value::Nil));
}

// ============================================================================
// World Map Globals
// ============================================================================

#[test]
fn test_world_map_frame_pin_pools() {
    let env = env();
    let is_table: bool = env
        .eval("return type(WorldMapFrame.pinPools) == 'table'")
        .unwrap();
    assert!(is_table, "WorldMapFrame.pinPools should be a table");
}

#[test]
fn test_world_map_frame_overlay_frames() {
    let env = env();
    let is_table: bool = env
        .eval("return type(WorldMapFrame.overlayFrames) == 'table'")
        .unwrap();
    assert!(is_table, "WorldMapFrame.overlayFrames should be a table");
}

// ============================================================================
// Misc Frame Globals
// ============================================================================

#[test]
fn test_buff_frame_aura_container_icon_scale() {
    let env = env();
    let scale: f64 = env
        .eval("return BuffFrame.AuraContainer.iconScale")
        .unwrap();
    assert!((scale - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_addon_compartment_frame_methods() {
    let env = env();
    // RegisterAddon and UnregisterAddon should be callable without error
    env.exec("AddonCompartmentFrame:RegisterAddon()").unwrap();
    env.exec("AddonCompartmentFrame:UnregisterAddon()").unwrap();
    let is_table: bool = env
        .eval("return type(AddonCompartmentFrame.registeredAddons) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_lfg_list_frame_search_panel() {
    let env = env();
    let exists: bool = env
        .eval("return LFGListFrame.SearchPanel ~= nil")
        .unwrap();
    assert!(exists, "LFGListFrame.SearchPanel should exist");
    let box_exists: bool = env
        .eval("return LFGListFrame.SearchPanel.SearchBox ~= nil")
        .unwrap();
    assert!(box_exists, "LFGListFrame.SearchPanel.SearchBox should exist");
}

#[test]
fn test_alert_frame_sub_systems() {
    let env = env();
    let is_table: bool = env
        .eval("return type(AlertFrame.alertFrameSubSystems) == 'table'")
        .unwrap();
    assert!(is_table, "AlertFrame.alertFrameSubSystems should be a table");
}

#[test]
fn test_container_frame_container_has_container_frames() {
    let env = env();
    let is_table: bool = env
        .eval("return type(ContainerFrameContainer.ContainerFrames) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_party_member_frame_pool() {
    let env = env();
    // EnumerateActive returns an iterator that yields nil immediately
    let count: i32 = env
        .eval(
            r#"
        local n = 0
        for _ in PartyMemberFramePool:EnumerateActive() do
            n = n + 1
        end
        return n
    "#,
        )
        .unwrap();
    assert_eq!(count, 0);

    let num: i32 = env
        .eval("return PartyMemberFramePool:GetNumActive()")
        .unwrap();
    assert_eq!(num, 0);
}

// ============================================================================
// Table Globals
// ============================================================================

#[test]
fn test_ui_special_frames_exists() {
    let env = env();
    let is_table: bool = env
        .eval("return type(UISpecialFrames) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_static_popup_dialogs_exists() {
    let env = env();
    let is_table: bool = env
        .eval("return type(StaticPopupDialogs) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_ui_panel_windows_exists() {
    let env = env();
    let is_table: bool = env
        .eval("return type(UIPanelWindows) == 'table'")
        .unwrap();
    assert!(is_table);
}

// ============================================================================
// Remaining Named Frame Globals
// ============================================================================

#[test]
fn test_all_named_frame_globals_exist() {
    let env = env();
    let frame_names = [
        "EventToastManagerFrame",
        "EditModeManagerFrame",
        "RolePollPopup",
        "TimerTracker",
        "TargetFrameSpellBar",
        "MinimapCluster",
        "ObjectiveTrackerFrame",
        "PlayerCastingBarFrame",
        "AlternatePowerBar",
        "MonkStaggerBar",
        "NamePlateDriverFrame",
        "UIErrorsFrame",
        "InterfaceOptionsFrame",
        "AuctionHouseFrame",
        "SideDressUpFrame",
        "ContainerFrameCombinedBags",
        "LootFrame",
        "ScenarioObjectiveTracker",
        "RaidWarningFrame",
        "GossipFrame",
        "FriendsFrame",
    ];
    for name in &frame_names {
        let exists: bool = env
            .eval(&format!("return {} ~= nil", name))
            .unwrap();
        assert!(exists, "{} should exist as a global", name);
    }
}
