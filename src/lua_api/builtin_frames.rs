//! Built-in WoW frames created at startup.

use crate::widget::{Frame, WidgetRegistry, WidgetType};

// ---------------------------------------------------------------------------
// Helpers: reduce repetition across all frame creation functions
// ---------------------------------------------------------------------------

/// Create a named frame with optional size, register it, and return its ID.
fn register_frame(
    widgets: &mut WidgetRegistry,
    widget_type: WidgetType,
    name: &str,
    parent: Option<u64>,
    size: Option<(f32, f32)>,
) -> u64 {
    let mut frame = Frame::new(widget_type, Some(name.to_string()), parent);
    if let Some((w, h)) = size {
        frame.width = w;
        frame.height = h;
    }
    widgets.register(frame)
}

/// Create a named frame that starts hidden, register it, and return its ID.
fn register_hidden_frame(
    widgets: &mut WidgetRegistry,
    widget_type: WidgetType,
    name: &str,
    parent: Option<u64>,
    size: Option<(f32, f32)>,
) -> u64 {
    let mut frame = Frame::new(widget_type, Some(name.to_string()), parent);
    if let Some((w, h)) = size {
        frame.width = w;
        frame.height = h;
    }
    frame.visible = false;
    widgets.register(frame)
}

/// Insert a child key into a parent frame's children_keys map.
fn link_child(widgets: &mut WidgetRegistry, parent_id: u64, key: &str, child_id: u64) {
    if let Some(parent) = widgets.get_mut(parent_id) {
        parent.children_keys.insert(key.to_string(), child_id);
    }
}

/// Register an anonymous FontString child under the given parent.
fn register_fontstring(widgets: &mut WidgetRegistry, parent: u64) -> u64 {
    let fs = Frame::new(WidgetType::FontString, None, Some(parent));
    widgets.register(fs)
}

/// Register an anonymous FontString with explicit size under the given parent.
fn register_sized_fontstring(
    widgets: &mut WidgetRegistry,
    parent: u64,
    w: f32,
    h: f32,
) -> u64 {
    let mut fs = Frame::new(WidgetType::FontString, None, Some(parent));
    fs.width = w;
    fs.height = h;
    widgets.register(fs)
}

/// Create three standard text children (LeftText, RightText, TextString) on a bar
/// and link them via children_keys. The third key is configurable.
fn add_bar_text_children(
    widgets: &mut WidgetRegistry,
    bar_id: u64,
    third_key: &str,
) {
    let left_id = register_fontstring(widgets, bar_id);
    let right_id = register_fontstring(widgets, bar_id);
    let text_id = register_fontstring(widgets, bar_id);
    link_child(widgets, bar_id, "LeftText", left_id);
    link_child(widgets, bar_id, "RightText", right_id);
    link_child(widgets, bar_id, third_key, text_id);
}

// ---------------------------------------------------------------------------
// create_builtin_frames  (was 93 lines -> orchestrator + helper)
// ---------------------------------------------------------------------------

/// Create all built-in WoW frames (UIParent, WorldFrame, unit frames, etc.).
pub fn create_builtin_frames(widgets: &mut WidgetRegistry) {
    let ui_parent_id = create_root_frames(widgets);

    create_world_map_frame(widgets, ui_parent_id);
    create_player_frame(widgets, ui_parent_id);
    create_target_frame(widgets, ui_parent_id);
    create_focus_frame(widgets, ui_parent_id);
    create_buff_frame(widgets, ui_parent_id);
    create_pet_frame(widgets, ui_parent_id);
    create_misc_frames(widgets, ui_parent_id);
}

