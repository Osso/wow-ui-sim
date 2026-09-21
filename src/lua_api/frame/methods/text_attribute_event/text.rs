//! Text-related RustFn methods: SetText, GetText, font, color, justification, wrapping.

mod formatting;
mod metrics;
mod simple_html;
mod style;
#[cfg(feature = "retail-12-0-0")]
pub(super) use style::{get_scale_animation_mode, set_scale_animation_mode};

pub(super) use formatting::{
    apply_default_text, get_font, get_font_height, get_font_object, get_unbounded_string_width,
    get_wrapped_width, is_truncated, scale_text_to_fit, set_font, set_font_height, set_font_object,
    set_font_objects_to_try, set_formatted_text, set_text_height, set_text_to_fit,
    try_apply_default_text,
};
use metrics::{approximate_text_height, approximate_text_width};
pub(super) use style::{
    can_non_space_wrap, can_word_wrap, get_hyperlink_format, get_hyperlinks_enabled,
    get_indented_word_wrap, get_justify_h, get_justify_v, get_max_lines, get_shadow_color,
    get_shadow_offset, get_text_color, get_text_scale, get_word_wrap, set_fixed_color,
    set_hyperlink_format, set_hyperlinks_enabled, set_indented_word_wrap, set_justify_h,
    set_justify_v, set_max_lines, set_non_space_wrap, set_shadow_color, set_shadow_offset,
    set_text_color, set_text_scale, set_word_wrap,
};

use super::helpers::val_to_f32;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, frame_id_from_stack,
    get_or_create_frame_fields, table_set,
};
use crate::lua_api::state::SimState;
use crate::lua_bridge::stack_val;
use crate::widget::{Color, WidgetType};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use simple_html::{build_simple_html_text_data, is_simple_html_frame};

use crate::lua_api::frame::methods::button_anchor_hierarchy::ensure_button_text_child;

const BUTTON_TEXT_CHILD_KEYS: [&str; 3] = ["Text", "text", "ButtonText"];

#[derive(Copy, Clone)]
struct TooltipLineValues {
    r: Val,
    g: Val,
    b: Val,
    a: Val,
    wrap: Val,
}

struct AutoTextHeightState {
    is_fontstring: bool,
    has_text: bool,
    width: f32,
    width_is_text_auto: bool,
    height_is_text_auto: bool,
    word_wrap: bool,
    height: f32,
}

struct LineCountProps {
    has_text: bool,
    line_height: f32,
    wrap_width: Option<f32>,
}

pub(super) fn set_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (value, secret) =
        crate::lua_api::frame::methods::secret_origin::unwrap_input(state, stack_val(state, 2))?;
    if cfg!(feature = "client-wowforever") && matches!(value, Val::Userdata(_)) {
        return Err(rilua::runtime_error("expected text, not userdata"));
    }
    let text = read_text_arg(state, value);
    let tooltip = read_tooltip_line_values(state);
    let stripped_text = stripped_text_for_frame(state, id, text.clone())?;
    let (is_tooltip, should_update_button_child) =
        update_text_frame(state, id, &text, &stripped_text)?;
    sync_button_text_child(state, id, &text, &stripped_text, should_update_button_child)?;
    set_text_secret_origin(state, id, secret)?;
    refresh_text_measurements(state, id);
    if is_tooltip {
        sync_tooltip_text(state, id, text, tooltip)?;
    }
    Ok(0)
}

fn read_tooltip_line_values(state: &LuaState) -> TooltipLineValues {
    TooltipLineValues {
        r: stack_val(state, 3),
        g: stack_val(state, 4),
        b: stack_val(state, 5),
        a: stack_val(state, 6),
        wrap: stack_val(state, 7),
    }
}

fn resolved_frame_width(state: &LuaState, id: u64) -> f32 {
    let Ok(sim) = borrow_state(state) else {
        return 0.0;
    };
    let mut cache = crate::layout::LayoutCache::default();
    crate::layout::compute_frame_rect_cached(
        &sim.widgets,
        id,
        sim.screen_width,
        sim.screen_height,
        &mut cache,
    )
    .rect
    .width
    .max(0.0)
}

