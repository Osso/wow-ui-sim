use super::super::{
    LINE_TYPE_ITEM_NAME, LINE_TYPE_SPELL_DESCRIPTION, LINE_TYPE_SPELL_NAME, TOOLTIP_TYPE_ITEM,
    TOOLTIP_TYPE_SPELL, TOOLTIP_TYPE_UNIT_AURA,
};
use super::builders::{
    create_identified_tooltip, empty_tooltip, item_quality_color, push_tooltip_line,
    stamp_data_instance, tooltip_for_item_id,
};
use crate::lua_api::game_data;
use crate::lua_api::methods::{borrow_state, table_get, table_set};
use crate::spells;
use rilua::Val;
use rilua::vm::state::LuaState;

const SPELL_TOOLTIP_WORD_WRAP_MIN_WIDTH: f64 = 240.0;

fn push_highlight_spell_line(state: &mut LuaState, lines: Val, index: i64, text: &str) -> Val {
    let color = spell_color_from_global(state, b"HIGHLIGHT_FONT_COLOR", (1.0, 1.0, 1.0));
    let line = push_tooltip_line(state, lines, index, 0.0, text, Some(color), false);
    super::restore_plain_line_type(state, line);
    line
}

fn push_spell_name_line(state: &mut LuaState, lines: Val, index: i64, text: &str) {
    let color = spell_color_from_global(state, b"HIGHLIGHT_FONT_COLOR", (1.0, 1.0, 1.0));
    push_tooltip_line(
        state,
        lines,
        index,
        LINE_TYPE_SPELL_NAME,
        text,
        Some(color),
        false,
    );
}

fn push_spell_description_line(state: &mut LuaState, lines: Val, index: i64, spell_id: u32) {
    let description = resolved_spell_description(state, spell_id);
    push_tooltip_line(
        state,
        lines,
        index,
        LINE_TYPE_SPELL_DESCRIPTION,
        &description,
        None,
        true,
    );
}

fn spell_color_from_global(
    state: &mut LuaState,
    color_name: &[u8],
    fallback: (f64, f64, f64),
) -> (f64, f64, f64) {
    std::str::from_utf8(color_name)
        .ok()
        .and_then(|name| crate::lua_api::global_colors::global_color_rgb(state, name))
        .unwrap_or(fallback)
}

pub(super) fn tooltip_line_count(state: &LuaState, lines: Val) -> i64 {
    let Val::Table(lines_ref) = lines else {
        return 0;
    };
    let mut index = 1;
    while state
        .gc
        .tables
        .get(lines_ref)
        .is_some_and(|table| table.get_int(index) != Val::Nil)
    {
        index += 1;
    }
    index - 1
}

pub(super) fn resolved_spell_description(state: &LuaState, spell_id: u32) -> String {
    match borrow_state(state) {
        Ok(sim) => crate::spell_description_resolver::resolve_spell_description(&sim, spell_id),
        Err(_) => "No description available.".to_string(),
    }
}

fn push_spell_tooltip_lines(
    state: &mut LuaState,
    lines: Val,
    spell_id: u32,
    spell_name: &str,
    is_link: bool,
) -> rilua::LuaResult<()> {
    let input = {
        let sim = borrow_state(state).expect("spell tooltip requires simulator state");
        sim.spell_tooltip_inputs
            .get(&spell_id)
            .cloned()
            .unwrap_or_default()
    };
    push_spell_name_line(state, lines, 1, spell_name);
    let mut index = 2;
    // R81 Q1: replacement is the first ordinary detail row; zero skips lookup.
    if !input.suppress_replacement && input.overrides_spell_id != 0 {
        if let Some(replaced) = spells::get_spell(input.overrides_spell_id) {
            let text = super::format_global_text(state, "REPLACES_SPELL", replaced.name)?;
            let line = push_tooltip_line(state, lines, index, 0.0, &text, None, false);
            super::restore_plain_line_type(state, line);
            index += 1;
        }
    }
    if let Some(text) = input
        .azerite_annotation
        .as_deref()
        .filter(|text| !text.is_empty())
    {
        let color = spell_color_from_global(state, b"AZERITE_ESSENCE_COLOR", (1.0, 0.82, 0.0));
        let line = push_tooltip_line(state, lines, index, 0.0, text, Some(color), false);
        super::restore_plain_line_type(state, line);
        index += 1;
    }
    index = push_cost_range_lines(state, lines, index, spell_id, &input);
    if input.cast_is_visible() {
        // R81 Q2: supplied final helper duration, not catalog base time.
        // The link flag reaches helper input selection, never the cost/range gates.
        if let Some(text) = spell_cast_text(state, &input, is_link)? {
            push_highlight_spell_line(state, lines, index, &text);
            index += 1;
        }
        if let Some(text) = input.recast_text.as_deref().filter(|text| !text.is_empty()) {
            push_highlight_spell_line(state, lines, index, text);
            index += 1;
        }
    }
    if let Some(text) = input
        .cooldown_text
        .as_deref()
        .filter(|text| !text.is_empty())
    {
        push_highlight_spell_line(state, lines, index, text);
        index += 1;
    }
    push_spell_description_line(state, lines, index, spell_id);
    Ok(())
}

