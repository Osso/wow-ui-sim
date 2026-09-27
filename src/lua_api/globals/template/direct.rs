//! Direct Rust property setters for frame creation.
//!
//! These functions bypass Lua compilation to set frame properties directly
//! on the Rust Frame struct, avoiding the ~30-50µs overhead per `lua.load().exec()`
//! call. Used during template application and XML frame loading.

use crate::lua_api::SimState;
mod frame_level;

pub use frame_level::{apply_xml_frame_level, apply_xml_frame_strata};

use crate::widget::AnchorPoint;
use crate::xml::{AnchorXml, FrameXml};
use std::cell::RefCell;
use std::rc::Rc;

/// Set a single anchor point on a frame.
pub(super) fn set_single_anchor(
    state: &mut SimState,
    frame_id: u64,
    anchor: &AnchorXml,
    frame_name: &str,
) {
    let Some((point, relative_point)) = anchor_points(anchor) else {
        return;
    };

    let (offset_x, offset_y) = anchor_offset(anchor);
    let relative_to_id = resolve_anchor_target(state, frame_id, anchor, frame_name);
    let unresolved_relative = unresolved_anchor_relative(anchor, relative_to_id);

    if anchor_would_create_cycle(state, frame_id, relative_to_id) {
        return;
    }

    update_anchor_dependents(state, frame_id, point, relative_to_id);
    set_frame_anchor(
        state,
        frame_id,
        point,
        relative_point,
        relative_to_id,
        unresolved_relative,
        (offset_x, offset_y),
    );
    state.widgets.mark_rect_dirty(frame_id);
}

fn unresolved_anchor_relative(anchor: &AnchorXml, relative_to_id: Option<u64>) -> Option<String> {
    if relative_to_id.is_some() {
        return None;
    }

    anchor
        .relative_key
        .as_ref()
        .cloned()
        .or_else(|| anchor.relative_to.as_ref().cloned())
}

fn anchor_would_create_cycle(state: &SimState, frame_id: u64, relative_to_id: Option<u64>) -> bool {
    let Some(rel_id) = relative_to_id else {
        return false;
    };

    state.widgets.would_create_anchor_cycle(frame_id, rel_id)
}

fn set_frame_anchor(
    state: &mut SimState,
    frame_id: u64,
    point: AnchorPoint,
    relative_point: AnchorPoint,
    relative_to_id: Option<u64>,
    unresolved_relative: Option<String>,
    offset: (f32, f32),
) {
    if let Some(frame) = state.widgets.get_mut_visual(frame_id) {
        let (offset_x, offset_y) = offset;
        if let Some(relative_expr) = unresolved_relative {
            frame.set_point_with_name(
                point,
                Some(relative_expr),
                relative_point,
                offset_x,
                offset_y,
            );
        } else {
            frame.set_point(
                point,
                relative_to_id.map(|id| id as usize),
                relative_point,
                offset_x,
                offset_y,
            );
        }
    }
}

fn anchor_points(anchor: &AnchorXml) -> Option<(AnchorPoint, AnchorPoint)> {
    let point_str = anchor.point.as_deref().unwrap_or("TOPLEFT");
    let relative_point_str = anchor.relative_point.as_deref().unwrap_or(point_str);
    let point = AnchorPoint::from_str(point_str)?;
    let relative_point = AnchorPoint::from_str(relative_point_str)?;
    Some((point, relative_point))
}

/// Resolve the relative_to target ID for an anchor element.
fn resolve_anchor_target(
    state: &SimState,
    frame_id: u64,
    anchor: &AnchorXml,
    frame_name: &str,
) -> Option<u64> {
    if anchor.relative_to.is_none() {
        if let Some(key) = anchor.relative_key.as_deref() {
            resolve_relative_key(state, frame_id, key)
        } else {
            resolve_relative_to(state, frame_id, None, frame_name)
        }
    } else {
        resolve_relative_to(state, frame_id, anchor.relative_to.as_deref(), frame_name)
    }
}

/// Remove old and add new anchor dependents for a point.
fn update_anchor_dependents(
    state: &mut SimState,
    frame_id: u64,
    point: AnchorPoint,
    new_target: Option<u64>,
) {
    if let Some(frame) = state.widgets.get(frame_id)
        && let Some(old_anchor) = frame.anchors.iter().find(|a| a.point == point)
        && let Some(old_target) = old_anchor.relative_to_id
    {
        state
            .widgets
            .remove_anchor_dependent(old_target as u64, frame_id);
    }
    if let Some(rel_id) = new_target {
        state.widgets.add_anchor_dependent(rel_id, frame_id);
    }
}

