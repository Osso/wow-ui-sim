use super::simple_html::{get_simple_html_font, is_simple_html_frame, set_simple_html_font};
use super::{
    frame_text_value, get_string_width, measure_text_height, measure_text_width,
    refresh_auto_text_height_after_width_change, set_text,
};
use crate::lua_api::frame::methods::button_anchor_hierarchy::{
    apply_font_object_snapshot, font_object_snapshot_changes_frame, read_font_object_fields,
};
use crate::lua_api::globals::font_strings_collection::fonts::create_font_object;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table,
    frame_id_from_stack, registry_get, registry_set, registry_table_or_create, table_get,
    table_get_static, table_set, table_set_static, val_to_string,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Copy, Clone)]
struct TruncationProps {
    width: f64,
    height: f64,
    word_wrap: bool,
    max_lines: u32,
    line_height: f64,
}

#[derive(Copy, Clone)]
struct FormattedTextUpdateState {
    text_matches: bool,
    width_is_text_auto: bool,
    current_width: f32,
}

struct CachedFormattedText {
    result: String,
    width_hint: Option<f32>,
}

pub(crate) fn is_truncated(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let props = read_truncation_props(state, id);
    let truncated = check_truncated(state, id, props);
    state.push(Val::Bool(truncated));
    Ok(1)
}

fn read_truncation_props(state: &LuaState, id: u64) -> TruncationProps {
    let sim = borrow_state(state).expect("sim state should exist");
    let frame = sim.widgets.get(id);
    TruncationProps {
        width: frame.map(|f| f.width as f64).unwrap_or(0.0),
        height: frame.map(|f| f.height as f64).unwrap_or(0.0),
        word_wrap: frame.map(|f| f.word_wrap).unwrap_or(false),
        max_lines: frame.map(|f| f.max_lines).unwrap_or(0),
        line_height: frame
            .map(|f| f.font_size as f64 * f.text_scale.max(0.0))
            .unwrap_or(0.0),
    }
}

fn check_truncated(state: &LuaState, id: u64, p: TruncationProps) -> bool {
    let width_overflow = p.width > 0.0 && measure_text_width(state, id) > p.width + 0.5;
    let vertical_overflow = check_vertical_overflow(state, id, &p);
    width_overflow || vertical_overflow
}

fn check_vertical_overflow(state: &LuaState, id: u64, p: &TruncationProps) -> bool {
    if !p.word_wrap || p.width <= 0.0 {
        return false;
    }
    let wrapped_height = measure_text_height(state, id, Some(p.width as f32));
    let max_lines_height = (p.max_lines > 0).then_some(p.line_height * p.max_lines as f64);
    let available_height = match (p.height > 0.0, max_lines_height) {
        (true, Some(lh)) => p.height.min(lh),
        (true, None) => p.height,
        (false, Some(lh)) => lh,
        (false, None) => 0.0,
    };
    available_height > 0.0 && wrapped_height > available_height + 0.5
}

pub(crate) fn set_formatted_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let formatter = table_get(state, Val::Table(state.global), "format");
    let signature = default_formatter_unchanged(state, formatter)
        .then(|| formatted_text_signature(state))
        .flatten();
    let cached = read_cached_formatted_text(state, id, formatter, signature.as_deref());
    let formatted_text = if let Some(cached) = &cached {
        cached.result.clone()
    } else {
        format_text_arg(state, formatter)?
    };

    let cached_width_hint = cached.as_ref().and_then(|cached| cached.width_hint);
    apply_formatted_text_update(state, id, formatted_text.clone(), cached_width_hint)?;
    if let Some(signature) = signature.as_deref() {
        write_cached_formatted_text(state, id, formatter, signature, &formatted_text);
    }
    Ok(0)
}

fn apply_formatted_text_update(
    state: &mut LuaState,
    id: u64,
    formatted_text: String,
    cached_width_hint: Option<f32>,
) -> LuaResult<()> {
    if should_skip_formatted_text_update(state, id, &formatted_text, cached_width_hint)? {
        return Ok(());
    }

    replace_text_stack_arg(state, &formatted_text);
    set_text(state)?;
    ensure_intrinsic_formatted_text_width(state, id)
}

fn format_text_arg(state: &mut LuaState, formatter: Val) -> LuaResult<String> {
    let format = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let args = collect_format_args(state, &format);
    let formatted = call_function_state(state, formatter, &args)?;
    Ok(val_to_string(state, formatted).unwrap_or(format))
}

