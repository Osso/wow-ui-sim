//! Tooltip ownership, anchoring, and frame-stack methods.

use super::super::shared::{opt_bool, opt_f32, opt_string};
use super::content::{fire_tooltip_script, fire_tooltip_script_with_args};
use super::line_frames::sync_tooltip_line_frame;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, frame_id_from_stack, frame_ref,
    get_or_create_frame_fields, table_set,
};
use crate::lua_api::script_helpers::collect_lua_error;
use crate::lua_api::tooltip::{TooltipLine, build_cursor_anchor};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// `Tooltip:SetOwner(frame, anchor, xOffset, yOffset)`
pub(super) fn set_owner(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let args = tooltip_owner_args(state);
    let mut sim = borrow_state_mut(state)?;
    if !apply_tooltip_owner(&mut sim, tooltip_id, &args) {
        return Ok(0);
    }
    drop(sim);
    let fields = get_or_create_frame_fields(state, tooltip_id);
    let anchor_value = create_string(state, &args.anchor_kind);
    table_set(state, fields, "anchor", anchor_value);
    fire_tooltip_script(state, tooltip_id, "OnTooltipCleared");
    Ok(0)
}

struct TooltipOwnerArgs {
    owner_id: Option<u64>,
    anchor_kind: String,
    x_offset: f32,
    y_offset: f32,
}

fn tooltip_owner_args(state: &mut LuaState) -> TooltipOwnerArgs {
    TooltipOwnerArgs {
        owner_id: frame_id_from_stack(state, 2).ok(),
        anchor_kind: tooltip_anchor_kind_arg(state),
        x_offset: opt_f32(state, 4).unwrap_or(0.0),
        y_offset: opt_f32(state, 5).unwrap_or(0.0),
    }
}

fn tooltip_anchor_kind_arg(state: &mut LuaState) -> String {
    normalized_tooltip_anchor_kind(state, 3)
}

fn normalized_tooltip_anchor_kind(state: &mut LuaState, stack_index: i32) -> String {
    let anchor_kind = opt_string(state, stack_index).unwrap_or_else(|| "ANCHOR_NONE".to_string());
    if is_valid_tooltip_anchor(&anchor_kind) {
        return anchor_kind;
    }

    let _ = collect_lua_error(
        state,
        &format!("invalid anchor type: {anchor_kind}; defaulting to ANCHOR_LEFT"),
    );
    "ANCHOR_LEFT".to_string()
}

fn apply_tooltip_owner(
    sim: &mut crate::lua_api::state::SimState,
    tooltip_id: u64,
    args: &TooltipOwnerArgs,
) -> bool {
    let mouse_position = sim.mouse_position;
    let Some(tooltip) = sim.widgets.get_mut_visual(tooltip_id) else {
        return false;
    };
    tooltip.tooltip_owner_id = args.owner_id;
    apply_tooltip_anchor(
        tooltip,
        &args.anchor_kind,
        args.owner_id,
        mouse_position,
        args.x_offset,
        args.y_offset,
    );
    record_tooltip_owner(sim, tooltip_id, args);
    true
}

fn record_tooltip_owner(
    sim: &mut crate::lua_api::state::SimState,
    tooltip_id: u64,
    args: &TooltipOwnerArgs,
) {
    let td = sim.tooltips.entry(tooltip_id).or_default();
    td.owner_id = args.owner_id;
    td.anchor_type = args.anchor_kind.clone();
    td.anchor_x_offset = args.x_offset;
    td.anchor_y_offset = args.y_offset;
    td.lines.clear();
    td.spell_id = None;
    sim.widgets.mark_rect_dirty(tooltip_id);
    sim.set_frame_visible(tooltip_id, false);
}

pub(super) fn set_object_tooltip_position(state: &mut LuaState) -> LuaResult<u32> {
    use crate::widget::{Anchor, AnchorPoint};

    let tooltip_id = frame_id_from_stack(state, 1)?;
    let owner_id = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(tooltip_id)
            .and_then(|tooltip| tooltip.tooltip_owner_id)
            .or_else(|| sim.tooltips.get(&tooltip_id).and_then(|td| td.owner_id))
    };
    let Some(owner_id) = owner_id else {
        return Ok(0);
    };

    let mut sim = borrow_state_mut(state)?;
    let Some(tooltip) = sim.widgets.get_mut_visual(tooltip_id) else {
        return Ok(0);
    };
    tooltip.anchors.clear();
    tooltip.anchors.push(Anchor {
        point: AnchorPoint::Bottom,
        relative_to: None,
        relative_to_id: Some(owner_id as usize),
        relative_point: AnchorPoint::Top,
        x_offset: 0.0,
        y_offset: 0.0,
    });
    sim.widgets.mark_rect_dirty(tooltip_id);
    Ok(0)
}

