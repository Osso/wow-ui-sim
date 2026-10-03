//! B82 inputs only: original school authentication; existing intellect/output model.
//! Authored, not compiled or executed. Main owns grouped RED before producer edits.
#![cfg(feature = "retail-12-0-5")]

use rilua::table_security::{wrap_host_secret_bool, wrap_host_secret_number, wrap_secret};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;

const SECRET_NAMES: [&str; 5] = [
    "SBNumber",
    "SBNil",
    "SBBool",
    "SBSecretTable",
    "SBSecretFrame",
];

const ASSERTIONS: &str = r#"
    SBTable = {marker = 'PRIVATE-B82-SCHOOL', count = 82}
    SBFrame = CreateFrame('Frame')
    SBFrame.marker = 'PRIVATE-B82-FRAME'
    SBFrame:SetAlpha(0.625)
    function SBProperties()
        assert(SBTable.marker == 'PRIVATE-B82-SCHOOL' and SBTable.count == 82)
        assert(SBFrame:GetObjectType() == 'Frame')
        assert(SBFrame.marker == 'PRIVATE-B82-FRAME' and SBFrame:GetAlpha() == 0.625)
    end
    function SBTainted(probe)
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'B82SpellBonus')
            probe()
            assert(debug.getstacktaint() == 'B82SpellBonus', 'caller taint retained')
        end
        debug.setobjecttaint(addon, 'B82SpellBonus')
        addon()
        assert(issecure(), 'outer secure context restored')
    end
    function SBResult(expected, restricted, ...)
        assert(select('#', ...) == 1, 'one existing stat output')
        local value = ...
        assert(issecretvalue(value) == restricted, 'explicit output policy')
        assert(canaccessvalue(value) == not restricted, 'existing access predicate')
        if restricted and not issecure() then
            assert(not pcall(secretunwrap, value), 'opaque addon output')
            assert(not pcall(function() return value + 1 end), 'no addon arithmetic')
        else
            if restricted then value = secretunwrap(value) end
            assert(type(value) == 'number' and value == expected, 'actual intellect proxy')
        end
    end
    function SBDenied(school)
        assert(issecretvalue(school), 'original authentic input')
        local before = debug.getstacktaint()
        local ok, err = pcall(GetSpellBonusDamage, school)
        assert(not ok, 'AllowedWhenUntainted school must be denied before output')
        assert(type(err) == 'string')
        assert(string.find(err, 'GetSpellBonusDamage', 1, true), 'API context')
        assert(string.find(err, 'argument 1', 1, true), 'original argument context')
        assert(string.find(err, 'requires an untainted caller', 1, true), 'authorization boundary')
        assert(not string.find(err, 'PRIVATE-B82', 1, true), 'no secret content leak')
        assert(debug.getstacktaint() == before)
        assert(issecretvalue(school), 'no input declassification')
    end
"#;

fn publish_secret(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let result = lua.set_global_val(name, value);
    lua.state_mut().pop();
    result.expect("permanent host wrapper root");
}

fn fixture_env(intellect: f64) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("real spell bonus API");
    env.state().borrow_mut().player.stats.intellect = intellect;
    env.exec(ASSERTIONS)
        .expect("assertions only, no query overrides");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    let number = wrap_host_secret_number(lua.state_mut(), 2.0);
    publish_secret(&mut *lua, "SBNumber", number);
    let nil = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish_secret(&mut *lua, "SBNil", nil);
    let boolean = wrap_host_secret_bool(lua.state_mut(), false);
    publish_secret(&mut *lua, "SBBool", boolean);
    for (original, name) in [("SBTable", "SBSecretTable"), ("SBFrame", "SBSecretFrame")] {
        let value = lua.get_global_val(original);
        let Val::Table(reference) = value else {
            panic!("actual original table/frame")
        };
        if original == "SBFrame" {
            assert!(
                lua.state_mut()
                    .gc
                    .tables
                    .get(reference)
                    .unwrap()
                    .backing()
                    .is_some()
            );
        }
        lua.state_mut().push(value);
        let secret = wrap_secret(lua.state_mut(), value).unwrap();
        publish_secret(&mut *lua, name, secret);
        lua.state_mut().pop();
    }
    drop(lua);
    env.exec("SBSecrets = {SBNumber, SBNil, SBBool, SBSecretTable, SBSecretFrame}; for _, value in ipairs(SBSecrets) do assert(issecretvalue(value)) end")
        .unwrap();
    env
}

fn snapshot_roots(env: &WowLuaEnv) -> (Val, Vec<(Val, u64)>) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("SBSecrets") else {
        panic!("permanent list")
    };
    let roots = SECRET_NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let value = lua.get_global_val(name);
            let Val::Userdata(reference) = value else {
                panic!("original wrapper {name}")
            };
            assert!(rilua::table_security::is_secret_value(
                lua.state_mut(),
                value
            ));
            assert_eq!(
                lua.state_mut()
                    .gc
                    .tables
                    .get(list)
                    .unwrap()
                    .get_int(index as i64 + 1),
                value,
                "actual global/list identities; no tainted secret BOOL equality"
            );
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .unwrap()
                .alloc_seq();
            (value, sequence)
        })
        .collect();
    (Val::Table(list), roots)
}

