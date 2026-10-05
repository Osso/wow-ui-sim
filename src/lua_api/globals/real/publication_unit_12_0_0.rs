//! Retail role, cast-recipient and threat queries using explicit host snapshots.
//! INFERRED: unknown roles false, unknown lead snapshot nil; only player casts
//! are modeled. Native display policies and threat thresholds are not invented.

use super::publication_12_0_0::PlainGlobalInputs;
use crate::lua_api::globals::unit_misc::existing_guid_for_unit;
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::table_security::{unwrap_secret, wrap_host_secret_number, wrap_host_secret_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};
use std::collections::HashSet;

fn read_unit(state: &LuaState, index: i32, nilable: bool) -> LuaResult<Option<String>> {
    let value = unwrap_secret(state, stack_val(state, index))?;
    match value {
        Val::Nil if nilable => Ok(None),
        Val::Str(_) => val_to_string(state, value)
            .map(Some)
            .ok_or_else(|| runtime_error("unit token must be UTF-8")),
        _ => Err(runtime_error("unit token must be a string")),
    }
}

fn push_role(
    state: &mut LuaState,
    nilable: bool,
    select: fn(&PlainGlobalInputs) -> &HashSet<String>,
) -> LuaResult<u32> {
    let unit = read_unit(state, 1, nilable)?;
    let result = {
        let sim = borrow_state(state)?;
        unit.as_deref()
            .and_then(|unit| existing_guid_for_unit(&sim, unit))
            .is_some_and(|guid| select(&sim.plain_global_inputs).contains(&guid))
    };
    state.push(Val::Bool(result));
    Ok(1)
}

fn lieutenant(state: &mut LuaState) -> LuaResult<u32> {
    push_role(state, false, |inputs| &inputs.lieutenant_guids)
}

fn minion(state: &mut LuaState) -> LuaResult<u32> {
    push_role(state, false, |inputs| &inputs.minion_guids)
}

fn npc_as_player(state: &mut LuaState) -> LuaResult<u32> {
    push_role(state, true, |inputs| &inputs.npc_as_player_guids)
}

fn should_display_spell_target_name(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_unit(state, 1, false)?;
    // INFERRED display policy: an explicit target on the modeled player cast
    // is displayable. Channels and unsupported casters are not player casts.
    let display = unit.as_deref() == Some("player")
        && borrow_state(state)?
            .casting
            .as_ref()
            .is_some_and(|cast| cast.target.is_some());
    state.push(Val::Bool(display));
    Ok(1)
}

fn spell_target_class(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_unit(state, 1, false)?;
    let class = if unit.as_deref() == Some("player") {
        let sim = borrow_state(state)?;
        sim.casting
            .as_ref()
            .and_then(|cast| cast.target.as_ref())
            .and_then(|target| {
                sim.plain_global_inputs
                    .cast_target_classes
                    .get(&target.guid)
            })
            .cloned()
    } else {
        None
    };
    let result = match class {
        Some(class) => wrap_host_secret_string(state, &class),
        None => Val::Nil,
    };
    state.push(result);
    Ok(1)
}

fn threat_lead_situation(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_unit(state, 1, false)?.expect("required unit");
    let mob = read_unit(state, 2, false)?.expect("required mob token");
    let (snapshot, restricted) = {
        let sim = borrow_state(state)?;
        let pair = existing_guid_for_unit(&sim, &unit).zip(existing_guid_for_unit(&sim, &mob));
        let snapshot =
            pair.and_then(|pair| sim.plain_global_inputs.threat_lead.get(&pair).copied());
        (snapshot, sim.plain_global_inputs.threat_state_restricted)
    };
    let Some(snapshot) = snapshot else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let category = threat_lead_category(snapshot)?;
    let result = if restricted {
        wrap_host_secret_number(state, category)
    } else {
        Val::Num(category)
    };
    state.push(result);
    Ok(1)
}

fn threat_lead_category(snapshot: super::publication_12_0_0::ThreatLeadSnapshot) -> LuaResult<f64> {
    const RED: u8 = 3;
    if snapshot.lead_state > RED {
        return Err(runtime_error("threat lead category must be in 0..3"));
    }
    let category = if snapshot.is_first {
        snapshot.lead_state
    } else {
        RED
    };
    Ok(category as f64)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "UnitIsLieutenant", lieutenant)?;
    LuaApiMut::register_function(lua, "UnitIsMinion", minion)?;
    LuaApiMut::register_function(lua, "UnitIsNPCAsPlayer", npc_as_player)?;
    LuaApiMut::register_function(
        lua,
        "UnitShouldDisplaySpellTargetName",
        should_display_spell_target_name,
    )?;
    LuaApiMut::register_function(lua, "UnitSpellTargetClass", spell_target_class)?;
    LuaApiMut::register_function(lua, "UnitThreatLeadSituation", threat_lead_situation)
}