fn apply_tooltip_anchor(
    tooltip: &mut crate::widget::Frame,
    anchor_kind: &str,
    owner_id: Option<u64>,
    mouse_position: Option<(f32, f32)>,
    x_offset: f32,
    y_offset: f32,
) {
    if anchor_kind == "ANCHOR_PRESERVE" {
        return;
    }
    tooltip.anchors.clear();
    if is_cursor_tooltip_anchor(anchor_kind) {
        if let Some((mx, my)) = mouse_position {
            tooltip
                .anchors
                .push(build_cursor_anchor(mx, my, x_offset, y_offset));
        }
        return;
    }

    if let Some(owner_id) = owner_id {
        push_owner_tooltip_anchor(tooltip, anchor_kind, owner_id, x_offset, y_offset);
    }
}

fn push_owner_tooltip_anchor(
    tooltip: &mut crate::widget::Frame,
    anchor_kind: &str,
    owner_id: u64,
    x_offset: f32,
    y_offset: f32,
) {
    let Some((point, relative_point)) = owner_tooltip_anchor_points(anchor_kind) else {
        return;
    };
    tooltip.anchors.push(crate::widget::Anchor {
        point,
        relative_to: None,
        relative_to_id: Some(owner_id as usize),
        relative_point,
        x_offset,
        y_offset,
    });
}

fn owner_tooltip_anchor_points(
    anchor_kind: &str,
) -> Option<(crate::widget::AnchorPoint, crate::widget::AnchorPoint)> {
    use crate::widget::AnchorPoint::{
        Bottom, BottomLeft, BottomRight, Left, Right, Top, TopLeft, TopRight,
    };

    match anchor_kind {
        "ANCHOR_RIGHT" => Some((Left, Right)),
        "ANCHOR_LEFT" => Some((Right, Left)),
        "ANCHOR_TOP" => Some((Bottom, Top)),
        "ANCHOR_BOTTOM" => Some((Top, Bottom)),
        "ANCHOR_TOPRIGHT" => Some((BottomRight, TopRight)),
        "ANCHOR_TOPLEFT" => Some((BottomLeft, TopLeft)),
        "ANCHOR_BOTTOMRIGHT" => Some((TopRight, BottomRight)),
        "ANCHOR_BOTTOMLEFT" => Some((TopLeft, BottomLeft)),
        _ => None,
    }
}

pub(super) fn get_owner(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let owner_id = {
        let sim = borrow_state(state)?;
        sim.widgets.get(tooltip_id).and_then(|f| f.tooltip_owner_id)
    };
    let val = match owner_id {
        Some(id) => frame_ref(state, id)?,
        None => Val::Nil,
    };
    state.push(val);
    Ok(1)
}

pub(super) fn is_owned(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let candidate_id = frame_id_from_stack(state, 2).ok();
    let matched = {
        let sim = borrow_state(state)?;
        let tooltip_owner = sim.widgets.get(tooltip_id).and_then(|f| f.tooltip_owner_id);
        match (tooltip_owner, candidate_id) {
            (Some(owner), Some(candidate)) => owner == candidate,
            _ => false,
        }
    };
    state.push(Val::Bool(matched));
    Ok(1)
}

pub(super) fn fade_out(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::frame::methods::core_state::hide(state)
}

pub(super) fn get_anchor_type(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let anchor = {
        let sim = borrow_state(state)?;
        sim.tooltips
            .get(&tooltip_id)
            .map(|td| td.anchor_type.clone())
            .unwrap_or_else(|| "ANCHOR_NONE".to_string())
    };
    let anchor_val = create_string(state, &anchor);
    state.push(anchor_val);
    Ok(1)
}