/// Create UIParent, WorldFrame, chat frames, and other top-level singletons.
/// Returns the UIParent id so callers can parent child frames to it.
fn create_root_frames(widgets: &mut WidgetRegistry) -> u64 {
    let ui_parent_id = register_frame(
        widgets,
        WidgetType::Frame,
        "UIParent",
        None,
        Some((500.0, 375.0)),
    );

    // Minimap (first registration - there is a second in create_misc_frames)
    register_frame(widgets, WidgetType::Frame, "Minimap", Some(ui_parent_id), None);

    // WorldFrame (3D world rendering area, same level as UIParent)
    register_frame(widgets, WidgetType::Frame, "WorldFrame", None, Some((500.0, 375.0)));

    // Chat frames
    register_frame(widgets, WidgetType::MessageFrame, "DEFAULT_CHAT_FRAME", Some(ui_parent_id), Some((430.0, 120.0)));
    register_frame(widgets, WidgetType::MessageFrame, "ChatFrame1", Some(ui_parent_id), Some((430.0, 120.0)));

    // Event / UI management frames
    register_frame(widgets, WidgetType::Frame, "EventToastManagerFrame", Some(ui_parent_id), Some((300.0, 100.0)));
    register_frame(widgets, WidgetType::Frame, "EditModeManagerFrame", Some(ui_parent_id), Some((400.0, 300.0)));
    register_frame(widgets, WidgetType::Frame, "RolePollPopup", Some(ui_parent_id), Some((200.0, 150.0)));
    register_frame(widgets, WidgetType::Frame, "TimerTracker", Some(ui_parent_id), Some((200.0, 50.0)));

    ui_parent_id
}

// ---------------------------------------------------------------------------
// create_world_map_frame  (was 77 lines -> orchestrator + helper)
// ---------------------------------------------------------------------------

fn create_world_map_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let wm_id = register_hidden_frame(
        widgets,
        WidgetType::Frame,
        "WorldMapFrame",
        Some(ui_parent_id),
        Some((1024.0, 768.0)),
    );

    let border_id = register_frame(widgets, WidgetType::Frame, "WorldMapBorderFrame", Some(wm_id), Some((1024.0, 768.0)));
    let scroll_id = register_frame(widgets, WidgetType::ScrollFrame, "WorldMapScrollContainer", Some(wm_id), None);

    link_child(widgets, wm_id, "BorderFrame", border_id);
    link_child(widgets, wm_id, "ScrollContainer", scroll_id);

    create_world_map_maximize_minimize(widgets, border_id);
}

fn create_world_map_maximize_minimize(widgets: &mut WidgetRegistry, border_id: u64) {
    let mm_id = register_frame(
        widgets,
        WidgetType::Frame,
        "WorldMapMaximizeMinimizeFrame",
        Some(border_id),
        Some((32.0, 32.0)),
    );
    let max_id = register_frame(widgets, WidgetType::Button, "WorldMapMaximizeButton", Some(mm_id), None);
    let min_id = register_frame(widgets, WidgetType::Button, "WorldMapMinimizeButton", Some(mm_id), None);

    link_child(widgets, border_id, "MaximizeMinimizeFrame", mm_id);
    link_child(widgets, mm_id, "MaximizeButton", max_id);
    link_child(widgets, mm_id, "MinimizeButton", min_id);
}

// ---------------------------------------------------------------------------
// create_player_frame  (was 131 lines -> orchestrator + 2 helpers)
// ---------------------------------------------------------------------------

fn create_player_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let pf_id = register_frame(widgets, WidgetType::Frame, "PlayerFrame", Some(ui_parent_id), Some((175.0, 76.0)));
    let pfc_id = register_frame(widgets, WidgetType::Frame, "PlayerFrameContent", Some(pf_id), Some((175.0, 76.0)));
    let pfcm_id = register_frame(widgets, WidgetType::Frame, "PlayerFrameContentMain", Some(pfc_id), Some((175.0, 76.0)));

    link_child(widgets, pf_id, "PlayerFrameContent", pfc_id);
    link_child(widgets, pfc_id, "PlayerFrameContentMain", pfcm_id);

    create_player_health_bar(widgets, pfcm_id);
    create_player_mana_bar(widgets, pfcm_id);
}

fn create_player_health_bar(widgets: &mut WidgetRegistry, content_main_id: u64) {
    let hbc_id = register_frame(widgets, WidgetType::Frame, "PlayerFrameHealthBarsContainer", Some(content_main_id), Some((120.0, 20.0)));
    let hb_id = register_frame(widgets, WidgetType::StatusBar, "PlayerFrameHealthBar", Some(hbc_id), Some((120.0, 20.0)));

    link_child(widgets, content_main_id, "HealthBarsContainer", hbc_id);
    link_child(widgets, hbc_id, "HealthBar", hb_id);

    add_bar_text_children(widgets, hb_id, "TextString");
}

