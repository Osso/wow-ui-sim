//! Exact row233 output-only contract. Numeric-field/zero secrecy is inferred,
//! not native parity. Compiled RED and subsequent gates remain parent-owned.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::{LuaApi, LuaApiMut, Val};
use std::time::{Duration, Instant};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::SpellCooldownState;

const NUMBER_FIELDS: [&str; 3] = ["startTime", "duration", "modRate"];
const SPELL: (f64, f64) = (312.0, 237.0);
const GCD: (f64, f64) = (330.0, 300.0);
const EMPTY: (f64, f64) = (0.0, 0.0);
const HAS_ACTIVE_FIELD: bool = cfg!(feature = "retail-12-1-0");

fn fixture(restricted: bool, spell: bool, gcd: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("row233 environment");
    {
        let mut state = env.state().borrow_mut();
        state.start_time = Instant::now() - Duration::from_secs(335);
        state.cooldowns_restricted = restricted;
        state.action_bars.insert(17, 19750);
        state.action_bars.remove(&18);
        state.spell_cooldowns.clear();
        if spell {
            state.spell_cooldowns.insert(
                19750,
                SpellCooldownState {
                    start: SPELL.0,
                    duration: SPELL.1,
                },
            );
        }
        state.gcd = gcd.then_some(GCD);
    }
    env
}

fn capture(env: &WowLuaEnv, slot: u32) {
    env.exec(&format!(
        "assert(select('#', C_ActionBar.GetActionCooldown({slot})) == 1); \
         ACInfo = C_ActionBar.GetActionCooldown({slot}); \
         assert(type(ACInfo) == 'table' and not issecretvalue(ACInfo)); \
         assert(canaccessvalue(ACInfo)); \
         ACNumbers = {{ACInfo.startTime, ACInfo.duration, ACInfo.modRate}}"
    ))
    .expect("ordinary table and Lua roots before host inspection");
}

fn assert_numbers(env: &WowLuaEnv, expected: (f64, f64), restricted: bool) {
    for (field, number) in NUMBER_FIELDS.into_iter().zip([expected.0, expected.1, 1.0]) {
        env.exec(&format!(
            "assert(type(ACInfo.{field}) == 'number'); \
             assert(issecretvalue(ACInfo.{field}) == {restricted})"
        ))
        .expect("each literal field retains numeric Lua type and secrecy");
        let value: Val = env.eval(&format!("return ACInfo.{field}")).unwrap();
        let lua = env.lua();
        assert!(
            rilua::api::state_is_secure(lua.state()),
            "fresh secure host entry"
        );
        assert_eq!(
            rilua::table_security::is_secret_value(lua.state(), value),
            restricted,
            "{field}"
        );
        let payload = if restricted {
            rilua::table_security::unwrap_secret(lua.state(), value)
                .expect("existing VM authentication at secure host entry")
        } else {
            value
        };
        assert_eq!(payload, Val::Num(number), "{field} payload");
    }
}

fn assert_shape(env: &WowLuaEnv, active: bool) {
    env.exec(&format!(
        r#"
        assert(type(ACInfo.isEnabled) == 'boolean')
        assert(not issecretvalue(ACInfo.isEnabled) and ACInfo.isEnabled == true)
        assert(ACInfo.isOnGCD == nil and ACInfo.activeCategory == nil)
        assert(ACInfo.timeUntilEndOfStartRecovery == nil)
        local count = 0
        for _ in pairs(ACInfo) do count = count + 1 end
        assert(count == {})
        if {} then
            assert(type(ACInfo.isActive) == 'boolean')
            assert(not issecretvalue(ACInfo.isActive) and ACInfo.isActive == {})
        else
            assert(ACInfo.isActive == nil)
        end
        "#,
        4 + usize::from(HAS_ACTIVE_FIELD),
        HAS_ACTIVE_FIELD,
        active
    ))
    .expect("unchanged four/five-field profile payload and public booleans");
}

