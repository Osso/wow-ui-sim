//! Bounty set maps projected only from explicitly supplied simulator state.
use super::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set_static, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !cfg!(any(feature = "client-retail", feature = "client-ptr")) {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_Map")?;
    table_set_rust_fn_static(state, namespace, "GetBountySetMaps", query_bounty_set_maps)
}

pub(super) fn register_quest_queries(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        let namespace = ensure_namespace(state, "C_QuestLog")?;
        // Retail QuestLogDocumentation.lua:125,692.
        table_set_rust_fn_static(state, namespace, "GetBountiesForMapID", query_map_bounties)?;
        table_set_rust_fn_static(state, namespace, "HasActiveThreats", query_active_threats)?;
    }
    Ok(())
}

fn query_map_bounties(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value).and_then(|s| s.trim().parse::<f64>().ok()),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error("GetBountiesForMapID requires a map ID"))?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(rilua::runtime_error(
            "GetBountiesForMapID map ID out of range",
        ));
    }
    let map_id = number as i32;
    let bounties = {
        let sim = borrow_state(state)?;
        let mut sets = sim
            .bounty_set_maps
            .iter()
            .filter(|(_, maps)| maps.contains(&map_id));
        let selected = sets.next().map(|(id, _)| *id);
        if sets.next().is_some() {
            return Err(rilua::runtime_error(
                "map belongs to multiple supplied bounty sets",
            ));
        }
        selected.map(|id| {
            sim.quest_log_entries
                .bounties_by_set
                .get(&id)
                .cloned()
                .unwrap_or_default()
        })
    };
    let Some(bounties) = bounties else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let result = create_table(state);
    state.push(result);
    for (index, bounty) in bounties.into_iter().enumerate() {
        let row = create_table(state);
        table_set_static(state, row, "questID", Val::Num(f64::from(bounty.quest_id)));
        table_set_static(
            state,
            row,
            "factionID",
            Val::Num(f64::from(bounty.faction_id)),
        );
        if let Some(icon) = bounty.icon {
            table_set_static(state, row, "icon", Val::Num(f64::from(icon)));
        }
        table_set_static(
            state,
            row,
            "numObjectives",
            Val::Num(f64::from(bounty.num_objectives)),
        );
        if let Some(text) = bounty.turnin_requirement_text {
            let text = create_string(state, &text);
            table_set_static(state, row, "turninRequirementText", text);
        }
        set_table_array(state, result, index as i64 + 1, row);
    }
    Ok(1)
}

fn query_active_threats(state: &mut LuaState) -> LuaResult<u32> {
    const EXCLUDED_FLAGS: u16 = 32 | 2048;
    const THREAT_MASK: u16 = 10;
    const THREAT_VALUE: u16 = 8;
    let active = {
        let sim = borrow_state(state)?;
        let log = &sim.quest_log_entries;
        log.entries.iter().any(|entry| {
            let flags = log
                .threat_entry_flags
                .get(&entry.quest_id)
                .copied()
                .unwrap_or(0);
            flags & EXCLUDED_FLAGS == 0 && flags & THREAT_MASK == THREAT_VALUE
        })
    };
    state.push(Val::Bool(active));
    Ok(1)
}

fn validate_bounty_set_id(value: Val) -> LuaResult<Option<i32>> {
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(
            "C_Map.GetBountySetMaps: bountySetID must be a number",
        ));
    };
    let in_range = (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number);
    let represented = number.is_finite() && number.fract() == 0.0 && in_range;
    Ok(represented.then(|| number as i32))
}

fn query_bounty_set_maps(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    // Authenticate before validation or lookup, preserving wrappers and taint.
    #[cfg(feature = "retail-12-1-0")]
    let value = rilua::table_security::unwrap_secret(state, value)?;
    let bounty_set_id = validate_bounty_set_id(value)?;
    let maps = {
        let sim = borrow_state(state)?;
        bounty_set_id
            .and_then(|id| sim.bounty_set_maps.get(&id))
            .cloned()
            .unwrap_or_default()
    };
    let result = create_table(state);
    state.push(result);
    for (index, id) in maps.into_iter().enumerate() {
        set_table_array(state, result, index as i64 + 1, Val::Num(f64::from(id)));
    }
    Ok(1)
}