fn create_player_mana_bar(widgets: &mut WidgetRegistry, content_main_id: u64) {
    let mba_id = register_frame(widgets, WidgetType::Frame, "PlayerFrameManaBarArea", Some(content_main_id), Some((120.0, 12.0)));
    let mb_id = register_frame(widgets, WidgetType::StatusBar, "PlayerFrameManaBar", Some(mba_id), Some((120.0, 12.0)));

    link_child(widgets, content_main_id, "ManaBarArea", mba_id);
    link_child(widgets, mba_id, "ManaBar", mb_id);

    add_bar_text_children(widgets, mb_id, "ManaBarText");
}

// ---------------------------------------------------------------------------
// create_target_frame  (was 128 lines -> orchestrator + 2 helpers)
// ---------------------------------------------------------------------------

fn create_target_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let tf_id = register_frame(widgets, WidgetType::Frame, "TargetFrame", Some(ui_parent_id), Some((175.0, 76.0)));
    let tfc_id = register_frame(widgets, WidgetType::Frame, "TargetFrameContent", Some(tf_id), Some((175.0, 76.0)));
    let tfcm_id = register_frame(widgets, WidgetType::Frame, "TargetFrameContentMain", Some(tfc_id), Some((175.0, 76.0)));

    link_child(widgets, tf_id, "TargetFrameContent", tfc_id);
    link_child(widgets, tfc_id, "TargetFrameContentMain", tfcm_id);

    let mana_bar_id = create_target_bars(widgets, tfcm_id);
    // Some addons access ManaBar directly on ContentMain
    link_child(widgets, tfcm_id, "ManaBar", mana_bar_id);

    create_target_tot_and_spellbar(widgets, tf_id);
}

/// Create health and mana bars for the target frame. Returns the mana bar ID
/// so it can also be linked directly on ContentMain.
fn create_target_bars(widgets: &mut WidgetRegistry, content_main_id: u64) -> u64 {
    let hbc_id = register_frame(widgets, WidgetType::Frame, "TargetFrameHealthBarsContainer", Some(content_main_id), Some((120.0, 20.0)));
    let hb_id = register_frame(widgets, WidgetType::StatusBar, "TargetFrameHealthBar", Some(hbc_id), Some((120.0, 20.0)));

    link_child(widgets, content_main_id, "HealthBarsContainer", hbc_id);
    link_child(widgets, hbc_id, "HealthBar", hb_id);

    let mba_id = register_frame(widgets, WidgetType::Frame, "TargetFrameManaBarArea", Some(content_main_id), Some((120.0, 12.0)));
    let mb_id = register_frame(widgets, WidgetType::StatusBar, "TargetFrameManaBar", Some(mba_id), Some((120.0, 12.0)));

    link_child(widgets, content_main_id, "ManaBarArea", mba_id);
    link_child(widgets, mba_id, "ManaBar", mb_id);

    mb_id
}

fn create_target_tot_and_spellbar(widgets: &mut WidgetRegistry, target_frame_id: u64) {
    // Target-of-target frame
    let tot_id = register_frame(widgets, WidgetType::Frame, "TargetFrameToTFrame", Some(target_frame_id), Some((80.0, 30.0)));
    let tot_hb_id = register_frame(widgets, WidgetType::StatusBar, "TargetFrameToTHealthBar", Some(tot_id), None);

    link_child(widgets, target_frame_id, "totFrame", tot_id);
    link_child(widgets, tot_id, "HealthBar", tot_hb_id);

    // Target cast bar
    register_frame(widgets, WidgetType::StatusBar, "TargetFrameSpellBar", Some(target_frame_id), Some((150.0, 16.0)));
}

// ---------------------------------------------------------------------------
// create_focus_frame  (was 112 lines -> orchestrator + 2 helpers)
// ---------------------------------------------------------------------------

fn create_focus_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let ff_id = register_frame(widgets, WidgetType::Frame, "FocusFrame", Some(ui_parent_id), Some((175.0, 76.0)));
    let ffc_id = register_frame(widgets, WidgetType::Frame, "FocusFrameContent", Some(ff_id), Some((175.0, 76.0)));
    let ffcm_id = register_frame(widgets, WidgetType::Frame, "FocusFrameContentMain", Some(ffc_id), Some((175.0, 76.0)));

    // FocusFrame uses "TargetFrameContent" as children_key name (matches WoW behavior)
    link_child(widgets, ff_id, "TargetFrameContent", ffc_id);
    link_child(widgets, ffc_id, "TargetFrameContentMain", ffcm_id);

    create_focus_bars(widgets, ffcm_id);
    create_focus_tot_and_spellbar(widgets, ff_id);
}

