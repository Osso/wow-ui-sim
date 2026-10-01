//! 12.0.5 input-first contract; actual RED and query implementation are parent-owned.
//! One explicit nil result and actual-cast-only scope are documented in the spec
//! as inferences, not native-verified behavior.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::api::state_is_secure;
use rilua::table_security::{is_secret_value, unwrap_secret, wrap_host_secret_string};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{CastTargetSnapshot, CastingState, TargetInfo};

const TARGET_NAME: &str = "Cast Recipient";

fn target_snapshot(is_player: bool) -> CastTargetSnapshot {
    CastTargetSnapshot {
        guid: if is_player {
            "Player-1-00000001"
        } else {
            "Creature-0-0-0-0-123-1"
        }
        .into(),
        name: TARGET_NAME.into(),
        is_player,
    }
}

fn cast(target: Option<CastTargetSnapshot>) -> CastingState {
    CastingState {
        spell_id: 19750,
        spell_name: "Flash of Light".into(),
        icon_path: String::new(),
        start_time: 0.0,
        end_time: 3600.0,
        cast_id: 1,
        empower: None,
        delay_time: 0.0,
        target,
    }
}

fn player_target_cast() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("cast target environment");
    env.state().borrow_mut().casting = Some(cast(Some(target_snapshot(true))));
    env
}

fn assert_nil_result(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(type(UnitSpellTargetName) == 'function', 'actual query must be registered')
        assert(select('#', UnitSpellTargetName('player')) == 1, 'inferred one nil result')
        assert(UnitSpellTargetName('player') == nil, 'absent player cast target name')
        "#,
    )
    .expect("documented absence with inferred explicit nil arity");
}

fn assert_secret_name(env: &WowLuaEnv) {
    // Inspect only after the addon frame has returned; never clear taint to read.
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let result = lua.get_global_val("CastTargetNameResult");
    let state = lua.state_mut();
    assert!(state_is_secure(state), "secure host inspection required");
    assert!(
        is_secret_value(state, result),
        "target name must stay opaque"
    );
    let Val::Str(string_ref) = unwrap_secret(state, result).expect("guarded secure inspection")
    else {
        panic!("target name payload must be a string");
    };
    let bytes = state
        .gc
        .string_arena
        .get(string_ref)
        .expect("rooted result")
        .data();
    assert_eq!(bytes, TARGET_NAME.as_bytes());
}

fn query_player_name(env: &WowLuaEnv) {
    env.exec(
        "assert(select('#', UnitSpellTargetName('player')) == 1); \
         CastTargetNameResult = UnitSpellTargetName('player')",
    )
    .expect("actual cast target query");
    assert_secret_name(env);
}

fn inject_secret_unit(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let secret = wrap_host_secret_string(lua.state_mut(), "player");
    lua.state_mut().push(secret);
    let inserted = lua.set_global_val("SecretCasterUnit", secret);
    lua.state_mut().pop();
    inserted.expect("root secret unit token");
}

#[test]
fn no_cast_returns_one_nil() {
    assert_nil_result(&WowLuaEnv::new().expect("idle environment"));
}

#[test]
fn cast_without_explicit_target_returns_one_nil() {
    let env = WowLuaEnv::new().expect("untargeted cast environment");
    env.state().borrow_mut().casting = Some(cast(None));
    assert_nil_result(&env);
}

#[test]
fn explicit_npc_cast_target_returns_one_nil() {
    let env = WowLuaEnv::new().expect("NPC target environment");
    env.state().borrow_mut().casting = Some(cast(Some(target_snapshot(false))));
    assert_nil_result(&env);
}

#[test]
fn explicit_player_cast_target_returns_exact_secret_name() {
    query_player_name(&player_target_cast());
}