fn strip_html_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn prepare_stripped_text(widget_type: WidgetType, text: Option<String>) -> Option<String> {
    text.map(|value| {
        if widget_type == WidgetType::SimpleHTML {
            crate::render::strip_wow_markup(&strip_html_tags(&value))
        } else {
            crate::render::strip_wow_markup(&value)
        }
    })
}

fn leading_wow_text_color(text: &Option<String>) -> Option<Color> {
    let text = text.as_deref()?;
    let color_code = text.strip_prefix("|c")?.get(..8)?;
    let argb = u32::from_str_radix(color_code, 16).ok()?;
    let a = ((argb >> 24) & 0xff) as f32 / 255.0;
    let r = ((argb >> 16) & 0xff) as f32 / 255.0;
    let g = ((argb >> 8) & 0xff) as f32 / 255.0;
    let b = (argb & 0xff) as f32 / 255.0;
    Some(Color::new(r, g, b, a))
}

fn stripped_text_for_frame(
    state: &LuaState,
    id: u64,
    text: Option<String>,
) -> LuaResult<Option<String>> {
    let widget_type = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.widget_type)
        .unwrap_or(WidgetType::FontString);
    Ok(prepare_stripped_text(widget_type, text))
}

fn update_text_frame(
    state: &mut LuaState,
    id: u64,
    text: &Option<String>,
    stripped_text: &Option<String>,
) -> LuaResult<(bool, bool)> {
    let mut sim = borrow_state_mut(state)?;
    let frame = sim.widgets.get(id);
    let current_text = frame.and_then(|frame| frame_text_value(&sim, frame, false));
    let current_stripped_text = frame.and_then(|frame| frame_text_value(&sim, frame, true));
    let is_tooltip = frame
        .map(|frame| frame.widget_type == WidgetType::GameTooltip)
        .unwrap_or(false);
    let is_button = matches!(
        frame.map(|frame| frame.widget_type),
        Some(WidgetType::Button | WidgetType::CheckButton)
    );
    let inline_color = leading_wow_text_color(text);
    let has_button_text_child = frame.and_then(button_text_child_id).is_some();
    let changed = is_tooltip || current_text != *text || current_stripped_text != *stripped_text;
    if changed && let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.text = text.clone();
        frame.text_stripped = stripped_text.clone();
        frame.text_segments.clear();
        if let Some(color) = inline_color {
            frame.text_color = color;
        }
    }
    let should_update_button_child =
        is_button && (changed || (!has_button_text_child && text.is_some()));
    Ok((is_tooltip, should_update_button_child))
}

fn sync_button_text_child(
    state: &mut LuaState,
    id: u64,
    text: &Option<String>,
    stripped_text: &Option<String>,
    should_update_button_child: bool,
) -> LuaResult<()> {
    if !should_update_button_child {
        return Ok(());
    }
    let Some(text_child_id) = ensure_button_text_child(state, id)? else {
        return Ok(());
    };
    {
        let mut sim = borrow_state_mut(state)?;
        let inline_color = leading_wow_text_color(text);
        if let Some(text_child) = sim.widgets.get_mut_visual(text_child_id) {
            text_child.text = text.clone();
            text_child.text_stripped = stripped_text.clone();
            if let Some(color) = inline_color {
                text_child.text_color = color;
            }
        }
    }
    refresh_text_measurements(state, text_child_id);
    Ok(())
}

fn update_auto_text_height(state: &mut LuaState, id: u64) {
    let Some(current) = read_auto_text_height_state(state, id) else {
        return;
    };
    if !should_update_auto_text_height(&current) {
        return;
    }
    let width_is_explicit = current.word_wrap && current.width > 0.0 && !current.width_is_text_auto;
    let wrap_width = width_is_explicit.then_some(current.width);
    let height = measure_text_height(state, id, wrap_width) as f32;
    let Ok(mut sim) = borrow_state_mut(state) else {
        return;
    };
    let Some(frame) = sim.widgets.get(id) else {
        return;
    };
    if (frame.height - height).abs() <= 0.5 && frame.height_is_text_auto {
        return;
    }
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return;
    };
    frame.height = height;
    frame.height_is_text_auto = true;
    sim.widgets.mark_rect_dirty(id);
}

