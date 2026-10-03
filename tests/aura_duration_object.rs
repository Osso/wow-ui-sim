//! Bounded Retail 12.0.5 `C_UnitAuras.GetAuraDuration` snapshots, not native parity proof.
//! Invalid-instance errors and the permanent-aura zero span are inferred simulator policy.
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
    let env = WowLuaEnv::new().expect("create aura duration environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            fixture_aura(301, true, 45.0),
            fixture_aura(302, true, 0.0),
            fixture_aura(303, false, 60.0),
        ];
        let party = state
            .party_members
            .first_mut()
            .expect("existing seeded roster");
        party.debuffs = vec![fixture_aura(402, false, 75.0)];
    }
    env.exec(
        r#"
        function AssertAuraDuration(unit, id, startTime, total)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one result')
                local duration = ...
                assert(type(duration) == 'table', 'duration object')
                assert(duration:GetTotalDuration() == total, 'total duration')
                assert(duration:GetStartTime() == startTime, 'start time')
                assert(duration:GetEndTime() == startTime + total, 'end time')
                assert(duration:GetModRate() == 1, 'unit rate')
                assert(duration:IsZero() == (total == 0), 'zero span only for permanent auras')
                return duration
            end
            return check(C_UnitAuras.GetAuraDuration(unit, id))
        end
        function RejectAuraDuration(...)
            assert(type(C_UnitAuras.GetAuraDuration) == 'function',
                'real query must be registered before testing rejection')
            local ok, err = pcall(C_UnitAuras.GetAuraDuration, ...)
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
        ("SecretDurationUnit", "player"),
        ("SecretUnknownDurationUnit", "missing-unit"),
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
fn timed_auras_return_one_duration_object_spanning_stored_times() {
    let env = fixture_env();
    env.exec(
        r#"
        AssertAuraDuration('player', 301, 15, 30)
        AssertAuraDuration('player', 303, 30, 30)
        AssertAuraDuration('party1', 402, 45, 30)
        "#,
    )
    .expect("start is expiration minus duration for helpful, harmful and party records");
}

#[test]
fn permanent_aura_yields_a_zero_span_object() {
    let env = fixture_env();
    env.exec("AssertAuraDuration('player', 302, 0, 0)")
        .expect("zero duration and expiration give a zero span");
}

#[test]
fn results_are_independent_snapshots_of_live_state() {
    let env = fixture_env();
    env.exec("FirstDuration = AssertAuraDuration('player', 301, 15, 30)")
        .expect("first snapshot");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs[0].duration = 20.0;
        state.player.buffs[0].expiration_time = 90.0;
    }
    env.exec(
        r#"
        local second = AssertAuraDuration('player', 301, 70, 20)
        assert(not rawequal(second, FirstDuration), 'distinct objects')
        assert(FirstDuration:GetTotalDuration() == 30 and FirstDuration:GetStartTime() == 15,
            'earlier snapshot unchanged')
        second:SetTimeFromStart(1, 2, 1)
        AssertAuraDuration('player', 301, 70, 20)
        "#,
    )
    .expect("live read, old snapshot kept, mutating a result does not touch state");
    let state = env.state().borrow();
    assert_eq!(state.player.buffs[0].duration, 20.0);
    assert_eq!(state.player.buffs[0].expiration_time, 90.0);
}

#[test]
fn unknown_or_malformed_arguments_error_and_recover() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectAuraDuration('player', 999)
        RejectAuraDuration('missing-unit', 301)
        RejectAuraDuration('party1', 301)
        RejectAuraDuration(nil, 301)
        RejectAuraDuration()
        RejectAuraDuration('player')
        RejectAuraDuration('player', nil)
        RejectAuraDuration('player', '301')
        RejectAuraDuration('player', 0/0)
        RejectAuraDuration(12, 301)
        RejectAuraDuration({}, 301)
        AssertAuraDuration('player', 301, 15, 30)
        "#,
    )
    .expect("no silent nil duration for an invalid instance");
}

#[test]
fn untainted_query_accepts_each_authentic_secret_argument_and_combination() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        AssertAuraDuration(SecretDurationUnit, 301, 15, 30)
        AssertAuraDuration('player', SecretTimedID, 15, 30)
        AssertAuraDuration(SecretDurationUnit, SecretTimedID, 15, 30)
        AssertAuraDuration(SecretDurationUnit, SecretPermanentID, 0, 0)
        RejectAuraDuration(SecretUnknownDurationUnit, SecretTimedID)
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretDurationUnit) and issecretvalue(SecretTimedID))
        assert(issecretvalue(SecretPermanentID) and issecretvalue(SecretUnknownDurationUnit))
        "#,
    )
    .expect("AllowedWhenUntainted authenticates secrets; does not declassify inputs");
}

#[test]
fn tainted_query_denies_each_secret_before_validation_or_lookup() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'AuraDurationProbe')
            local _, denial = pcall(C_UnitAuras.GetAuraDuration, 'player', SecretTimedID)
            local _, invalid = pcall(C_UnitAuras.GetAuraDuration, 'player', 999)
            local _, malformed = pcall(C_UnitAuras.GetAuraDuration, 12, 301)
            assert(denial ~= invalid and denial ~= malformed, 'denial is its own error')
            for _, args in ipairs({
                {SecretDurationUnit, 301}, {SecretDurationUnit, SecretTimedID},
                {SecretUnknownDurationUnit, 301}, {'missing-unit', SecretTimedID},
                {12, SecretTimedID}, {true, SecretTimedID},
            }) do
                local ok, err = pcall(C_UnitAuras.GetAuraDuration, args[1], args[2])
                assert(not ok and err == denial, 'secret denial precedes validation and lookup')
            end
            assert(debug.getstacktaint() == before)
            for _, value in ipairs({SecretDurationUnit, SecretTimedID}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
            AssertAuraDuration('player', 301, 15, 30)
            assert(debug.getstacktaint() == before, 'public recovery must retain taint')
        end
        debug.setobjecttaint(probe, 'AuraDurationProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertAuraDuration(SecretDurationUnit, SecretTimedID, 15, 30)
        "#,
    )
    .expect("VM-authenticated denial before validation, unchanged taint and public recovery");
}