fn collect_format_args(state: &mut LuaState, format: &str) -> Vec<Val> {
    let nargs = (state.top as i32 - state.base as i32) as usize;
    let mut args = Vec::with_capacity(nargs.saturating_sub(1));
    args.push(create_string(state, format));
    for index in 3..=nargs {
        args.push(stack_val(state, index as i32));
    }
    args
}

fn formatted_text_signature(state: &LuaState) -> Option<String> {
    let nargs = (state.top as i32 - state.base as i32) as usize;
    let mut sig = String::new();
    for index in 2..=nargs {
        match stack_val(state, index as i32) {
            Val::Nil => sig.push_str("n;"),
            Val::Bool(value) => {
                sig.push_str("b:");
                sig.push(if value { '1' } else { '0' });
                sig.push(';');
            }
            Val::Num(value) => {
                sig.push_str("d:");
                sig.push_str(&value.to_bits().to_string());
                sig.push(';');
            }
            Val::Str(value) => {
                let text = val_to_string(state, Val::Str(value))?;
                sig.push_str("s:");
                sig.push_str(&text.len().to_string());
                sig.push(':');
                sig.push_str(&text);
                sig.push(';');
            }
            _ => return None,
        }
    }
    Some(sig)
}

fn get_or_create_formatted_text_cache_store(state: &mut LuaState) -> Val {
    registry_table_or_create(state, "__formatted_text_cache")
}

fn capture_default_formatter(state: &mut LuaState) -> Val {
    const DEFAULT_FORMATTER_KEY: &str = "__formatted_text_default_formatter";
    let cached = registry_get(state, DEFAULT_FORMATTER_KEY);
    if !matches!(cached, Val::Nil) {
        return cached;
    }
    let formatter = table_get(state, Val::Table(state.global), "format");
    registry_set(state, DEFAULT_FORMATTER_KEY, formatter);
    formatter
}

fn default_formatter_unchanged(state: &mut LuaState, formatter: Val) -> bool {
    capture_default_formatter(state) == formatter
}

fn read_cached_formatted_text(
    state: &mut LuaState,
    id: u64,
    formatter: Val,
    signature: Option<&str>,
) -> Option<CachedFormattedText> {
    let signature = signature?;
    let store = get_or_create_formatted_text_cache_store(state);
    let id_key = id.to_string();
    let entry = table_get(state, store, &id_key);
    if !matches!(entry, Val::Table(_)) {
        return None;
    }
    let cached_formatter = table_get_static(state, entry, "__formatter");
    if cached_formatter != formatter {
        return None;
    }
    let cached_signature_val = table_get_static(state, entry, "__signature");
    let cached_signature = val_to_string(state, cached_signature_val)?;
    if cached_signature != signature {
        return None;
    }
    let cached_result_val = table_get_static(state, entry, "__result");
    let result = val_to_string(state, cached_result_val)?;
    let width_hint = match table_get_static(state, entry, "__width_hint") {
        Val::Num(width) => Some(width as f32),
        _ => None,
    };
    Some(CachedFormattedText { result, width_hint })
}

fn write_cached_formatted_text(
    state: &mut LuaState,
    id: u64,
    formatter: Val,
    signature: &str,
    result: &str,
) {
    let store = get_or_create_formatted_text_cache_store(state);
    let id_key = id.to_string();
    let mut entry = table_get(state, store, &id_key);
    if !matches!(entry, Val::Table(_)) {
        entry = create_table(state);
    }
    let width_hint = borrow_state(state)
        .ok()
        .and_then(|sim| sim.widgets.get(id).map(|frame| frame.width));
    let signature_val = create_string(state, signature);
    let result_val = create_string(state, result);
    table_set_static(state, entry, "__formatter", formatter);
    table_set_static(state, entry, "__signature", signature_val);
    table_set_static(state, entry, "__result", result_val);
    if let Some(width_hint) = width_hint {
        table_set_static(state, entry, "__width_hint", Val::Num(width_hint as f64));
    } else {
        table_set_static(state, entry, "__width_hint", Val::Nil);
    }
    table_set(state, store, &id_key, entry);
}