fn check(env: &WowLuaEnv, slot: u32, expected: (f64, f64), restricted: bool) {
    capture(env, slot);
    assert_numbers(env, expected, restricted);
    assert_shape(env, expected != EMPTY);
}

fn wrapper_metadata(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    assert!(rilua::api::state_is_secure(lua.state_mut()));
    let Val::Table(root) = lua.get_global_val("ACNumbers") else {
        panic!("numeric root list missing")
    };
    (1..=3)
        .map(|index| {
            let value = lua
                .state_mut()
                .gc
                .tables
                .get(root)
                .expect("live root list")
                .get_int(index);
            assert!(rilua::table_security::is_secret_value(
                lua.state_mut(),
                value
            ));
            let Val::Userdata(reference) = value else {
                panic!("actual VM numeric wrapper missing")
            };
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .expect("root membership keeps original wrapper live")
                .alloc_seq();
            (value, sequence)
        })
        .collect()
}

fn assert_copy_identity(env: &WowLuaEnv, original: &[(Val, u64)]) {
    assert_eq!(
        wrapper_metadata(env),
        original,
        "original rooted wrapper identity/allocation"
    );
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(copy) = lua.get_global_val("ACCopyNumbers") else {
        panic!("copied field root list missing")
    };
    for (index, (value, _)) in original.iter().enumerate() {
        assert_eq!(
            lua.state_mut()
                .gc
                .tables
                .get(copy)
                .expect("live copy root")
                .get_int(index as i64 + 1),
            *value,
            "host-only copied wrapper identity"
        );
    }
}

#[test]
fn unrestricted_spell_interval_is_public_and_exact() {
    check(&fixture(false, true, false), 17, SPELL, false);
}

#[test]
fn unrestricted_gcd_only_interval_is_public_and_exact() {
    check(&fixture(false, false, true), 17, GCD, false);
}

#[test]
fn unrestricted_overlap_keeps_latest_ending_gcd_selection() {
    check(&fixture(false, true, true), 17, GCD, false);
}

#[test]
fn unrestricted_empty_and_unassigned_positive_slots_keep_zero_payloads() {
    let env = fixture(false, false, false);
    check(&env, 17, EMPTY, false);
    env.state().borrow_mut().gcd = Some(GCD);
    check(&env, 18, EMPTY, false);
}

#[test]
fn restricted_spell_interval_wraps_all_three_actual_numbers() {
    check(&fixture(true, true, false), 17, SPELL, true);
}

#[test]
fn restricted_gcd_only_interval_wraps_all_three_actual_numbers() {
    check(&fixture(true, false, true), 17, GCD, true);
}

#[test]
fn restricted_overlap_keeps_latest_ending_gcd_selection() {
    check(&fixture(true, true, true), 17, GCD, true);
}

#[test]
fn restricted_assigned_spell_without_cooldown_wraps_zero_interval() {
    check(&fixture(true, false, false), 17, EMPTY, true);
}

#[test]
fn restricted_unassigned_positive_slot_keeps_secret_zero_payload() {
    check(&fixture(true, true, true), 18, EMPTY, true);
}

#[test]
fn booleans_and_fieldset_remain_public_for_active_and_empty_results() {
    let env = fixture(true, true, false);
    check(&env, 17, SPELL, true);
    env.state().borrow_mut().spell_cooldowns.clear();
    check(&env, 17, EMPTY, true);
}

#[test]
fn live_restriction_toggle_keeps_old_secrets_and_returns_fresh_public_numbers() {
    let env = fixture(false, true, false);
    check(&env, 17, SPELL, false);
    env.state().borrow_mut().cooldowns_restricted = true;
    check(&env, 17, SPELL, true);
    let original = wrapper_metadata(&env);
    env.state().borrow_mut().cooldowns_restricted = false;
    env.exec(
        "ACFresh = C_ActionBar.GetActionCooldown(17); assert(ACFresh ~= ACInfo); \
              ACCopyNumbers = {ACInfo.startTime, ACInfo.duration, ACInfo.modRate}",
    )
    .unwrap();
    assert_copy_identity(&env, &original);
    assert_numbers(&env, SPELL, true);
    env.exec("ACInfo = ACFresh").unwrap();
    assert_numbers(&env, SPELL, false);
}