#[test]
fn secure_original_secret_school_returns_existing_intellect_and_preserves_trust() {
    let env = fixture_env(2500.0);
    let roots = snapshot_roots(&env);
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            "assert(issecure()); SBResult(2500, {restricted}, GetSpellBonusDamage(SBNumber)); assert(issecure()); assert(issecretvalue(SBNumber)); SBProperties()"
        )).unwrap();
    }
    assert_eq!(snapshot_roots(&env), roots);
    assert_eq!(env.state().borrow().player.stats.intellect, 2500.0);
}

#[test]
fn tainted_original_secret_school_is_denied_even_for_zero_or_restricted_output() {
    let env = fixture_env(2500.0);
    let roots = snapshot_roots(&env);
    for (intellect, restricted) in [(2500.0, false), (0.0, false), (0.0, true)] {
        {
            let mut state = env.state().borrow_mut();
            state.player.stats.intellect = intellect;
            state.unit_stats_restricted = restricted;
        }
        env.exec(&format!(
            "SBTainted(function() SBDenied(SBNumber); SBResult({intellect}, {restricted}, GetSpellBonusDamage(2)) end); SBResult({intellect}, {restricted}, GetSpellBonusDamage(2)); SBProperties()"
        )).unwrap();
        let state = env.state().borrow();
        assert_eq!(state.player.stats.intellect, intellect);
        assert_eq!(state.unit_stats_restricted, restricted);
    }
    assert_eq!(snapshot_roots(&env), roots);
}

#[test]
fn tainted_secret_nil_bool_table_and_actual_frame_authenticate_before_unused_payload() {
    let env = fixture_env(2500.0);
    let roots = snapshot_roots(&env);
    env.exec(
        r#"
        SBTainted(function()
            for _, value in ipairs({SBNil, SBBool, SBSecretTable, SBSecretFrame}) do
                SBDenied(value)
                SBResult(2500, false, GetSpellBonusDamage(2))
                SBProperties()
            end
        end)
        SBProperties()
    "#,
    )
    .unwrap();
    assert_eq!(snapshot_roots(&env), roots);
    assert_eq!(env.state().borrow().player.stats.intellect, 2500.0);
}

#[test]
fn public_tainted_school_follows_explicit_output_flag_and_retains_caller_taint() {
    let env = fixture_env(2500.0);
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            "SBTainted(function() SBResult(2500, {restricted}, GetSpellBonusDamage(2)) end); SBResult(2500, {restricted}, GetSpellBonusDamage(2))"
        )).unwrap();
    }
}

#[test]
fn damage_and_unchanged_healing_share_value_arity_and_false_true_false_output_control() {
    let env = fixture_env(2500.0);
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            r#"
            assert(C_Secrets.ShouldUnitStatsBeSecret() == {restricted})
            assert(not issecretvalue(C_Secrets.ShouldUnitStatsBeSecret()))
            SBResult(2500, {restricted}, GetSpellBonusDamage(2))
            SBResult(2500, {restricted}, GetSpellBonusHealing())
            SBTainted(function()
                SBResult(2500, {restricted}, GetSpellBonusDamage(2))
                SBResult(2500, {restricted}, GetSpellBonusHealing())
            end)
        "#
        ))
        .unwrap();
    }
}

#[test]
fn live_intellect_mutation_and_restriction_are_environment_local() {
    let env = fixture_env(2500.0);
    let other = fixture_env(731.0);
    env.exec("SBResult(2500, false, GetSpellBonusDamage(SBNumber))")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.stats.intellect = 819.0;
        state.unit_stats_restricted = true;
    }
    env.exec("SBResult(819, true, GetSpellBonusDamage(SBNumber)); SBResult(819, true, GetSpellBonusHealing())").unwrap();
    other.exec("SBResult(731, false, GetSpellBonusDamage(SBNumber)); SBResult(731, false, GetSpellBonusHealing()); assert(not C_Secrets.ShouldUnitStatsBeSecret())").unwrap();
    env.state().borrow_mut().unit_stats_restricted = false;
    env.exec("SBResult(819, false, GetSpellBonusDamage(SBNumber)); SBResult(819, false, GetSpellBonusHealing())").unwrap();
    assert_eq!(other.state().borrow().player.stats.intellect, 731.0);
    assert!(!other.state().borrow().unit_stats_restricted);
}

#[test]
fn permanent_global_list_and_wrapper_allocations_survive_gc_calls_denials_and_toggles() {
    let env = fixture_env(2500.0);
    let roots = snapshot_roots(&env);
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!(
            r#"
            local garbage = {{}}
            for i=1,128 do garbage[i] = {{index = i, nested = {{i}}}} end
            collectgarbage('collect')
            SBResult(2500, {restricted}, GetSpellBonusDamage(SBNumber))
            SBTainted(function()
                for _, value in ipairs(SBSecrets) do SBDenied(value) end
                SBResult(2500, {restricted}, GetSpellBonusDamage(2))
            end)
            collectgarbage('collect')
            for _, value in ipairs(SBSecrets) do assert(issecretvalue(value)) end
            SBProperties()
        "#
        ))
        .unwrap();
        assert_eq!(
            snapshot_roots(&env),
            roots,
            "same permanent list and original wrappers"
        );
    }
}
