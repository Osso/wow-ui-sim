//! Player identity globals backed by `SimState`.

use crate::lua_api::game_data::{RACE_DATA, class_info_by_index};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const SIM_REALM: &str = "SimRealm";

fn get_realm_name(state: &mut LuaState) -> LuaResult<u32> {
    let realm = borrow_state(state)?
        .player
        .realm_name
        .clone()
        .unwrap_or_else(|| "SimulatedRealm".to_string());
    let realm = create_string(state, &realm);
    state.push(realm);
    Ok(1)
}

fn get_normalized_realm_name(state: &mut LuaState) -> LuaResult<u32> {
    let realm = borrow_state(state)?
        .player
        .realm_name
        .as_deref()
        .unwrap_or("SimulatedRealm")
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '-')
        .collect::<String>();
    let realm = create_string(state, &realm);
    state.push(realm);
    Ok(1)
}

fn get_player_info_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?;
    if guid.as_deref() != Some(SEEDED_LOCAL_CHARACTER_GUID) {
        state.push(Val::Nil);
        return Ok(1);
    }

    let (localized_class, english_class, localized_race, english_race, sex, name, realm) = {
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
            sim.player.realm_name.clone(),
        )
    };

    let localized_class = create_string(state, localized_class);
    let english_class = create_string(state, english_class);
    let localized_race = create_string(state, localized_race);
    let english_race = create_string(state, english_race);
    let name = create_string(state, &name);
    let realm = create_string(state, realm.as_deref().unwrap_or(SIM_REALM));

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
    let (name, realm) = {
        let sim = borrow_state(state)?;
        (sim.player.name.clone(), sim.player.realm_name.clone())
    };
    let name = create_string(state, &name);
    let realm = create_string(state, realm.as_deref().unwrap_or(SIM_REALM));
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

fn regional_unique_names_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.player.regional_unique_names_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn can_port_graveyard(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.player.can_port_graveyard;
    state.push(Val::Bool(available));
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetRealmName", get_realm_name)?;
    LuaApiMut::register_function(lua, "GetNormalizedRealmName", get_normalized_realm_name)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "CanPortGraveyard", can_port_graveyard)?;
        LuaApiMut::register_function(
            lua,
            "RegionalUniqueNamesEnabled",
            regional_unique_names_enabled,
        )?;
    }
    LuaApiMut::register_function(lua, "GetPlayerInfoByGUID", get_player_info_by_guid)?;
    #[cfg(feature = "player-cast-durations")]
    LuaApiMut::register_function(lua, "UnitNameFromGUID", unit_name_from_guid)?;
    #[cfg(feature = "player-cast-durations")]
    LuaApiMut::register_function(lua, "UnitClassFromGUID", unit_class_from_guid)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn supplied_player_realm_reaches_name_and_guid_queries() {
        let env = WowLuaEnv::new().unwrap();
        let other = WowLuaEnv::new().unwrap();
        let defaults: (String, String, String) = env
            .eval(
                "local _, realm = UnitFullName('player'); \
                 return GetRealmName(), GetNormalizedRealmName(), realm",
            )
            .unwrap();
        {
            let mut state = env.state().borrow_mut();
            state.player.name = "Wörkspacé".into();
            state.player.realm_name = Some("Test Realm-One".into());
        }
        env.exec(
            r#"
            assert(GetRealmName() == 'Test Realm-One')
            assert(GetNormalizedRealmName() == 'TestRealmOne')
            assert(select('#', GetRealmName()) == 1)
            assert(select('#', GetNormalizedRealmName()) == 1)
            local name, realm = UnitFullName('player')
            assert(name == 'Wörkspacé' and realm == 'Test Realm-One')
            local guid = UnitGUID('player')
            local _, _, _, _, _, guidName, guidRealm = GetPlayerInfoByGUID(guid)
            assert(guidName == name and guidRealm == realm)
            assert(select('#', GetPlayerInfoByGUID(guid)) == 7)
            "#,
        )
        .unwrap();
        #[cfg(feature = "player-cast-durations")]
        env.exec(
            "local name, realm = UnitNameFromGUID(UnitGUID('player')); \
             assert(name == 'Wörkspacé' and realm == 'Test Realm-One'); \
             assert(select('#', UnitNameFromGUID(UnitGUID('player'))) == 2)",
        )
        .unwrap();
        assert_eq!(
            other
                .eval::<(String, String, String)>(
                    "local _, realm = UnitFullName('player'); \
                     return GetRealmName(), GetNormalizedRealmName(), realm",
                )
                .unwrap(),
            defaults
        );
        env.state().borrow_mut().player.realm_name = None;
        assert_eq!(
            env.eval::<(String, String, String)>(
                "local _, realm = UnitFullName('player'); \
                 return GetRealmName(), GetNormalizedRealmName(), realm",
            )
            .unwrap(),
            defaults
        );
    }
}
