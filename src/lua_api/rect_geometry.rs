//! Rect / geometry query RustFns for WoW frame methods.
//!
//! Ported from master-era `frame/methods/methods_rect.rs`. Every method
//! reads the resolved layout rect for the frame (computing it if the
//! rect is missing or dirty), converts screen-space coordinates to
//! WoW UI coordinates (y-axis inverted, divided by effective scale),
//! and returns the values Lua expects.
//!
//! Registered by `register_rect_methods_on_table` onto the frame
//! metatable. `timer_layout::register_layout_fns_on_table` delegates
//! here so the historical entry point keeps working.
//!
//! Method surface:
//!
//! | Lua name | Returns |
//! |---|---|
//! | `GetRect` | `(left, bottom, width, height)` in WoW UI coords |
//! | `GetScaledRect` | `(left, bottom, width, height)` in screen-space coords |
//! | `GetLeft` / `GetRight` / `GetTop` / `GetBottom` | single edge in WoW UI coords |
//! | `GetCenter` | `(x, y)` center point in WoW UI coords |
//! | `GetWidth` / `GetHeight` | single dimension in WoW UI coords |
//! | `GetSize` | `(width, height)` in WoW UI coords |
//!
//! Coordinate conventions: screen-space origin is top-left (grows
//! down); WoW UI origin is bottom-left (y grows up). The `screen_height
//! - y - height` terms convert between them.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::table_set_rust_fn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use rustc_hash::FxHashSet;

fn resolve_frame_rect_if_dirty(state: &mut LuaState, id: u64, force: bool) -> LuaResult<bool> {
    let (layout_missing, is_dirty, is_root) = {
        let sim = borrow_state(state)?;
        let Some(frame) = sim.widgets.get(id) else {
            return Ok(false);
        };
        (
            frame.layout_rect.is_none(),
            sim.widgets.is_rect_dirty(id),
            frame.name.as_deref() == Some("UIParent") || (frame.parent_id.is_none() && id == 1),
        )
    };
    let needs_resolve = layout_missing || is_dirty;
    if !needs_resolve && !force && !is_root {
        return Ok(false);
    }

    let mut sim = borrow_state_mut(state)?;
    if is_dirty {
        sim.resolve_rect_if_dirty(id);
    } else if layout_missing || force {
        sim.invalidate_layout(id);
    }
    if is_root && let Some(frame) = sim.widgets.get_mut(id) {
        frame.rect_valid = true;
        frame.anchor_invalidated = false;
    }
    Ok(needs_resolve || force)
}

fn anchor_targets(state: &LuaState, id: u64) -> LuaResult<(Vec<(u64, bool)>, bool, bool)> {
    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok((Vec::new(), true, false));
    };
    let has_layout_path = crate::layout::frame_has_layout_anchor(&sim.widgets, frame, id);
    let mut targets = Vec::new();
    let mut unresolved_name = false;

    if frame.anchors.is_empty() {
        if has_layout_path && let Some(parent_id) = frame.parent_id {
            let needs_resolve = sim.widgets.get(parent_id).is_some_and(|parent| {
                parent.layout_rect.is_none() || sim.widgets.is_rect_dirty(parent_id)
            });
            targets.push((parent_id, needs_resolve));
        }
    } else {
        for anchor in &frame.anchors {
            if anchor.relative_to_id.is_none() && anchor.relative_to.is_some() {
                unresolved_name = true;
                continue;
            }
            let Some(target_id) = anchor
                .relative_to_id
                .map(|target_id| target_id as u64)
                .or(frame.parent_id)
            else {
                continue;
            };
            let needs_resolve = sim.widgets.get(target_id).is_some_and(|target| {
                target.layout_rect.is_none() || sim.widgets.is_rect_dirty(target_id)
            });
            targets.push((target_id, needs_resolve));
        }
    }

    Ok((targets, unresolved_name, has_layout_path))
}

fn resolve_anchor_targets(
    state: &mut LuaState,
    id: u64,
    visiting: &mut FxHashSet<u64>,
) -> LuaResult<bool> {
    if !visiting.insert(id) {
        return Ok(false);
    }

    let (targets, unresolved_name, has_layout_path) = anchor_targets(state, id)?;
    let mut targets_valid = !unresolved_name;
    let mut target_changed = false;
    for (target_id, was_dirty) in targets {
        let target_chain_changed = resolve_anchor_targets(state, target_id, visiting)?;
        let target_was_resolved = was_dirty || target_chain_changed;
        target_changed |= target_was_resolved;
        let target_is_usable = {
            let sim = borrow_state(state)?;
            sim.widgets.get(target_id).is_some_and(|target| {
                target.rect_valid && (!target_was_resolved || !target.anchor_invalidated)
            })
        };
        targets_valid &= target_is_usable;
    }

    visiting.remove(&id);
    let frame_changed = resolve_frame_rect_if_dirty(state, id, target_changed)?;
    if !has_layout_path || !targets_valid {
        if let Some(frame) = borrow_state_mut(state)?.widgets.get_mut(id) {
            frame.rect_valid = false;
        }
    }
    Ok(target_changed || frame_changed)
}

