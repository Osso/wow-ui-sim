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

/// Resolve the frame's layout rect, computing it if dirty or unset.
///
/// Returns `None` when the frame or any ancestor has no queryable
/// layout anchor, when the frame isn't in the registry, or when the
/// rect still can't be resolved after an invalidation pass.
///
/// Returned tuple is `(rect, effective_scale, screen_height)` — all
/// three are needed by the per-edge helpers to do coordinate
/// conversion.
fn resolve_queryable_rect(
    state: &mut LuaState,
    id: u64,
) -> LuaResult<Option<(crate::LayoutRect, f32, f32)>> {
    crate::lua_api::frame::methods::secret_origin::require_geometry_readable(state, id)?;
    let needs_resolve = {
        let sim = borrow_state(state)?;
        if !crate::layout::frame_has_render_layout(&sim.widgets, id) {
            return Ok(None);
        };
        let Some(frame) = sim.widgets.get(id) else {
            return Ok(None);
        };
        frame.layout_rect.is_none() || sim.widgets.is_rect_dirty(id)
    };

    if needs_resolve {
        let mut sim = borrow_state_mut(state)?;
        if sim
            .widgets
            .get(id)
            .and_then(|frame| frame.layout_rect)
            .is_none()
        {
            sim.invalidate_layout(id);
        }
        sim.resolve_rect_if_dirty(id);
    }

    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok(None);
    };
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

/// `frame:GetWidth()` — frame width in WoW UI coordinates.
fn get_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, _)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let width = rect.width / eff_scale;
    state.push(Val::Num(width as f64));
    Ok(1)
}

/// `frame:GetHeight()` — frame height in WoW UI coordinates.
fn get_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, _)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let height = rect.height / eff_scale;
    state.push(Val::Num(height as f64));
    Ok(1)
}

/// `frame:GetSize()` — `(width, height)` in WoW UI coordinates.
fn get_size(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((rect, eff_scale, _)) = resolve_queryable_rect(state, id)? else {
        return Ok(0);
    };
    let width = rect.width / eff_scale;
    let height = rect.height / eff_scale;
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
