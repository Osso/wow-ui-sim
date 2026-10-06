#![cfg(feature = "retail-12-0-5")]
use rilua::{LuaApi, LuaApiMut};
use wow_ui_sim::lua_api::{WowLuaEnv, state::{CastTargetSnapshot, CastingState}};
use wow_ui_sim::lua_api::globals::real::totems::Totem;

#[test]
fn patch_12_0_1_totem_slots_and_duration_share_info_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(GetNumTotemSlots() == 4)
        assert(GetTotemDuration(1):GetTotalDuration() == 0)
        assert(GetTotemDuration(1):GetRemainingDuration() == 0)
    "#).unwrap();
    env.state().borrow_mut().totem_slots[0] = Some(Totem {
        name: "Healing Stream".into(), start_time: 0.0, duration: 3600.0, icon: 135127,
    });
    env.state().borrow_mut().totem_slots[1] = Some(Totem {
        name: "Expired".into(), start_time: -20.0, duration: 1.0, icon: 135128,
    });
    env.exec(r#"
        local active, name, start, duration, icon = GetTotemInfo(1)
        assert(active and name == 'Healing Stream' and start == 0 and duration == 3600 and icon == 135127)
        assert(GetTotemDuration(1):GetTotalDuration() == 3600)
        assert(GetTotemDuration(1):GetRemainingDuration() > 3500)
        assert(GetTotemInfo(2) == false)
        assert(GetTotemDuration(2):GetTotalDuration() == 0)
        assert(GetTotemInfo(99) == false)
    "#).unwrap();
}

fn assert_secret_target(env: &WowLuaEnv, unit: &str, expected: bool) {
    env.exec(&format!("assert(type(PlayerIsSpellTarget) == 'function'); SpellTargetResult = PlayerIsSpellTarget('{unit}')")).unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let value = lua.get_global_val("SpellTargetResult");
    assert!(rilua::table_security::is_secret_value(lua.state(), value));
    assert_eq!(rilua::table_security::unwrap_secret(lua.state(), value).unwrap(), rilua::Val::Bool(expected));
}

#[test]
fn patch_12_0_1_player_spell_target_is_a_secret_boolean() {
    let env = WowLuaEnv::new().unwrap();
    assert_secret_target(&env, "player", false);
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    env.state().borrow_mut().casting = Some(CastingState {
        spell_id: 19750, spell_name: "Flash of Light".into(), icon_path: String::new(),
        start_time: 0.0, duration: 3600.0, cast_id: 1, empower: None, delay_time: 0.0,
        target: Some(CastTargetSnapshot { guid, name: "Self".into(), is_player: true }),
    });
    assert_secret_target(&env, "player", true);
    assert_secret_target(&env, "missingunit", false);
    env.state().borrow_mut().casting.as_mut().unwrap().target.as_mut().unwrap().guid = "Player-Other".into();
    assert_secret_target(&env, "player", false);
}
