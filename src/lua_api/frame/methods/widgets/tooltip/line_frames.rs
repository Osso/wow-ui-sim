//! Tooltip line frame helpers.

use super::super::shared::{opt_f32, opt_string};
use super::sizing::refresh_tooltip_geometry;
use crate::lua_api::globals::create_frame::create_frame_instance;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, frame_id_from_stack, frame_ref, table_get, val_to_string,
};
use crate::lua_api::tooltip::{TooltipLine, TooltipTextSegment, TooltipTexture};
use crate::lua_bridge::stack_val;
use crate::widget::{TextJustify, WidgetType};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

fn tooltip_line_exists(state: &LuaState, tooltip_id: u64, line_index: usize) -> bool {
    if line_index == 0 {
        return false;
    }
    let Ok(sim) = borrow_state(state) else {
        return false;
    };
    let line_count = sim.tooltips.get(&tooltip_id).map(|td| td.lines.len());
    line_count.is_some_and(|count| line_index <= count)
}

fn tooltip_line_name(
    state: &LuaState,
    tooltip_id: u64,
    right_side: bool,
    line_index: usize,
) -> Option<String> {
    let sim = borrow_state(state).ok()?;
    let tooltip_name = sim.widgets.get(tooltip_id)?.name.as_ref()?;
    let suffix = if right_side { "Right" } else { "Left" };
    Some(format!("{tooltip_name}Text{suffix}{line_index}"))
}

fn find_existing_tooltip_line_id(
    state: &LuaState,
    tooltip_id: u64,
    right_side: bool,
    line_index: usize,
) -> Option<u64> {
    let sim = borrow_state(state).ok()?;
    let td = sim.tooltips.get(&tooltip_id)?;
    let existing = if right_side {
        td.right_line_ids.get(line_index - 1).copied()
    } else {
        td.left_line_ids.get(line_index - 1).copied()
    };
    if let Some(id) = existing
        && sim.widgets.get(id).is_some_and(|frame| {
            frame.parent_id == Some(tooltip_id) && frame.widget_type == WidgetType::FontString
        })
    {
        return Some(id);
    }

    let name = tooltip_line_name(state, tooltip_id, right_side, line_index)?;
    let child_id = sim.widgets.get_id_by_name(&name)?;
    sim.widgets.get(child_id).and_then(|frame| {
        (frame.parent_id == Some(tooltip_id) && frame.widget_type == WidgetType::FontString)
            .then_some(child_id)
    })
}

fn ensure_tooltip_line_id(
    state: &mut LuaState,
    tooltip_id: u64,
    right_side: bool,
    line_index: usize,
) -> LuaResult<Option<u64>> {
    if !tooltip_line_exists(state, tooltip_id, line_index) {
        return Ok(None);
    }
    if let Some(existing_id) =
        find_existing_tooltip_line_id(state, tooltip_id, right_side, line_index)
    {
        if matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Wrath
                | crate::client_profile::ClientProfile::Retail
                | crate::client_profile::ClientProfile::Ptr
        ) {
            let mut sim = borrow_state_mut(state)?;
            record_tooltip_line_child_id(&mut sim, tooltip_id, existing_id, right_side, line_index);
        }
        return Ok(Some(existing_id));
    }

    let name = tooltip_line_name(state, tooltip_id, right_side, line_index);
    let child_id = create_frame_instance(
        state,
        WidgetType::FontString,
        "FontString",
        name,
        Some(tooltip_id),
        true,
        None,
    )?;
    {
        let mut sim = borrow_state_mut(state)?;
        configure_tooltip_line_child(&mut sim, child_id, right_side, line_index);
        record_tooltip_line_child_id(&mut sim, tooltip_id, child_id, right_side, line_index);
    }
    Ok(Some(child_id))
}

fn configure_tooltip_line_child(
    sim: &mut crate::lua_api::state::SimState,
    child_id: u64,
    right_side: bool,
    line_index: usize,
) {
    let Some(child) = sim.widgets.get_mut_visual(child_id) else {
        return;
    };
    child.parent_key = Some(tooltip_line_parent_key(right_side, line_index));
    child.justify_h = tooltip_line_justify(right_side);
}

fn tooltip_line_parent_key(right_side: bool, line_index: usize) -> String {
    if right_side {
        format!("TextRight{line_index}")
    } else {
        format!("TextLeft{line_index}")
    }
}

fn tooltip_line_justify(right_side: bool) -> TextJustify {
    if right_side {
        TextJustify::Right
    } else {
        TextJustify::Left
    }
}

fn record_tooltip_line_child_id(
    sim: &mut crate::lua_api::state::SimState,
    tooltip_id: u64,
    child_id: u64,
    right_side: bool,
    line_index: usize,
) {
    let td = sim.tooltips.entry(tooltip_id).or_default();
    let ids = if right_side {
        &mut td.right_line_ids
    } else {
        &mut td.left_line_ids
    };
    if ids.len() < line_index {
        ids.resize(line_index, 0);
    }
    ids[line_index - 1] = child_id;
}

