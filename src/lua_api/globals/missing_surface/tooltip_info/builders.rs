use super::super::{
    LINE_TYPE_EQUIP_SLOT, LINE_TYPE_ITEM_BINDING, LINE_TYPE_ITEM_LEVEL, LINE_TYPE_ITEM_NAME,
    LINE_TYPE_SPELL_NAME, TOOLTIP_TYPE_ITEM, ensure_namespace, set_table_array,
};
use crate::items;
use crate::lua_api::frame::methods::widgets::tooltip::text_markup::{
    TooltipColorSegment, tooltip_line_text_and_color,
};
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[cfg(all(test, any(feature = "profile-retail", feature = "client-ptr")))]
#[path = "builder_contract_tests.rs"]
mod builder_contract_tests;

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

pub(crate) fn push_tooltip_line(
    state: &mut LuaState,
    lines: Val,
    index: i64,
    line_type: f64,
    left_text: &str,
    left_color: Option<(f64, f64, f64)>,
    wrap: bool,
) -> Val {
    // CreateColor executes Lua and may collect; Rust-held Vals are not GC roots.
    let saved_top = state.top;
    state.push(lines);
    let line = create_table(state);
    set_table_array(state, lines, index, line);
    let line_type = if uses_modern_tooltip_data() {
        remap_line_type(state, line_type, left_color)
    } else {
        line_type
    };
    table_set(state, line, "type", Val::Num(line_type));
    if uses_modern_tooltip_data() {
        let rgb = left_color
            .or_else(|| global_color(state, b"NORMAL_FONT_COLOR"))
            .unwrap_or((1.0, 0.82, 0.0));
        let color = color_table(state, rgb.0, rgb.1, rgb.2, 1.0);
        crate::c_api::tooltip_data::publish_rgb(state, color, rgb);
        crate::c_api::tooltip_data::publish_side(state, line, false, Some(left_text), color);
        if wrap {
            table_set(state, line, "wrapText", Val::Bool(true));
        }
        state.top = saved_top;
        return line;
    }
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
    state.top = saved_top;
    line
}

pub(super) fn uses_modern_tooltip_data() -> bool {
    crate::c_api::tooltip_data::enabled()
}

/// D56 builder initialization: populated results get the next `dataInstanceID`.
/// Miss results keep their prior shape because WM82 leaves miss publication UNKNOWN.
pub(super) fn stamp_data_instance(state: &mut LuaState, tooltip: Val) {
    if !uses_modern_tooltip_data() {
        return;
    }
    crate::c_api::tooltip_data::stamp_instance(state, tooltip);
}

fn remap_line_type(state: &mut LuaState, line_type: f64, color: Option<(f64, f64, f64)>) -> f64 {
    if line_type != 0.0 || color.is_none() {
        return line_type;
    }
    if color == global_color(state, b"ERROR_COLOR") {
        41.0
    } else if color == global_color(state, b"DISABLED_FONT_COLOR") {
        42.0
    } else {
        line_type
    }
}

fn color_segment_table(state: &mut LuaState, segments: Vec<TooltipColorSegment>) -> Val {
    let saved_top = state.top;
    let table = create_table(state);
    state.push(table);
    for (index, segment) in segments.into_iter().enumerate() {
        let entry = create_table(state);
        set_table_array(state, table, index as i64 + 1, entry);
        let text = create_string(state, &segment.text);
        table_set(state, entry, "text", text);
        let (r, g, b) = segment.color;
        let color = color_table(state, r, g, b, 1.0);
        table_set(state, entry, "color", color);
    }
    state.top = saved_top;
    table
}

pub(super) fn empty_tooltip(state: &mut LuaState, tooltip_type: f64) -> Val {
    let tooltip = create_table(state);
    let lines = create_table(state);
    table_set(state, tooltip, "lines", lines);
    if uses_modern_tooltip_data() {
        crate::c_api::tooltip_data::assign_identity(state, tooltip, tooltip_type, 0, Val::Nil);
    } else {
        table_set(state, tooltip, "type", Val::Num(tooltip_type));
    }
    tooltip
}