pub(super) fn set_anchor_type(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let anchor_kind = tooltip_anchor_kind_from_stack(state);
    let x_offset = opt_f32(state, 3).unwrap_or(0.0);
    let y_offset = opt_f32(state, 4).unwrap_or(0.0);
    let owner_id = current_tooltip_owner_id(state, tooltip_id)?;
    apply_anchor_type_to_tooltip(
        state,
        tooltip_id,
        &anchor_kind,
        owner_id,
        x_offset,
        y_offset,
    )?;
    record_tooltip_anchor_field(state, tooltip_id, &anchor_kind);
    Ok(0)
}

fn tooltip_anchor_kind_from_stack(state: &mut LuaState) -> String {
    normalized_tooltip_anchor_kind(state, 2)
}

fn current_tooltip_owner_id(state: &mut LuaState, tooltip_id: u64) -> LuaResult<Option<u64>> {
    let sim = borrow_state(state)?;
    let widget_owner_id = sim
        .widgets
        .get(tooltip_id)
        .and_then(|tooltip| tooltip.tooltip_owner_id);
    Ok(widget_owner_id.or_else(|| sim.tooltips.get(&tooltip_id).and_then(|td| td.owner_id)))
}

fn apply_anchor_type_to_tooltip(
    state: &mut LuaState,
    tooltip_id: u64,
    anchor_kind: &str,
    owner_id: Option<u64>,
    x_offset: f32,
    y_offset: f32,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let mouse_position = sim.mouse_position;
    let Some(tooltip) = sim.widgets.get_mut_visual(tooltip_id) else {
        return Ok(());
    };
    apply_tooltip_anchor(
        tooltip,
        anchor_kind,
        owner_id,
        mouse_position,
        x_offset,
        y_offset,
    );
    let td = sim.tooltips.entry(tooltip_id).or_default();
    td.anchor_type = anchor_kind.to_string();
    td.anchor_x_offset = x_offset;
    td.anchor_y_offset = y_offset;
    sim.widgets.mark_rect_dirty(tooltip_id);
    Ok(())
}

fn record_tooltip_anchor_field(state: &mut LuaState, tooltip_id: u64, anchor_kind: &str) {
    let fields = get_or_create_frame_fields(state, tooltip_id);
    let anchor_value = create_string(state, anchor_kind);
    table_set(state, fields, "anchor", anchor_value);
}

fn is_valid_tooltip_anchor(anchor_kind: &str) -> bool {
    matches!(
        anchor_kind,
        "ANCHOR_NONE"
            | "ANCHOR_PRESERVE"
            | "ANCHOR_RIGHT"
            | "ANCHOR_LEFT"
            | "ANCHOR_TOP"
            | "ANCHOR_BOTTOM"
            | "ANCHOR_TOPRIGHT"
            | "ANCHOR_TOPLEFT"
            | "ANCHOR_BOTTOMRIGHT"
            | "ANCHOR_BOTTOMLEFT"
            | "ANCHOR_CURSOR"
            | "ANCHOR_CURSOR_RIGHT"
            | "ANCHOR_CURSOR_LEFT"
    )
}

fn is_cursor_tooltip_anchor(anchor_kind: &str) -> bool {
    matches!(
        anchor_kind,
        "ANCHOR_CURSOR" | "ANCHOR_CURSOR_RIGHT" | "ANCHOR_CURSOR_LEFT"
    )
}

pub(super) fn copy_tooltip(state: &mut LuaState) -> LuaResult<u32> {
    let target_id = frame_id_from_stack(state, 1)?;
    let Ok(source_id) = frame_id_from_stack(state, 2) else {
        return Ok(0);
    };
    let source = {
        let sim = borrow_state(state)?;
        sim.tooltips.get(&source_id).cloned()
    };
    let Some(source) = source else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    let target = sim.tooltips.entry(target_id).or_default();
    let preserved_owner = target.owner_id;
    let preserved_anchor = target.anchor_type.clone();
    let preserved_x = target.anchor_x_offset;
    let preserved_y = target.anchor_y_offset;
    *target = source;
    target.owner_id = preserved_owner;
    target.anchor_type = preserved_anchor;
    target.anchor_x_offset = preserved_x;
    target.anchor_y_offset = preserved_y;
    Ok(0)
}