/// Resolve the relative_to target for an anchor, returning the target frame ID.
fn resolve_relative_to(
    state: &SimState,
    frame_id: u64,
    relative_to: Option<&str>,
    frame_name: &str,
) -> Option<u64> {
    match relative_to {
        Some("$parent") => state.widgets.get(frame_id).and_then(|f| f.parent_id),
        Some(rel) if rel.contains("$parent") || rel.contains("$Parent") => {
            // $parent in relativeTo refers to the anchoring frame's parent.
            // Derive from frame_id so we don't depend on callers threading the
            // correct parent name through every codepath (some runtime template
            // paths pass the frame's own name instead of its parent's).
            let parent_name = state
                .widgets
                .get(frame_id)
                .and_then(|f| f.parent_id)
                .and_then(|pid| state.widgets.get(pid))
                .and_then(|p| p.name.as_deref())
                .unwrap_or(frame_name);
            let resolved = rel
                .replace("$parent", parent_name)
                .replace("$Parent", parent_name);
            state.widgets.get_id_by_name(&resolved)
        }
        Some(rel) => state.widgets.get_id_by_name(rel),
        None => state.widgets.get(frame_id).and_then(|f| f.parent_id),
    }
}

/// Resolve a `relativeKey` expression like `$parent.HeroSpecButton` to a frame ID.
///
/// Supported patterns:
/// - `$parent` → parent frame
/// - `$parent.ChildKey` → child of parent with matching parentKey
/// - `$parent.ChildKey.GrandchildKey` → nested child lookup
fn resolve_relative_key(state: &SimState, frame_id: u64, key: &str) -> Option<u64> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() || parts[0] != "$parent" {
        return None;
    }
    let mut current_id = state.widgets.get(frame_id)?.parent_id?;
    for &segment in &parts[1..] {
        if segment == "$parent" || segment == "$Parent" || segment == "$parentKey" {
            current_id = state.widgets.get(current_id)?.parent_id?;
        } else {
            let frame = state.widgets.get(current_id)?;
            current_id = *frame.children_keys.get(segment)?;
        }
    }
    Some(current_id)
}

/// Inner: clear all points and fill parent with TOPLEFT+BOTTOMRIGHT anchors.
///
/// Matches Lua `SetAllPoints(true)`: stores `relative_to_id = None` (implicit parent)
/// and does NOT add anchor dependents (the layout system uses parent implicitly).
fn set_all_points_inner(state: &mut SimState, frame_id: u64) {
    // Remove old anchor dependents
    state.widgets.remove_all_anchor_dependents_for(frame_id);

    let parent_id = state.widgets.get(frame_id).and_then(|f| f.parent_id);
    if let Some(frame) = state.widgets.get_mut_visual(frame_id) {
        frame.clear_all_points();
        frame.set_point(
            AnchorPoint::TopLeft,
            parent_id.map(|p| p as usize),
            AnchorPoint::TopLeft,
            0.0,
            0.0,
        );
        frame.set_point(
            AnchorPoint::BottomRight,
            parent_id.map(|p| p as usize),
            AnchorPoint::BottomRight,
            0.0,
            0.0,
        );
    }

    state.widgets.mark_rect_dirty(frame_id);
}

/// Set frame alpha directly.
pub fn set_alpha(state: &Rc<RefCell<SimState>>, frame_id: u64, alpha: f32) {
    let clamped = alpha.clamp(0.0, 1.0);
    let mut s = state.borrow_mut();
    let parent_eff = s
        .widgets
        .get(frame_id)
        .and_then(|f| f.parent_id)
        .and_then(|pid| s.widgets.get(pid))
        .map(|p| p.effective_alpha)
        .unwrap_or(1.0);
    if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
        frame.alpha = clamped;
    }
    s.widgets.propagate_effective_alpha(frame_id, parent_eff);
}