pub(super) fn sync_tooltip_line_frame(
    state: &mut LuaState,
    tooltip_id: u64,
    right_side: bool,
    line_index: usize,
) -> LuaResult<Option<u64>> {
    let Some(line_id) = ensure_tooltip_line_id(state, tooltip_id, right_side, line_index)? else {
        return Ok(None);
    };
    let line = {
        let sim = borrow_state(state)?;
        sim.tooltips
            .get(&tooltip_id)
            .and_then(|td| td.lines.get(line_index - 1).cloned())
    };
    let Some(line) = line else {
        return Ok(None);
    };
    let wrap = line.wrap;
    let (text, color, text_segments) = tooltip_line_frame_content(line, right_side);
    let stripped = text
        .as_ref()
        .map(|value| crate::render::strip_wow_markup(value));
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(line_id) {
        frame.text = text;
        frame.text_stripped = stripped;
        frame.text_color = color;
        frame.text_segments = text_segments;
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
            frame.word_wrap = !right_side && wrap;
            frame.visible = frame.text.as_ref().is_some_and(|text| !text.is_empty());
        }
    }
    Ok(Some(line_id))
}

fn tooltip_line_frame_content(
    line: TooltipLine,
    right_side: bool,
) -> (
    Option<String>,
    crate::widget::Color,
    Vec<crate::widget::TextSegment>,
) {
    if right_side {
        (
            line.right_text,
            tooltip_line_frame_color(line.right_color),
            tooltip_line_frame_segments(line.right_segments),
        )
    } else {
        (
            Some(line.left_text),
            tooltip_line_frame_color(line.left_color),
            tooltip_line_frame_segments(line.left_segments),
        )
    }
}

fn tooltip_line_frame_color(color: (f32, f32, f32)) -> crate::widget::Color {
    let (r, g, b) = color;
    crate::widget::Color::new(r, g, b, 1.0)
}

fn tooltip_line_frame_segments(
    segments: Vec<TooltipTextSegment>,
) -> Vec<crate::widget::TextSegment> {
    segments
        .into_iter()
        .map(|segment| crate::widget::TextSegment {
            text: segment.text,
            color: crate::widget::Color::new(
                segment.color.0,
                segment.color.1,
                segment.color.2,
                1.0,
            ),
        })
        .collect()
}

fn push_tooltip_line_ref(
    state: &mut LuaState,
    tooltip_id: u64,
    right_side: bool,
    line_index: usize,
) -> LuaResult<u32> {
    if line_index == 0 {
        state.push(Val::Nil);
        return Ok(1);
    }
    let line_id = sync_tooltip_line_frame(state, tooltip_id, right_side, line_index)?;
    let Some(line_id) = line_id else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let line_ref = frame_ref(state, line_id)?;
    state.push(line_ref);
    Ok(1)
}

fn add_texture_line(
    state: &mut LuaState,
    tooltip_id: u64,
    texture: TooltipTexture,
) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.tooltips
        .entry(tooltip_id)
        .or_default()
        .lines
        .push(TooltipLine {
            left_text: String::new(),
            left_color: (1.0, 1.0, 1.0),
            left_segments: Vec::new(),
            right_text: None,
            right_color: (1.0, 1.0, 1.0),
            right_segments: Vec::new(),
            wrap: false,
            texture: Some(texture),
        });
    refresh_tooltip_geometry(&mut sim, tooltip_id);
    Ok(0)
}

pub(super) fn add_texture(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        return super::wrath::attach_texture(state, tooltip_id);
    }
    let texture = match stack_val(state, 2) {
        Val::Num(value) if value >= 0.0 => Some(TooltipTexture::FileDataId(value as u32)),
        value => val_to_string(state, value)
            .and_then(|text| text.parse::<u32>().ok())
            .map(TooltipTexture::FileDataId),
    };
    let Some(texture) = texture else {
        #[cfg(feature = "client-retail")]
        attach_path_to_blank_line(state, tooltip_id)?;
        return Ok(0);
    };
    add_texture_line(state, tooltip_id, texture)
}

#[cfg(feature = "client-retail")]
fn read_path_attachment(
    state: &mut LuaState,
) -> Option<crate::lua_api::tooltip::TooltipPathAttachment> {
    let path = opt_string(state, 2)?;
    let options = stack_val(state, 3);
    let anchor = table_get(state, options, "anchor");
    const TOOLTIP_TEXTURE_ANCHOR_ALL: f64 = 6.0;
    if !matches!(anchor, Val::Num(value) if value == TOOLTIP_TEXTURE_ANCHOR_ALL) {
        return None;
    }
    let Val::Num(height) = table_get(state, options, "height") else {
        return None;
    };
    let margin = table_get(state, options, "margin");
    let mut margins = [0.0; 4];
    for (index, key) in ["left", "right", "top", "bottom"].iter().enumerate() {
        let Val::Num(value) = table_get(state, margin, key) else {
            return None;
        };
        margins[index] = value as f32;
    }
    Some(crate::lua_api::tooltip::TooltipPathAttachment {
        path,
        height: height as f32,
        margins,
    })
}

