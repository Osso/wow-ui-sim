//! Built-in WoW frames created at startup.
//!
//! Only frames that are truly engine-created (UIParent, WorldFrame) or stubs
//! for addons not yet in the BLIZZARD_ADDONS loading list belong here.
//! Frames from loaded addons are created by the XML loader and should NOT
//! be duplicated here — doing so creates orphan ghosts in the widget registry.

use crate::widget::{Frame, WidgetRegistry, WidgetType};

// ---------------------------------------------------------------------------
// Helpers
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
// Public entry point
// ---------------------------------------------------------------------------

/// Create built-in WoW frames.
///
/// Only includes engine-created frames and stubs for not-yet-loaded addons.
/// Frames from loaded Blizzard addons (Blizzard_FrameXML, Blizzard_UIPanels_Game,
/// Blizzard_EditMode, Blizzard_Settings_Shared, Blizzard_Minimap, etc.) are
/// created by the XML loader and must NOT be duplicated here.
pub fn create_builtin_frames(widgets: &mut WidgetRegistry) {
    let ui_parent_id = create_engine_frames(widgets);

    // Stubs for addons not yet in BLIZZARD_ADDONS:
    create_world_map_frame(widgets, ui_parent_id); // Blizzard_WorldMap
    create_player_frame(widgets, ui_parent_id);     // Blizzard_UnitFrame
    create_target_frame(widgets, ui_parent_id);     // Blizzard_UnitFrame
    create_focus_frame(widgets, ui_parent_id);      // Blizzard_UnitFrame
    create_buff_frame(widgets, ui_parent_id);       // Blizzard_BuffFrame
    create_pet_frame(widgets, ui_parent_id);        // Blizzard_UnitFrame
    create_stub_frames(widgets, ui_parent_id);      // Various not-loaded addons
}

// ---------------------------------------------------------------------------
// Engine-created frames (no XML definition, or must exist before XML loads)
// ---------------------------------------------------------------------------

fn create_engine_frames(widgets: &mut WidgetRegistry) -> u64 {
    let ui_parent_id = register_frame(
        widgets,
        WidgetType::Frame,
        "UIParent",
        None,
        Some((500.0, 375.0)),
    );

    // WorldFrame (3D world rendering area, same level as UIParent)
    register_frame(widgets, WidgetType::Frame, "WorldFrame", None, Some((500.0, 375.0)));

    // Chat frames (Blizzard_ChatFrameBase not loaded)
    register_frame(widgets, WidgetType::MessageFrame, "DEFAULT_CHAT_FRAME", Some(ui_parent_id), Some((430.0, 120.0)));
    register_frame(widgets, WidgetType::MessageFrame, "ChatFrame1", Some(ui_parent_id), Some((430.0, 120.0)));

    ui_parent_id
}

// ---------------------------------------------------------------------------
// Stubs: Blizzard_WorldMap (not loaded)
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

    let mm_id = register_frame(widgets, WidgetType::Frame, "WorldMapMaximizeMinimizeFrame", Some(border_id), Some((32.0, 32.0)));
    let max_id = register_frame(widgets, WidgetType::Button, "WorldMapMaximizeButton", Some(mm_id), None);
    let min_id = register_frame(widgets, WidgetType::Button, "WorldMapMinimizeButton", Some(mm_id), None);

    link_child(widgets, border_id, "MaximizeMinimizeFrame", mm_id);
    link_child(widgets, mm_id, "MaximizeButton", max_id);
    link_child(widgets, mm_id, "MinimizeButton", min_id);
}

// ---------------------------------------------------------------------------
// Stubs: Blizzard_UnitFrame (not loaded)
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