/// Set frame scale directly.
pub fn set_scale(state: &Rc<RefCell<SimState>>, frame_id: u64, scale: f32) {
    if scale <= 0.0 {
        return;
    }
    let mut s = state.borrow_mut();
    let parent_eff = s
        .widgets
        .get(frame_id)
        .and_then(|f| f.parent_id)
        .and_then(|pid| s.widgets.get(pid))
        .map(|p| p.effective_scale)
        .unwrap_or(1.0);
    if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
        frame.scale = scale;
    }
    s.widgets.propagate_effective_scale(frame_id, parent_eff);
    s.widgets.mark_rect_dirty(frame_id);
}

/// Set top-level show ordering directly.
pub fn set_toplevel(state: &Rc<RefCell<SimState>>, frame_id: u64, toplevel: bool) {
    state.borrow_mut().set_frame_toplevel(frame_id, toplevel);
}

/// Set enableMouse directly.
pub fn enable_mouse(state: &Rc<RefCell<SimState>>, frame_id: u64, enable: bool) {
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut(frame_id) {
        frame.mouse_enabled = enable;
    }
}

/// Set hit rect insets directly.
pub fn set_hit_rect_insets(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
) {
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut(frame_id) {
        frame.hit_rect_insets = (left, right, top, bottom);
    }
}

/// Set clamped to screen directly.
pub fn set_clamped_to_screen(state: &Rc<RefCell<SimState>>, frame_id: u64, clamped: bool) {
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut(frame_id) {
        frame.clamped_to_screen = clamped;
    }
}

/// Set frame ID (from XML `id` attribute) directly.
pub fn set_id(state: &Rc<RefCell<SimState>>, frame_id: u64, id: i32) {
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut(frame_id) {
        frame.user_id = id;
    }
}

/// Extract offset values from an anchor XML element.
fn anchor_offset(anchor: &AnchorXml) -> (f32, f32) {
    if let Some(offset) = &anchor.offset {
        let abs = offset.abs_dimension.as_ref();
        if let Some(abs) = abs {
            (abs.x.unwrap_or(0.0), abs.y.unwrap_or(0.0))
        } else {
            (offset.x.unwrap_or(0.0), offset.y.unwrap_or(0.0))
        }
    } else {
        (anchor.x.unwrap_or(0.0), anchor.y.unwrap_or(0.0))
    }
}

// --- XML-path helpers (Phase 2): resolve properties from template chain + instance ---

/// Resolve and apply size from template chain + instance XML for the XML loading path.
pub fn apply_xml_size(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut final_width: Option<f32> = None;
    let mut final_height: Option<f32> = None;

    if !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            merge_size(&mut final_width, &mut final_height, entry.frame.size());
        }
    }
    merge_size(&mut final_width, &mut final_height, frame.size());

    if final_width.is_some() || final_height.is_some() {
        let mut s = state.borrow_mut();
        if let Some(f) = s.widgets.get_mut_visual(frame_id) {
            if let Some(width) = final_width {
                f.width = width;
                f.width_is_text_auto = false;
            }
            if let Some(height) = final_height {
                f.height = height;
            }
        }
        s.widgets.mark_rect_dirty(frame_id);
    }
}

/// Resolve and apply anchors from template chain + instance XML.
pub fn apply_xml_anchors(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
    parent_name: &str,
) {
    if let Some(anchors) = frame.anchors() {
        let mut s = state.borrow_mut();
        for anchor in &anchors.anchors {
            set_single_anchor(&mut s, frame_id, anchor, parent_name);
        }
    } else if !inherits.is_empty() {
        // No direct anchors — most derived template with anchors wins
        let chain = crate::xml::get_template_chain(inherits);
        for entry in chain.iter().rev() {
            if let Some(anchors) = entry.frame.anchors() {
                let mut s = state.borrow_mut();
                for anchor in &anchors.anchors {
                    set_single_anchor(&mut s, frame_id, anchor, parent_name);
                }
                break;
            }
        }
    }
}

/// Resolve and apply hidden from template chain + instance XML.
pub fn apply_xml_hidden(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut hidden = frame.hidden;
    if hidden.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(h) = entry.frame.hidden {
                hidden = Some(h);
                break;
            }
        }
    }
    if let Some(hidden) = hidden {
        state.borrow_mut().set_frame_visible(frame_id, !hidden);
    }
}

/// Resolve and apply toplevel from template chain + instance XML.
pub fn apply_xml_toplevel(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let toplevel = frame.toplevel.or_else(|| {
        if inherits.is_empty() {
            return None;
        }
        crate::xml::get_template_chain(inherits)
            .iter()
            .find_map(|e| e.frame.toplevel)
    });
    if toplevel == Some(true) {
        set_toplevel(state, frame_id, true);
    }
}

