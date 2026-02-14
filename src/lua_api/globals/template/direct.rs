//! Direct Rust property setters for frame creation.
//!
//! These functions bypass Lua compilation to set frame properties directly
//! on the Rust Frame struct, avoiding the ~30-50µs overhead per `lua.load().exec()`
//! call. Used during template application and XML frame loading.

use crate::lua_api::SimState;
use crate::widget::{AnchorPoint, FrameStrata};
use crate::xml::{AnchorXml, FrameXml};
use std::cell::RefCell;
use std::rc::Rc;

/// Set frame size directly from a FrameXml's `<Size>` element.
pub fn set_size(state: &Rc<RefCell<SimState>>, frame_id: u64, template: &FrameXml) {
    let Some(size) = template.size() else { return };
    let (width, height) = super::get_size_values(size);
    let (Some(w), Some(h)) = (width, height) else { return };
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
        frame.set_size(w, h);
    }
    s.widgets.mark_rect_dirty(frame_id);
    s.invalidate_layout_with_dependents(frame_id);
}

/// Set partial or full size from XML (handles width-only and height-only).
pub fn set_size_partial(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    template: &FrameXml,
) {
    let Some(size) = template.size() else { return };
    let (w, h) = super::get_size_values(size);
    let mut s = state.borrow_mut();
    match (w, h) {
        (Some(w), Some(h)) => {
            if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
                frame.set_size(w, h);
            }
        }
        (Some(w), None) => {
            if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
                frame.width = w;
            }
        }
        (None, Some(h)) => {
            if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
                frame.height = h;
            }
        }
        (None, None) => return,
    }
    s.widgets.mark_rect_dirty(frame_id);
    s.invalidate_layout_with_dependents(frame_id);
}

/// Set frame anchors directly from a FrameXml's `<Anchors>` element.
pub fn set_anchors(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    template: &FrameXml,
    frame_name: &str,
) {
    let Some(anchors) = template.anchors() else { return };
    let mut s = state.borrow_mut();
    for anchor in &anchors.anchors {
        set_single_anchor(&mut s, frame_id, anchor, frame_name);
    }
}

/// Set a single anchor point on a frame.
fn set_single_anchor(
    state: &mut SimState,
    frame_id: u64,
    anchor: &AnchorXml,
    frame_name: &str,
) {
    let point_str = anchor.point.as_str();
    let relative_point_str = anchor.relative_point.as_deref().unwrap_or(point_str);

    let Some(point) = AnchorPoint::from_str(point_str) else { return };
    let Some(relative_point) = AnchorPoint::from_str(relative_point_str) else {
        return;
    };

    let (offset_x, offset_y) = anchor_offset(anchor);

    // Resolve relative_to target
    let relative_to_id =
        resolve_relative_to(state, frame_id, anchor.relative_to.as_deref(), frame_name);

    // Cycle detection
    if let Some(rel_id) = relative_to_id {
        if state.widgets.would_create_anchor_cycle(frame_id, rel_id) {
            return;
        }
    }

    // Remove old anchor dependent for this point
    if let Some(frame) = state.widgets.get(frame_id) {
        if let Some(old_anchor) = frame.anchors.iter().find(|a| a.point == point) {
            if let Some(old_target) = old_anchor.relative_to_id {
                state
                    .widgets
                    .remove_anchor_dependent(old_target as u64, frame_id);
            }
        }
    }

    // Add new anchor dependent
    if let Some(rel_id) = relative_to_id {
        state.widgets.add_anchor_dependent(rel_id, frame_id);
    }

    // Set the anchor
    if let Some(frame) = state.widgets.get_mut_visual(frame_id) {
        frame.set_point(
            point,
            relative_to_id.map(|id| id as usize),
            relative_point,
            offset_x,
            offset_y,
        );
    }

    state.widgets.mark_rect_dirty(frame_id);
    state.invalidate_layout_with_dependents(frame_id);
}

/// Resolve the relative_to target for an anchor, returning the target frame ID.
fn resolve_relative_to(
    state: &SimState,
    frame_id: u64,
    relative_to: Option<&str>,
    frame_name: &str,
) -> Option<u64> {
    match relative_to {
        Some(rel) if rel == "$parent" => state.widgets.get(frame_id).and_then(|f| f.parent_id),
        Some(rel) => {
            let resolved = rel.replace("$parent", frame_name);
            state.widgets.get_id_by_name(&resolved)
        }
        None => state.widgets.get(frame_id).and_then(|f| f.parent_id),
    }
}

/// Set SetAllPoints from template (clears anchors, adds TOPLEFT+BOTTOMRIGHT to parent).
pub fn set_all_points(state: &Rc<RefCell<SimState>>, frame_id: u64, template: &FrameXml) {
    if template.set_all_points != Some(true) {
        return;
    }
    let mut s = state.borrow_mut();
    set_all_points_inner(&mut s, frame_id);
}

/// Inner: clear all points and fill parent with TOPLEFT+BOTTOMRIGHT anchors.
///
/// Matches Lua `SetAllPoints(true)`: stores `relative_to_id = None` (implicit parent)
/// and does NOT add anchor dependents (the layout system uses parent implicitly).
fn set_all_points_inner(state: &mut SimState, frame_id: u64) {
    // Remove old anchor dependents
    state.widgets.remove_all_anchor_dependents_for(frame_id);

    if let Some(frame) = state.widgets.get_mut_visual(frame_id) {
        frame.clear_all_points();
        frame.set_point(AnchorPoint::TopLeft, None, AnchorPoint::TopLeft, 0.0, 0.0);
        frame.set_point(AnchorPoint::BottomRight, None, AnchorPoint::BottomRight, 0.0, 0.0);
    }

    state.widgets.mark_rect_dirty(frame_id);
    state.invalidate_layout(frame_id);
}