#[test]
fn explicit_flag_not_combat_or_stat_policy_controls_output() {
    let env = fixture(false, true, false);
    for (restricted, combat, stats) in [
        (false, true, true),
        (true, false, false),
        (false, false, true),
    ] {
        {
            let mut state = env.state().borrow_mut();
            state.cooldowns_restricted = restricted;
            state.player.in_combat = combat;
            state.unit_stats_restricted = stats;
        }
        env.exec(&format!(
            "local flag = C_Secrets.ShouldCooldownsBeSecret(); \
            assert(not issecretvalue(flag) and flag == {restricted})"
        ))
        .unwrap();
        check(&env, 17, SPELL, restricted);
    }
}

#[test]
fn secure_and_tainted_public_callers_preserve_context_and_field_observation() {
    let env = fixture(true, true, false);
    env.exec(
        r#"
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ActionCooldownOutputProbe')
            ACInfo = C_ActionBar.GetActionCooldown(17)
            assert(type(ACInfo) == 'table' and not issecretvalue(ACInfo))
            for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                assert(type(ACInfo[field]) == 'number' and issecretvalue(ACInfo[field]))
                assert(not canaccessvalue(ACInfo[field]))
            end
            assert(not issecretvalue(ACInfo.isEnabled) and ACInfo.isEnabled == true)
            assert(debug.getstacktaint() == 'ActionCooldownOutputProbe')
        end
        debug.setobjecttaint(addon, 'ActionCooldownOutputProbe')
        addon()
        assert(issecure())
        ACNumbers = {ACInfo.startTime, ACInfo.duration, ACInfo.modRate}
    "#,
    )
    .unwrap();
    assert_numbers(&env, SPELL, true);
    assert_shape(&env, true);
}