/// WM94: totems have two plain lines and identity type 18 with zero native ID.
pub(super) fn tooltip_for_totem_text(
    state: &mut LuaState,
    name: &str,
    remaining: &str,
) -> LuaResult<Val> {
    let highlight = global_color(state, b"HIGHLIGHT_FONT_COLOR")
        .ok_or_else(|| rilua::runtime_error("GetTotem requires HIGHLIGHT_FONT_COLOR"))?;
    let saved_top = state.top;
    let tooltip = empty_tooltip(state, 18.0);
    state.push(tooltip);
    stamp_data_instance(state, tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_tooltip_line(state, lines, 1, 0.0, name, None, false);
    push_tooltip_line(state, lines, 2, 0.0, remaining, Some(highlight), false);
    state.top = saved_top;
    Ok(tooltip)
}

pub(super) fn create_identified_tooltip(state: &mut LuaState, tooltip_type: f64, id: u32) -> Val {
    let tooltip = empty_tooltip(state, tooltip_type);
    table_set(state, tooltip, "id", Val::Num(id as f64));
    tooltip
}

pub(crate) fn item_quality_color(quality: u8) -> (f64, f64, f64) {
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

pub(super) fn global_color(state: &mut LuaState, key: &[u8]) -> Option<(f64, f64, f64)> {
    let name = std::str::from_utf8(key).ok()?;
    crate::lua_api::global_colors::global_color_rgb(state, name)
}

fn global_color_or_fallback(
    state: &mut LuaState,
    key: &[u8],
    fallback: (f64, f64, f64),
) -> (f64, f64, f64) {
    global_color(state, key).unwrap_or(fallback)
}

fn push_item_stat_lines(
    state: &mut LuaState,
    lines: Val,
    stats: &[crate::c_api::item_tooltip::ItemTooltipStat],
    next_index: &mut i64,
) {
    let stat_color = global_color_or_fallback(state, b"GREEN_FONT_COLOR", (0.12, 1.0, 0.0));
    // R80 Q2: ordinary item-mod amounts are not proved by the admitted calculator.
    // Keep the existing position, but publish only final amounts supplied by the host.
    for stat in stats {
        let line_text = format!("{:+} {}", stat.amount, stat.label);
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

fn publish_item_metadata(
    state: &mut LuaState,
    tooltip: Val,
    input: &crate::c_api::item_tooltip::ItemTooltipInput,
) {
    for (key, value) in [("guid", &input.guid), ("hyperlink", &input.hyperlink)] {
        if let Some(value) = value {
            let value = create_string(state, value);
            table_set(state, tooltip, key, value);
        }
    }
    if let Some(level) = input.override_item_level.filter(|level| *level != 0) {
        table_set(
            state,
            tooltip,
            "overrideItemLevel",
            Val::Num(f64::from(level)),
        );
    }
    for (key, value) in [
        ("isCorruptedItem", input.is_corrupted_item),
        ("isAzeriteEmpoweredItem", input.is_azerite_empowered_item),
        ("isAzeriteItem", input.is_azerite_item),
    ] {
        if let Some(value) = value {
            table_set(state, tooltip, key, Val::Bool(value));
        }
    }
}

pub(super) fn tooltip_for_item_id(state: &mut LuaState, item_id: u32) -> Val {
    let Some(item) = items::get_item(item_id) else {
        return empty_tooltip(state, TOOLTIP_TYPE_ITEM);
    };
    tooltip_for_item_info(state, item_id, item)
}

pub(super) fn tooltip_for_item_info(
    state: &mut LuaState,
    item_id: u32,
    item: &items::ItemInfo,
) -> Val {
    let tooltip = create_identified_tooltip(state, TOOLTIP_TYPE_ITEM, item_id);
    stamp_data_instance(state, tooltip);
    populate_item_tooltip_lines(state, tooltip, item_id, item);
    tooltip
}

/// WM97 DQ4: relic lines run inside the item producer, after the shared item
/// lines. The native line order is UNKNOWN; only the texts are asserted.
pub(super) fn tooltip_for_socketed_relic(
    state: &mut LuaState,
    item_id: u32,
    item: &items::ItemInfo,
    relic: &crate::c_api::item_descriptor::RelicMetadata,
) -> LuaResult<Val> {
    let tooltip = tooltip_for_item_info(state, item_id, item);
    append_socketed_relic_lines(state, tooltip, relic)?;
    Ok(tooltip)
}

fn append_socketed_relic_lines(
    state: &mut LuaState,
    tooltip: Val,
    relic: &crate::c_api::item_descriptor::RelicMetadata,
) -> LuaResult<()> {
    let type_text = relic_type_line(state, relic.slot_type)?;
    let increase_text = if relic.item_levels > 0 {
        Some(format_global_string(
            state,
            b"RELIC_TOOLTIP_ILVL_INCREASE",
            "+%d Item Levels",
            &[Val::Num(f64::from(relic.item_levels))],
        )?)
    } else {
        None
    };
    let saved_top = state.top;
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    let mut index = super::spell::tooltip_line_count(state, lines) + 1;
    let artifact_color = global_color_or_fallback(state, b"ITEM_ARTIFACT_COLOR", (0.9, 0.8, 0.5));
    push_tooltip_line(
        state,
        lines,
        index,
        LINE_TYPE_SPELL_NAME,
        &type_text,
        Some(artifact_color),
        false,
    );
    index += 1;
    if let Some(text) = increase_text {
        push_tooltip_line(
            state,
            lines,
            index,
            LINE_TYPE_SPELL_NAME,
            &text,
            None,
            false,
        );
    }
    state.top = saved_top;
    Ok(())
}

fn relic_type_line(
    state: &mut LuaState,
    slot_type: Option<crate::c_api::item_descriptor::RelicSlotType>,
) -> LuaResult<String> {
    let label = match slot_type {
        Some(slot_type) => {
            let key = format!("RELIC_SLOT_TYPE_{}", slot_type.token);
            global_string_or_fallback(state, key.as_bytes(), slot_type.label)
        }
        None => global_string_or_fallback(state, b"UNKNOWN", "Unknown"),
    };
    let label_val = create_string(state, &label);
    format_global_string(
        state,
        b"RELIC_TOOLTIP_TYPE",
        "%s Artifact Relic",
        &[label_val],
    )
}

/// Apply a global format string through `string.format`, preserving positional
/// specifiers. Failures propagate; a missing localised shape is not masked.
fn format_global_string(
    state: &mut LuaState,
    key: &[u8],
    fallback: &str,
    args: &[Val],
) -> LuaResult<String> {
    let format = global_string_or_fallback(state, key, fallback);
    let format_val = create_string(state, &format);
    let globals = Val::Table(state.global);
    let string_table = table_get(state, globals, "string");
    let format_fn = table_get(state, string_table, "format");
    let mut call_args = Vec::with_capacity(args.len() + 1);
    call_args.push(format_val);
    call_args.extend_from_slice(args);
    let formatted = call_function_state(state, format_fn, &call_args)?;
    val_to_string(state, formatted)
        .ok_or_else(|| rilua::runtime_error("string.format did not return a string"))
}

pub(super) fn populate_item_tooltip_lines(
    state: &mut LuaState,
    tooltip: Val,
    item_id: u32,
    item: &items::ItemInfo,
) {
    let saved_top = state.top;
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_item_name_line(state, lines, item);
    push_item_level_line(state, lines, item);

    let mut next_index = 3;
    push_item_equip_slot_line(state, lines, item.inventory_type, &mut next_index);
    push_item_binding_line(state, lines, &mut next_index, item);
    // A standalone Lua state has no host input provider: publish headers only.
    let input = borrow_state(state).ok().and_then(|sim| {
        sim.item_tooltip_inputs
            .get(&(item_id, item.item_level))
            .cloned()
    });
    if let Some(input) = input {
        publish_item_metadata(state, tooltip, &input);
        push_item_stat_lines(state, lines, &input.stats, &mut next_index);
    }
    state.top = saved_top;
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
