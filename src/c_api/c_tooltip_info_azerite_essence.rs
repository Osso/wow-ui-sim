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
    append_line, append_spell_to_tooltip, compose_tooltip, format_global_text, item_quality_color,
    spell_description_text,
};
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_string, table_set, val_to_string,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

const ESSENCE_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetAzeriteEssence(essenceID [, rank])";
const ESSENCE_SLOT_USAGE: &str = "Usage: local data = C_TooltipInfo.GetAzeriteEssenceSlot(slot)";
const MAX_ESSENCE_SLOT: u8 = 3;
const SOURCE_STAGE_DISABLED_NEXT_RANK: u32 = 4;
const TOOLTIP_TYPE_AZERITE_ESSENCE: f64 = 8.0;
// Simulator policy for name and heading rows; their native line type is UNKNOWN.
const LINE_TYPE_ESSENCE_TEXT: f64 = 0.0;
const LINE_TYPE_SOURCE_DESCRIPTION: f64 = 0.0;
const LINE_TYPE_POWER_DESCRIPTION: f64 = 5.0;

pub(crate) fn register_essence(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAzeriteEssence",
        produce_azerite_essence_tooltip,
    )
}

pub(crate) fn register_slot(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAzeriteEssenceSlot",
        produce_azerite_essence_slot_tooltip,
    )
}

fn produce_azerite_essence_tooltip(state: &mut LuaState) -> LuaResult<u32> {
    let essence_id = tooltip_getter_args::round7_signed(state, 1, ESSENCE_USAGE)?;
    let requested_rank = tooltip_getter_args::round7_optional_unsigned(state, 2, ESSENCE_USAGE)?;
    if ACTIVE != ClientProfile::Retail || essence_id < 0 {
        return no_values();
    }
    let tooltip = compose_tooltip(
        state,
        TOOLTIP_TYPE_AZERITE_ESSENCE,
        Some(f64::from(essence_id)),
    );
    build_essence_tooltip(state, tooltip, essence_id, requested_rank)
}

fn build_essence_tooltip(
    state: &mut LuaState,
    tooltip: Val,
    essence_id: i32,
    requested_rank: Option<u32>,
) -> LuaResult<u32> {
    let Some(essence) = essence_record(essence_id) else {
        return no_values();
    };
    let Some((source_rank, rank)) = selected_ranks(state, essence_id, requested_rank) else {
        return no_values();
    };
    let Some(next_rank) = source_stage_for_rank(essence_id, source_rank) else {
        return no_values();
    };
    let Some((rank_one_power, selected_power)) = required_power_records(essence_id, rank) else {
        return no_values();
    };
    let (main_spell, passive_spell) =
        selected_essence_spell_ids(state, essence_id, rank_one_power, selected_power)?;
    let source_stage = source_stage_for_next_rank(essence_id, next_rank);
    publish_essence_tooltip(
        state,
        tooltip,
        Some((essence.name, rank)),
        Some(main_spell),
        passive_spell,
        source_stage,
    )
}