fn read_auto_text_height_state(state: &LuaState, id: u64) -> Option<AutoTextHeightState> {
    let sim = borrow_state(state).ok()?;
    sim.widgets.get(id).map(|frame| AutoTextHeightState {
        is_fontstring: frame.widget_type == WidgetType::FontString,
        has_text: frame.text.as_ref().is_some_and(|text| !text.is_empty()),
        width: frame.width,
        width_is_text_auto: frame.width_is_text_auto,
        height_is_text_auto: frame.height_is_text_auto,
        word_wrap: frame.word_wrap,
        height: frame.height,
    })
}

fn should_update_auto_text_height(current: &AutoTextHeightState) -> bool {
    current.is_fontstring
        && current.has_text
        && (current.height.abs() <= f32::EPSILON || current.height_is_text_auto)
}

pub(crate) fn refresh_auto_text_height_after_width_change(state: &mut LuaState, id: u64) {
    let should_refresh = {
        let Ok(sim) = borrow_state(state) else {
            return;
        };
        sim.widgets.get(id).is_some_and(|frame| {
            frame.widget_type == WidgetType::FontString
                && (frame.height_is_text_auto
                    || (frame.height.abs() <= f32::EPSILON
                        && frame.text.as_ref().is_some_and(|text| !text.is_empty())))
        })
    };
    if should_refresh {
        update_auto_text_height(state, id);
    }
}

pub(crate) fn update_auto_text_width(state: &mut LuaState, id: u64) {
    if apply_anchor_pinned_width(state, id) {
        return;
    }

    let should_update = {
        let Ok(sim) = borrow_state(state) else {
            return;
        };
        sim.widgets.get(id).is_some_and(|frame| {
            frame.widget_type == WidgetType::FontString
                && (frame.width <= 0.0 || frame.width_is_text_auto)
        })
    };
    if !should_update {
        return;
    }

    let width = measure_text_width(state, id) as f32;
    let Ok(mut sim) = borrow_state_mut(state) else {
        return;
    };
    let Some(frame) = sim.widgets.get_mut(id) else {
        return;
    };
    if frame.width > 0.0 && !frame.width_is_text_auto {
        return;
    }
    let width_changed = (frame.width - width).abs() > 0.5;
    let auto_flag_changed = !frame.width_is_text_auto;
    if !width_changed && !auto_flag_changed {
        return;
    }
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return;
    };
    frame.width = width;
    frame.width_is_text_auto = true;
    sim.widgets.mark_rect_dirty(id);
    drop(sim);
    crate::lua_api::frame::methods::core_state::size::mark_nearest_layout_parent_dirty(state, id);
}

/// If the frame's anchors pin both its left and right edges, set its width from
/// the relative frame's width and clear `width_is_text_auto` so word-wrap works.
/// Returns true when handling the frame, false if anchors do not pin both edges.
fn apply_anchor_pinned_width(state: &mut LuaState, id: u64) -> bool {
    let Some(pinned_width) = anchor_pinned_horizontal_width(state, id) else {
        return false;
    };
    let Ok(mut sim) = borrow_state_mut(state) else {
        return true;
    };
    let Some(frame) = sim.widgets.get(id) else {
        return true;
    };
    let width_changed = (frame.width - pinned_width).abs() > 0.5;
    let auto_flag_changed = frame.width_is_text_auto;
    if !width_changed && !auto_flag_changed {
        return true;
    }
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return true;
    };
    frame.width = pinned_width;
    frame.width_is_text_auto = false;
    sim.widgets.mark_rect_dirty(id);
    true
}

