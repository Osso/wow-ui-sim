//! `C_PingSecure` UI receiver and world target selection (WM47 Q8, WM51-F4/F5).

use super::sends::{FAILED_GENERIC, SendResult, push_result, read_number};
use crate::layout::{LayoutCache, apply_scroll_offsets, compute_frame_rect_cached};
use crate::lua_api::globals::real::protected_globals::refuse_tainted_caller;
use crate::lua_api::methods::{borrow_state, frame_ref};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::widget::{AttributeValue, Frame, FrameStrata, WidgetType};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

/// Enum.PingSetTargetState.Failed (PingConstantsDocumentation.lua:37-48).
const TARGET_FAILED: f64 = 1.0;

type Member = fn(&mut LuaState) -> LuaResult<u32>;

const TARGETING_MEMBERS: &[(&str, Member)] = &[
    ("GetTargetPingReceiver", get_target_ping_receiver),
    ("SetHitTestPingTarget", set_hit_test_ping_target),
    (
        "SetHitTestTargetAndSendPing",
        set_hit_test_target_and_send_ping,
    ),
];

pub(super) fn register(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    for (name, member) in TARGETING_MEMBERS {
        table_set_rust_fn_static(state, table_ref, name, *member)?;
    }
    Ok(())
}

/// WM47 Q8: the eligible frame that draws on top, or nil.
fn get_target_ping_receiver(state: &mut LuaState) -> LuaResult<u32> {
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let (x, y) = read_cursor_position(state)?;
    let receiver = find_receiver(&*borrow_state(state)?, x, y);
    let value = match receiver {
        Some(id) => frame_ref(state, id)?,
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}

/// WM51-F4: the simulator has no world geometry, so the world query always
/// misses: selection state 1, before any pending information is stored.
fn set_hit_test_ping_target(state: &mut LuaState) -> LuaResult<u32> {
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    read_cursor_position(state)?;
    state.push(Val::Num(TARGET_FAILED));
    Ok(1)
}

/// WM51-F4: an immediate-mode miss returns result 1 without a type, skipping
/// the pending send, eligibility and cooldown bookkeeping.
fn set_hit_test_target_and_send_ping(state: &mut LuaState) -> LuaResult<u32> {
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    push_result(state, SendResult::failed(FAILED_GENERIC))
}

/// WM38: two consecutive numeric-convertible arguments.
fn read_cursor_position(state: &LuaState) -> LuaResult<(f32, f32)> {
    let coordinate = |index: i32| {
        let value = unwrap_secret(state, stack_val(state, index))?;
        read_number(state, value)
            .map(|number| number as f32)
            .ok_or_else(|| runtime_error("Usage: C_PingSecure(mousePosX, mousePosY)"))
    };
    Ok((coordinate(1)?, coordinate(2)?))
}

/// WM47 Q8: a forward walk keeps the eligible frame with the highest strata,
/// then frame level; equal keys keep the first frame. INFERRED: the frame list
/// is in creation (ID) order, and GetCursorPosition space is the layout canvas
/// with Y counted from the bottom (WM51-F5: one global factor).
fn find_receiver(sim: &SimState, x: f32, y: f32) -> Option<u64> {
    let point = (x, sim.screen_height - y);
    let mut ids: Vec<u64> = sim.widgets.iter_ids().collect();
    ids.sort_unstable();
    let mut layout = LayoutCache::default();
    let mut best: Option<(u64, (FrameStrata, i32))> = None;
    for id in ids {
        let Some(frame) = sim.widgets.get(id) else {
            continue;
        };
        if !is_candidate(sim, id, frame) || !hit_rect_contains(sim, id, frame, point, &mut layout) {
            continue;
        }
        let key = (frame.frame_strata, frame.frame_level);
        if best.is_none_or(|(_, top)| key > top) {
            best = Some((id, key));
        }
    }
    best.map(|(id, _)| id)
}

/// WM47 Q8: an effectively visible frame with nonzero effective alpha. Mouse
/// input is not required.
fn is_candidate(sim: &SimState, id: u64, frame: &Frame) -> bool {
    !matches!(
        frame.widget_type,
        WidgetType::Texture | WidgetType::FontString | WidgetType::Line
    ) && accepts_pings(frame)
        && frame.effective_alpha > 0.0
        && sim.widgets.is_ancestor_visible(id)
}

/// WM47 Q8: a top-level frame qualifies unless pass-through is set; any other
/// frame needs ping-receiver. Any non-nil value is set, and names match
/// case-insensitively.
fn accepts_pings(frame: &Frame) -> bool {
    if frame.toplevel {
        !has_attribute(frame, "ping-top-level-pass-through")
    } else {
        has_attribute(frame, "ping-receiver")
    }
}

fn has_attribute(frame: &Frame, name: &str) -> bool {
    frame
        .attributes
        .iter()
        .any(|(key, value)| key.eq_ignore_ascii_case(name) && !matches!(value, AttributeValue::Nil))
}

/// The simulator's hit rectangle: the rendered rect minus scaled insets.
/// INFERRED: half-open edges.
fn hit_rect_contains(
    sim: &SimState,
    id: u64,
    frame: &Frame,
    (x, y): (f32, f32),
    layout: &mut LayoutCache,
) -> bool {
    if !crate::layout::frame_has_render_layout(&sim.widgets, id) {
        return false;
    }
    let rect = compute_frame_rect_cached(
        &sim.widgets,
        id,
        sim.screen_width,
        sim.screen_height,
        layout,
    )
    .rect;
    let rect = apply_scroll_offsets(&sim.widgets, id, rect);
    let scale = frame.effective_scale;
    let (left, right, top, bottom) = frame.hit_rect_insets;
    let (x0, y0) = (rect.x + left * scale, rect.y + top * scale);
    let (x1, y1) = (
        rect.x + rect.width - right * scale,
        rect.y + rect.height - bottom * scale,
    );
    x0 <= x && x < x1 && y0 <= y && y < y1
}