fn should_skip_formatted_text_update(
    state: &mut LuaState,
    id: u64,
    formatted_text: &str,
    cached_width_hint: Option<f32>,
) -> LuaResult<bool> {
    let Some(update_state) = read_formatted_text_update_state(state, id, formatted_text)? else {
        return Ok(true);
    };
    if !update_state.text_matches {
        return Ok(false);
    }
    if update_state.width_is_text_auto
        && cached_width_hint
            .is_some_and(|cached_width| (update_state.current_width - cached_width).abs() <= 0.5)
    {
        return Ok(true);
    }
    if let Some(skip) = skip_matching_formatted_text(update_state, None) {
        return Ok(skip);
    }
    let measured_width = measure_text_width(state, id) as f32;
    Ok(skip_matching_formatted_text(update_state, Some(measured_width)).unwrap_or(false))
}

fn read_formatted_text_update_state(
    state: &LuaState,
    id: u64,
    formatted_text: &str,
) -> LuaResult<Option<FormattedTextUpdateState>> {
    let sim = borrow_state(state)?;
    let Some(frame) = sim.widgets.get(id) else {
        return Ok(None);
    };
    let current_text = frame_text_value(&sim, frame, false);
    let current_stripped_text = frame_text_value(&sim, frame, true);
    Ok(Some(FormattedTextUpdateState {
        text_matches: current_text.as_deref() == Some(formatted_text)
            && current_stripped_text.as_deref() == Some(formatted_text),
        width_is_text_auto: frame.width_is_text_auto,
        current_width: frame.width,
    }))
}

fn skip_matching_formatted_text(
    update_state: FormattedTextUpdateState,
    measured_width: Option<f32>,
) -> Option<bool> {
    if !update_state.text_matches {
        return Some(false);
    }
    if !update_state.width_is_text_auto && update_state.current_width > 0.0 {
        return Some(true);
    }
    measured_width.map(|width| (update_state.current_width - width).abs() <= 0.5)
}

fn replace_text_stack_arg(state: &mut LuaState, text: &str) {
    let formatted_value = create_string(state, text);
    // stack_val uses base-relative indexing (index 2 = stack[base + 1]);
    // stack_set is absolute. Match the reader so set_text sees the replacement.
    state.stack_set(state.base + 1, formatted_value);
}

fn ensure_intrinsic_formatted_text_width(state: &mut LuaState, id: u64) -> LuaResult<()> {
    let needs_intrinsic_width = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).is_some_and(|frame| {
            frame.width <= 0.0 && !frame.text.as_deref().unwrap_or("").is_empty()
        })
    };
    if !needs_intrinsic_width {
        return Ok(());
    }

    let width = measure_text_width(state, id) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id)
        && frame.width <= 0.0
    {
        frame.width = width;
    }
    Ok(())
}

pub(crate) fn set_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id) {
        let font = val_to_string(state, stack_val(state, 3));
        let size = match stack_val(state, 4) {
            Val::Num(n) => Some(n as f32),
            _ => None,
        };
        let flags = val_to_string(state, stack_val(state, 5));
        set_simple_html_font(state, id, text_type, font, size, flags);
        return Ok(0);
    }
    let font = val_to_string(state, stack_val(state, 2));
    if font.is_none() {
        state.push(Val::Bool(false));
        return Ok(1);
    }
    let size = match stack_val(state, 3) {
        Val::Num(n) => Some(n as f32),
        _ => None,
    };
    let flags = val_to_string(state, stack_val(state, 4));
    let mut sim = borrow_state_mut(state)?;
    let changed = sim
        .widgets
        .get_mut(id)
        .is_some_and(|frame| apply_font_args(frame, font, size, flags));
    // Re-applying the current font leaves the text as drawn.
    if changed {
        sim.widgets.mark_visual_dirty(id);
    }
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}

/// Apply the given font fields; returns whether any of them changed.
fn apply_font_args(
    frame: &mut crate::widget::Frame,
    font: Option<String>,
    size: Option<f32>,
    flags: Option<String>,
) -> bool {
    let font = font.filter(|f| frame.font.as_deref() != Some(f.as_str()));
    let size = size.filter(|&s| frame.font_size != s);
    let outline = flags
        .map(|f| crate::widget::TextOutline::from_wow_str(&f))
        .filter(|&o| frame.font_outline != o);
    let changed = font.is_some() || size.is_some() || outline.is_some();
    if let Some(f) = font {
        frame.font = Some(f);
    }
    if let Some(s) = size {
        frame.font_size = s;
    }
    if let Some(o) = outline {
        frame.font_outline = o;
    }
    changed
}