fn anchor_pinned_horizontal_width(state: &LuaState, id: u64) -> Option<f32> {
    let sim = borrow_state(state).ok()?;
    let frame = sim.widgets.get(id)?;
    if frame.anchors.len() < 2 {
        return None;
    }

    let mut left: Option<(u64, &crate::widget::Anchor)> = None;
    let mut right: Option<(u64, &crate::widget::Anchor)> = None;
    for anchor in &frame.anchors {
        let target_id = anchor
            .relative_to_id
            .map(|i| i as u64)
            .or(frame.parent_id)?;
        if anchor.point.pins_left_edge() && left.is_none() {
            left = Some((target_id, anchor));
        } else if anchor.point.pins_right_edge() && right.is_none() {
            right = Some((target_id, anchor));
        }
    }
    let (left_target, left_anchor) = left?;
    let (right_target, right_anchor) = right?;
    if left_target != right_target {
        return None;
    }
    let target = sim.widgets.get(left_target)?;
    if target.width <= 0.0 {
        return None;
    }

    let left_x =
        target.width * left_anchor.relative_point.horizontal_factor() + left_anchor.x_offset;
    let right_x =
        target.width * right_anchor.relative_point.horizontal_factor() + right_anchor.x_offset;
    let width = right_x - left_x;
    (width > 0.0).then_some(width)
}

fn refresh_text_measurements(state: &mut LuaState, id: u64) {
    update_auto_text_width(state, id);
    update_auto_text_height(state, id);
}

fn mirror_tooltip_text_fields(
    state: &mut LuaState,
    id: u64,
    text: Option<String>,
    tooltip: TooltipLineValues,
) {
    let fields = get_or_create_frame_fields(state, id);
    let text_val = text.map_or(Val::Nil, |value| create_string(state, &value));
    table_set(state, fields, "text", text_val);
    table_set(state, fields, "r", tooltip.r);
    table_set(state, fields, "g", tooltip.g);
    table_set(state, fields, "b", tooltip.b);
    table_set(state, fields, "a", tooltip.a);
    table_set(state, fields, "wrap", tooltip.wrap);

    let args = create_table(state);
    if let Val::Table(args_ref) = args {
        if let Some(table) = state.gc.tables.get_mut(args_ref) {
            let _ = table.raw_set(Val::Num(1.0), text_val, &state.gc.string_arena);
            let _ = table.raw_set(Val::Num(2.0), tooltip.r, &state.gc.string_arena);
            let _ = table.raw_set(Val::Num(3.0), tooltip.g, &state.gc.string_arena);
            let _ = table.raw_set(Val::Num(4.0), tooltip.b, &state.gc.string_arena);
            let _ = table.raw_set(Val::Num(5.0), tooltip.a, &state.gc.string_arena);
            let _ = table.raw_set(Val::Num(6.0), tooltip.wrap, &state.gc.string_arena);
        }
        state.gc.barrier_back(args_ref);
    }
    table_set(state, fields, "args", args);
}

fn replace_tooltip_lines(
    state: &mut LuaState,
    id: u64,
    text: Option<String>,
    tooltip: TooltipLineValues,
) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let td = sim.tooltips.entry(id).or_default();
    td.lines.clear();
    if let Some(text) = text {
        td.lines.push(crate::lua_api::tooltip::TooltipLine {
            left_text: text,
            left_color: (
                val_to_f32(tooltip.r, 1.0),
                val_to_f32(tooltip.g, 1.0),
                val_to_f32(tooltip.b, 1.0),
            ),
            left_segments: Vec::new(),
            right_text: None,
            right_color: (1.0, 1.0, 1.0),
            right_segments: Vec::new(),
            wrap: matches!(tooltip.wrap, Val::Bool(true)),
            texture: None,
        });
    }
    td.spell_id = None;
    Ok(())
}

fn sync_tooltip_text(
    state: &mut LuaState,
    id: u64,
    text: Option<String>,
    tooltip: TooltipLineValues,
) -> LuaResult<()> {
    mirror_tooltip_text_fields(state, id, text.clone(), tooltip);
    replace_tooltip_lines(state, id, text, tooltip)
}

fn set_text_secret_origin(state: &LuaState, id: u64, secret: bool) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let child = sim.widgets.get(id).and_then(button_text_child_id);
    for frame_id in std::iter::once(id).chain(child) {
        if let Some(frame) = sim.widgets.get_mut(frame_id) {
            frame.secret_text = secret;
        }
    }
    Ok(())
}

