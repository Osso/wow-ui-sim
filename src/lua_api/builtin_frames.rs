//! Built-in WoW frames created at startup.
//!
//! Only frames that are truly engine-created (UIParent, WorldFrame) or stubs
//! for addons not yet in the BLIZZARD_ADDONS loading list belong here.
//! Frames from loaded addons are created by the XML loader and should NOT
//! be duplicated here — doing so creates orphan ghosts in the widget registry.

use crate::widget::{AttributeValue, Frame, FrameStrata, WidgetRegistry, WidgetType};

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
    owner: u16,
) -> u64 {
    register_builtin_frame(widgets, widget_type, name, parent, size, owner, true, None)
}

fn register_hidden_frame_with_strata(
    widgets: &mut WidgetRegistry,
    widget_type: WidgetType,
    name: &str,
    parent: Option<u64>,
    size: Option<(f32, f32)>,
    owner: u16,
    strata: FrameStrata,
) -> u64 {
    register_builtin_frame(
        widgets,
        widget_type,
        name,
        parent,
        size,
        owner,
        false,
        Some(strata),
    )
}

fn register_builtin_frame(
    widgets: &mut WidgetRegistry,
    widget_type: WidgetType,
    name: &str,
    parent: Option<u64>,
    size: Option<(f32, f32)>,
    owner: u16,
    visible: bool,
    fixed_strata: Option<FrameStrata>,
) -> u64 {
    let mut frame = Frame::new(widget_type, Some(name.to_string()), parent);
    frame.is_simulator_placeholder = true;
    frame.owner_addon = Some(owner);
    frame.visible = visible;
    if let Some(strata) = fixed_strata {
        frame.frame_strata = strata;
        frame.has_fixed_frame_strata = true;
    }
    if let Some((w, h)) = size {
        frame.width = w;
        frame.height = h;
    }
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
pub fn create_builtin_frames(
    widgets: &mut WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
    owner: u16,
) {
    let ui_parent_id = create_engine_frames(widgets, screen_width, screen_height, owner);

    // Stubs for addons not yet in BLIZZARD_ADDONS:
    create_buff_frame(widgets, ui_parent_id, owner); // Blizzard_BuffFrame
    create_debuff_frame(widgets, ui_parent_id, owner); // Blizzard_BuffFrame (referenced by Blizzard_UnitFrame)
    create_stub_frames(widgets, ui_parent_id, owner); // Various not-loaded addons
}

// ---------------------------------------------------------------------------
// Engine-created frames (no XML definition, or must exist before XML loads)
// ---------------------------------------------------------------------------

fn create_engine_frames(
    widgets: &mut WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
    o: u16,
) -> u64 {
    let ui_parent_id = create_ui_parent(widgets, screen_width, screen_height, o);
    create_engine_frame_children(widgets, ui_parent_id, screen_width, screen_height, o);

    ui_parent_id
}

fn create_engine_frame_children(
    widgets: &mut WidgetRegistry,
    ui_parent_id: u64,
    screen_width: f32,
    screen_height: f32,
    owner: u16,
) {
    set_ui_parent_panel_attributes(widgets, ui_parent_id);
    create_world_frame(widgets, screen_width, screen_height, owner);
    create_minimap_cluster(widgets, ui_parent_id, owner);
    create_tooltip_frames(widgets, ui_parent_id, owner);
    create_default_chat_frame(widgets, ui_parent_id, owner);
}

fn create_ui_parent(
    widgets: &mut WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
    owner: u16,
) -> u64 {
    register_frame(
        widgets,
        WidgetType::Frame,
        "UIParent",
        None,
        Some((screen_width, screen_height)),
        owner,
    )
}

fn create_world_frame(
    widgets: &mut WidgetRegistry,
    screen_width: f32,
    screen_height: f32,
    owner: u16,
) {
    register_frame(
        widgets,
        WidgetType::WorldFrame,
        "WorldFrame",
        None,
        Some((screen_width, screen_height)),
        owner,
    );
}

fn create_tooltip_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64, owner: u16) {
    register_standard_tooltips(widgets, ui_parent_id, owner);
    register_tooltip_container(widgets, ui_parent_id, owner);
    register_friends_tooltip(widgets, ui_parent_id, owner);
}

fn register_standard_tooltips(widgets: &mut WidgetRegistry, ui_parent_id: u64, owner: u16) {
    for name in STANDARD_TOOLTIP_FRAMES {
        register_standard_tooltip(widgets, ui_parent_id, owner, name);
    }
}

fn register_standard_tooltip(
    widgets: &mut WidgetRegistry,
    ui_parent_id: u64,
    owner: u16,
    name: &str,
) {
    register_hidden_frame_with_strata(
        widgets,
        WidgetType::GameTooltip,
        name,
        Some(ui_parent_id),
        Some((128.0, 64.0)),
        owner,
        FrameStrata::Tooltip,
    );
}