fn create_focus_bars(widgets: &mut WidgetRegistry, content_main_id: u64) {
    let hbc_id = register_frame(widgets, WidgetType::Frame, "FocusFrameHealthBarsContainer", Some(content_main_id), Some((120.0, 20.0)));
    let hb_id = register_frame(widgets, WidgetType::StatusBar, "FocusFrameHealthBar", Some(hbc_id), Some((120.0, 20.0)));

    link_child(widgets, content_main_id, "HealthBarsContainer", hbc_id);
    link_child(widgets, hbc_id, "HealthBar", hb_id);

    let mb_id = register_frame(widgets, WidgetType::StatusBar, "FocusFrameManaBar", Some(content_main_id), Some((120.0, 12.0)));
    link_child(widgets, content_main_id, "ManaBar", mb_id);
}

fn create_focus_tot_and_spellbar(widgets: &mut WidgetRegistry, focus_frame_id: u64) {
    let tot_id = register_frame(widgets, WidgetType::Frame, "FocusFrameToTFrame", Some(focus_frame_id), Some((80.0, 30.0)));
    let tot_hb_id = register_frame(widgets, WidgetType::StatusBar, "FocusFrameToTHealthBar", Some(tot_id), None);

    link_child(widgets, focus_frame_id, "totFrame", tot_id);
    link_child(widgets, tot_id, "HealthBar", tot_hb_id);

    // Focus cast bar
    register_frame(widgets, WidgetType::StatusBar, "FocusFrameSpellBar", Some(focus_frame_id), Some((150.0, 16.0)));
}

// ---------------------------------------------------------------------------
// create_buff_frame  (already under 50 lines, unchanged)
// ---------------------------------------------------------------------------

fn create_buff_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let bf_id = register_frame(widgets, WidgetType::Frame, "BuffFrame", Some(ui_parent_id), Some((300.0, 100.0)));
    let ac_id = register_frame(widgets, WidgetType::Frame, "BuffFrameAuraContainer", Some(bf_id), Some((300.0, 100.0)));
    link_child(widgets, bf_id, "AuraContainer", ac_id);
}

// ---------------------------------------------------------------------------
// create_pet_frame  (was 77 lines -> orchestrator + helper)
// ---------------------------------------------------------------------------

fn create_pet_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let pf_id = register_frame(widgets, WidgetType::Frame, "PetFrame", Some(ui_parent_id), Some((128.0, 53.0)));

    let hb_id = create_pet_health_bar(widgets, pf_id);
    let mb_id = create_pet_mana_bar(widgets, pf_id);

    link_child(widgets, pf_id, "healthbar", hb_id);
    link_child(widgets, pf_id, "manabar", mb_id);
}

fn create_pet_health_bar(widgets: &mut WidgetRegistry, pet_frame_id: u64) -> u64 {
    let hb_id = register_frame(widgets, WidgetType::StatusBar, "PetFrameHealthBar", Some(pet_frame_id), Some((90.0, 12.0)));

    let left_id = register_sized_fontstring(widgets, hb_id, 40.0, 12.0);
    let right_id = register_sized_fontstring(widgets, hb_id, 40.0, 12.0);
    let text_id = register_sized_fontstring(widgets, hb_id, 80.0, 12.0);

    link_child(widgets, hb_id, "LeftText", left_id);
    link_child(widgets, hb_id, "RightText", right_id);
    link_child(widgets, hb_id, "TextString", text_id);

    hb_id
}

fn create_pet_mana_bar(widgets: &mut WidgetRegistry, pet_frame_id: u64) -> u64 {
    let mb_id = register_frame(widgets, WidgetType::StatusBar, "PetFrameManaBar", Some(pet_frame_id), None);

    add_bar_text_children(widgets, mb_id, "TextString");

    mb_id
}

// ---------------------------------------------------------------------------
// create_misc_frames  (was 95 lines -> orchestrator + helper)
// ---------------------------------------------------------------------------