#[test]
fn tainted_arithmetic_denial_keeps_roots_context_and_secure_recovery() {
    let env = fixture(true, true, false);
    capture(&env, 17);
    let original = wrapper_metadata(&env);
    env.exec(
        r#"
        local function addon()
            for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                local ok, err = pcall(function() return ACInfo[field] + 1 end)
                assert(not ok and type(err) == 'string' and #err > 0)
                assert(not string.find(err, '312', 1, true))
                assert(not string.find(err, '237', 1, true))
                assert(issecretvalue(ACInfo[field]))
                assert(debug.getstacktaint() == 'ActionCooldownOutputProbe')
            end
            ACCopyNumbers = {ACInfo.startTime, ACInfo.duration, ACInfo.modRate}
        end
        debug.setobjecttaint(addon, 'ActionCooldownOutputProbe')
        addon()
        assert(issecure())
        ACRecovery = C_ActionBar.GetActionCooldown(17)
        assert(type(ACRecovery) == 'table' and issecretvalue(ACRecovery.duration))
    "#,
    )
    .unwrap();
    assert_copy_identity(&env, &original);
    assert_numbers(&env, SPELL, true);
}

#[test]
fn tainted_table_copy_preserves_actual_wrapper_identity() {
    let env = fixture(true, true, false);
    capture(&env, 17);
    let original = wrapper_metadata(&env);
    env.exec(
        r#"
        local function addon()
            ACCopy = {}
            for key, value in pairs(ACInfo) do ACCopy[key] = value end
            assert(not issecretvalue(ACCopy))
            for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                assert(type(ACCopy[field]) == 'number' and issecretvalue(ACCopy[field]))
                assert(not canaccessvalue(ACCopy[field]))
            end
            assert(not issecretvalue(ACCopy.isEnabled) and ACCopy.isEnabled == true)
            ACCopyNumbers = {ACCopy.startTime, ACCopy.duration, ACCopy.modRate}
            assert(debug.getstacktaint() == 'ActionCooldownOutputProbe')
        end
        debug.setobjecttaint(addon, 'ActionCooldownOutputProbe')
        addon()
        assert(issecure())
    "#,
    )
    .unwrap();
    assert_copy_identity(&env, &original);
    assert_numbers(&env, SPELL, true);
}

#[test]
fn result_mutation_and_replacement_do_not_change_model_or_other_dtos() {
    let env = fixture(true, true, true);
    let mappings = env.state().borrow().action_bars.clone();
    let charges = env.state().borrow().spell_charges.clone();
    env.exec(
        r#"
        ACOther = C_ActionBar.GetActionCooldown(17)
        ACInfo = C_ActionBar.GetActionCooldown(17)
        assert(ACInfo ~= ACOther)
        ACInfo.startTime = -99
        ACInfo.duration = 999
        ACInfo.modRate = 9
        ACInfo.isEnabled = false
        ACInfo = {duration = 123}
        assert(issecretvalue(ACOther.duration) and ACOther.isEnabled == true)
    "#,
    )
    .unwrap();
    {
        let state = env.state().borrow();
        assert_eq!(state.action_bars, mappings);
        assert!(state.cooldowns_restricted);
        assert_eq!(state.gcd, Some(GCD));
        let spell = state.spell_cooldowns.get(&19750).unwrap();
        assert_eq!((spell.start, spell.duration), SPELL);
        assert_eq!(state.spell_charges, charges, "no fabricated charge inputs");
    }
    env.exec("ACInfo = ACOther; ACNumbers = {ACInfo.startTime, ACInfo.duration, ACInfo.modRate}")
        .unwrap();
    assert_numbers(&env, GCD, true);
    check(&env, 17, GCD, true);
}

#[test]
fn live_interval_changes_expiry_and_independent_envs_use_current_state() {
    let env = fixture(true, true, true);
    let independent = fixture(false, true, false);
    check(&env, 17, GCD, true);
    env.state().borrow_mut().gcd = None;
    check(&env, 17, SPELL, true);
    env.state().borrow_mut().spell_cooldowns.insert(
        19750,
        SpellCooldownState {
            start: 320.0,
            duration: 400.0,
        },
    );
    check(&env, 17, (320.0, 400.0), true);
    env.state().borrow_mut().start_time = Instant::now() - Duration::from_secs(900);
    check(&env, 17, EMPTY, true);
    check(&independent, 17, SPELL, false);
    assert!(!independent.state().borrow().cooldowns_restricted);
}

#[test]
fn forced_gc_keeps_original_wrappers_roots_and_fresh_result_payloads() {
    let env = fixture(true, true, false);
    capture(&env, 17);
    let original = wrapper_metadata(&env);
    env.exec(
        r#"
        ACCopyNumbers = {ACInfo.startTime, ACInfo.duration, ACInfo.modRate}
        for i = 1, 80 do
            ACFresh = C_ActionBar.GetActionCooldown(17)
            assert(ACFresh ~= ACInfo)
            assert(issecretvalue(ACFresh.startTime))
            assert(issecretvalue(ACFresh.duration))
            assert(issecretvalue(ACFresh.modRate))
            local garbage = {}
            for j = 1, 40 do garbage[j] = {i, j, tostring(i) .. ':' .. tostring(j)} end
            if i % 8 == 0 then collectgarbage('collect') end
        end
        collectgarbage('collect')
    "#,
    )
    .unwrap();
    assert_copy_identity(&env, &original);
    assert_numbers(&env, SPELL, true);
    env.exec("ACInfo = ACFresh").unwrap();
    assert_numbers(&env, SPELL, true);
}
