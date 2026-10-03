//! Tooltip line and layout methods.

use super::super::shared::{opt_f32, opt_string, val_to_bool, val_to_f64};
use super::content::fire_tooltip_script;
use super::line_frames::{
    line_color_segments, normal_font_color, sync_tooltip_line_frame, table_array_get,
};
use super::sizing::refresh_tooltip_geometry;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, frame_id_from_stack, frame_ref, table_get,
    val_to_string,
};
use crate::lua_api::tooltip::{TooltipLine, TooltipTextSegment};
use crate::lua_bridge::{IntoStack, stack_val};
use crate::widget::WidgetType;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn clear_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    let line_ids = {
        let td = sim.tooltips.entry(id).or_default();
        td.lines.clear();
        td.spell_id = None;
        td.unit_token = None;
        td.unit_name = None;
        td.unit_guid = None;
        td.left_line_ids
            .iter()
            .chain(&td.right_line_ids)
            .copied()
            .collect::<Vec<_>>()
    };
    clear_cached_tooltip_line_text(&mut sim, &line_ids);
    refresh_tooltip_geometry(&mut sim, id);
    drop(sim);
    fire_tooltip_script(state, id, "OnTooltipCleared");
    Ok(0)
}

fn clear_cached_tooltip_line_text(sim: &mut crate::lua_api::state::SimState, line_ids: &[u64]) {
    for &line_id in line_ids {
        if let Some(frame) = sim.widgets.get_mut_visual(line_id) {
            frame.text = None;
            frame.text_stripped = None;
            frame.text_segments.clear();
            frame.secret_text = false;
        }
    }
}

pub(super) fn add_line(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::tooltip::TooltipLine;
    let id = frame_id_from_stack(state, 1)?;
    let text = opt_string(state, 2).unwrap_or_default();
    let (default_r, default_g, default_b) = normal_font_color(state);
    let r = opt_f32(state, 3).unwrap_or(default_r);
    let g = opt_f32(state, 4).unwrap_or(default_g);
    let b = opt_f32(state, 5).unwrap_or(default_b);
    let wrap = val_to_bool(stack_val(state, 6));
    let line_index = next_tooltip_line_index(state, id);
    let left_segments = processing_line_color_segments(
        state,
        id,
        line_index,
        "leftText",
        &text,
        "leftColorSegments",
    );
    let mut sim = borrow_state_mut(state)?;
    let td = sim.tooltips.entry(id).or_default();
    td.lines.push(TooltipLine {
        left_text: text,
        left_color: (r, g, b),
        left_segments,
        right_text: None,
        right_color: (1.0, 1.0, 1.0),
        right_segments: Vec::new(),
        wrap,
        texture: None,
    });
    refresh_tooltip_geometry(&mut sim, id);
    drop(sim);
    let _ = sync_tooltip_line_frame(state, id, false, line_index)?;
    Ok(0)
}

pub(super) fn add_double_line(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let args = double_line_args(state);
    let line_index = next_tooltip_line_index(state, id);
    let left_segments = processing_line_color_segments(
        state,
        id,
        line_index,
        "leftText",
        &args.left_text,
        "leftColorSegments",
    );
    let right_segments = processing_right_line_color_segments(state, id, line_index, &args);
    push_double_tooltip_line(state, id, args, left_segments, right_segments)?;
    let _ = sync_tooltip_line_frame(state, id, false, line_index)?;
    let _ = sync_tooltip_line_frame(state, id, true, line_index)?;

    Ok(0)
}

struct DoubleLineArgs {
    left_text: String,
    right_text: Option<String>,
    left_color: (f32, f32, f32),
    right_color: (f32, f32, f32),
    wrap: bool,
}

fn double_line_args(state: &mut LuaState) -> DoubleLineArgs {
    let left_text = opt_string(state, 2).unwrap_or_default();
    let right_text = opt_string(state, 3);
    let (default_r, default_g, default_b) = normal_font_color(state);
    let left_color = (
        opt_f32(state, 4).unwrap_or(default_r),
        opt_f32(state, 5).unwrap_or(default_g),
        opt_f32(state, 6).unwrap_or(default_b),
    );
    let right_color = (
        opt_f32(state, 7).unwrap_or(left_color.0),
        opt_f32(state, 8).unwrap_or(left_color.1),
        opt_f32(state, 9).unwrap_or(left_color.2),
    );
    let wrap = val_to_bool(stack_val(state, 10));
    DoubleLineArgs {
        left_text,
        right_text,
        left_color,
        right_color,
        wrap,
    }
}

fn processing_right_line_color_segments(
    state: &mut LuaState,
    tooltip_id: u64,
    line_index: usize,
    args: &DoubleLineArgs,
) -> Vec<TooltipTextSegment> {
    args.right_text
        .as_deref()
        .map(|text| {
            processing_line_color_segments(
                state,
                tooltip_id,
                line_index,
                "rightText",
                text,
                "rightColorSegments",
            )
        })
        .unwrap_or_default()
}

fn push_double_tooltip_line(
    state: &mut LuaState,
    tooltip_id: u64,
    args: DoubleLineArgs,
    left_segments: Vec<TooltipTextSegment>,
    right_segments: Vec<TooltipTextSegment>,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let td = sim.tooltips.entry(tooltip_id).or_default();
    td.lines.push(TooltipLine {
        left_text: args.left_text,
        left_color: args.left_color,
        left_segments,
        right_text: args.right_text,
        right_color: args.right_color,
        right_segments,
        wrap: args.wrap,
        texture: None,
    });
    refresh_tooltip_geometry(&mut sim, tooltip_id);
    Ok(())
}