fn create_misc_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    create_minimap_and_tracker_frames(widgets, ui_parent_id);
    create_settings_panel(widgets, ui_parent_id);
    create_misc_bar_and_party_frames(widgets, ui_parent_id);
    create_lfg_frames(widgets, ui_parent_id);
    create_utility_frames(widgets, ui_parent_id);
}

fn create_minimap_and_tracker_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Second Minimap registration (matches original - duplicate)
    register_frame(widgets, WidgetType::Frame, "Minimap", Some(ui_parent_id), Some((140.0, 140.0)));
    register_frame(widgets, WidgetType::Frame, "MinimapCluster", Some(ui_parent_id), Some((192.0, 192.0)));
    register_frame(widgets, WidgetType::Frame, "ObjectiveTrackerFrame", Some(ui_parent_id), Some((248.0, 600.0)));
}

fn create_settings_panel(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let sp_id = register_hidden_frame(widgets, WidgetType::Frame, "SettingsPanel", Some(ui_parent_id), Some((800.0, 600.0)));
    let fc_id = register_frame(widgets, WidgetType::Frame, "SettingsPanelFrameContainer", Some(sp_id), Some((780.0, 580.0)));
    link_child(widgets, sp_id, "FrameContainer", fc_id);
}

fn create_misc_bar_and_party_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    register_frame(widgets, WidgetType::StatusBar, "PlayerCastingBarFrame", Some(ui_parent_id), Some((200.0, 20.0)));
    register_frame(widgets, WidgetType::Frame, "PartyFrame", Some(ui_parent_id), Some((200.0, 400.0)));
    register_frame(widgets, WidgetType::StatusBar, "AlternatePowerBar", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::StatusBar, "MonkStaggerBar", Some(ui_parent_id), None);
}

// ---------------------------------------------------------------------------
// create_lfg_frames  (already under 50 lines, unchanged logic)
// ---------------------------------------------------------------------------

fn create_lfg_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let lfg_id = register_hidden_frame(widgets, WidgetType::Frame, "LFGListFrame", Some(ui_parent_id), Some((400.0, 500.0)));
    let sp_id = register_frame(widgets, WidgetType::Frame, "LFGListSearchPanel", Some(lfg_id), Some((380.0, 450.0)));
    let sf_id = register_frame(widgets, WidgetType::ScrollFrame, "LFGListSearchPanelScrollFrame", Some(sp_id), Some((360.0, 400.0)));

    link_child(widgets, lfg_id, "SearchPanel", sp_id);
    link_child(widgets, sp_id, "ScrollFrame", sf_id);
}

// ---------------------------------------------------------------------------
// create_utility_frames  (was 131 lines -> orchestrator + 2 helpers)
// ---------------------------------------------------------------------------

fn create_utility_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    create_simple_utility_frames(widgets, ui_parent_id);
    create_loot_and_dialog_frames(widgets, ui_parent_id);
}

/// Simple utility frames that have no children_keys.
fn create_simple_utility_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    register_frame(widgets, WidgetType::Frame, "AlertFrame", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "LFGEventFrame", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "NamePlateDriverFrame", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::MessageFrame, "UIErrorsFrame", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "InterfaceOptionsFrame", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "AuctionHouseFrame", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "SideDressUpFrame", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "ContainerFrameContainer", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "ContainerFrameCombinedBags", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::Frame, "ScenarioObjectiveTracker", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::MessageFrame, "RaidWarningFrame", Some(ui_parent_id), None);
}

/// Utility frames that have children or specific dimensions.
fn create_loot_and_dialog_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // LootFrame with ScrollBox child
    let lf_id = register_frame(widgets, WidgetType::Frame, "LootFrame", Some(ui_parent_id), None);
    let sb_id = register_frame(widgets, WidgetType::ScrollFrame, "LootFrameScrollBox", Some(lf_id), None);
    link_child(widgets, lf_id, "ScrollBox", sb_id);

    // NPC dialog frames
    register_frame(widgets, WidgetType::Frame, "GossipFrame", Some(ui_parent_id), Some((338.0, 479.0)));
    register_frame(widgets, WidgetType::Frame, "QuestFrame", Some(ui_parent_id), Some((384.0, 512.0)));
}
