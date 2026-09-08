use super::super::{
    LINE_TYPE_EQUIP_SLOT, LINE_TYPE_ITEM_BINDING, LINE_TYPE_ITEM_LEVEL, LINE_TYPE_ITEM_NAME,
    LINE_TYPE_SPELL_NAME, TOOLTIP_TYPE_ITEM, ensure_namespace, set_table_array,
};
use crate::items;
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn color_table(state: &mut LuaState, r: f64, g: f64, b: f64, a: f64) -> Val {
    let key = state.gc.intern_string(b"CreateColor");
    let create_color = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    match call_function_state(
        state,
        create_color,
        &[Val::Num(r), Val::Num(g), Val::Num(b), Val::Num(a)],
    ) {
        Ok(color) => color,
        Err(_) => create_table(state),
    }
}

pub(super) fn push_tooltip_line(
    state: &mut LuaState,
    lines: Val,
    index: i64,
    line_type: f64,
    left_text: &str,
    left_color: Option<(f64, f64, f64)>,
    wrap: bool,
) {
    let line = create_table(state);
    table_set(state, line, "type", Val::Num(line_type));
    let parsed = tooltip_line_text_and_color(state, left_text, left_color);
    let left_text_val = create_string(state, &parsed.text);
    table_set(state, line, "leftText", left_text_val);
    if let Some((r, g, b)) = parsed.color {
        let color = color_table(state, r, g, b, 1.0);
        table_set(state, line, "leftColor", color);
    }
    if !parsed.segments.is_empty() {
        let segments = color_segment_table(state, parsed.segments);
        table_set(state, line, "leftColorSegments", segments);
    }
    if wrap {
        table_set(state, line, "wrapText", Val::Bool(true));
    }
    set_table_array(state, lines, index, line);
}

struct ParsedTooltipText {
    text: String,
    color: Option<(f64, f64, f64)>,
    segments: Vec<TooltipColorSegment>,
}

struct TooltipColorSegment {
    text: String,
    color: (f64, f64, f64),
}

