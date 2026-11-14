//! R88n.2 Retail `C_TooltipInfo` Azerite Essence getters.

use crate::c_api::azerite_essence_spell_selector::{
    AzeriteEssenceSpellSelector, spell_for_selector,
};
use crate::c_api::tooltip_azerite_catalog_data::{
    AZERITE_ESSENCE_POWERS, AZERITE_ESSENCES, AzeriteEssenceCatalogRecord,
    AzeriteEssencePowerRecord,
};
use crate::c_api::tooltip_getter_args;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, append_spell_to_tooltip, compose_tooltip, item_quality_color,
};
use crate::lua_api::methods::{borrow_state, call_function_state, create_string, val_to_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

const ESSENCE_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetAzeriteEssence(essenceID [, rank])";
const ESSENCE_SLOT_USAGE: &str = "Usage: local data = C_TooltipInfo.GetAzeriteEssenceSlot(slot)";
const TOOLTIP_TYPE_AZERITE_ESSENCE: f64 = 8.0;
// INFERRED: R88n.2 establishes the heading content and quality rank, not the line type.
const LINE_TYPE_ESSENCE_HEADING: f64 = 0.0;

pub(crate) fn register_essence(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetAzeriteEssence", get_azerite_essence)
}

pub(crate) fn register_slot(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAzeriteEssenceSlot",
        get_azerite_essence_slot,
    )
}

fn get_azerite_essence(state: &mut LuaState) -> LuaResult<u32> {
    let essence_id = tooltip_getter_args::round7_signed(state, 1, ESSENCE_USAGE)?;
    let requested_rank = tooltip_getter_args::round7_optional_unsigned(state, 2, ESSENCE_USAGE)?;
    if ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let Some(essence) = essence_record(essence_id) else {
        return no_values();
    };
    let Some(rank) = selected_rank(state, essence_id, requested_rank) else {
        return no_values();
    };
    let Some(rank_one_power) = power_record(essence_id, 1) else {
        return no_values();
    };
    let Some(selected_power) = power_record(essence_id, rank) else {
        return no_values();
    };
    let (is_major_active, is_active) = active_slot_roles(state, essence_id)?;

    let main_spell = if is_major_active {
        spell_for_selector(
            rank_one_power,
            AzeriteEssenceSpellSelector::MajorDescription,
        )
    } else {
        spell_for_selector(selected_power, AzeriteEssenceSpellSelector::MajorActual)
    };
    let passive_spell = if is_active {
        spell_for_selector(
            rank_one_power,
            AzeriteEssenceSpellSelector::MinorDescription,
        )
    } else {
        spell_for_selector(selected_power, AzeriteEssenceSpellSelector::MinorActual)
    };
    publish_essence_tooltip(
        state,
        essence_id,
        Some((essence.name, rank)),
        Some(main_spell),
        passive_spell,
    )
}

fn get_azerite_essence_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = tooltip_getter_args::round7_signed(state, 1, ESSENCE_SLOT_USAGE)? as u8;
    if slot > 3 {
        return Err(rilua::runtime_error(format!(
            "outside of expected range 0 to 3 - {ESSENCE_SLOT_USAGE}"
        )));
    }
    if ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let Some(essence_id) = active_essence_for_slot(state, i32::from(slot)) else {
        return no_values();
    };
    let Some(rank) = selected_rank(state, essence_id, None) else {
        return no_values();
    };
    let Some(_essence) = essence_record(essence_id) else {
        return no_values();
    };
    let Some(rank_one_power) = power_record(essence_id, 1) else {
        return no_values();
    };
    let Some(_selected_power) = power_record(essence_id, rank) else {
        return no_values();
    };

    let main_spell = (slot == 0).then(|| {
        spell_for_selector(
            rank_one_power,
            AzeriteEssenceSpellSelector::MajorDescription,
        )
    });
    let passive_spell = spell_for_selector(
        rank_one_power,
        AzeriteEssenceSpellSelector::MinorDescription,
    );
    publish_essence_tooltip(state, essence_id, None, main_spell, passive_spell)
}

