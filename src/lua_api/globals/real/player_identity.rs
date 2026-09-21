//! Player identity globals backed by `SimState`.

use crate::lua_api::game_data::{RACE_DATA, class_info_by_index};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const SIM_REALM: &str = "SimRealm";

fn get_player_info_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?;
    if guid.as_deref() != Some(SEEDED_LOCAL_CHARACTER_GUID) {
        state.push(Val::Nil);
        return Ok(1);
    }

    let (localized_class, english_class, localized_race, english_race, sex, name) = {
        let sim = borrow_state(state)?;
        let (localized_class, english_class, _) = class_info_by_index(sim.player.class_index);
        let race_index = sim.player.race_index.min(RACE_DATA.len().saturating_sub(1));
        let (localized_race, english_race, _) = RACE_DATA[race_index];
        (
            localized_class,
            english_class,
            localized_race,
            english_race,
            sim.player.sex,
            sim.player.name.clone(),
        )
    };

    let localized_class = create_string(state, localized_class);
    let english_class = create_string(state, english_class);
    let localized_race = create_string(state, localized_race);
    let english_race = create_string(state, english_race);
    let name = create_string(state, &name);
    let realm = create_string(state, SIM_REALM);

    state.push(localized_class);
    state.push(english_class);
    state.push(localized_race);
    state.push(english_race);
    state.push(Val::Num(sex as f64));
    state.push(name);
    state.push(realm);
    Ok(7)
}

#[cfg(feature = "player-cast-durations")]
fn unit_name_from_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    if guid != SEEDED_LOCAL_CHARACTER_GUID {
        return Ok(0);
    }
    let name = borrow_state(state)?.player.name.clone();
    let name = create_string(state, &name);
    let realm = create_string(state, SIM_REALM);
    state.push(name);
    state.push(realm);
    Ok(2)
}

#[cfg(feature = "player-cast-durations")]
fn unit_class_from_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    if guid != SEEDED_LOCAL_CHARACTER_GUID {
        return Ok(0);
    }
    let (localized, token, id) = class_info_by_index(borrow_state(state)?.player.class_index);
    let localized = create_string(state, localized);
    let token = create_string(state, token);
    state.push(localized);
    state.push(token);
    state.push(Val::Num(id as f64));
    Ok(3)
}

#[cfg(feature = "client-wowforever")]
fn regional_unique_names_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.player.regional_unique_names_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn can_port_graveyard(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.player.can_port_graveyard;
    state.push(Val::Bool(available));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(feature = "client-wowforever")]
    LuaApiMut::register_function(lua, "CanPortGraveyard", can_port_graveyard)?;
    #[cfg(feature = "client-wowforever")]
    LuaApiMut::register_function(
        lua,
        "RegionalUniqueNamesEnabled",
        regional_unique_names_enabled,
    )?;
    LuaApiMut::register_function(lua, "GetPlayerInfoByGUID", get_player_info_by_guid)?;
    #[cfg(feature = "player-cast-durations")]
    LuaApiMut::register_function(lua, "UnitNameFromGUID", unit_name_from_guid)?;
    #[cfg(feature = "player-cast-durations")]
    LuaApiMut::register_function(lua, "UnitClassFromGUID", unit_class_from_guid)?;
    Ok(())
}