fn tooltip_line_text_and_color(
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

fn parse_tooltip_color_segments(
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

fn color_segment_table(state: &mut LuaState, segments: Vec<TooltipColorSegment>) -> Val {
    let table = create_table(state);
    for (index, segment) in segments.into_iter().enumerate() {
        let entry = create_table(state);
        let text = create_string(state, &segment.text);
        table_set(state, entry, "text", text);
        let (r, g, b) = segment.color;
        let color = color_table(state, r, g, b, 1.0);
        table_set(state, entry, "color", color);
        set_table_array(state, table, index as i64 + 1, entry);
    }
    table
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

pub(super) fn empty_tooltip(state: &mut LuaState, tooltip_type: f64) -> Val {
    let tooltip = create_table(state);
    let lines = create_table(state);
    table_set(state, tooltip, "type", Val::Num(tooltip_type));
    table_set(state, tooltip, "lines", lines);
    tooltip
}

pub(super) fn create_identified_tooltip(state: &mut LuaState, tooltip_type: f64, id: u32) -> Val {
    let tooltip = empty_tooltip(state, tooltip_type);
    table_set(state, tooltip, "id", Val::Num(id as f64));
    tooltip
}

pub(super) fn item_quality_color(quality: u8) -> (f64, f64, f64) {
    match quality {
        0 => (0.62, 0.62, 0.62),
        1 => (1.0, 1.0, 1.0),
        2 => (0.12, 1.0, 0.0),
        3 => (0.0, 0.44, 0.87),
        4 => (0.64, 0.21, 0.93),
        5 => (1.0, 0.5, 0.0),
        _ => (1.0, 1.0, 1.0),
    }
}

fn item_equip_slot_label(inv_type: u8) -> &'static str {
    ITEM_EQUIP_SLOT_LABELS
        .iter()
        .find(|(item_inv_type, _)| *item_inv_type == inv_type)
        .map(|(_, label)| *label)
        .unwrap_or("")
}

const ITEM_EQUIP_SLOT_LABELS: &[(u8, &str)] = &[
    (1, "Head"),
    (2, "Neck"),
    (3, "Shoulder"),
    (4, "Shirt"),
    (5, "Chest"),
    (6, "Waist"),
    (7, "Legs"),
    (8, "Feet"),
    (9, "Wrist"),
    (10, "Hands"),
    (11, "Finger"),
    (12, "Trinket"),
    (13, "One-Hand"),
    (14, "Shield"),
    (15, "Ranged"),
    (16, "Back"),
    (17, "Two-Hand"),
    (20, "Chest"),
    (21, "Main Hand"),
    (22, "Off Hand"),
    (23, "Held In Off-hand"),
];

fn push_item_name_line(state: &mut LuaState, lines: Val, item: &items::ItemInfo) {
    push_tooltip_line(
        state,
        lines,
        1,
        LINE_TYPE_ITEM_NAME,
        item.name,
        Some(item_quality_color(item.quality)),
        false,
    );
}

fn tooltip_detail_color(state: &mut LuaState) -> (f64, f64, f64) {
    global_color_or_fallback(state, b"HIGHLIGHT_FONT_COLOR", (1.0, 0.82, 0.0))
}

fn push_item_level_line(state: &mut LuaState, lines: Val, item: &items::ItemInfo) {
    let item_level = format!("Item Level {}", item.item_level);
    let detail_color = tooltip_detail_color(state);
    push_tooltip_line(
        state,
        lines,
        2,
        LINE_TYPE_ITEM_LEVEL,
        &item_level,
        Some(detail_color),
        false,
    );
}

fn item_binding_text(state: &mut LuaState, bonding: u8) -> Option<String> {
    if bonding != 1 {
        return None;
    }
    let item_bind_key = state.gc.intern_string(b"ITEM_BIND_ON_PICKUP");
    state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(item_bind_key, &state.gc.string_arena))
        .and_then(|value| val_to_string(state, value))
        .or_else(|| Some("Binds when picked up".to_string()))
}

fn push_item_equip_slot_line(
    state: &mut LuaState,
    lines: Val,
    inventory_type: u8,
    next_index: &mut i64,
) {
    let equip_slot = item_equip_slot_label(inventory_type);
    if equip_slot.is_empty() {
        return;
    }
    let detail_color = tooltip_detail_color(state);
    push_tooltip_line(
        state,
        lines,
        *next_index,
        LINE_TYPE_EQUIP_SLOT,
        equip_slot,
        Some(detail_color),
        false,
    );
    *next_index += 1;
}

fn push_item_binding_line(
    state: &mut LuaState,
    lines: Val,
    next_index: &mut i64,
    item: &items::ItemInfo,
) {
    let Some(binding) = item_binding_text(state, item.bonding) else {
        return;
    };
    let detail_color = tooltip_detail_color(state);
    push_tooltip_line(
        state,
        lines,
        *next_index,
        LINE_TYPE_ITEM_BINDING,
        &binding,
        Some(detail_color),
        false,
    );
    *next_index += 1;
}

fn global_string_or_fallback(state: &mut LuaState, key: &[u8], fallback: &str) -> String {
    let interned_key = state.gc.intern_string(key);
    state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(interned_key, &state.gc.string_arena))
        .and_then(|value| val_to_string(state, value))
        .unwrap_or_else(|| fallback.to_string())
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

fn global_color_or_fallback(
    state: &mut LuaState,
    key: &[u8],
    fallback: (f64, f64, f64),
) -> (f64, f64, f64) {
    global_color(state, key).unwrap_or(fallback)
}

fn slot_budget_multiplier(inventory_type: u8) -> f64 {
    match inventory_type {
        1 | 3 | 5 | 6 | 7 | 8 | 9 | 10 => 2.0, // armor pieces
        2 | 11 | 12 | 16 => 1.25,              // neck, rings, trinkets, cloak
        13 | 17 | 21 => 2.7,                   // one-hand, two-hand, main hand
        14 | 22 => 1.6,                        // shield/off hand
        15 => 2.2,                             // ranged
        _ => 1.0,
    }
}

fn primary_stat_for_class(class_index: i32) -> i16 {
    match class_index {
        1 | 2 | 6 => 4,       // Strength classes
        3 | 4 | 10 | 12 => 3, // Agility classes
        _ => 5,               // Intellect classes
    }
}

fn resolve_hybrid_stat_id(raw_stat_id: i16, class_index: i32) -> i16 {
    let primary = primary_stat_for_class(class_index);
    match raw_stat_id {
        71 => primary, // Agi/Str/Int
        72 => {
            if primary == 4 {
                4
            } else {
                3
            }
        } // Agi/Str
        73 => {
            if primary == 3 {
                3
            } else {
                5
            }
        } // Agi/Int
        74 => {
            if primary == 4 {
                4
            } else {
                5
            }
        } // Str/Int
        _ => raw_stat_id,
    }
}

fn stat_display_spec(stat_id: i16) -> Option<(&'static [u8], &'static str, f64)> {
    match stat_id {
        3 => Some((b"ITEM_MOD_AGILITY_SHORT", "Agility", 1.2)),
        4 => Some((b"ITEM_MOD_STRENGTH_SHORT", "Strength", 1.2)),
        5 => Some((b"ITEM_MOD_INTELLECT_SHORT", "Intellect", 1.2)),
        7 => Some((b"ITEM_MOD_STAMINA_SHORT", "Stamina", 1.8)),
        32 => Some((b"ITEM_MOD_CRIT_RATING_SHORT", "Critical Strike", 0.75)),
        36 => Some((b"ITEM_MOD_HASTE_RATING_SHORT", "Haste", 0.75)),
        40 => Some((b"ITEM_MOD_VERSATILITY", "Versatility", 0.75)),
        49 => Some((b"ITEM_MOD_MASTERY_RATING_SHORT", "Mastery", 0.75)),
        50 => Some((b"ITEM_MOD_EXTRA_ARMOR_SHORT", "Bonus Armor", 0.9)),
        61 => Some((b"ITEM_MOD_CR_SPEED_SHORT", "Speed", 0.35)),
        62 => Some((b"ITEM_MOD_CR_LIFESTEAL_SHORT", "Leech", 0.35)),
        63 => Some((b"ITEM_MOD_CR_AVOIDANCE_SHORT", "Avoidance", 0.35)),
        _ => None,
    }
}

fn estimate_stat_value(item: &items::ItemInfo, stat_percent: u16, scale: f64) -> i32 {
    let budget = f64::from(item.item_level) * slot_budget_multiplier(item.inventory_type);
    let raw = budget * (f64::from(stat_percent) / 10_000.0) * scale;
    raw.round().max(1.0) as i32
}

fn push_item_stat_lines(
    state: &mut LuaState,
    lines: Val,
    item: &items::ItemInfo,
    next_index: &mut i64,
) {
    let stat_color = global_color_or_fallback(state, b"GREEN_FONT_COLOR", (0.12, 1.0, 0.0));
    let class_index = borrow_state(state)
        .ok()
        .map(|sim| sim.player.class_index)
        .unwrap_or(2);

    for (&raw_stat_id, &stat_percent) in item
        .stat_modifier_bonus_stat
        .iter()
        .zip(item.stat_percent_editor.iter())
    {
        if raw_stat_id < 0 || stat_percent == 0 {
            continue;
        }
        let stat_id = resolve_hybrid_stat_id(raw_stat_id, class_index);
        let Some((label_key, label_fallback, scale)) = stat_display_spec(stat_id) else {
            continue;
        };
        let label = global_string_or_fallback(state, label_key, label_fallback);
        let value = estimate_stat_value(item, stat_percent, scale);
        let line_text = format!("+{value} {label}");
        push_tooltip_line(
            state,
            lines,
            *next_index,
            LINE_TYPE_SPELL_NAME,
            &line_text,
            Some(stat_color),
            false,
        );
        *next_index += 1;
    }
}

pub(super) fn tooltip_for_item_id(state: &mut LuaState, item_id: u32) -> Val {
    let Some(item) = items::get_item(item_id) else {
        return empty_tooltip(state, TOOLTIP_TYPE_ITEM);
    };
    let tooltip = create_identified_tooltip(state, TOOLTIP_TYPE_ITEM, item_id);
    populate_item_tooltip_lines(state, tooltip, item);
    tooltip
}

pub(super) fn populate_item_tooltip_lines(
    state: &mut LuaState,
    tooltip: Val,
    item: &items::ItemInfo,
) {
    let lines = table_get(state, tooltip, "lines");
    push_item_name_line(state, lines, item);
    push_item_level_line(state, lines, item);

    let mut next_index = 3;
    push_item_equip_slot_line(state, lines, item.inventory_type, &mut next_index);
    push_item_binding_line(state, lines, &mut next_index, item);
    push_item_stat_lines(state, lines, item, &mut next_index);
}

pub(super) fn push_plain_line(state: &mut LuaState, lines: Val, index: i64, text: &str) {
    push_tooltip_line(state, lines, index, LINE_TYPE_SPELL_NAME, text, None, false);
}

/// Ensure `C_PetInfo` namespace exists, register `_state()` accessor,
/// and return the mutable state sub-table.
pub(super) fn ensure_pet_info_state(state: &mut LuaState) -> Val {
    let ns_ref = ensure_namespace(state, "C_PetInfo").expect("C_PetInfo namespace");
    let ns = Val::Table(ns_ref);
    let pet_state = match table_get(state, ns, "_pet_state_data") {
        table @ Val::Table(_) => table,
        _ => {
            let t = create_table(state);
            // sub-tables used by tests / C_PetInfo._state
            let spell_map = create_table(state);
            table_set(state, t, "spellByPetActionID", spell_map);
            let action_map = create_table(state);
            table_set(state, t, "petActionsByID", action_map);
            let tamers_map = create_table(state);
            table_set(state, t, "petTamersByMapID", tamers_map);
            let passive_actions = create_table(state);
            table_set(state, t, "passivePetActionIDs", passive_actions);
            table_set(state, ns, "_pet_state_data", t);
            table_set(state, ns, "_state", t);
            t
        }
    };
    if !matches!(table_get(state, ns, "_state"), Val::Table(_)) {
        table_set(state, ns, "_state", pet_state);
    }
    pet_state
}

pub(super) fn table_get_int(state: &LuaState, table: Val, index: i32) -> Val {
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    state
        .gc
        .tables
        .get(table_ref)
        .map(|t| t.get_int(index as i64))
        .unwrap_or(Val::Nil)
}

pub(super) fn pet_action_spell_id(state: &mut LuaState, slot: i32) -> Option<u32> {
    let pet_state = ensure_pet_info_state(state);
    // spellByPetActionID[slot] → spell id (direct integer key mapping)
    let spell_map = table_get(state, pet_state, "spellByPetActionID");
    if let Some(spell_id) = table_get_u32(state, table_get_int(state, spell_map, slot)) {
        return Some(spell_id);
    }
    // petActionsByID[slot].spellID fallback
    let action_map = table_get(state, pet_state, "petActionsByID");
    let entry = table_get_int(state, action_map, slot);
    let spell_value = table_get(state, entry, "spellID");
    if let Some(spell_id) = table_get_u32(state, spell_value) {
        return Some(spell_id);
    }
    None
}

fn pet_info_slot_from_stack(state: &LuaState, slot_index: i32) -> Option<i32> {
    match stack_val(state, slot_index) {
        Val::Num(n) => Some(n as i32),
        Val::Str(_) => val_to_string(state, stack_val(state, slot_index))
            .and_then(|text| text.parse::<i32>().ok()),
        _ => None,
    }
}

fn table_get_u32(state: &LuaState, value: Val) -> Option<u32> {
    match value {
        Val::Num(n) if n >= 0.0 => Some(n as u32),
        Val::Str(_) => val_to_string(state, value).and_then(|text| text.parse::<u32>().ok()),
        _ => None,
    }
}

fn clone_pet_tamer_entry(state: &mut LuaState, entry: Val) -> Val {
    let copy = create_table(state);
    for key in ["areaPoiID", "name", "atlasName", "textureIndex"] {
        let value = table_get(state, entry, key);
        if !matches!(value, Val::Nil) {
            table_set(state, copy, key, value);
        }
    }
    let position = table_get(state, entry, "position");
    if matches!(position, Val::Table(_)) {
        let position_copy = create_table(state);
        for key in ["x", "y"] {
            let value = table_get(state, position, key);
            if !matches!(value, Val::Nil) {
                table_set(state, position_copy, key, value);
            }
        }
        table_set(state, copy, "position", position_copy);
    }
    copy
}

fn clone_pet_tamers_for_map(state: &mut LuaState, tamers: Val) -> Val {
    let clone = create_table(state);
    let mut index = 1_i64;
    loop {
        let entry = table_get_int(state, tamers, index as i32);
        if matches!(entry, Val::Nil) {
            break;
        }
        let entry_copy = clone_pet_tamer_entry(state, entry);
        set_table_array(state, clone, index, entry_copy);
        index += 1;
    }
    clone
}

pub(super) fn get_pet_tamers_for_map(state: &mut LuaState) -> LuaResult<u32> {
    let Some(map_id) = pet_info_slot_from_stack(state, 1) else {
        let empty = create_table(state);
        state.push(empty);
        return Ok(1);
    };
    let pet_state = ensure_pet_info_state(state);
    let tamers_by_map = table_get(state, pet_state, "petTamersByMapID");
    let tamers = table_get_int(state, tamers_by_map, map_id);
    let result = match tamers {
        Val::Table(_) => clone_pet_tamers_for_map(state, tamers),
        _ => create_table(state),
    };
    state.push(result);
    Ok(1)
}

pub(super) fn get_spell_for_pet_action(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = pet_info_slot_from_stack(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    match pet_action_spell_id(state, slot) {
        Some(spell_id) => state.push(Val::Num(spell_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn is_pet_action_passive(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = pet_info_slot_from_stack(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let pet_state = ensure_pet_info_state(state);
    let passive_actions = table_get(state, pet_state, "passivePetActionIDs");
    let action_map = table_get(state, pet_state, "petActionsByID");
    let action_entry = table_get_int(state, action_map, slot);
    let is_passive = matches!(table_get_int(state, passive_actions, slot), Val::Bool(true))
        || matches!(table_get(state, action_entry, "isPassive"), Val::Bool(true));
    state.push(Val::Bool(is_passive));
    Ok(1)
}