/// Set frame hidden state directly from template.
pub fn set_hidden(state: &Rc<RefCell<SimState>>, frame_id: u64, template: &FrameXml) {
    if template.hidden != Some(true) {
        return;
    }
    state.borrow_mut().set_frame_visible(frame_id, false);
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

/// Set frame strata directly.
pub fn set_frame_strata(state: &Rc<RefCell<SimState>>, frame_id: u64, strata_str: &str) {
    let Some(strata) = FrameStrata::from_str(strata_str) else {
        return;
    };
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
        frame.frame_strata = strata;
        frame.has_fixed_frame_strata = true;
    }
    // Propagate to descendants that don't have fixed strata
    let mut queue: Vec<u64> = s
        .widgets
        .get(frame_id)
        .map(|f| f.children.clone())
        .unwrap_or_default();
    while let Some(child_id) = queue.pop() {
        let Some(child) = s.widgets.get_mut_visual(child_id) else {
            continue;
        };
        if child.has_fixed_frame_strata {
            continue;
        }
        child.frame_strata = strata;
        queue.extend(child.children.iter().copied());
    }
    // Invalidate strata buckets since strata changed.
    s.strata_buckets = None;
}

/// Set frame level directly.
pub fn set_frame_level(state: &Rc<RefCell<SimState>>, frame_id: u64, level: i32) {
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut_visual(frame_id) {
        frame.frame_level = level;
        frame.has_fixed_frame_level = true;
    }
    crate::lua_api::frame::propagate_strata_level_pub(&mut s.widgets, frame_id);
}

/// Set toplevel directly.
pub fn set_toplevel(state: &Rc<RefCell<SimState>>, frame_id: u64, toplevel: bool) {
    let mut s = state.borrow_mut();
    if let Some(frame) = s.widgets.get_mut(frame_id) {
        frame.toplevel = toplevel;
    }
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
        (
            abs.and_then(|d| d.x).unwrap_or(0.0),
            abs.and_then(|d| d.y).unwrap_or(0.0),
        )
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
        for entry in &crate::xml::get_template_chain(inherits) {
            merge_size(&mut final_width, &mut final_height, entry.frame.size());
        }
    }
    merge_size(&mut final_width, &mut final_height, frame.size());

    if let (Some(w), Some(h)) = (final_width, final_height) {
        let mut s = state.borrow_mut();
        if let Some(f) = s.widgets.get_mut_visual(frame_id) {
            f.set_size(w, h);
        }
        s.widgets.mark_rect_dirty(frame_id);
        s.invalidate_layout_with_dependents(frame_id);
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

/// Resolve and apply frame strata from template chain + instance XML.
pub fn apply_xml_frame_strata(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let strata: Option<String> = frame.frame_strata.clone().or_else(|| {
        if inherits.is_empty() {
            return None;
        }
        crate::xml::get_template_chain(inherits)
            .iter()
            .find_map(|e| e.frame.frame_strata.clone())
    });
    if let Some(ref s) = strata {
        set_frame_strata(state, frame_id, s);
    }
}

/// Resolve and apply frame level from template chain + instance XML.
pub fn apply_xml_frame_level(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let level = frame.frame_level.or_else(|| {
        if inherits.is_empty() {
            return None;
        }
        crate::xml::get_template_chain(inherits)
            .iter()
            .find_map(|e| e.frame.frame_level)
    });
    if let Some(l) = level {
        set_frame_level(state, frame_id, l);
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
        for entry in &crate::xml::get_template_chain(inherits) {
            if let Some(h) = entry.frame.hidden {
                hidden = Some(h);
                break;
            }
        }
    }
    if hidden == Some(true) {
        state.borrow_mut().set_frame_visible(frame_id, false);
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
        for entry in &crate::xml::get_template_chain(inherits) {
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

/// Resolve and apply enableMouse from template chain + instance XML.
pub fn apply_xml_enable_mouse(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let mut em = frame.enable_mouse;
    if em.is_none() && !inherits.is_empty() {
        for entry in &crate::xml::get_template_chain(inherits) {
            if let Some(e) = entry.frame.enable_mouse {
                em = Some(e);
            }
        }
    }
    if let Some(enabled) = em {
        enable_mouse(state, frame_id, enabled);
    }
}

/// Apply hitRectInsets from instance XML (no template chain resolution).
pub fn apply_xml_hit_rect_insets(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
) {
    if let Some(insets) = frame.hit_rect_insets() {
        set_hit_rect_insets(
            state,
            frame_id,
            insets.left.unwrap_or(0.0),
            insets.right.unwrap_or(0.0),
            insets.top.unwrap_or(0.0),
            insets.bottom.unwrap_or(0.0),
        );
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
        for entry in &crate::xml::get_template_chain(inherits) {
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
        for entry in &crate::xml::get_template_chain(inherits) {
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
        set_all_points_inner(&mut s, frame_id);
    }
}

/// Apply frame ID from XML `id` attribute.
pub fn apply_xml_id(state: &Rc<RefCell<SimState>>, frame_id: u64, frame: &FrameXml) {
    if let Some(id) = frame.xml_id {
        set_id(state, frame_id, id);
    }
}

/// Merge size values from a SizeXml into accumulators.
fn merge_size(width: &mut Option<f32>, height: &mut Option<f32>, size: Option<&crate::xml::SizeXml>) {
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