#[test]
fn selected_target_changes_do_not_reinterpret_cast_snapshot() {
    let env = player_target_cast();
    query_player_name(&env);
    env.state().borrow_mut().current_target = Some(TargetInfo {
        unit_id: "target".into(),
        name: "Selected Stranger".into(),
        class_index: 1,
        level: 60,
        health: 100,
        health_max: 100,
        power: 0,
        power_max: 0,
        power_type: 1,
        power_type_name: "RAGE".into(),
        is_player: true,
        is_enemy: false,
        guid: "Player-1-00000002".into(),
        classification: "normal".into(),
        creature_type: "Humanoid".into(),
        reaction: 5,
        interaction: Default::default(),
    });
    query_player_name(&env);
    env.state().borrow_mut().casting = Some(cast(None));
    assert_nil_result(&env);
    env.state().borrow_mut().casting = Some(cast(Some(target_snapshot(true))));
    env.state().borrow_mut().current_target = None;
    query_player_name(&env);
}

#[test]
fn clearing_or_replacing_cast_removes_previous_target() {
    let env = player_target_cast();
    query_player_name(&env);
    env.state().borrow_mut().casting = None;
    assert_nil_result(&env);
    env.state().borrow_mut().casting = Some(cast(Some(target_snapshot(true))));
    query_player_name(&env);
    env.state().borrow_mut().casting = Some(cast(None));
    assert_nil_result(&env);
}

#[test]
fn channel_only_does_not_supply_actual_cast_target() {
    let env = WowLuaEnv::new().expect("channel-only environment");
    env.state().borrow_mut().channeling = Some(cast(Some(target_snapshot(true))));
    assert_nil_result(&env);
}

#[test]
fn tainted_public_unit_gets_opaque_name_without_changing_taint() {
    let env = player_target_cast();
    env.exec(
        r#"
        debug.settaintmode(true)
        local function addon()
            assert(debug.getstacktaint() == 'CastTargetProbe')
            assert(select('#', UnitSpellTargetName('player')) == 1)
            local result = UnitSpellTargetName('player')
            assert(debug.getstacktaint() == 'CastTargetProbe')
            assert(issecretvalue(result) and not canaccessvalue(result))
            assert(not pcall(secretunwrap, result), 'addon cannot unwrap target name')
            assert(debug.getstacktaint() == 'CastTargetProbe')
            return result
        end
        debug.setobjecttaint(addon, 'CastTargetProbe')
        CastTargetNameResult = addon()
        assert(debug.getstacktaint() == nil, 'caller restored after addon returns')
        collectgarbage('collect')
        "#,
    )
    .expect("public unit query preserves taint and opaque output");
    assert_secret_name(&env);
}

#[test]
fn untainted_secret_unit_resolves_actual_cast_target() {
    let env = player_target_cast();
    inject_secret_unit(&env);
    env.exec(
        "debug.settaintmode(true); assert(debug.getstacktaint() == nil); \
         assert(select('#', UnitSpellTargetName(SecretCasterUnit)) == 1); \
         CastTargetNameResult = UnitSpellTargetName(SecretCasterUnit); \
         assert(debug.getstacktaint() == nil); collectgarbage('collect')",
    )
    .expect("AllowedWhenUntainted secret input");
    assert_secret_name(&env);
}

#[test]
fn tainted_secret_unit_is_rejected_without_changing_taint() {
    let env = player_target_cast();
    inject_secret_unit(&env);
    env.exec(
        r#"
        debug.settaintmode(true)
        assert(type(UnitSpellTargetName) == 'function', 'rejection must come from actual query')
        local function addon()
            assert(debug.getstacktaint() == 'CastTargetProbe')
            assert(not pcall(secretunwrap, SecretCasterUnit), 'input stays opaque')
            assert(not pcall(UnitSpellTargetName, SecretCasterUnit), 'secret unit rejected')
            assert(debug.getstacktaint() == 'CastTargetProbe')
        end
        debug.setobjecttaint(addon, 'CastTargetProbe')
        addon()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("existing unwrap_secret caller policy rejects tainted secret input");
}