fn read_text_arg(state: &LuaState, value: Val) -> Option<String> {
    match value {
        Val::Str(s) => state
            .gc
            .string_arena
            .get(s)
            .map(|ls| String::from_utf8_lossy(ls.data()).to_string()),
        Val::Num(n) => Some(n.to_string()),
        _ => None,
    }
}

pub(super) fn get_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    crate::lua_api::frame::methods::secret_origin::require_text_readable(state, id)?;
    let sim = borrow_state(state)?;
    let frame = sim.widgets.get(id);
    if let Some(child) = frame.and_then(button_text_child_id) {
        crate::lua_api::frame::methods::secret_origin::require_text_readable(state, child)?;
    }
    let use_stripped = frame
        .map(|f| f.widget_type == WidgetType::SimpleHTML)
        .unwrap_or(false);
    let is_editbox = frame
        .map(|f| f.widget_type == WidgetType::EditBox)
        .unwrap_or(false);
    let text = frame.and_then(|f| frame_text_value(&sim, f, use_stripped));
    drop(sim);
    push_text_result(state, text, is_editbox)
}

fn push_text_result(
    state: &mut LuaState,
    text: Option<String>,
    is_editbox: bool,
) -> LuaResult<u32> {
    match text {
        Some(t) => {
            let s = create_string(state, &t);
            state.push(s);
        }
        None if is_editbox => {
            let s = create_string(state, "");
            state.push(s);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn clear_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    clear_frame_text(&mut sim, id);
    let text_child_id = sim.widgets.get(id).and_then(button_text_child_id);
    if let Some(child_id) = text_child_id {
        clear_frame_text(&mut sim, child_id);
    }
    Ok(0)
}

fn button_text_child_id(frame: &crate::widget::Frame) -> Option<u64> {
    BUTTON_TEXT_CHILD_KEYS
        .iter()
        .find_map(|key| frame.children_keys.get(*key).copied())
}

fn clear_frame_text(sim: &mut SimState, id: u64) {
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.secret_text = false;
        frame.text = Some(String::new());
        frame.text_stripped = Some(String::new());
        frame.text_segments.clear();
    }
}

pub(super) fn frame_text_value(
    sim: &SimState,
    frame: &crate::widget::Frame,
    stripped: bool,
) -> Option<String> {
    let own_text = || {
        if stripped {
            frame.text_stripped.clone().or_else(|| frame.text.clone())
        } else {
            frame.text.clone()
        }
    };

    if !matches!(
        frame.widget_type,
        WidgetType::Button | WidgetType::CheckButton
    ) {
        return own_text();
    }

    frame
        .children_keys
        .get("Text")
        .and_then(|&cid| sim.widgets.get(cid))
        .and_then(|child| {
            if stripped {
                child.text_stripped.clone().or_else(|| child.text.clone())
            } else {
                child.text.clone()
            }
        })
        .or_else(own_text)
}

fn frame_text_measurement(state: &LuaState, id: u64) -> (String, Option<String>, f32) {
    let sim = borrow_state(state).expect("sim state should exist");
    let Some(frame) = sim.widgets.get(id) else {
        return (String::new(), None, 12.0);
    };
    let measurement_frame = frame
        .children_keys
        .get("Text")
        .and_then(|&child_id| sim.widgets.get(child_id))
        .unwrap_or(frame);
    let text = frame_text_value(&sim, measurement_frame, true).unwrap_or_default();
    (
        text,
        measurement_frame.font.clone(),
        measurement_frame.font_size,
    )
}

fn frame_text_scale_value(state: &LuaState, id: u64) -> f64 {
    borrow_state(state)
        .expect("sim state should exist")
        .widgets
        .get(id)
        .map(|frame| frame.text_scale.max(0.0))
        .unwrap_or(1.0)
}

pub(super) fn measure_text_width(state: &LuaState, id: u64) -> f64 {
    let (text, font, font_size) = frame_text_measurement(state, id);
    if text.is_empty() {
        return 0.0;
    }
    let text_scale = frame_text_scale_value(state, id);
    if let Some(app) = state.app_data::<crate::lua_api::env::WowLuaAppData>()
        && let Some(font_system) = app.font_system.as_ref()
    {
        return font_system
            .borrow_mut()
            .measure_text_width(&text, font.as_deref(), font_size) as f64
            * text_scale;
    }
    approximate_text_width(&text, font_size) as f64 * text_scale
}

fn measure_text_height(state: &LuaState, id: u64, wrap_width: Option<f32>) -> f64 {
    let (text, font, font_size) = frame_text_measurement(state, id);
    if text.is_empty() {
        return 0.0;
    }
    let text_scale = frame_text_scale_value(state, id);
    if let Some(app) = state.app_data::<crate::lua_api::env::WowLuaAppData>()
        && let Some(font_system) = app.font_system.as_ref()
    {
        return font_system.borrow_mut().measure_text_height(
            &text,
            font.as_deref(),
            font_size,
            wrap_width,
        ) as f64
            * text_scale;
    }
    approximate_text_height(&text, font_size, wrap_width) as f64 * text_scale
}

pub(super) fn get_string_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    state.push(Val::Num(measure_text_width(state, id)));
    Ok(1)
}