pub(super) fn num_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let is_tooltip = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .is_some_and(|frame| frame.widget_type == WidgetType::GameTooltip)
    };
    if !is_tooltip {
        return crate::lua_api::frame::methods::text_attribute_event::get_text_num_lines(state);
    }
    let line_count = {
        let sim = borrow_state(state)?;
        sim.tooltips.get(&id).map(|td| td.lines.len()).unwrap_or(0)
    };
    (line_count as f64).into_stack(state)
}

fn next_tooltip_line_index(state: &mut LuaState, tooltip_id: u64) -> usize {
    borrow_state(state)
        .ok()
        .and_then(|sim| sim.tooltips.get(&tooltip_id).map(|td| td.lines.len() + 1))
        .unwrap_or(1)
}

fn processing_line_color_segments(
    state: &mut LuaState,
    tooltip_id: u64,
    line_index: usize,
    text_key: &str,
    text: &str,
    segment_key: &str,
) -> Vec<TooltipTextSegment> {
    let Some(line) = processing_tooltip_line(state, tooltip_id, line_index, text_key, text) else {
        return Vec::new();
    };
    line_color_segments(state, line, segment_key)
}

fn processing_tooltip_line(
    state: &mut LuaState,
    tooltip_id: u64,
    line_index: usize,
    text_key: &str,
    text: &str,
) -> Option<Val> {
    let tooltip = frame_ref(state, tooltip_id).ok()?;
    let processing_info = table_get(state, tooltip, "processingInfo");
    let tooltip_data = table_get(state, processing_info, "tooltipData");
    let lines = table_get(state, tooltip_data, "lines");
    processing_tooltip_line_at(state, lines, line_index, text_key, text)
        .or_else(|| find_processing_tooltip_line(state, lines, text_key, text))
}

fn processing_tooltip_line_at(
    state: &mut LuaState,
    lines: Val,
    line_index: usize,
    text_key: &str,
    text: &str,
) -> Option<Val> {
    let line = table_array_get(state, lines, line_index as i64);
    processing_line_text_matches(state, line, text_key, text).then_some(line)
}

fn find_processing_tooltip_line(
    state: &mut LuaState,
    lines: Val,
    text_key: &str,
    text: &str,
) -> Option<Val> {
    let mut index = 1;
    loop {
        let line = table_array_get(state, lines, index);
        if !matches!(line, Val::Table(_)) {
            return None;
        }
        if processing_line_text_matches(state, line, text_key, text) {
            return Some(line);
        }
        index += 1;
    }
}

fn processing_line_text_matches(
    state: &mut LuaState,
    line: Val,
    text_key: &str,
    text: &str,
) -> bool {
    let line_text = table_get(state, line, text_key);
    val_to_string(state, line_text).as_deref() == Some(text)
}

pub(super) fn set_custom_line_spacing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let spacing = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips.entry(id).or_default().line_spacing = Some(spacing);
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}

pub(super) fn get_custom_line_spacing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .tooltips
        .get(&id)
        .and_then(|td| td.line_spacing)
        .map(|s| s as f64)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_minimum_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let width = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips.entry(id).or_default().min_width = width;
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}

pub(super) fn get_minimum_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .tooltips
        .get(&id)
        .map(|td| td.min_width as f64)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_allow_show_with_no_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips.entry(id).or_default().allow_show_with_no_lines = value;
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}

pub(super) fn set_custom_word_wrap_min_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let width = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips
        .entry(id)
        .or_default()
        .custom_word_wrap_min_width = Some(width);
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}

pub(super) fn set_shrink_to_fit_wrapped(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips.entry(id).or_default().shrink_to_fit_wrapped = value;
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}

pub(super) fn get_spell(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let spell_id = {
        let sim = borrow_state(state)?;
        sim.tooltips.get(&id).and_then(|td| td.spell_id)
    };
    match spell_id {
        Some(id) => {
            let name = crate::spells::get_spell(id)
                .map(|s| s.name.to_string())
                .unwrap_or_else(|| format!("Spell {}", id));
            let name_val = create_string(state, &name);
            name_val.into_stack(state)?;
            (id as f64).into_stack(state)?;
            Ok(2)
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Nil);
            Ok(2)
        }
    }
}

pub(super) fn get_unit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let unit = {
        let sim = borrow_state(state)?;
        sim.tooltips.get(&id).and_then(|td| {
            Some((
                td.unit_name.clone()?,
                td.unit_token.clone()?,
                td.unit_guid.clone()?,
            ))
        })
    };
    match unit {
        Some((name, token, guid)) => {
            let name = create_string(state, &name);
            let token = create_string(state, &token);
            let guid = create_string(state, &guid);
            state.push(name);
            state.push(token);
            state.push(guid);
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Nil);
            state.push(Val::Nil);
        }
    }
    Ok(3)
}

pub(super) fn get_item(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(2)
}

pub(super) fn set_padding(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let padding = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips.entry(id).or_default().padding = padding;
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}

pub(super) fn get_padding(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .tooltips
        .get(&id)
        .map(|td| td.padding as f64)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn clear_padding(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(td) = sim.tooltips.get_mut(&id) {
        td.padding = 0.0;
    }
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}

pub(super) fn append_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text = opt_string(state, 2).unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    if let Some(last) = sim.tooltips.entry(id).or_default().lines.last_mut() {
        last.left_text.push_str(&text);
    }
    refresh_tooltip_geometry(&mut sim, id);
    Ok(0)
}