/// Resolve the frame and dirty anchor targets before reading cached validity.
///
/// The returned tuple is `(rect, effective_scale, screen_height)` for Lua
/// coordinate conversion. An invalid cached rectangle returns `None`.
pub(crate) fn resolve_queryable_rect(
    state: &mut LuaState,
    id: u64,
) -> LuaResult<Option<(crate::LayoutRect, f32, f32)>> {
    crate::lua_api::frame::methods::secret_origin::require_geometry_readable(state, id)?;
    resolve_anchor_targets(state, id, &mut FxHashSet::default())?;

    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok(None);
    };
    if !frame.rect_valid {
        return Ok(None);
    }
    let Some(rect) = frame.layout_rect else {
        return Ok(None);
    };
    Ok(Some((
        rect,
        frame.effective_scale.max(1e-6),
        sim.screen_height,
    )))
}

/// `frame:GetRect()` — `(left, bottom, width, height)` in WoW UI coords.
fn get_frame_rect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, screen_height)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let left = rect.x / eff_scale;
    let bottom = (screen_height - rect.y - rect.height) / eff_scale;
    let width = rect.width / eff_scale;
    let height = rect.height / eff_scale;
    state.push(Val::Num(left as f64));
    state.push(Val::Num(bottom as f64));
    state.push(Val::Num(width as f64));
    state.push(Val::Num(height as f64));
    Ok(4)
}

/// `frame:GetScaledRect()` — rect in screen-space coordinates (no
/// effective-scale division). y axis still converted to WoW UI.
fn get_scaled_rect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, _eff_scale, screen_height)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let left = rect.x;
    let bottom = screen_height - rect.y - rect.height;
    state.push(Val::Num(left as f64));
    state.push(Val::Num(bottom as f64));
    state.push(Val::Num(rect.width as f64));
    state.push(Val::Num(rect.height as f64));
    Ok(4)
}

/// `frame:GetLeft()` — left edge in WoW UI coordinates.
fn get_left(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, _)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let left = rect.x / eff_scale;
    state.push(Val::Num(left as f64));
    Ok(1)
}

/// `frame:GetRight()` — right edge in WoW UI coordinates.
fn get_right(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, _)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let right = (rect.x + rect.width) / eff_scale;
    state.push(Val::Num(right as f64));
    Ok(1)
}

/// `frame:GetTop()` — top edge in WoW UI coordinates (inverted: large
/// values = near top of screen).
fn get_top(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, screen_height)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let top = (screen_height - rect.y) / eff_scale;
    state.push(Val::Num(top as f64));
    Ok(1)
}

/// `frame:GetBottom()` — bottom edge in WoW UI coordinates.
fn get_bottom(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, screen_height)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let bottom = (screen_height - rect.y - rect.height) / eff_scale;
    state.push(Val::Num(bottom as f64));
    Ok(1)
}

/// `frame:GetCenter()` — center point `(x, y)` in WoW UI coords.
fn get_center(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, screen_height)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let center_x = (rect.x + rect.width / 2.0) / eff_scale;
    let center_y = (screen_height - rect.y - rect.height / 2.0) / eff_scale;
    state.push(Val::Num(center_x as f64));
    state.push(Val::Num(center_y as f64));
    Ok(2)
}

fn queryable_size(state: &mut LuaState, id: u64) -> LuaResult<(f32, f32)> {
    if let Some((rect, effective_scale, _)) = resolve_queryable_rect(state, id)? {
        return Ok((rect.width / effective_scale, rect.height / effective_scale));
    }

    Ok(borrow_state(state)?.widgets.stored_size(id))
}

/// `frame:GetWidth()` — resolved width, or stored width without a valid rect.
fn get_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (width, _) = queryable_size(state, id)?;
    state.push(Val::Num(width as f64));
    Ok(1)
}

/// `frame:GetHeight()` — resolved height, or stored height without a valid rect.
fn get_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (_, height) = queryable_size(state, id)?;
    state.push(Val::Num(height as f64));
    Ok(1)
}

/// `frame:GetSize()` — `(width, height)` in WoW UI coordinates.
fn get_size(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (width, height) = queryable_size(state, id)?;
    state.push(Val::Num(width as f64));
    state.push(Val::Num(height as f64));
    Ok(2)
}

const RECT_METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    ("GetRect", get_frame_rect),
    ("GetScaledRect", get_scaled_rect),
    ("GetLeft", get_left),
    ("GetRight", get_right),
    ("GetTop", get_top),
    ("GetBottom", get_bottom),
    ("GetCenter", get_center),
    ("GetWidth", get_width),
    ("GetHeight", get_height),
    ("GetSize", get_size),
];

/// Install the rect / geometry query family on a table (typically the
/// frame metatable). Covered by `tests/rect_geometry.rs`.
pub fn register_rect_methods_on_table(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in RECT_METHODS {
        table_set_rust_fn(state, table, name, *func)?;
    }
    Ok(())
}