pub(super) fn set_frame_stack(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let _show_hidden = opt_bool(state, 2).unwrap_or(false);
    let _show_regions = opt_bool(state, 3).unwrap_or(false);
    let frame_stack_index = opt_f32(state, 4).unwrap_or(0.0).max(0.0) as usize;
    let highlight_id = hovered_frame_id(state)?;
    let Some(highlight_id) = highlight_id else {
        clear_tooltip_lines(state, tooltip_id)?;
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(frame_info) = frame_stack_info(state, highlight_id)? else {
        clear_tooltip_lines(state, tooltip_id)?;
        state.push(Val::Nil);
        return Ok(1);
    };
    let highlight = frame_global_or_ref_local(state, highlight_id)?;
    write_frame_stack_tooltip(state, tooltip_id, frame_stack_index, frame_info)?;
    fire_tooltip_script_with_args(
        state,
        tooltip_id,
        "OnTooltipSetFramestack",
        &[highlight.clone()],
    );
    state.push(highlight);
    Ok(1)
}

struct FrameStackInfo {
    primary: String,
    parent_label: String,
}

fn hovered_frame_id(state: &mut LuaState) -> LuaResult<Option<u64>> {
    let sim = borrow_state(state)?;
    Ok(sim.hovered_frame)
}

fn frame_stack_info(state: &mut LuaState, frame_id: u64) -> LuaResult<Option<FrameStackInfo>> {
    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(frame_id) else {
        return Ok(None);
    };
    let primary = frame
        .name
        .clone()
        .or_else(|| frame.object_type_name.clone())
        .unwrap_or_else(|| "Frame".into());
    let parent_label = frame
        .parent_id
        .and_then(|pid| sim.widgets.get(pid))
        .and_then(|parent| parent.name.clone())
        .unwrap_or_else(|| frame.widget_type.as_str().to_string());
    Ok(Some(FrameStackInfo {
        primary,
        parent_label,
    }))
}

fn write_frame_stack_tooltip(
    state: &mut LuaState,
    tooltip_id: u64,
    frame_stack_index: usize,
    frame_info: FrameStackInfo,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let td = sim.tooltips.entry(tooltip_id).or_default();
    td.frame_stack_index = frame_stack_index;
    td.lines.clear();
    td.lines.push(frame_stack_primary_line(frame_info.primary));
    td.lines
        .push(frame_stack_parent_line(frame_info.parent_label));
    sim.set_frame_visible(tooltip_id, true);
    Ok(())
}

fn frame_stack_primary_line(primary: String) -> TooltipLine {
    TooltipLine {
        left_text: primary,
        left_color: (1.0, 1.0, 1.0),
        left_segments: Vec::new(),
        right_text: None,
        right_color: (1.0, 1.0, 1.0),
        right_segments: Vec::new(),
        wrap: false,
        texture: None,
    }
}

fn frame_stack_parent_line(parent_label: String) -> TooltipLine {
    TooltipLine {
        left_text: format!("Parent: {parent_label}"),
        left_color: (0.8, 0.8, 0.8),
        left_segments: Vec::new(),
        right_text: None,
        right_color: (1.0, 1.0, 1.0),
        right_segments: Vec::new(),
        wrap: false,
        texture: None,
    }
}

fn clear_tooltip_lines(state: &mut LuaState, tooltip_id: u64) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips.entry(tooltip_id).or_default().lines.clear();
    Ok(())
}

fn frame_global_or_ref_local(state: &mut LuaState, id: u64) -> LuaResult<Val> {
    let frame_name = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|frame| frame.name.clone())
    };
    if let Some(name) = frame_name {
        let key = state.gc.intern_string(name.as_bytes());
        let global = state
            .gc
            .tables
            .get(state.global)
            .map(|table| table.get_str(key, &state.gc.string_arena))
            .unwrap_or(Val::Nil);
        if global != Val::Nil {
            return Ok(global);
        }
    }
    frame_ref(state, id)
}

pub(super) fn add_font_strings(_state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(_state, 1)?;
    let line_count = {
        let sim = borrow_state(_state)?;
        sim.tooltips
            .get(&tooltip_id)
            .map(|td| td.lines.len())
            .unwrap_or(0)
    };
    for line_index in 1..=line_count {
        let _ = sync_tooltip_line_frame(_state, tooltip_id, false, line_index)?;
        let _ = sync_tooltip_line_frame(_state, tooltip_id, true, line_index)?;
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// register_tooltip
// ---------------------------------------------------------------------------
