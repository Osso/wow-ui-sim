use crate::lua_api::SimState;
use crate::widget::FrameStrata;
use crate::xml::FrameXml;
use std::cell::RefCell;
use std::rc::Rc;

/// Resolve and apply frame strata from template chain + instance XML.
pub fn apply_xml_frame_strata(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let strata = resolve_frame_strata(frame, inherits);
    if let Some(ref strata) = strata {
        apply_resolved_frame_strata(state, frame_id, strata);
    }
}

fn resolve_frame_strata(frame: &FrameXml, inherits: &str) -> Option<String> {
    let inherited = inherited_frame_strata(inherits);
    match frame.frame_strata.as_deref() {
        Some(strata) if strata.eq_ignore_ascii_case("PARENT") && inherited.is_some() => inherited,
        Some(strata) if strata.eq_ignore_ascii_case("PARENT") => Some(strata.to_owned()),
        Some(strata) if FrameStrata::from_str(strata).is_some() => Some(strata.to_owned()),
        Some(_) | None => inherited,
    }
}

fn inherited_frame_strata(inherits: &str) -> Option<String> {
    let mut resolved = None;
    for entry in &*crate::xml::get_template_chain(inherits) {
        let Some(strata) = entry.frame.frame_strata.as_deref() else {
            continue;
        };
        if strata.eq_ignore_ascii_case("PARENT") {
            resolved.get_or_insert_with(|| strata.to_owned());
        } else if FrameStrata::from_str(strata).is_some() {
            resolved = Some(strata.to_owned());
        }
    }
    resolved
}

/// Resolve and apply frame level from template chain + instance XML.
pub fn apply_xml_frame_level(
    state: &Rc<RefCell<SimState>>,
    frame_id: u64,
    frame: &FrameXml,
    inherits: &str,
) {
    let frame_level = resolve_frame_level(frame, inherits);
    if frame_level.use_parent_level == Some(true) {
        set_xml_frame_level_to_parent(state, frame_id);
        return;
    }
    if let Some(level) = frame_level.level {
        // Real client (XmlFrameLevelProbe, 12.0.5): a bare XML `frameLevel` is
        // an absolute level that is NOT fixed — it shifts with parent level
        // changes. Only an explicit `fixedFrameLevel="true"` pins it.
        set_xml_frame_level(state, frame_id, level, frame_level.fixed.unwrap_or(false));
    }
}

#[derive(Default)]
struct ResolvedFrameLevel {
    level: Option<i32>,
    fixed: Option<bool>,
    use_parent_level: Option<bool>,
}

fn resolve_frame_level(frame: &FrameXml, inherits: &str) -> ResolvedFrameLevel {
    let mut resolved = inherited_frame_level(inherits);
    if let Some(frame_level) = frame.frame_level {
        resolved.level = Some(frame_level);
    }
    if let Some(frame_fixed) = frame.fixed_frame_level {
        resolved.fixed = Some(frame_fixed);
    }
    if let Some(frame_use_parent_level) = frame.use_parent_level {
        resolved.use_parent_level = Some(frame_use_parent_level);
    }
    resolved
}

fn inherited_frame_level(inherits: &str) -> ResolvedFrameLevel {
    let mut resolved = ResolvedFrameLevel::default();
    if inherits.is_empty() {
        return resolved;
    }
    for entry in &*crate::xml::get_template_chain(inherits) {
        if let Some(level) = entry.frame.frame_level {
            resolved.level = Some(level);
        }
        if let Some(fixed) = entry.frame.fixed_frame_level {
            resolved.fixed = Some(fixed);
        }
        if let Some(use_parent_level) = entry.frame.use_parent_level {
            resolved.use_parent_level = Some(use_parent_level);
        }
    }
    resolved
}

fn apply_resolved_frame_strata(state: &Rc<RefCell<SimState>>, frame_id: u64, strata: &str) {
    if strata.eq_ignore_ascii_case("PARENT") {
        set_frame_strata_to_parent(state, frame_id);
    } else {
        set_frame_strata(state, frame_id, strata);
    }
}