#[cfg(feature = "client-retail")]
fn attach_path_to_blank_line(state: &mut LuaState, tooltip_id: u64) -> LuaResult<()> {
    let Some(attachment) = read_path_attachment(state) else {
        return Ok(());
    };
    let mut sim = borrow_state_mut(state)?;
    let Some(line) = sim
        .tooltips
        .get_mut(&tooltip_id)
        .and_then(|td| td.lines.last_mut())
    else {
        return Ok(());
    };
    // SocialUIUtil supplies this blank wrapped row before AddTexture.
    // Other attachment forms remain outside the U1 contract.
    if !line.wrap || line.left_text != " " || line.texture.is_some() {
        return Ok(());
    }
    line.texture = Some(TooltipTexture::PathAttachment(attachment));
    refresh_tooltip_geometry(&mut sim, tooltip_id);
    Ok(())
}

pub(super) fn add_atlas(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let Some(atlas) = opt_string(state, 2) else {
        return Ok(0);
    };
    add_texture_line(state, tooltip_id, TooltipTexture::Atlas(atlas))
}

pub(super) fn get_left_line(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let line_index = opt_f32(state, 2).unwrap_or(0.0).max(0.0) as usize;
    push_tooltip_line_ref(state, tooltip_id, false, line_index)
}

pub(super) fn get_right_line(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip_id = frame_id_from_stack(state, 1)?;
    let line_index = opt_f32(state, 2).unwrap_or(0.0).max(0.0) as usize;
    push_tooltip_line_ref(state, tooltip_id, true, line_index)
}

pub(super) fn table_array_get(state: &LuaState, table: Val, index: i64) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.get_int(index))
        .unwrap_or(Val::Nil)
}

fn line_color_component(state: &mut LuaState, color: Val, key: &str, default: f32) -> f32 {
    match table_get(state, color, key) {
        Val::Num(value) => value as f32,
        _ => default,
    }
}

pub(super) fn normal_font_color(state: &mut LuaState) -> (f32, f32, f32) {
    let globals = Val::Table(state.global);
    let normal = table_get(state, globals, "NORMAL_FONT_COLOR");
    if !matches!(normal, Val::Table(_)) {
        return (1.0, 0.82, 0.0);
    }
    (
        line_color_component(state, normal, "r", 1.0),
        line_color_component(state, normal, "g", 0.82),
        line_color_component(state, normal, "b", 0.0),
    )
}

fn line_color(state: &mut LuaState, line: Val, key: &str) -> (f32, f32, f32) {
    let color = table_get(state, line, key);
    if !matches!(color, Val::Table(_)) {
        return normal_font_color(state);
    }
    (
        line_color_component(state, color, "r", 1.0),
        line_color_component(state, color, "g", 1.0),
        line_color_component(state, color, "b", 1.0),
    )
}

pub(super) fn line_color_segments(
    state: &mut LuaState,
    line: Val,
    key: &str,
) -> Vec<TooltipTextSegment> {
    let segments_table = table_get(state, line, key);
    let mut segments = Vec::new();
    let mut index = 1;

    loop {
        let segment = table_array_get(state, segments_table, index);
        if !matches!(segment, Val::Table(_)) {
            break;
        }
        let text_val = table_get(state, segment, "text");
        let text = val_to_string(state, text_val).unwrap_or_default();
        let color = line_color(state, segment, "color");
        segments.push(TooltipTextSegment { text, color });
        index += 1;
    }

    segments
}

pub(super) fn tooltip_line_from_table(state: &mut LuaState, line: Val) -> Option<TooltipLine> {
    if !matches!(line, Val::Table(_)) {
        return None;
    }
    let left_text_val = table_get(state, line, "leftText");
    let right_text_val = table_get(state, line, "rightText");
    let wrap_val = table_get(state, line, "wrapText");
    let left_text = val_to_string(state, left_text_val).unwrap_or_default();
    let right_text = val_to_string(state, right_text_val);
    let wrap = matches!(wrap_val, Val::Bool(true));
    Some(TooltipLine {
        left_text,
        left_color: line_color(state, line, "leftColor"),
        left_segments: line_color_segments(state, line, "leftColorSegments"),
        right_text,
        right_color: line_color(state, line, "rightColor"),
        right_segments: line_color_segments(state, line, "rightColorSegments"),
        wrap,
        texture: None,
    })
}