pub(crate) fn get_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    if is_simple_html_frame(state, id)
        && let Some((font_path, font_size, flags)) = get_simple_html_font(state, id, text_type)
    {
        let font_path_val = create_string(state, &font_path);
        let flags_val = create_string(state, &flags);
        state.push(font_path_val);
        state.push(Val::Num(font_size as f64));
        state.push(flags_val);
        return Ok(3);
    }
    let sim = borrow_state(state)?;
    let frame = sim.widgets.get(id);
    let font_path = frame
        .and_then(|f| f.font.as_deref())
        .unwrap_or("Fonts\\FRIZQT__.TTF")
        .to_string();
    let font_size = frame.map(|f| f.font_size).unwrap_or(12.0);
    let flags = outline_to_str(frame).to_string();
    drop(sim);
    let font_path_val = create_string(state, &font_path);
    state.push(font_path_val);
    state.push(Val::Num(font_size as f64));
    let flags_val = create_string(state, &flags);
    state.push(flags_val);
    Ok(3)
}

fn outline_to_str(frame: Option<&crate::widget::Frame>) -> &'static str {
    frame
        .map(|f| match f.font_outline {
            crate::widget::TextOutline::None => "",
            crate::widget::TextOutline::Outline => "OUTLINE",
            crate::widget::TextOutline::ThickOutline => "THICKOUTLINE",
        })
        .unwrap_or("")
}

pub(crate) fn set_font_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let height = match stack_val(state, 2) {
        Val::Num(n) => n as f32,
        _ => return Ok(0),
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.font_size = height;
    }
    Ok(0)
}

pub(crate) fn set_text_height(state: &mut LuaState) -> LuaResult<u32> {
    set_font_height(state)
}

pub(crate) fn get_font_height(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let size = sim.widgets.get(id).map(|f| f.font_size).unwrap_or(12.0);
    drop(sim);
    state.push(Val::Num(size as f64));
    Ok(1)
}

fn get_or_create_font_object_store(state: &mut LuaState) -> Val {
    registry_table_or_create(state, "__font_objects")
}

pub(crate) fn set_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    if is_simple_html_frame(state, id) {
        return set_simple_html_font_object(state, id);
    }

    let font_object = resolve_font_object_arg(state, stack_val(state, 2))?;
    let store = get_or_create_font_object_store(state);
    if table_get(state, store, &id.to_string()) == font_object {
        return Ok(0);
    }
    let fields = read_font_object_fields(state, font_object);
    table_set(state, store, &id.to_string(), font_object);
    let should_apply = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .is_some_and(|frame| font_object_snapshot_changes_frame(frame, &fields))
    };
    if !should_apply {
        return Ok(0);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        apply_font_object_snapshot(frame, &fields);
    }
    drop(sim);
    refresh_auto_text_height_after_width_change(state, id);
    Ok(0)
}

fn set_simple_html_font_object(state: &mut LuaState, id: u64) -> LuaResult<u32> {
    let text_type = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let font_object = resolve_font_object_arg(state, stack_val(state, 3))?;
    let fields = read_font_object_fields(state, font_object);
    set_simple_html_font(
        state,
        id,
        text_type,
        fields.font,
        fields.font_size,
        fields.font_outline.map(simple_html_font_flags),
    );
    Ok(0)
}

fn resolve_font_object_arg(state: &mut LuaState, value: Val) -> LuaResult<Val> {
    match value {
        Val::Table(_) => Ok(value),
        Val::Str(_) => {
            let name = val_to_string(state, value)
                .ok_or_else(|| runtime_error("SetFontObject requires a font object"))?;
            let font_object = table_get(state, Val::Table(state.global), &name);
            matches!(font_object, Val::Table(_))
                .then_some(font_object)
                .ok_or_else(|| runtime_error("SetFontObject requires a font object"))
        }
        _ => Err(runtime_error("SetFontObject requires a font object")),
    }
}

fn simple_html_font_flags(outline: crate::widget::TextOutline) -> String {
    match outline {
        crate::widget::TextOutline::None => String::new(),
        crate::widget::TextOutline::Outline => "OUTLINE".to_string(),
        crate::widget::TextOutline::ThickOutline => "THICKOUTLINE".to_string(),
    }
}

pub(crate) fn get_font_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let store = get_or_create_font_object_store(state);
    let font_object = table_get(state, store, &id.to_string());
    if !matches!(font_object, Val::Nil) {
        state.push(font_object);
        return Ok(1);
    }
    let auto_font = create_font_object(state, None);
    table_set(state, store, &id.to_string(), auto_font);
    state.push(auto_font);
    Ok(1)
}