fn push_cost_range_lines(
    state: &mut LuaState,
    lines: Val,
    mut index: i64,
    spell_id: u32,
    input: &crate::c_api::spell_tooltip::SpellTooltipInput,
) -> i64 {
    let cost = {
        let sim = borrow_state(state).expect("spell tooltip requires simulator state");
        crate::c_api::spell_tooltip_cost::spell_cost_line(&sim, spell_id).map(str::to_owned)
    };
    let mut ranges = input
        .ranges
        .iter()
        .filter(|text| !text.is_empty() && input.range_is_visible());
    if let Some(cost) = cost {
        let line = push_highlight_spell_line(state, lines, index, &cost);
        if let Some(range) = ranges.next() {
            // The parent line already roots its color before any further Lua allocation.
            let color = table_get(state, line, "leftColor");
            crate::c_api::tooltip_data::publish_side(state, line, true, Some(range), color);
        }
        index += 1;
    }
    for range in ranges {
        push_highlight_spell_line(state, lines, index, range);
        index += 1;
    }
    index
}

fn spell_cast_text(
    state: &mut LuaState,
    input: &crate::c_api::spell_tooltip::SpellTooltipInput,
    is_link: bool,
) -> rilua::LuaResult<Option<String>> {
    let cast_ms = if is_link {
        input.link_cast_ms.or(input.cast_ms)
    } else {
        input.cast_ms
    };
    match cast_ms {
        Some(ms) if ms > 0 => {
            let (token, divisor) = if ms < 60000 {
                ("SPELL_CAST_TIME_SEC", 1000.0)
            } else {
                ("SPELL_CAST_TIME_MIN", 60000.0)
            };
            super::format_global_numeric(state, token, f64::from(ms) / divisor).map(Some)
        }
        _ => Ok(input
            .alternate_cast_text
            .clone()
            .filter(|text| !text.is_empty())),
    }
}

pub(super) fn tooltip_for_spell_id(state: &mut LuaState, spell_id: u32) -> Val {
    tooltip_for_spell_options(state, spell_id, false, false)
}

pub(super) fn tooltip_for_spell_options(
    state: &mut LuaState,
    spell_id: u32,
    dont_override: bool,
    is_link: bool,
) -> Val {
    let effective = if dont_override {
        spell_id
    } else {
        borrow_state(state)
            .ok()
            .and_then(|sim| {
                sim.spell_tooltip_inputs
                    .get(&spell_id)
                    .and_then(|input| input.resolved_spell_id)
            })
            .filter(|id| *id != 0)
            .unwrap_or(spell_id)
    };
    tooltip_for_spell_source(state, effective, is_link)
}

pub(super) fn tooltip_for_spell_hyperlink(state: &mut LuaState, spell_id: u32) -> Val {
    tooltip_for_spell_source(state, spell_id, true)
}

fn tooltip_for_spell_source(state: &mut LuaState, spell_id: u32, is_link: bool) -> Val {
    let tooltip = create_identified_tooltip(state, TOOLTIP_TYPE_SPELL, spell_id);
    let Some(spell) = spells::get_spell(spell_id) else {
        return tooltip;
    };
    let saved_top = state.top;
    state.push(tooltip);
    stamp_data_instance(state, tooltip);
    let lines = table_get(state, tooltip, "lines");
    if let Err(error) = push_spell_tooltip_lines(state, lines, spell_id, spell.name, is_link) {
        crate::lua_api::script_helpers::collect_lua_error(
            state,
            &format!("spell tooltip: {error}"),
        );
    }
    set_spell_tooltip_width_hint(state, tooltip);
    state.top = saved_top;
    tooltip
}