/// Resolve and apply alpha from template chain + instance XML.
pub fn apply_xml_alpha(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut alpha = frame.alpha;
    if alpha.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(a) = entry.frame.alpha {
                alpha = Some(a);
                break;
            }
        }
    }
    if let Some(a) = alpha {
        set_alpha(state, frame_id, a);
    }
}

/// Resolve and apply scale from template chain + instance XML.
pub fn apply_xml_scale(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut scale = frame.scale;
    if scale.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(s) = entry.frame.scale {
                scale = Some(s);
                break;
            }
        }
    }
    if let Some(s) = scale {
        set_scale(state, frame_id, s);
    }
}

/// Resolve and apply enableMouse from template chain + instance XML.
pub fn apply_xml_enable_mouse(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut em = frame.enable_mouse;
    if em.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(e) = entry.frame.enable_mouse {
                em = Some(e);
            }
        }
    }
    if let Some(enabled) = em {
        enable_mouse(state, frame_id, enabled);
    }
}

/// Resolve and apply enableKeyboard from template chain + instance XML.
pub fn apply_xml_enable_keyboard(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut enabled = frame.enable_keyboard;
    if enabled.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(template_enabled) = entry.frame.enable_keyboard {
                enabled = Some(template_enabled);
            }
        }
    }
    if let Some(enabled) = enabled
        && let Some(frame) = state.borrow_mut().widgets.get_mut(frame_id)
    {
        frame.keyboard_enabled = enabled;
    }
}

/// Resolve and apply propagateMouseInput / propagateMouseInputMask from XML.
pub fn apply_xml_propagate_mouse_input(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut clicks = false;
    let mut motion = false;

    if !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(spec) = entry.frame.propagate_mouse_input_mask.as_deref() {
                merge_propagate_mouse_input_spec(spec, &mut clicks, &mut motion);
            }
            if let Some(spec) = entry.frame.propagate_mouse_input.as_deref() {
                merge_propagate_mouse_input_spec(spec, &mut clicks, &mut motion);
            }
        }
    }

    if let Some(spec) = frame.propagate_mouse_input_mask.as_deref() {
        merge_propagate_mouse_input_spec(spec, &mut clicks, &mut motion);
    }
    if let Some(spec) = frame.propagate_mouse_input.as_deref() {
        merge_propagate_mouse_input_spec(spec, &mut clicks, &mut motion);
    }

    if clicks || motion {
        let mut s = state.borrow_mut();
        if let Some(widget) = s.widgets.get_mut(frame_id) {
            widget.propagate_mouse_clicks = clicks;
            widget.propagate_mouse_motion = motion;
        }
    }
}

fn merge_propagate_mouse_input_spec(propagate_spec: &str, clicks: &mut bool, motion: &mut bool) {
    for token in propagate_spec
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
    {
        match token.to_ascii_lowercase().as_str() {
            "all" => {
                *clicks = true;
                *motion = true;
            }
            "clicks" => *clicks = true,
            "motion" => *motion = true,
            _ => {}
        }
    }
}

/// Resolve and apply clipChildren from template chain + instance XML.
pub fn apply_xml_clips_children(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut clips_children = frame.clip_children;
    if clips_children.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(c) = entry.frame.clip_children {
                clips_children = Some(c);
            }
        }
    }
    if let Some(clips) = clips_children
        && let Some(f) = state.borrow_mut().widgets.get_mut_visual(frame_id)
    {
        f.clips_children = clips;
    }
}

/// Apply hitRectInsets from instance XML (no template chain resolution).
pub fn apply_xml_hit_rect_insets(state: &Rc<RefCell<SimState>>, frame_id: u64, frame: &FrameXml) {
    if let Some(insets) = frame.hit_rect_insets() {
        set_hit_rect_insets(
            state,
            frame_id,
            insets.left(),
            insets.right(),
            insets.top(),
            insets.bottom(),
        );
    }
}

/// Apply text insets from instance XML.
pub fn apply_xml_text_insets(state: &Rc<RefCell<SimState>>, frame_id: u64, frame: &FrameXml) {
    if let Some(insets) = frame.text_insets()
        && let Some(frame) = state.borrow_mut().widgets.get_mut_visual(frame_id)
    {
        frame.editbox_text_insets = (insets.left(), insets.right(), insets.top(), insets.bottom());
    }
}