pub(crate) fn set_font_objects_to_try(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    for index in 2..=8 {
        let font_object = stack_val(state, index);
        if !matches!(font_object, Val::Nil) {
            let store = get_or_create_font_object_store(state);
            table_set(state, store, &id.to_string(), font_object);
            break;
        }
    }
    Ok(0)
}

pub(crate) fn get_unbounded_string_width(state: &mut LuaState) -> LuaResult<u32> {
    get_string_width(state)
}

pub(crate) fn get_wrapped_width(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let string_width = measure_text_width(state, id);
    let wrap_width = read_active_wrap_width(state, id);
    let wrapped_width = wrap_width.map_or(string_width, |width| string_width.min(width));
    state.push(Val::Num(wrapped_width));
    Ok(1)
}

fn read_active_wrap_width(state: &LuaState, id: u64) -> Option<f64> {
    let sim = borrow_state(state).ok()?;
    let frame = sim.widgets.get(id)?;
    (frame.word_wrap && frame.width > 0.0).then_some(f64::from(frame.width))
}

pub(crate) fn set_text_to_fit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    set_text(state)?;
    let measured_width = measure_text_width(state, id) as f32;
    apply_text_to_fit_size(state, id, measured_width);
    Ok(0)
}

/// Real WoW's `FontString:SetTextToFit` unconditionally resizes the font
/// string to the unclipped extents of the new text so menu compositors can
/// measure element widths. Menu dropdowns rely on this — without it, element
/// font strings report width 0 and menus render as 1-row stubs.
fn apply_text_to_fit_size(state: &mut LuaState, id: u64, measured_width: f32) {
    const LINE_HEIGHT_MULTIPLIER: f32 = 1.2;
    let font_size = {
        let Ok(sim) = borrow_state(state) else {
            return;
        };
        sim.widgets
            .get(id)
            .map(|frame| frame.font_size)
            .unwrap_or(0.0)
    };
    let Ok(mut sim) = borrow_state_mut(state) else {
        return;
    };
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return;
    };
    frame.width = measured_width;
    frame.width_is_text_auto = true;
    if frame.height <= 0.0 && font_size > 0.0 {
        frame.height = (font_size * LINE_HEIGHT_MULTIPLIER).ceil();
    }
    sim.widgets.mark_rect_dirty(id);
}

pub(crate) fn scale_text_to_fit(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub(crate) fn apply_default_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let default_text = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        store_default_text_attrs(frame, &default_text, true);
    }
    Ok(0)
}

fn store_default_text_attrs(
    frame: &mut crate::widget::Frame,
    default_text: &str,
    mark_enabled: bool,
) {
    use crate::widget::AttributeValue;
    frame.attributes.insert(
        "__default_text".to_string(),
        AttributeValue::String(default_text.to_string()),
    );
    if mark_enabled {
        frame.attributes.insert(
            "__default_text_enabled".to_string(),
            AttributeValue::Boolean(true),
        );
    }
    if frame.text.as_deref().unwrap_or_default().is_empty() {
        frame.text = Some(default_text.to_string());
        frame
            .attributes
            .insert("__defaulted".to_string(), AttributeValue::Boolean(true));
    }
}

pub(crate) fn try_apply_default_text(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let default_text = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|frame| match frame.attributes.get("__default_text") {
                Some(crate::widget::AttributeValue::String(text)) => Some(text.clone()),
                _ => None,
            })
    };
    let Some(default_text) = default_text else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id)
        && frame.text.as_deref().unwrap_or_default().is_empty()
    {
        frame.text = Some(default_text.clone());
        frame.attributes.insert(
            "__defaulted".to_string(),
            crate::widget::AttributeValue::Boolean(true),
        );
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::{FormattedTextUpdateState, skip_matching_formatted_text};

    #[test]
    fn skip_matching_formatted_text_skips_when_text_matches_and_width_is_stable() {
        let update_state = FormattedTextUpdateState {
            text_matches: true,
            width_is_text_auto: true,
            current_width: 24.0,
        };

        assert_eq!(
            skip_matching_formatted_text(update_state, Some(24.2)),
            Some(true)
        );
    }

    #[test]
    fn skip_matching_formatted_text_requires_measurement_for_auto_width() {
        let update_state = FormattedTextUpdateState {
            text_matches: true,
            width_is_text_auto: true,
            current_width: 24.0,
        };

        assert_eq!(skip_matching_formatted_text(update_state, None), None);
    }
}
