//! Built-in WoW frames created at startup.
//!
//! Only frames that are truly engine-created (UIParent, WorldFrame) or stubs
//! for addons not yet in the BLIZZARD_ADDONS loading list belong here.
//! Frames from loaded addons are created by the XML loader and should NOT
//! be duplicated here — doing so creates orphan ghosts in the widget registry.

use crate::widget::{AttributeValue, Frame, WidgetRegistry, WidgetType};

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

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Create built-in WoW frames.
///
/// Only includes engine-created frames and stubs for not-yet-loaded addons.
/// Frames from loaded Blizzard addons (Blizzard_FrameXML, Blizzard_UIPanels_Game,
/// Blizzard_EditMode, Blizzard_Settings_Shared, Blizzard_Minimap, etc.) are
/// created by the XML loader and must NOT be duplicated here.
pub fn create_builtin_frames(widgets: &mut WidgetRegistry, screen_width: f32, screen_height: f32) {
    let ui_parent_id = create_engine_frames(widgets, screen_width, screen_height);

    // Stubs for addons not yet in BLIZZARD_ADDONS:
    create_buff_frame(widgets, ui_parent_id);       // Blizzard_BuffFrame
    create_debuff_frame(widgets, ui_parent_id);     // Blizzard_BuffFrame (referenced by Blizzard_UnitFrame)
    create_stub_frames(widgets, ui_parent_id);      // Various not-loaded addons
}

// ---------------------------------------------------------------------------
// Engine-created frames (no XML definition, or must exist before XML loads)
// ---------------------------------------------------------------------------

fn create_engine_frames(widgets: &mut WidgetRegistry, screen_width: f32, screen_height: f32) -> u64 {
    let ui_parent_id = register_frame(
        widgets,
        WidgetType::Frame,
        "UIParent",
        None,
        Some((screen_width, screen_height)),
    );

    // Set UIParent panel attributes (from UIParent.xml <Attributes>).
    // Must be present before UIParentPanelManager loads, which reads
    // them via GetAttribute during SetAttribute callbacks.
    if let Some(frame) = widgets.get_mut(ui_parent_id) {
        let attrs = &mut frame.attributes;
        attrs.insert("DEFAULT_FRAME_WIDTH".into(), AttributeValue::Number(384.0));
        attrs.insert("TOP_OFFSET".into(), AttributeValue::Number(-116.0));
        attrs.insert("LEFT_OFFSET".into(), AttributeValue::Number(16.0));
        attrs.insert("CENTER_OFFSET".into(), AttributeValue::Number(384.0));
        attrs.insert("RIGHT_OFFSET".into(), AttributeValue::Number(768.0));
        attrs.insert("RIGHT_OFFSET_BUFFER".into(), AttributeValue::Number(80.0));
        attrs.insert("PANEl_SPACING_X".into(), AttributeValue::Number(32.0));
    }

    // WorldFrame (3D world rendering area, same level as UIParent)
    register_frame(widgets, WidgetType::Frame, "WorldFrame", None, Some((screen_width, screen_height)));

    // DEFAULT_CHAT_FRAME fallback (overwritten by show_chat_frame workaround when chat addons load)
    register_frame(widgets, WidgetType::MessageFrame, "DEFAULT_CHAT_FRAME", Some(ui_parent_id), Some((430.0, 120.0)));

    ui_parent_id
}

// ---------------------------------------------------------------------------
// Stubs: Blizzard_BuffFrame (not loaded)
// ---------------------------------------------------------------------------

fn create_buff_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let bf_id = register_frame(widgets, WidgetType::Frame, "BuffFrame", Some(ui_parent_id), Some((300.0, 100.0)));
    let ac_id = register_frame(widgets, WidgetType::Frame, "BuffFrameAuraContainer", Some(bf_id), Some((300.0, 100.0)));
    link_child(widgets, bf_id, "AuraContainer", ac_id);
}

fn create_debuff_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    let df_id = register_frame(widgets, WidgetType::Frame, "DebuffFrame", Some(ui_parent_id), Some((300.0, 100.0)));
    let ac_id = register_frame(widgets, WidgetType::Frame, "DebuffFrameAuraContainer", Some(df_id), Some((300.0, 100.0)));
    link_child(widgets, df_id, "AuraContainer", ac_id);
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

    // Blizzard_NamePlates (not loaded)
    register_frame(widgets, WidgetType::Frame, "NamePlateDriverFrame", Some(ui_parent_id), None);

    // Blizzard_AuctionHouseUI (not loaded)
    register_frame(widgets, WidgetType::Frame, "AuctionHouseFrame", Some(ui_parent_id), None);

    // Engine-only (no XML definition)
    register_frame(widgets, WidgetType::Frame, "InterfaceOptionsFrame", Some(ui_parent_id), None);
}