pub(super) fn get_string_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let wrap_width = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|frame| (frame.word_wrap && frame.width > 0.0).then_some(frame.width))
    };
    state.push(Val::Num(measure_text_height(state, id, wrap_width)));
    Ok(1)
}

pub(super) fn get_text_width(state: &mut LuaState) -> LuaResult<u32> {
    get_string_width(state)
}

pub(super) fn get_text_height(state: &mut LuaState) -> LuaResult<u32> {
    get_string_height(state)
}

pub(super) fn get_content_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let has_text = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|frame| frame_text_value(&sim, frame, true))
            .is_some_and(|text| !text.is_empty())
    };
    let height = if has_text {
        let wrap_width = resolved_frame_width(state, id);
        measure_text_height(state, id, (wrap_width > 0.0).then_some(wrap_width))
    } else {
        0.0
    };
    state.push(Val::Num(height));
    Ok(1)
}

pub(super) fn get_text_data(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    crate::lua_api::frame::methods::secret_origin::require_text_readable(state, id)?;
    let text = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|frame| frame.text.clone().or_else(|| frame.text_stripped.clone()))
    };
    let data = if is_simple_html_frame(state, id) {
        build_simple_html_text_data(state, id, text)
    } else {
        create_table(state)
    };
    state.push(data);
    Ok(1)
}

pub(super) fn get_line_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let height = sim
        .widgets
        .get(id)
        .map(|frame| (frame.font_size as f64 * frame.text_scale.max(0.0)) as f32)
        .unwrap_or(0.0);
    drop(sim);
    state.push(Val::Num(height as f64));
    Ok(1)
}

pub(super) fn get_num_lines(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(props) = read_line_count_props(state, id)? else {
        state.push(Val::Num(0.0));
        return Ok(1);
    };

    let line_count = if props.has_text && props.line_height > 0.0 {
        let text_height = measure_text_height(state, id, props.wrap_width);
        (text_height / f64::from(props.line_height)).ceil().max(1.0)
    } else {
        0.0
    };
    state.push(Val::Num(line_count));
    Ok(1)
}

fn read_line_count_props(state: &LuaState, id: u64) -> LuaResult<Option<LineCountProps>> {
    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok(None);
    };
    Ok(Some(LineCountProps {
        has_text: frame_text_value(&sim, frame, true).is_some_and(|text| !text.is_empty()),
        line_height: measured_line_height(frame),
        wrap_width: (frame.word_wrap && frame.width > 0.0).then_some(frame.width),
    }))
}

fn measured_line_height(frame: &crate::widget::Frame) -> f32 {
    (frame.font_size * 1.2).ceil() * frame.text_scale.max(0.0) as f32
}

#[cfg(test)]
mod tests;