const STANDARD_TOOLTIP_FRAMES: &[&str] = &[
    "GameTooltip",
    "ShoppingTooltip1",
    "ShoppingTooltip2",
    "ItemRefShoppingTooltip1",
    "ItemRefShoppingTooltip2",
    "ItemRefTooltip",
];

fn register_tooltip_container(widgets: &mut WidgetRegistry, ui_parent_id: u64, owner: u16) {
    register_hidden_frame_with_strata(
        widgets,
        WidgetType::Frame,
        "GameTooltipDefaultContainer",
        Some(ui_parent_id),
        None,
        owner,
        FrameStrata::Low,
    );
}

fn register_friends_tooltip(widgets: &mut WidgetRegistry, ui_parent_id: u64, owner: u16) {
    register_hidden_frame_with_strata(
        widgets,
        WidgetType::Frame,
        "FriendsTooltip",
        Some(ui_parent_id),
        None,
        owner,
        FrameStrata::Tooltip,
    );
}

fn create_default_chat_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64, owner: u16) {
    // Overwritten by show_chat_frame workaround when chat addons load.
    register_frame(
        widgets,
        WidgetType::MessageFrame,
        "DEFAULT_CHAT_FRAME",
        Some(ui_parent_id),
        Some((430.0, 120.0)),
        owner,
    );
}

fn create_minimap_cluster(widgets: &mut WidgetRegistry, ui_parent_id: u64, owner: u16) {
    let minimap_cluster_id = register_frame(
        widgets,
        WidgetType::Frame,
        "MinimapCluster",
        Some(ui_parent_id),
        Some((256.0, 256.0)),
        owner,
    );
    let minimap_id = register_frame(
        widgets,
        WidgetType::Minimap,
        "Minimap",
        Some(minimap_cluster_id),
        Some((198.0, 198.0)),
        owner,
    );
    link_child(widgets, minimap_cluster_id, "Minimap", minimap_id);
}

fn set_ui_parent_panel_attributes(widgets: &mut WidgetRegistry, ui_parent_id: u64) {
    // From UIParent.xml <Attributes>. Must exist before UIParentPanelManager loads.
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
}

// ---------------------------------------------------------------------------
// Stubs: Blizzard_BuffFrame (not loaded)
// ---------------------------------------------------------------------------

fn create_buff_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64, o: u16) {
    let bf_id = register_frame(
        widgets,
        WidgetType::Frame,
        "BuffFrame",
        Some(ui_parent_id),
        Some((300.0, 100.0)),
        o,
    );
    let ac_id = register_frame(
        widgets,
        WidgetType::Frame,
        "BuffFrameAuraContainer",
        Some(bf_id),
        Some((300.0, 100.0)),
        o,
    );
    link_child(widgets, bf_id, "AuraContainer", ac_id);
}

fn create_debuff_frame(widgets: &mut WidgetRegistry, ui_parent_id: u64, o: u16) {
    let df_id = register_frame(
        widgets,
        WidgetType::Frame,
        "DebuffFrame",
        Some(ui_parent_id),
        Some((300.0, 100.0)),
        o,
    );
    let ac_id = register_frame(
        widgets,
        WidgetType::Frame,
        "DebuffFrameAuraContainer",
        Some(df_id),
        Some((300.0, 100.0)),
        o,
    );
    link_child(widgets, df_id, "AuraContainer", ac_id);
}

// ---------------------------------------------------------------------------
// Simple stubs: frames from various not-loaded addons (no children needed)
// ---------------------------------------------------------------------------

fn create_stub_frames(widgets: &mut WidgetRegistry, ui_parent_id: u64, o: u16) {
    register_stub_frame_specs(widgets, ui_parent_id, o, VISIBLE_STUB_FRAMES, true);
    register_stub_frame_specs(widgets, ui_parent_id, o, HIDDEN_STUB_FRAMES, false);
}

const VISIBLE_STUB_FRAMES: &[(&str, Option<(f32, f32)>)] = &[
    ("ObjectiveTrackerFrame", Some((248.0, 600.0))),
    ("ScenarioObjectiveTracker", None),
    ("LFGEventFrame", None),
    ("NamePlateDriverFrame", None),
    ("InterfaceOptionsFrame", None),
];

const HIDDEN_STUB_FRAMES: &[(&str, Option<(f32, f32)>)] = &[("LFGListFrame", Some((400.0, 500.0)))];

fn register_stub_frame_specs(
    widgets: &mut WidgetRegistry,
    ui_parent_id: u64,
    owner: u16,
    specs: &[(&str, Option<(f32, f32)>)],
    visible: bool,
) {
    for (name, size) in specs {
        register_builtin_frame(
            widgets,
            WidgetType::Frame,
            name,
            Some(ui_parent_id),
            *size,
            owner,
            visible,
            None,
        );
    }
}