fn produce_azerite_essence_slot_tooltip(state: &mut LuaState) -> LuaResult<u32> {
    let slot = tooltip_getter_args::round7_signed(state, 1, ESSENCE_SLOT_USAGE)? as u8;
    if slot > MAX_ESSENCE_SLOT {
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
    let Some((_, rank)) = selected_ranks(state, essence_id, None) else {
        return no_values();
    };
    let tooltip = compose_tooltip(
        state,
        TOOLTIP_TYPE_AZERITE_ESSENCE,
        Some(f64::from(essence_id)),
    );
    let Some((main_spell, passive_spell)) = slot_spell_ids(essence_id, rank, slot) else {
        return no_values();
    };
    publish_essence_tooltip(state, tooltip, None, main_spell, passive_spell, None)
}

fn publish_essence_tooltip(
    state: &mut LuaState,
    tooltip: Val,
    heading: Option<(&str, u32)>,
    main_spell: Option<u32>,
    passive_spell: u32,
    source_stage: Option<(i32, u32)>,
) -> LuaResult<u32> {
    let saved_top = state.top;
    state.push(tooltip);
    let result = append_essence_tooltip_rows(
        state,
        tooltip,
        heading,
        main_spell,
        passive_spell,
        source_stage,
    );
    state.top = saved_top;
    if let Err(error) = result {
        return failed_producer(state, error);
    }
    state.push(tooltip);
    Ok(1)
}

fn append_essence_tooltip_rows(
    state: &mut LuaState,
    tooltip: Val,
    heading: Option<(&str, u32)>,
    main_spell: Option<u32>,
    passive_spell: u32,
    source_stage: Option<(i32, u32)>,
) -> LuaResult<()> {
    if let Some((name, rank)) = heading {
        let text = format_essence_heading(state, name, rank)?;
        let quality = (rank + 1) as u8;
        let color = item_quality_color(quality);
        append_line(state, tooltip, LINE_TYPE_ESSENCE_TEXT, &text, Some(color));
    }
    if let Some(spell_id) = main_spell {
        append_spell_to_tooltip(state, tooltip, spell_id)?;
    }
    append_spell_to_tooltip(state, tooltip, passive_spell)?;
    if let Some((essence_id, next_rank)) = source_stage {
        append_next_rank_rows(state, tooltip, essence_id, next_rank)?;
    }
    Ok(())
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

fn source_stage_for_rank(essence_id: i32, source_rank: u32) -> Option<u32> {
    let next_rank = source_rank.wrapping_add(1);
    if next_rank != SOURCE_STAGE_DISABLED_NEXT_RANK {
        power_record(essence_id, next_rank)?;
    }
    Some(next_rank)
}

fn source_stage_for_next_rank(essence_id: i32, next_rank: u32) -> Option<(i32, u32)> {
    (next_rank != SOURCE_STAGE_DISABLED_NEXT_RANK).then_some((essence_id, next_rank))
}

fn required_power_records(
    essence_id: i32,
    rank: u32,
) -> Option<(
    &'static AzeriteEssencePowerRecord,
    &'static AzeriteEssencePowerRecord,
)> {
    Some((
        power_record(essence_id, 1)?,
        power_record(essence_id, rank)?,
    ))
}

fn selected_essence_spell_ids(
    state: &LuaState,
    essence_id: i32,
    rank_one_power: &AzeriteEssencePowerRecord,
    selected_power: &AzeriteEssencePowerRecord,
) -> LuaResult<(u32, u32)> {
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
    Ok((main_spell, passive_spell))
}

fn slot_spell_ids(essence_id: i32, rank: u32, slot: u8) -> Option<(Option<u32>, u32)> {
    essence_record(essence_id)?;
    let (rank_one_power, _) = required_power_records(essence_id, rank)?;
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
    Some((main_spell, passive_spell))
}

fn selected_ranks(state: &LuaState, essence_id: i32, requested: Option<u32>) -> Option<(u32, u32)> {
    let source_rank = match requested {
        Some(rank) => rank,
        None => player_rank_or_list_miss(state, essence_id)?,
    };
    Some((source_rank, source_rank.max(1)))
}

fn player_rank_or_list_miss(state: &LuaState, essence_id: i32) -> Option<u32> {
    let sim = borrow_state(state).ok()?;
    let essences = &sim.azerite_essence.essences;
    if !sim.player.present || essences.is_empty() {
        return None;
    }
    let rank = essences.get(&essence_id).map_or(0, |essence| essence.rank);
    u32::try_from(rank).ok()
}

fn append_next_rank_rows(
    state: &mut LuaState,
    tooltip: Val,
    essence_id: i32,
    next_rank: u32,
) -> LuaResult<()> {
    let Some(power) = power_record(essence_id, next_rank) else {
        return Ok(());
    };
    let source = selected_source_text(state, power)?;
    if next_rank == 1 {
        if source.is_empty() {
            return Ok(());
        }
        append_source_heading(state, tooltip, next_rank)?;
        append_source_description(state, tooltip, &source);
        return Ok(());
    }
    append_source_heading(state, tooltip, next_rank)?;
    append_source_description(state, tooltip, &source);
    append_next_rank_power_rows(state, tooltip, essence_id, next_rank, power)
}

fn selected_source_text(state: &LuaState, power: &AzeriteEssencePowerRecord) -> LuaResult<String> {
    let sim = borrow_state(state)?;
    let source_override = i32::try_from(power.essence_id).ok().and_then(|id| {
        sim.azerite_essence
            .tooltip_source_overrides
            .get(&(id, power.rank))
    });
    let (first, second) = source_override
        .map(|(first, second)| (first.as_str(), second.as_str()))
        .unwrap_or((power.source_alliance, power.source_horde));
    let use_second = sim.player.present
        && !second.is_empty()
        && sim.azerite_essence.source_faction_selector_result == Some(0);
    Ok(if use_second { second } else { first }.to_owned())
}

fn append_source_description(state: &mut LuaState, tooltip: Val, source: &str) {
    let line = append_line(state, tooltip, LINE_TYPE_SOURCE_DESCRIPTION, source, None);
    table_set(state, line, "wrapText", Val::Bool(true));
}

fn append_next_rank_power_rows(
    state: &mut LuaState,
    tooltip: Val,
    essence_id: i32,
    next_rank: u32,
    power: &AzeriteEssencePowerRecord,
) -> LuaResult<()> {
    if !player_owns_essence(state, essence_id)? {
        return Ok(());
    }
    let (power_state, heading_key) = major_power_row_style(next_rank);
    append_inline_power_row(
        state,
        tooltip,
        power.major_power_description,
        heading_key,
        power_state,
    )?;
    if next_rank < SOURCE_STAGE_DISABLED_NEXT_RANK {
        append_inline_power_row(
            state,
            tooltip,
            power.minor_power_description,
            Some("AZERITE_ESSENCE_TOOLTIP_PASSIVE_POWER_INLINE"),
            "minor",
        )?;
    }
    Ok(())
}

fn major_power_row_style(next_rank: u32) -> (&'static str, Option<&'static str>) {
    if next_rank < SOURCE_STAGE_DISABLED_NEXT_RANK {
        ("major", Some("AZERITE_ESSENCE_TOOLTIP_MAIN_POWER_INLINE"))
    } else {
        ("gray", None)
    }
}

fn player_owns_essence(state: &LuaState, essence_id: i32) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(sim.player.present
        && sim
            .azerite_essence
            .essences
            .get(&essence_id)
            .is_some_and(|essence| essence.unlocked))
}

fn append_inline_power_row(
    state: &mut LuaState,
    tooltip: Val,
    spell_id: u32,
    heading_key: Option<&str>,
    power_state: &str,
) -> LuaResult<()> {
    if crate::spell_descriptions::get_spell_description(spell_id).is_none() {
        return Ok(());
    }
    let description = spell_description_text(state, spell_id);
    if description.is_empty() {
        return Ok(());
    }
    let text = match heading_key {
        Some(key) => format_global_text(state, key, &description)?,
        None => description,
    };
    let line = append_line(state, tooltip, LINE_TYPE_POWER_DESCRIPTION, &text, None);
    table_set(state, line, "wrapText", Val::Bool(true));
    let power_state = create_string(state, power_state);
    table_set(state, line, "powerState", power_state);
    Ok(())
}

fn append_source_heading(state: &mut LuaState, tooltip: Val, next_rank: u32) -> LuaResult<()> {
    let heading_key = if next_rank == 1 {
        "SOURCE"
    } else {
        "NEXT_RANK_COLON"
    };
    let heading = localized_source_heading(state, heading_key)?;
    append_line(state, tooltip, LINE_TYPE_ESSENCE_TEXT, &heading, None);
    Ok(())
}

fn localized_source_heading(state: &mut LuaState, key: &str) -> LuaResult<String> {
    let value = crate::c_api::global_val(state, key);
    val_to_string(state, value).ok_or_else(|| {
        rilua::runtime_error(format!(
            "localized Azerite source heading {key} returned no text"
        ))
    })
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
