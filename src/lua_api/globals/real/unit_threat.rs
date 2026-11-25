use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

#[derive(Clone, Copy, Debug)]
pub struct ThreatEntry {
    pub is_tanking: bool,
    pub status: u8,
    pub scaled_percent: f64,
    pub raw_percent: f64,
    pub threat_value: f64,
    pub percentage_of_lead: f64,
}

pub(crate) fn register_api(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "UnitDetailedThreatSituation", detailed)?;
    LuaApiMut::register_function(lua, "UnitThreatPercentageOfLead", percentage_of_lead)?;
    LuaApiMut::register_function(lua, "UnitThreatSituation", situation)?;
    LuaApiMut::register_function(lua, "IsThreatWarningEnabled", is_threat_warning_enabled)?;
    LuaApiMut::register_function(
        lua,
        "UnitHasEffectivelyTankAura",
        unit_has_effectively_tank_aura,
    )?;
    Ok(())
}

fn detailed(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let mob = String::from_stack(state, 2)?;
    let entry = borrow_state(state)?.unit_threats.get(&(unit, mob)).copied();
    let Some(entry) = entry else { return Ok(0) };
    state.push(Val::Bool(entry.is_tanking));
    state.push(Val::Num(entry.status as f64));
    state.push(Val::Num(entry.scaled_percent));
    state.push(Val::Num(entry.raw_percent));
    state.push(Val::Num(entry.threat_value));
    Ok(5)
}

fn situation(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let mob = Option::<String>::from_stack(state, 2)?;
    let status = {
        let sim = borrow_state(state)?;
        sim.unit_threats
            .iter()
            .filter(|((candidate, target), _)| {
                candidate == &unit && mob.as_ref().is_none_or(|mob| mob == target)
            })
            .map(|(_, entry)| entry.status)
            .max()
    };
    if let Some(status) = status {
        state.push(Val::Num(status as f64));
    }
    Ok(u32::from(status.is_some()))
}

fn percentage_of_lead(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let mob = String::from_stack(state, 2)?;
    let entry = borrow_state(state)?.unit_threats.get(&(unit, mob)).copied();
    let Some(entry) = entry else { return Ok(0) };
    state.push(Val::Num(entry.percentage_of_lead));
    Ok(1)
}

// INFERRED (R88dz (native evidence pending)): absent host input means no effective tank aura.
fn unit_has_effectively_tank_aura(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?;
    let active = match unit {
        Some(unit) => borrow_state(state)?
            .unit_effectively_tank_auras
            .get(&unit)
            .copied()
            .unwrap_or(false),
        None => false,
    };
    state.push(Val::Bool(active));
    Ok(1)
}

// INFERRED (R88dz (native evidence pending)): threatWarning gates threat UI by group and instance state.
fn is_threat_warning_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = {
        let sim = borrow_state(state)?;
        let in_instance = sim.world.in_instance;
        let in_group = sim.party_group_active;
        match sim
            .cvars
            .cached_integer("threatWarning")
            .unwrap_or_default()
        {
            1 => in_instance,
            2 => in_group || in_instance,
            3 => true,
            _ => false,
        }
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_threat(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let mob = String::from_stack(state, 2)?;
    let status = i32::from_stack(state, 3)?;
    let scaled_percent = f64::from_stack(state, 4)?;
    let raw_percent = f64::from_stack(state, 5)?;
    let threat_value = f64::from_stack(state, 6)?;
    let is_tanking = bool::from_stack(state, 7)?;
    let percentage_of_lead = f64::from_stack(state, 8)?;
    if !(0..=3).contains(&status) {
        return Err(rilua::LuaError::Runtime(rilua::RuntimeError {
            message: "threat status must be between 0 and 3".into(),
            level: 1,
            traceback: vec![],
        }));
    }
    borrow_state_mut(state)?.unit_threats.insert(
        (unit, mob),
        ThreatEntry {
            is_tanking,
            status: status as u8,
            scaled_percent,
            raw_percent,
            threat_value,
            percentage_of_lead,
        },
    );
    Ok(0)
}

fn set_effectively_tank_aura(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let active = bool::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .unit_effectively_tank_auras
        .insert(unit, active);
    Ok(0)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetUnitThreatSituation", set_threat)?
        .set_function("SetUnitEffectivelyTankAura", set_effectively_tank_aura)
}