pub(super) fn tooltip_for_unit_aura(
    state: &mut LuaState,
    aura: Option<game_data::AuraInfo>,
) -> Val {
    let Some(aura) = aura else {
        return empty_tooltip(state, TOOLTIP_TYPE_UNIT_AURA);
    };
    let saved_top = state.top;
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_UNIT_AURA);
    state.push(tooltip);
    stamp_data_instance(state, tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_spell_name_line(state, lines, 1, &aura.name);
    let remaining = borrow_state(state)
        .ok()
        .and_then(|sim| {
            sim.aura_tooltip_inputs
                .get(&aura.aura_instance_id)
                .and_then(|input| input.remaining_text.clone())
        })
        .filter(|text| !text.is_empty());
    let mut index = 2;
    if let Some(text) = remaining {
        push_spell_name_line(state, lines, index, &text);
        index += 1;
    }
    push_spell_description_line(state, lines, index, aura.spell_id as u32);
    state.top = saved_top;
    tooltip
}

pub(super) fn tooltip_for_toy_item_id(state: &mut LuaState, item_id: u32) -> Val {
    if crate::items::get_item(item_id).is_some() {
        return tooltip_for_item_id(state, item_id);
    }

    let toy_name = borrow_state(state).ok().and_then(|st| {
        st.world
            .toys
            .iter()
            .find(|toy| toy.item_id == item_id)
            .map(|toy| toy.name.clone())
    });
    let Some(toy_name) = toy_name else {
        return empty_tooltip(state, TOOLTIP_TYPE_ITEM);
    };

    let saved_top = state.top;
    let tooltip = create_identified_tooltip(state, TOOLTIP_TYPE_ITEM, item_id);
    state.push(tooltip);
    stamp_data_instance(state, tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_tooltip_line(
        state,
        lines,
        1,
        LINE_TYPE_ITEM_NAME,
        &toy_name,
        Some(item_quality_color(1)),
        false,
    );
    state.top = saved_top;
    tooltip
}

pub(super) fn tooltip_for_mount_spell_id(state: &mut LuaState, spell_id: u32) -> Val {
    if spells::get_spell(spell_id).is_some() {
        return tooltip_for_spell_id(state, spell_id);
    }
    let Some(mount_name) = find_mount_name_for_spell(state, spell_id) else {
        return create_identified_tooltip(state, TOOLTIP_TYPE_SPELL, spell_id);
    };
    build_mount_tooltip(state, spell_id, &mount_name)
}

fn find_mount_name_for_spell(state: &mut LuaState, spell_id: u32) -> Option<String> {
    borrow_state(state).ok().and_then(|st| {
        st.world
            .mounts
            .iter()
            .find(|mount| mount.spell_id == spell_id)
            .map(|mount| mount.name.clone())
    })
}

fn build_mount_tooltip(state: &mut LuaState, spell_id: u32, mount_name: &str) -> Val {
    let saved_top = state.top;
    let tooltip = create_identified_tooltip(state, TOOLTIP_TYPE_SPELL, spell_id);
    state.push(tooltip);
    stamp_data_instance(state, tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_tooltip_line(
        state,
        lines,
        1,
        LINE_TYPE_SPELL_NAME,
        mount_name,
        None,
        false,
    );
    push_tooltip_line(
        state,
        lines,
        3,
        LINE_TYPE_SPELL_DESCRIPTION,
        "Summons this mount.",
        None,
        true,
    );
    set_spell_tooltip_width_hint(state, tooltip);
    state.top = saved_top;
    tooltip
}

fn set_spell_tooltip_width_hint(state: &mut LuaState, tooltip: Val) {
    table_set(
        state,
        tooltip,
        "wordWrapMinWidth",
        Val::Num(SPELL_TOOLTIP_WORD_WRAP_MIN_WIDTH),
    );
}

pub(super) fn spell_id_for_talent_id(state: &LuaState, talent_id: u32) -> Option<u32> {
    let selected_entry_id = borrow_state(state)
        .ok()
        .and_then(|sim| sim.talents.node_selections.get(&talent_id).copied());
    crate::c_api::talent_spell::spell_id_for_talent(talent_id, selected_entry_id)
}

pub(super) fn lookup_player_aura(state: &LuaState, index: i32) -> Option<game_data::AuraInfo> {
    let sim = borrow_state(state).ok()?;
    sim.player.buffs.get((index - 1).max(0) as usize).cloned()
}

pub(super) fn lookup_player_aura_by_instance_id(
    state: &LuaState,
    aura_instance_id: i32,
) -> Option<game_data::AuraInfo> {
    let sim = borrow_state(state).ok()?;
    sim.player
        .buffs
        .iter()
        .find(|aura| aura.aura_instance_id == aura_instance_id)
        .cloned()
}