fn set_frame_strata_to_parent(state: &Rc<RefCell<SimState>>, frame_id: u64) {
    let mut sim = state.borrow_mut();
    let parent_strata = sim
        .widgets
        .get(frame_id)
        .and_then(|frame| frame.parent_id)
        .and_then(|parent_id| sim.widgets.get(parent_id))
        .map(|parent| parent.frame_strata)
        .unwrap_or_default();
    if let Some(frame) = sim.widgets.get_mut_visual(frame_id) {
        frame.frame_strata = parent_strata;
        frame.has_fixed_frame_strata = false;
    }
    propagate_unfixed_child_strata(&mut sim, frame_id, parent_strata);
    sim.invalidate_strata_buckets();
}

/// Set frame strata directly.
fn set_frame_strata(state: &Rc<RefCell<SimState>>, frame_id: u64, strata_str: &str) {
    let Some(strata) = FrameStrata::from_str(strata_str) else {
        return;
    };
    let mut sim = state.borrow_mut();
    if let Some(frame) = sim.widgets.get_mut_visual(frame_id) {
        frame.frame_strata = strata;
        frame.has_fixed_frame_strata = false;
    }
    propagate_unfixed_child_strata(&mut sim, frame_id, strata);
    sim.invalidate_strata_buckets();
}

fn propagate_unfixed_child_strata(sim: &mut SimState, frame_id: u64, strata: FrameStrata) {
    let mut queue = sim
        .widgets
        .get(frame_id)
        .map(|frame| frame.children.clone())
        .unwrap_or_default();
    while let Some(child_id) = queue.pop() {
        let Some(child) = sim.widgets.get_mut_visual(child_id) else {
            continue;
        };
        if child.has_fixed_frame_strata {
            continue;
        }
        child.frame_strata = strata;
        queue.extend(child.children.iter().copied());
    }
}

fn set_xml_frame_level(state: &Rc<RefCell<SimState>>, frame_id: u64, level: i32, fixed: bool) {
    let mut sim = state.borrow_mut();
    if fixed {
        set_fixed_xml_frame_level(&mut sim, frame_id, level);
    } else {
        set_parent_relative_xml_frame_level(&mut sim, frame_id, level);
    }
    crate::lua_api::frame::recalculate_strata_level_pub(&mut sim.widgets, frame_id);
}

fn set_fixed_xml_frame_level(sim: &mut SimState, frame_id: u64, level: i32) {
    if let Some(frame) = sim.widgets.get_mut_visual(frame_id) {
        frame.frame_level_offset = None;
        frame.has_fixed_frame_level = true;
        frame.uses_parent_level = false;
        frame.frame_level = level;
    }
}

fn set_parent_relative_xml_frame_level(sim: &mut SimState, frame_id: u64, level: i32) {
    // The XML `frameLevel` value is the absolute level (real client). The frame
    // still tracks its parent: later parent level changes shift it by the
    // parent's delta, which the propagation pass applies as
    // `parent_level + frame_level_offset`. So the stored offset is the gap
    // captured now: (absolute level - parent level at this moment).
    let parent_level = parent_frame_level(sim, frame_id).unwrap_or(0);
    if let Some(frame) = sim.widgets.get_mut_visual(frame_id) {
        frame.has_fixed_frame_level = false;
        frame.uses_parent_level = false;
        frame.frame_level = level;
        frame.frame_level_offset = Some(level - parent_level);
    }
}

/// Force a frame to share its parent's level (XML `useParentLevel="true"`).
fn set_xml_frame_level_to_parent(state: &Rc<RefCell<SimState>>, frame_id: u64) {
    let mut sim = state.borrow_mut();
    let parent_level = parent_frame_level(&sim, frame_id);
    if let Some(frame) = sim.widgets.get_mut_visual(frame_id) {
        frame.frame_level_offset = Some(0);
        frame.has_fixed_frame_level = false;
        frame.uses_parent_level = true;
        if let Some(parent_level) = parent_level {
            frame.frame_level = parent_level;
        }
    }
    crate::lua_api::frame::recalculate_strata_level_pub(&mut sim.widgets, frame_id);
}

fn parent_frame_level(sim: &SimState, frame_id: u64) -> Option<i32> {
    sim.widgets
        .get(frame_id)
        .and_then(|frame| frame.parent_id)
        .and_then(|parent_id| sim.widgets.get(parent_id))
        .map(|parent| parent.frame_level)
}
