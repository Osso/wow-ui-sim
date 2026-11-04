//! Presentation-only interpretation of copied tooltip color markup.
use rilua::{Val, vm::state::LuaState};

pub(super) fn presentation_segments(
    state: &mut LuaState,
    text: &str,
    color: (f32, f32, f32),
) -> Vec<crate::lua_api::tooltip::TooltipTextSegment> {
    parse_tooltip_color_segments(
        state,
        text,
        (f64::from(color.0), f64::from(color.1), f64::from(color.2)),
    )
    .1
    .into_iter()
    .map(|segment| crate::lua_api::tooltip::TooltipTextSegment {
        text: segment.text,
        color: (
            segment.color.0 as f32,
            segment.color.1 as f32,
            segment.color.2 as f32,
        ),
    })
    .collect()
}

pub(crate) struct ParsedTooltipText {
    pub(crate) text: String,
    pub(crate) color: Option<(f64, f64, f64)>,
    pub(crate) segments: Vec<TooltipColorSegment>,
}

pub(crate) struct TooltipColorSegment {
    pub(crate) text: String,
    pub(crate) color: (f64, f64, f64),
}

pub(crate) fn tooltip_line_text_and_color(
    state: &mut LuaState,
    text: &str,
    explicit_color: Option<(f64, f64, f64)>,
) -> ParsedTooltipText {
    let color = explicit_color.or_else(|| full_line_color_markup(state, text));
    let base_color = color
        .or_else(|| global_color(state, b"NORMAL_FONT_COLOR"))
        .unwrap_or((1.0, 0.82, 0.0));
    let (text, segments) = parse_tooltip_color_segments(state, text, base_color);
    ParsedTooltipText {
        text,
        color,
        segments,
    }
}

fn full_line_color_markup(state: &mut LuaState, text: &str) -> Option<(f64, f64, f64)> {
    if let Some(color) = full_line_named_color_markup(state, text) {
        return Some(color);
    }
    full_line_hex_color_markup(text)
}

fn full_line_named_color_markup(state: &mut LuaState, text: &str) -> Option<(f64, f64, f64)> {
    let rest = text.strip_prefix("|cn")?;
    let (color_name, colored_text) = rest.split_once(':')?;
    has_closing_color_reset(colored_text).then(|| global_color(state, color_name.as_bytes()))?
}

fn full_line_hex_color_markup(text: &str) -> Option<(f64, f64, f64)> {
    let rest = text.strip_prefix("|c")?;
    let (hex, colored_text) = rest.split_at_checked(8)?;
    if !is_hex_color(hex) || !has_closing_color_reset(colored_text) {
        return None;
    }
    rgb_from_argb_hex(hex)
}

fn has_closing_color_reset(text: &str) -> bool {
    text.ends_with("|r") || text.ends_with("|R")
}

pub(crate) fn parse_tooltip_color_segments(
    state: &mut LuaState,
    text: &str,
    base_color: (f64, f64, f64),
) -> (String, Vec<TooltipColorSegment>) {
    let mut stripped = String::with_capacity(text.len());
    let mut segments = Vec::new();
    let mut segment_text = String::new();
    let mut current_color = base_color;
    let mut color_stack = Vec::new();
    let mut saw_markup = false;
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];
        if let Some(markup) = parse_color_markup(state, rest, base_color) {
            push_color_segment(&mut segments, &mut segment_text, current_color);
            match markup.action {
                ColorMarkupAction::Push(color) => {
                    color_stack.push(current_color);
                    current_color = color;
                }
                ColorMarkupAction::Pop => {
                    current_color = color_stack.pop().unwrap_or(base_color);
                }
            }
            saw_markup = true;
            index += markup.len;
            continue;
        }

        let ch = rest.chars().next().expect("index should be in bounds");
        stripped.push(ch);
        segment_text.push(ch);
        index += ch.len_utf8();
    }

    push_color_segment(&mut segments, &mut segment_text, current_color);
    if !saw_markup {
        segments.clear();
    }

    (stripped, segments)
}

struct ParsedColorMarkup {
    len: usize,
    action: ColorMarkupAction,
}

enum ColorMarkupAction {
    Push((f64, f64, f64)),
    Pop,
}

fn parse_color_markup(
    state: &mut LuaState,
    text: &str,
    _base_color: (f64, f64, f64),
) -> Option<ParsedColorMarkup> {
    if let Some(rest) = text.strip_prefix("|cn")
        && let Some(colon_index) = rest.find(':')
    {
        let name = &rest[..colon_index];
        let color = global_color(state, name.as_bytes())?;
        return Some(ParsedColorMarkup {
            len: "|cn".len() + colon_index + ":".len(),
            action: ColorMarkupAction::Push(color),
        });
    }
    if let Some(rest) = text.strip_prefix("|c") {
        let (hex, _) = rest.split_at_checked(8)?;
        if is_hex_color(hex) {
            let color = rgb_from_argb_hex(hex)?;
            return Some(ParsedColorMarkup {
                len: "|c".len() + hex.len(),
                action: ColorMarkupAction::Push(color),
            });
        }
    }
    (text.starts_with("|r") || text.starts_with("|R")).then_some(ParsedColorMarkup {
        len: 2,
        action: ColorMarkupAction::Pop,
    })
}

fn push_color_segment(
    segments: &mut Vec<TooltipColorSegment>,
    segment_text: &mut String,
    color: (f64, f64, f64),
) {
    if segment_text.is_empty() {
        return;
    }
    segments.push(TooltipColorSegment {
        text: std::mem::take(segment_text),
        color,
    });
}

fn is_hex_color(text: &str) -> bool {
    text.len() == 8 && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn rgb_from_argb_hex(hex: &str) -> Option<(f64, f64, f64)> {
    let red = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let green = u8::from_str_radix(&hex[4..6], 16).ok()?;
    let blue = u8::from_str_radix(&hex[6..8], 16).ok()?;
    Some((
        f64::from(red) / 255.0,
        f64::from(green) / 255.0,
        f64::from(blue) / 255.0,
    ))
}

fn table_color_component(state: &mut LuaState, table: Val, key: &[u8]) -> Option<f64> {
    let Val::Table(table_ref) = table else {
        return None;
    };
    let component_key = state.gc.intern_string(key);
    let value = state
        .gc
        .tables
        .get(table_ref)
        .map(|tbl| tbl.get_str(component_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    match value {
        Val::Num(component) => Some(component),
        _ => None,
    }
}

fn global_color(state: &mut LuaState, key: &[u8]) -> Option<(f64, f64, f64)> {
    let global_key = state.gc.intern_string(key);
    let color = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(global_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let r = table_color_component(state, color, b"r")?;
    let g = table_color_component(state, color, b"g")?;
    let b = table_color_component(state, color, b"b")?;
    Some((r, g, b))
}