fn create_target_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let tf_id = register_frame(widgets, WidgetType::Frame, "TargetFrame", Some(ui_parent_id), Some((175.0, 76.0)));
    let tfc_id = register_frame(widgets, WidgetType::Frame, "TargetFrameContent", Some(tf_id), Some((175.0, 76.0)));
    let tfcm_id = register_frame(widgets, WidgetType::Frame, "TargetFrameContentMain", Some(tfc_id), Some((175.0, 76.0)));

    link_child(widgets, tf_id, "TargetFrameContent", tfc_id);
    link_child(widgets, tfc_id, "TargetFrameContentMain", tfcm_id);

    let mana_bar_id = create_target_bars(widgets, tfcm_id);
    link_child(widgets, tfcm_id, "ManaBar", mana_bar_id);

    create_target_tot_and_spellbar(widgets, tf_id);
}

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
    let tot_id = register_frame(widgets, WidgetType::Frame, "TargetFrameToTFrame", Some(target_frame_id), Some((80.0, 30.0)));
    let tot_hb_id = register_frame(widgets, WidgetType::StatusBar, "TargetFrameToTHealthBar", Some(tot_id), None);

    link_child(widgets, target_frame_id, "totFrame", tot_id);
    link_child(widgets, tot_id, "HealthBar", tot_hb_id);

    register_frame(widgets, WidgetType::StatusBar, "TargetFrameSpellBar", Some(target_frame_id), Some((150.0, 16.0)));
}

fn create_focus_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let ff_id = register_frame(widgets, WidgetType::Frame, "FocusFrame", Some(ui_parent_id), Some((175.0, 76.0)));
    let ffc_id = register_frame(widgets, WidgetType::Frame, "FocusFrameContent", Some(ff_id), Some((175.0, 76.0)));
    let ffcm_id = register_frame(widgets, WidgetType::Frame, "FocusFrameContentMain", Some(ffc_id), Some((175.0, 76.0)));

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

    register_frame(widgets, WidgetType::StatusBar, "FocusFrameSpellBar", Some(focus_frame_id), Some((150.0, 16.0)));
}

// ---------------------------------------------------------------------------
// Stubs: Blizzard_BuffFrame (not loaded)
// ---------------------------------------------------------------------------

fn create_buff_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let bf_id = register_frame(widgets, WidgetType::Frame, "BuffFrame", Some(ui_parent_id), Some((300.0, 100.0)));
    let ac_id = register_frame(widgets, WidgetType::Frame, "BuffFrameAuraContainer", Some(bf_id), Some((300.0, 100.0)));
    link_child(widgets, bf_id, "AuraContainer", ac_id);
}

// ---------------------------------------------------------------------------
// Stubs: Blizzard_UnitFrame (not loaded) - PetFrame
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
// Simple stubs: frames from various not-loaded addons (no children needed)
// ---------------------------------------------------------------------------

fn create_stub_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // Blizzard_ObjectiveTracker (not loaded)
    register_frame(widgets, WidgetType::Frame, "ObjectiveTrackerFrame", Some(ui_parent_id), Some((248.0, 600.0)));
    register_frame(widgets, WidgetType::Frame, "ScenarioObjectiveTracker", Some(ui_parent_id), None);

    // Blizzard_GroupFinder (not loaded)
    register_hidden_frame(widgets, WidgetType::Frame, "LFGListFrame", Some(ui_parent_id), Some((400.0, 500.0)));
    register_frame(widgets, WidgetType::Frame, "LFGEventFrame", Some(ui_parent_id), None);

    // Blizzard_UnitFrame (not loaded) - simple frames without children
    register_frame(widgets, WidgetType::Frame, "PartyFrame", Some(ui_parent_id), Some((200.0, 400.0)));
    register_frame(widgets, WidgetType::StatusBar, "AlternatePowerBar", Some(ui_parent_id), None);
    register_frame(widgets, WidgetType::StatusBar, "MonkStaggerBar", Some(ui_parent_id), None);

    // Blizzard_NamePlates (not loaded)
    register_frame(widgets, WidgetType::Frame, "NamePlateDriverFrame", Some(ui_parent_id), None);

    // Blizzard_AuctionHouseUI (not loaded)
    register_frame(widgets, WidgetType::Frame, "AuctionHouseFrame", Some(ui_parent_id), None);

    // Engine-only (no XML definition)
    register_frame(widgets, WidgetType::Frame, "InterfaceOptionsFrame", Some(ui_parent_id), None);
}
