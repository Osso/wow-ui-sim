//! Bounded Retail 12.0.5 `C_UnitAuras.DoesAuraHaveExpirationTime` inputs, not native parity proof.
//! Miss, nil-unit and strict representation results are inferred simulator policy.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn fixture_aura(id: i32, helpful: bool, expiration_time: f64) -> AuraInfo {
    AuraInfo {
        name: format!("Expiration fixture {id}"),
        spell_id: 98000 + id,
        icon: 134973,
        duration: if expiration_time == 0.0 { 0.0 } else { 30.0 },
        expiration_time,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: helpful,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: id,
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create aura expiration environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            fixture_aura(301, true, 45.0),
            fixture_aura(302, true, 0.0),
            fixture_aura(303, false, 60.0),
            fixture_aura(304, false, 0.0),
        ];
        let party = state
            .party_members
            .first_mut()
            .expect("existing seeded roster");
        party.buffs = vec![fixture_aura(401, true, 0.0)];
        party.debuffs = vec![fixture_aura(402, false, 75.0)];
    }
    env.exec(
        r#"
        function AssertExpires(unit, id, expected)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one query result')
                local value = ...
                assert(type(value) == 'boolean', 'boolean result')
                assert(not issecretvalue(value), 'public result')
                assert(value == expected, 'expiration result mismatch')
            end
            check(C_UnitAuras.DoesAuraHaveExpirationTime(unit, id))
        end
        function RejectExpires(...)
            assert(type(C_UnitAuras.DoesAuraHaveExpirationTime) == 'function',
                'real query must be registered before testing rejection')
            local ok, err = pcall(C_UnitAuras.DoesAuraHaveExpirationTime, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
        end
        "#,
    )
    .expect("install assertions without replacing the query");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("install VM security helpers");
    for (name, text) in [
        ("SecretExpirationUnit", "player"),
        ("SecretUnknownExpirationUnit", "missing-unit"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret STRING");
    }
    for (name, number) in [
        ("SecretTimedID", 301.0),
        ("SecretPermanentID", 302.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret NUMBER");
    }
}

#[test]
fn timed_and_permanent_player_auras_return_exactly_one_public_boolean() {
    let env = fixture_env();
    env.exec(
        r#"
        AssertExpires('player', 301, true)
        AssertExpires('player', 302, false)
        AssertExpires('player', 303, true)
        AssertExpires('player', 304, false)
        "#,
    )
    .expect("stored expiration time decides the result for helpful and harmful records");
}

#[test]
fn party_records_are_unit_and_instance_isolated() {
    let env = fixture_env();
    env.exec(
        r#"
        AssertExpires('party1', 401, false)
        AssertExpires('party1', 402, true)
        AssertExpires('party1', 301, false)
        AssertExpires('player', 402, false)
        "#,
    )
    .expect("party buffs and debuffs resolve only on their own unit");
}

#[test]
fn stored_expiration_changes_are_read_live() {
    let env = fixture_env();
    env.exec("AssertExpires('player', 302, false)")
        .expect("permanent before mutation");
    env.state().borrow_mut().player.buffs[1].expiration_time = 90.0;
    env.exec("AssertExpires('player', 302, true)")
        .expect("timed after mutation");
    env.state().borrow_mut().player.buffs[1].expiration_time = 0.0;
    env.exec("AssertExpires('player', 302, false)")
        .expect("permanent again after reset");
}

#[test]
fn unknown_units_instances_and_nil_unit_return_false() {
    let env = fixture_env();
    env.exec(
        r#"
        AssertExpires('player', 999, false)
        AssertExpires('player', -301, false)
        AssertExpires('missing-unit', 301, false)
        AssertExpires(nil, 301, false)
        "#,
    )
    .expect("inferred miss policy: one public false");
}

#[test]
fn malformed_arguments_error_before_lookup() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectExpires(12, 301)
        RejectExpires({}, 301)
        RejectExpires(true, 301)
        RejectExpires('player')
        RejectExpires('player', nil)
        RejectExpires('player', '301')
        RejectExpires('player', 0/0)
        RejectExpires('player', math.huge)
        RejectExpires('missing-unit', 'not-a-number')
        AssertExpires('player', 301, true)
        "#,
    )
    .expect("strict representation errors, then ordinary recovery");
}