fn publish_essence_tooltip(
    state: &mut LuaState,
    essence_id: i32,
    heading: Option<(&str, u32)>,
    main_spell: Option<u32>,
    passive_spell: u32,
) -> LuaResult<u32> {
    let tooltip = compose_tooltip(
        state,
        TOOLTIP_TYPE_AZERITE_ESSENCE,
        Some(f64::from(essence_id)),
    );
    if let Some((name, rank)) = heading {
        let text = match format_essence_heading(state, name, rank) {
            Ok(text) => text,
            Err(error) => return failed_producer(state, error),
        };
        let quality = (rank + 1) as u8;
        let color = item_quality_color(quality);
        append_line(
            state,
            tooltip,
            LINE_TYPE_ESSENCE_HEADING,
            &text,
            Some(color),
        );
    }
    if let Some(spell_id) = main_spell {
        if let Err(error) = append_spell_to_tooltip(state, tooltip, spell_id) {
            return failed_producer(state, error);
        }
    }
    if let Err(error) = append_spell_to_tooltip(state, tooltip, passive_spell) {
        return failed_producer(state, error);
    }
    state.push(tooltip);
    Ok(1)
}

fn format_essence_heading(state: &mut LuaState, name: &str, rank: u32) -> LuaResult<String> {
    let template = crate::c_api::global_val(state, "AZERITE_ESSENCE_TOOLTIP_NAME_RANK");
    let formatter = crate::c_api::global_val(state, "format");
    let name = create_string(state, name);
    let text = call_function_state(
        state,
        formatter,
        &[template, name, Val::Num(f64::from(rank))],
    )?;
    val_to_string(state, text).ok_or_else(|| {
        rilua::runtime_error("localized Azerite Essence name/rank formatting returned no text")
    })
}

fn essence_record(essence_id: i32) -> Option<&'static AzeriteEssenceCatalogRecord> {
    let essence_id = u32::try_from(essence_id).ok()?;
    let index = AZERITE_ESSENCES
        .binary_search_by_key(&essence_id, |record| record.id)
        .ok()?;
    AZERITE_ESSENCES.get(index)
}

fn power_record(essence_id: i32, rank: u32) -> Option<&'static AzeriteEssencePowerRecord> {
    let essence_id = u32::try_from(essence_id).ok()?;
    let key = (essence_id, rank);
    let index = AZERITE_ESSENCE_POWERS
        .binary_search_by_key(&key, |record| (record.essence_id, record.rank))
        .ok()?;
    AZERITE_ESSENCE_POWERS.get(index)
}

fn selected_rank(state: &LuaState, essence_id: i32, requested: Option<u32>) -> Option<u32> {
    match requested {
        Some(rank) => Some(rank.max(1)),
        None => player_rank_or_list_miss(state, essence_id),
    }
}

fn player_rank_or_list_miss(state: &LuaState, essence_id: i32) -> Option<u32> {
    let sim = borrow_state(state).ok()?;
    let essences = &sim.azerite_essence.essences;
    if !sim.player.present || essences.is_empty() {
        return None;
    }
    let rank = essences.get(&essence_id).map_or(0, |essence| essence.rank);
    Some(u32::try_from(rank).ok()?.max(1))
}

fn active_slot_roles(state: &LuaState, essence_id: i32) -> LuaResult<(bool, bool)> {
    let sim = borrow_state(state)?;
    let mut is_major = false;
    let mut is_active = false;
    for milestone in &sim.azerite_essence.milestones {
        if milestone.active_essence_id == Some(essence_id) {
            is_active = true;
            is_major |= milestone.slot == Some(0);
        }
    }
    Ok((is_major, is_active))
}

fn active_essence_for_slot(state: &LuaState, slot: i32) -> Option<i32> {
    borrow_state(state)
        .ok()?
        .azerite_essence
        .milestones
        .iter()
        .find(|milestone| milestone.slot == Some(slot))
        .and_then(|milestone| milestone.active_essence_id)
}

fn failed_producer(state: &mut LuaState, error: impl std::fmt::Display) -> LuaResult<u32> {
    crate::lua_api::script_helpers::collect_lua_error(
        state,
        &format!("Azerite Essence tooltip: {error}"),
    );
    no_values()
}

fn no_values() -> LuaResult<u32> {
    Ok(0)
}
