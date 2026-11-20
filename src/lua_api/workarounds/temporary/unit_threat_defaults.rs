//! Threat APIs backed by explicitly supplied simulator state.

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::{FromStack, TableBuilder};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::vm::state::LuaState;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::{LuaApiMut, LuaResult, Val};
#[derive(Clone, Copy, Debug)]
pub struct ThreatEntry {
    pub status: u8,
    pub scaled_percent: f64,
    pub raw_percent: f64,
    pub threat_value: f64,
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_api(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "UnitDetailedThreatSituation", detailed)?;
    LuaApiMut::register_function(lua, "UnitThreatPercentageOfLead", percentage_of_lead)?;
    LuaApiMut::register_function(lua, "UnitThreatSituation", situation)?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn detailed(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let mob = String::from_stack(state, 2)?;
    let entry = borrow_state(state)?.unit_threats.get(&(unit, mob)).copied();
    let Some(entry) = entry else { return Ok(0) };
    state.push(Val::Bool(entry.status >= 2));
    state.push(Val::Num(entry.status as f64));
    state.push(Val::Num(entry.scaled_percent));
    state.push(Val::Num(entry.raw_percent));
    state.push(Val::Num(entry.threat_value));
    Ok(5)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
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

// INFERRED (R88cg): the threat lead is the highest threat value recorded for this mob.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn percentage_of_lead(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let mob = String::from_stack(state, 2)?;
    let percentage = {
        let sim = borrow_state(state)?;
        let Some(entry) = sim.unit_threats.get(&(unit, mob.clone())) else {
            return Ok(0);
        };
        let lead = sim
            .unit_threats
            .iter()
            .filter(|((_, target), _)| target == &mob)
            .map(|(_, entry)| entry.threat_value)
            .fold(0.0, f64::max);
        if lead > 0.0 {
            entry.threat_value * 100.0 / lead
        } else {
            0.0
        }
    };
    state.push(Val::Num(percentage));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_threat(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let mob = String::from_stack(state, 2)?;
    let status = i32::from_stack(state, 3)?;
    let scaled_percent = f64::from_stack(state, 4)?;
    let raw_percent = f64::from_stack(state, 5)?;
    let threat_value = f64::from_stack(state, 6)?;
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
            status: status as u8,
            scaled_percent,
            raw_percent,
            threat_value,
        },
    );
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetUnitThreatSituation", set_threat)
}

const UNIT_THREAT_DEFAULTS_LUA: &str = r#"
if UnitDetailedThreatSituation == nil then
  function UnitDetailedThreatSituation(_unit, _other_unit) return false, 0, 0, 0, 0 end
end
if UnitThreatPercentageOfLead == nil then
  function UnitThreatPercentageOfLead(_unit, _other_unit) return 0 end
end
if GetThreatStatusColor == nil then
  local colors = { [0] = { 0.69, 0.69, 0.69 }, [1] = { 1.00, 1.00, 0.47 }, [2] = { 1.00, 0.60, 0.00 }, [3] = { 1.00, 0.00, 0.00 } }
  function GetThreatStatusColor(status)
    local color = colors[status]
    if color == nil then return nil end
    return color[1], color[2], color[3]
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(UNIT_THREAT_DEFAULTS_LUA)?;
    Ok(())
}