#[test]
fn untainted_query_accepts_each_authentic_secret_argument_and_combination() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretExpirationUnit) and secretunwrap(SecretExpirationUnit) == 'player')
        assert(issecretvalue(SecretTimedID) and secretunwrap(SecretTimedID) == 301)
        assert(issecretvalue(SecretPermanentID) and secretunwrap(SecretPermanentID) == 302)
        AssertExpires(SecretExpirationUnit, 301, true)
        AssertExpires(SecretExpirationUnit, 302, false)
        AssertExpires('player', SecretTimedID, true)
        AssertExpires('player', SecretPermanentID, false)
        AssertExpires(SecretExpirationUnit, SecretTimedID, true)
        AssertExpires(SecretExpirationUnit, SecretPermanentID, false)
        AssertExpires(SecretUnknownExpirationUnit, SecretTimedID, false)
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretExpirationUnit) and issecretvalue(SecretUnknownExpirationUnit))
        assert(issecretvalue(SecretTimedID) and issecretvalue(SecretPermanentID))
        AssertExpires('player', 301, true)
        "#,
    )
    .expect("AllowedWhenUntainted authenticates secrets; does not declassify inputs");
}

#[test]
fn tainted_query_denies_each_secret_even_before_unknown_unit_lookup() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'AuraExpirationProbe')
            RejectExpires(SecretExpirationUnit, 301)
            RejectExpires('player', SecretTimedID)
            RejectExpires(SecretExpirationUnit, SecretTimedID)
            RejectExpires(SecretUnknownExpirationUnit, 301)
            RejectExpires('missing-unit', SecretTimedID)
            assert(debug.getstacktaint() == before)
            for _, value in ipairs({SecretExpirationUnit, SecretTimedID}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
            AssertExpires('player', 301, true)
            AssertExpires('player', 302, false)
            AssertExpires('missing-unit', 301, false)
            assert(debug.getstacktaint() == before, 'public recovery must retain taint')
        end
        debug.setobjecttaint(probe, 'AuraExpirationProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertExpires(SecretExpirationUnit, SecretTimedID, true)
        "#,
    )
    .expect("VM-authenticated denial before lookup, unchanged taint and public recovery");
}

#[test]
fn secret_arguments_survive_gc_without_declassification() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local unit, id = SecretExpirationUnit, SecretTimedID
        collectgarbage('collect')
        collectgarbage('collect')
        assert(rawequal(unit, SecretExpirationUnit))
        assert(issecretvalue(unit) and secretunwrap(unit) == 'player')
        assert(issecretvalue(id) and secretunwrap(id) == 301)
        AssertExpires(unit, id, true)
        local function probe()
            assert(debug.getstacktaint() == 'AuraExpirationGCProbe')
            RejectExpires(unit, 301)
            RejectExpires('player', id)
            assert(issecretvalue(unit) and not pcall(secretunwrap, unit))
            assert(issecretvalue(id) and not pcall(secretunwrap, id))
            AssertExpires('player', 301, true)
            assert(debug.getstacktaint() == 'AuraExpirationGCProbe')
        end
        debug.setobjecttaint(probe, 'AuraExpirationGCProbe')
        probe()
        collectgarbage('collect')
        assert(debug.getstacktaint() == nil)
        AssertExpires(unit, id, true)
        assert(issecretvalue(unit) and issecretvalue(id))
        "#,
    )
    .expect("rooted real secrets survive GC and stay secret");
}

#[test]
fn queries_leave_aura_records_unchanged_and_environments_isolated() {
    let env = fixture_env();
    let other = WowLuaEnv::new().expect("second environment");
    let before: Vec<(i32, f64)> = env
        .state()
        .borrow()
        .player
        .buffs
        .iter()
        .map(|aura| (aura.aura_instance_id, aura.expiration_time))
        .collect();
    env.exec(
        r#"
        for _ = 1, 3 do
            AssertExpires('player', 301, true)
            AssertExpires('player', 302, false)
            AssertExpires('party1', 402, true)
        end
        "#,
    )
    .expect("repeated reads");
    let after: Vec<(i32, f64)> = env
        .state()
        .borrow()
        .player
        .buffs
        .iter()
        .map(|aura| (aura.aura_instance_id, aura.expiration_time))
        .collect();
    assert_eq!(before, after);
    let unseeded: bool = other
        .eval("return C_UnitAuras.DoesAuraHaveExpirationTime('player', 301)")
        .expect("query in unseeded environment");
    assert!(!unseeded, "fixture records must not leak across environments");
}