/// Resolve and apply clampedToScreen from template chain + instance XML.
pub fn apply_xml_clamped_to_screen(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut clamped = frame.clamped_to_screen;
    if clamped.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(c) = entry.frame.clamped_to_screen {
                clamped = Some(c);
            }
        }
    }
    if let Some(c) = clamped {
        set_clamped_to_screen(state, frame_id, c);
    }
}

/// Resolve and apply setAllPoints from template chain + instance XML.
pub fn apply_xml_set_all_points(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut has = false;
    if !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if entry.frame.set_all_points == Some(true) {
                has = true;
                break;
            }
        }
    }
    if frame.set_all_points == Some(true) {
        has = true;
    }
    if has {
        let mut s = state.borrow_mut();
        if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
            frame.xml_set_all_points = true;
        }
        set_all_points_inner(&mut s, frame_id);
    }
}

/// Resolve and apply protected from template chain + instance XML.
pub fn apply_xml_protected(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let protected = frame.protected.or_else(|| {
        if inherits.is_empty() {
            return None;
        }
        crate::xml::get_template_chain(inherits)
            .iter()
            .find_map(|e| e.frame.protected)
    });
    if protected == Some(true)
        && let Some(f) = state.borrow_mut().widgets.get_mut_visual(frame_id)
    {
        f.is_protected = true;
    }
}

/// Apply frame ID from XML `id` attribute.
pub fn apply_xml_id(state: &Rc<RefCell<SimState>>, frame_id: u64, frame: &FrameXml) {
    if let Some(id) = frame.xml_id {
        set_id(state, frame_id, id);
    }
}

/// Apply declared EditBox limits, resolving each field independently.
/// Explicit values (including zero) override the most-derived template value.
pub fn apply_xml_editbox_limits(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let chain = crate::xml::get_template_chain(inherits);
    let letters = frame
        .letters
        .or_else(|| chain.iter().rev().find_map(|entry| entry.frame.letters));
    let bytes = frame
        .bytes
        .or_else(|| chain.iter().rev().find_map(|entry| entry.frame.bytes));
    if letters.is_none() && bytes.is_none() {
        return;
    }
    let mut sim = state.borrow_mut();
    let Some(widget) = sim.widgets.get_mut_visual(frame_id) else {
        return;
    };
    if let Some(value) = letters {
        widget.editbox_max_letters = value;
    }
    if let Some(value) = bytes {
        widget.editbox_max_bytes = value;
    }
}

/// Resolve and apply StatusBar texture rotation from template chain + instance XML.
pub fn apply_xml_statusbar_rotation(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let rotates = frame.rotates_texture.or_else(|| {
        crate::xml::get_template_chain(inherits)
            .iter()
            .rev()
            .find_map(|entry| entry.frame.rotates_texture)
    });
    if let Some(rotates) = rotates {
        let mut sim = state.borrow_mut();
        if sim
            .widgets
            .get(frame_id)
            .is_some_and(|frame| frame.widget_type == crate::widget::WidgetType::StatusBar)
        {
            crate::lua_api::frame::methods::widgets::set_statusbar_rotation(
                &mut sim, frame_id, rotates,
            );
        }
    }
}

/// Resolve and apply Slider `orientation` from template chain + instance XML.
pub fn apply_xml_slider_orientation(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut orientation = frame.orientation.clone();
    if orientation.is_none() && !inherits.is_empty() {
        for entry in &*crate::xml::get_template_chain(inherits) {
            if let Some(value) = entry.frame.orientation.clone() {
                orientation = Some(value);
            }
        }
    }
    if let Some(value) = orientation
        && let Some(f) = state.borrow_mut().widgets.get_mut_visual(frame_id)
    {
        f.slider_orientation = value.to_uppercase();
    }
}

/// Merge size values from a SizeXml into accumulators.
fn merge_size(
    width: &mut Option<f32>,
    height: &mut Option<f32>,
    size: Option<&crate::xml::SizeXml>,
) {
    if let Some(size) = size {
        let (x, y) = super::get_size_values(size);
        if let Some(x) = x {
            *width = Some(x);
        }
        if let Some(y) = y {
            *height = Some(y);
        }
    }
}

#[cfg(test)]
#[path = "direct_tests.rs"]
mod tests;
