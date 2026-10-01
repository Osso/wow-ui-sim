//! Stack-rooted serializers. No taint clearing, secret unwrapping or shared Lua rows.

use super::*;
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{create_table, table_set_static};

/// Every builder leaves one root on the VM stack across nested allocations.
fn push_table(state: &mut LuaState) -> Val {
    let row = create_table(state);
    state.push(row);
    row
}

fn attach_child(state: &mut LuaState, parent: Val, key: &'static str, child: Val) {
    table_set_static(state, parent, key, child);
    state.top -= 1;
}

fn push_sequence<T>(
    state: &mut LuaState,
    entries: &[T],
    push_row: fn(&mut LuaState, &T) -> Val,
) -> Val {
    let sequence = push_table(state);
    for (index, input) in entries.iter().enumerate() {
        let row = push_row(state, input);
        set_table_array(state, sequence, (index + 1) as i64, row);
        state.top -= 1;
    }
    sequence
}

fn set_text(state: &mut LuaState, row: Val, key: &'static str, text: &str) {
    let text = create_string(state, text);
    table_set_static(state, row, key, text);
}

fn set_number(state: &mut LuaState, row: Val, key: &'static str, number: f64) {
    table_set_static(state, row, key, Val::Num(number));
}

fn set_optional_number(state: &mut LuaState, row: Val, key: &'static str, number: Option<f64>) {
    if let Some(number) = number {
        set_number(state, row, key, number);
    }
}

pub(super) fn push_available_sessions(
    state: &mut LuaState,
    entries: &[DamageMeterAvailableCombatSession],
) -> Val {
    push_sequence(state, entries, push_available_session)
}

fn push_available_session(state: &mut LuaState, input: &DamageMeterAvailableCombatSession) -> Val {
    let row = push_table(state);
    set_number(state, row, "sessionID", input.session_id as f64);
    set_text(state, row, "name", &input.name);
    set_optional_number(state, row, "durationSeconds", input.duration_seconds);
    row
}

pub(super) fn push_session(state: &mut LuaState, input: &DamageMeterCombatSession) -> Val {
    let row = push_table(state);
    set_number(state, row, "maxAmount", input.max_amount);
    set_number(state, row, "totalAmount", input.total_amount);
    set_optional_number(state, row, "durationSeconds", input.duration_seconds);
    let sources = push_sequence(state, &input.combat_sources, push_aggregate_source);
    attach_child(state, row, "combatSources", sources);
    row
}

fn push_aggregate_source(state: &mut LuaState, input: &DamageMeterCombatSource) -> Val {
    let row = push_table(state);
    if let Some(guid) = &input.source_guid {
        set_text(state, row, "sourceGUID", guid);
    }
    set_optional_number(
        state,
        row,
        "sourceCreatureID",
        input.source_creature_id.map(|id| id as f64),
    );
    for (key, text) in [
        ("name", input.name.as_str()),
        ("classFilename", input.class_filename.as_str()),
        ("classification", input.classification.as_str()),
    ] {
        set_text(state, row, key, text);
    }
    if let Some(faction) = &input.faction_group {
        set_text(state, row, "factionGroup", faction);
    }
    set_aggregate_metrics(state, row, input);
    row
}

fn set_aggregate_metrics(state: &mut LuaState, row: Val, input: &DamageMeterCombatSource) {
    for (key, number) in [
        ("specIconID", input.spec_icon_id as f64),
        ("totalAmount", input.total_amount),
        ("amountPerSecond", input.amount_per_second),
        ("deathRecapID", input.death_recap_id as f64),
        ("deathTimeSeconds", input.death_time_seconds),
        ("sourceDisplayType", f64::from(input.source_display_type)),
    ] {
        set_number(state, row, key, number);
    }
    table_set_static(
        state,
        row,
        "isLocalPlayer",
        Val::Bool(input.is_local_player),
    );
}

pub(super) fn push_source_details(
    state: &mut LuaState,
    input: &DamageMeterCombatSessionSource,
) -> Val {
    let row = push_table(state);
    set_number(state, row, "maxAmount", input.max_amount);
    set_number(state, row, "totalAmount", input.total_amount);
    let spells = push_sequence(state, &input.combat_spells, push_spell);
    attach_child(state, row, "combatSpells", spells);
    row
}

fn push_spell(state: &mut LuaState, input: &DamageMeterCombatSpell) -> Val {
    let row = push_table(state);
    for (key, number) in [
        ("spellID", input.spell_id as f64),
        ("totalAmount", input.total_amount),
        ("amountPerSecond", input.amount_per_second),
        ("overkillAmount", input.overkill_amount),
    ] {
        set_number(state, row, key, number);
    }
    set_text(state, row, "creatureName", &input.creature_name);
    table_set_static(state, row, "isAvoidable", Val::Bool(input.is_avoidable));
    table_set_static(state, row, "isDeadly", Val::Bool(input.is_deadly));
    let details = push_spell_unit_details(state, &input.combat_spell_details);
    attach_child(state, row, "combatSpellDetails", details);
    row
}

fn push_spell_unit_details(state: &mut LuaState, input: &DamageMeterCombatSpellUnitDetails) -> Val {
    let row = push_table(state);
    for (key, text) in [
        ("unitName", input.unit_name.as_str()),
        ("unitClassFilename", input.unit_class_filename.as_str()),
        ("classification", input.classification.as_str()),
    ] {
        set_text(state, row, key, text);
    }
    table_set_static(state, row, "isPet", Val::Bool(input.is_pet));
    table_set_static(state, row, "isMob", Val::Bool(input.is_mob));
    set_number(state, row, "amount", input.amount);
    set_number(state, row, "specIconID", input.spec_icon_id as f64);
    row
}
