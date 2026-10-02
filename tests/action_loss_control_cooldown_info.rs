//! Row239 only: inferred snapshot/default/input and field-secrecy policies.
//! Data fixtures are not native LoC captures. RED/GREEN remain pending.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::{LuaApi, LuaApiMut, Val};
use wow_ui_sim::lua_api::{LossOfControlInfo, WowLuaEnv};

const FIELDS: [&str; 3] = ["startTime", "duration", "modRate"];
const FIRST: (f64, f64, f32, bool, bool) = (312.0, 237.0, 1.25, true, true);
const SECOND: (f64, f64, f32, bool, bool) = (11.0, 27.0, 0.5, true, false);
const EMPTY: (f64, f64, f32, bool, bool) = (0.0, 0.0, 1.0, false, false);
type Payload = (f64, f64, f32, bool, bool);

fn record(value: Payload) -> LossOfControlInfo {
    LossOfControlInfo {
        start_time: value.0,
        duration: value.1,
        mod_rate: value.2,
        is_active: value.3,
        should_replace_normal_cooldown: value.4,
    }
}

fn fixture(restricted: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("row239 environment");
    {
        let mut state = env.state().borrow_mut();
        state.cooldowns_restricted = restricted;
        state.action_bars.clear();
        state.action_bars.insert(17, 19750);
        state.action_bars.insert(19, 642);
        state.spell_loss_of_control.clear();
        state.spell_loss_of_control.insert(19750, record(FIRST));
        state.spell_loss_of_control.insert(642, record(SECOND));
    }
    env
}

fn capture(env: &WowLuaEnv, selector: &str) {
    env.exec(&format!(
        "assert(select('#', C_ActionBar.GetActionLossOfControlCooldownInfo({selector})) == 1); \
         Info = C_ActionBar.GetActionLossOfControlCooldownInfo({selector}); \
         assert(type(Info) == 'table'); assert(not issecretvalue(Info)); \
         assert(canaccessvalue(Info)); Numbers = {{Info.startTime, Info.duration, Info.modRate}}"
    ))
    .expect("one ordinary table, rooted actual numeric fields");
}

fn assert_payload(env: &WowLuaEnv, expected: Payload, restricted: bool) {
    for (field, number) in FIELDS
        .into_iter()
        .zip([expected.0, expected.1, f64::from(expected.2)])
    {
        env.exec(&format!(
            "assert(issecretvalue(Info.{field}) == {restricted}); \
             if not {restricted} then assert(type(Info.{field}) == 'number') end"
        ))
        .unwrap();
        let value: Val = env.eval(&format!("return Info.{field}")).unwrap();
        let lua = env.lua();
        assert!(rilua::api::state_is_secure(lua.state()));
        assert_eq!(
            rilua::table_security::is_secret_value(lua.state(), value),
            restricted
        );
        let payload = rilua::table_security::unwrap_secret(lua.state(), value)
            .expect("secure host authentication, not declassification");
        assert_eq!(payload, Val::Num(number), "{field}");
    }
    env.exec(&format!(
        "assert(type(Info.isActive) == 'boolean'); assert(not issecretvalue(Info.isActive)); \
         assert(Info.isActive == {}); assert(type(Info.shouldReplaceNormalCooldown) == 'boolean'); \
         assert(not issecretvalue(Info.shouldReplaceNormalCooldown)); \
         assert(Info.shouldReplaceNormalCooldown == {}); \
         local count = 0; for _ in pairs(Info) do count = count + 1 end; assert(count == 5)",
        expected.3, expected.4
    ))
    .expect("exact five fields and ordinary copied flags");
}

fn check(env: &WowLuaEnv, slot: u32, expected: Payload, restricted: bool) {
    capture(env, &slot.to_string());
    assert_payload(env, expected, restricted);
}

fn secret_slot(env: &WowLuaEnv, slot: f64) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let value = rilua::table_security::wrap_host_secret_number(lua.state_mut(), slot);
    lua.set_global_val("SecretSlot", value)
        .expect("publish authentic secret slot input");
}

fn wrapper_metadata(env: &WowLuaEnv, root_name: &str) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    assert!(rilua::api::state_is_secure(lua.state_mut()));
    let Val::Table(root) = lua.get_global_val(root_name) else {
        panic!("missing rooted numeric list")
    };
    (1..=3)
        .map(|index| {
            let value = lua.state_mut().gc.tables.get(root).unwrap().get_int(index);
            assert!(rilua::table_security::is_secret_value(
                lua.state_mut(),
                value
            ));
            let Val::Userdata(reference) = value else {
                panic!("missing authentic numeric wrapper")
            };
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .unwrap()
                .alloc_seq();
            (value, sequence)
        })
        .collect()
}

fn assert_error(env: &WowLuaEnv, expression: &str) {
    env.exec(&format!(
        "local ok, err = pcall(function() return C_ActionBar.GetActionLossOfControlCooldownInfo({expression}) end); \
         assert(not ok); assert(type(err) == 'string'); assert(#err > 0); \
         assert(string.find(err, 'C_ActionBar.GetActionLossOfControlCooldownInfo', 1, true)); \
         assert(string.find(err, '1', 1, true))"
    ))
    .expect("contextual public API argument-one error");
}

#[test]
fn assigned_first_slot_returns_all_five_concrete_values() {
    check(&fixture(false), 17, FIRST, false);
}

#[test]
fn assigned_second_slot_selects_independent_record_and_false_replacement() {
    let env = fixture(false);
    check(&env, 19, SECOND, false);
    check(&env, 17, FIRST, false);
}

#[test]
fn unassigned_positive_slots_return_exact_inactive_default() {
    let env = fixture(false);
    check(&env, 18, EMPTY, false);
    check(&env, u32::MAX, EMPTY, false);
}

#[test]
fn assigned_missing_record_returns_default_without_affecting_other_slot() {
    let env = fixture(false);
    env.state()
        .borrow_mut()
        .spell_loss_of_control
        .remove(&19750);
    check(&env, 17, EMPTY, false);
    check(&env, 19, SECOND, false);
}

#[test]
fn unrestricted_action_and_existing_spell_dtos_match_concrete_record() {
    let env = fixture(false);
    check(&env, 17, FIRST, false);
    env.exec(
        "local spell = C_Spell.GetSpellLossOfControlCooldownInfo(19750); \
         assert(type(spell) == 'table'); assert(spell ~= Info); \
         for _, field in ipairs({'startTime', 'duration', 'modRate', 'isActive', 'shouldReplaceNormalCooldown'}) do \
         assert(spell[field] == Info[field]); assert(not issecretvalue(spell[field])) end"
    ).unwrap();
}

#[test]
fn host_interval_replacement_is_visible_without_expiry_recomputation() {
    let env = fixture(false);
    check(&env, 17, FIRST, false);
    let changed = (1.0, 2.0, 0.5, true, true);
    env.state()
        .borrow_mut()
        .spell_loss_of_control
        .insert(19750, record(changed));
    check(&env, 17, changed, false);
}

#[test]
fn explicit_flags_are_copied_even_when_interval_suggests_other_activity() {
    let env = fixture(false);
    for flags in [(false, true), (true, false), (false, false), (true, true)] {
        let input = (0.0, 0.0, 1.25, flags.0, flags.1);
        env.state()
            .borrow_mut()
            .spell_loss_of_control
            .insert(19750, record(input));
        check(&env, 17, input, false);
    }
}

#[test]
fn mapping_reassignment_and_host_clear_use_current_inputs() {
    let env = fixture(false);
    check(&env, 17, FIRST, false);
    env.state().borrow_mut().action_bars.insert(17, 642);
    check(&env, 17, SECOND, false);
    env.state().borrow_mut().spell_loss_of_control.clear();
    check(&env, 17, EMPTY, false);
    check(&env, 19, EMPTY, false);
}

#[test]
fn reads_and_dto_mutation_leave_model_and_other_snapshots_unchanged() {
    let env = fixture(false);
    let mappings = env.state().borrow().action_bars.clone();
    check(&env, 17, FIRST, false);
    env.exec(
        "Other = C_ActionBar.GetActionLossOfControlCooldownInfo(17); assert(Other ~= Info); \
         Info.startTime = -99; Info.duration = 999; Info.modRate = 9; \
         Info.isActive = false; Info.shouldReplaceNormalCooldown = false; \
         Info = {duration = 123}; Info = Other",
    )
    .unwrap();
    assert_payload(&env, FIRST, false);
    assert_eq!(env.state().borrow().action_bars, mappings);
    let input = env
        .state()
        .borrow()
        .spell_loss_of_control
        .get(&19750)
        .unwrap()
        .clone();
    assert_eq!(
        (
            input.start_time,
            input.duration,
            input.mod_rate,
            input.is_active,
            input.should_replace_normal_cooldown
        ),
        FIRST
    );
    check(&env, 17, FIRST, false);
}

#[test]
fn independent_environments_do_not_share_records_or_restriction() {
    let first = fixture(true);
    let second = fixture(false);
    first.state().borrow_mut().spell_loss_of_control.clear();
    check(&first, 17, EMPTY, true);
    check(&second, 17, FIRST, false);
}

#[test]
fn restricted_assigned_records_wrap_three_numbers_and_keep_flags_public() {
    let env = fixture(true);
    check(&env, 17, FIRST, true);
    check(&env, 19, SECOND, true);
}

#[test]
fn restricted_missing_and_unassigned_defaults_wrap_zero_numbers() {
    let env = fixture(true);
    check(&env, 18, EMPTY, true);
    env.state()
        .borrow_mut()
        .spell_loss_of_control
        .remove(&19750);
    check(&env, 17, EMPTY, true);
}

#[test]
fn explicit_cooldown_flag_not_combat_or_stat_policy_controls_secrecy() {
    let env = fixture(false);
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
        check(&env, 17, FIRST, restricted);
    }
}

#[test]
fn restriction_toggle_returns_fresh_public_dto_without_unwrapping_old_fields() {
    let env = fixture(false);
    check(&env, 17, FIRST, false);
    env.state().borrow_mut().cooldowns_restricted = true;
    check(&env, 17, FIRST, true);
    let original = wrapper_metadata(&env, "Numbers");
    env.state().borrow_mut().cooldowns_restricted = false;
    env.exec("Fresh = C_ActionBar.GetActionLossOfControlCooldownInfo(17); assert(Fresh ~= Info)")
        .unwrap();
    assert_eq!(wrapper_metadata(&env, "Numbers"), original);
    assert_payload(&env, FIRST, true);
    env.exec("Info = Fresh").unwrap();
    assert_payload(&env, FIRST, false);
}

#[test]
fn tainted_public_slot_is_allowed_and_preserves_context_and_public_flags() {
    for restricted in [false, true] {
        let env = fixture(restricted);
        env.exec(&format!("ExpectedSecret = {restricted}")).unwrap();
        env.exec(
            r#"
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'LoCInfoProbe')
            Info = C_ActionBar.GetActionLossOfControlCooldownInfo(19)
            assert(type(Info) == 'table')
            assert(not issecretvalue(Info))
            for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                assert(issecretvalue(Info[field]) == ExpectedSecret)
                assert(canaccessvalue(Info[field]) == (not ExpectedSecret))
                if not ExpectedSecret then
                    local expected = {startTime = 11, duration = 27, modRate = 0.5}
                    assert(Info[field] == expected[field])
                end
            end
            assert(not issecretvalue(Info.isActive))
            assert(Info.isActive == true)
            assert(not issecretvalue(Info.shouldReplaceNormalCooldown))
            assert(Info.shouldReplaceNormalCooldown == false)
            assert(debug.getstacktaint() == 'LoCInfoProbe')
        end
        debug.setobjecttaint(addon, 'LoCInfoProbe')
        addon()
        assert(issecure())
    "#,
        )
        .unwrap();
        assert_payload(&env, SECOND, restricted);
    }
}

#[test]
fn tainted_copy_arithmetic_denial_and_recovery_preserve_real_wrapper_identity() {
    let env = fixture(true);
    check(&env, 17, FIRST, true);
    let original = wrapper_metadata(&env, "Numbers");
    env.exec(
        r#"
        local function addon()
            Copy = {}
            for key, value in pairs(Info) do Copy[key] = value end
            assert(not issecretvalue(Copy))
            for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                assert(issecretvalue(Copy[field]))
                assert(not canaccessvalue(Copy[field]))
                local ok, err = pcall(function() return Copy[field] + 1 end)
                assert(not ok)
                assert(type(err) == 'string')
                assert(#err > 0)
                for _, payload in ipairs({'312', '237', '1.25'}) do
                    assert(not string.find(err, payload, 1, true))
                end
                assert(debug.getstacktaint() == 'LoCInfoProbe')
            end
            assert(Copy.isActive == true)
            assert(not issecretvalue(Copy.isActive))
            assert(Copy.shouldReplaceNormalCooldown == true)
            assert(not issecretvalue(Copy.shouldReplaceNormalCooldown))
            CopyNumbers = {Copy.startTime, Copy.duration, Copy.modRate}
        end
        debug.setobjecttaint(addon, 'LoCInfoProbe')
        addon()
        assert(issecure())
        Recovery = C_ActionBar.GetActionLossOfControlCooldownInfo(17)
        assert(Recovery ~= Info)
    "#,
    )
    .unwrap();
    assert_eq!(wrapper_metadata(&env, "CopyNumbers"), original);
    assert_payload(&env, FIRST, true);
    env.exec("Info = Recovery").unwrap();
    assert_payload(&env, FIRST, true);
}

#[test]
fn forced_gc_keeps_copied_wrappers_and_fresh_namespace_results_alive() {
    let env = fixture(true);
    check(&env, 17, FIRST, true);
    let original = wrapper_metadata(&env, "Numbers");
    env.exec(
        r#"
        CopyNumbers = {Info.startTime, Info.duration, Info.modRate}
        for i = 1, 80 do
            Fresh = C_ActionBar.GetActionLossOfControlCooldownInfo(19)
            assert(Fresh ~= Info)
            for _, field in ipairs({'startTime', 'duration', 'modRate'}) do
                assert(issecretvalue(Fresh[field]))
            end
            local garbage = {}
            for j = 1, 40 do garbage[j] = {i, j, tostring(i) .. ':' .. tostring(j)} end
            if i % 8 == 0 then collectgarbage('collect') end
        end
        collectgarbage('collect')
    "#,
    )
    .unwrap();
    assert_eq!(wrapper_metadata(&env, "Numbers"), original);
    assert_eq!(wrapper_metadata(&env, "CopyNumbers"), original);
    assert_payload(&env, FIRST, true);
    env.exec("Info = Fresh").unwrap();
    assert_payload(&env, SECOND, true);
}

#[test]
fn secure_authentic_secret_slots_select_both_concrete_records() {
    let env = fixture(true);
    for (slot, expected) in [(17.0, FIRST), (19.0, SECOND)] {
        secret_slot(&env, slot);
        capture(&env, "SecretSlot");
        assert_payload(&env, expected, true);
        env.exec("assert(issecure()); assert(issecretvalue(SecretSlot))")
            .unwrap();
    }
}

#[test]
fn tainted_secret_slots_are_denied_before_known_unknown_or_missing_lookup() {
    let env = fixture(false);
    env.state().borrow_mut().spell_loss_of_control.remove(&642);
    for slot in [17.0, 18.0, 19.0] {
        secret_slot(&env, slot);
        env.exec(
            r#"
            local function addon()
                local ok, err = pcall(C_ActionBar.GetActionLossOfControlCooldownInfo, SecretSlot)
                assert(not ok)
                assert(type(err) == 'string')
                assert(#err > 0)
                for _, payload in ipairs({'17', '18', '19', '312', '237'}) do
                    assert(not string.find(err, payload, 1, true))
                end
                assert(issecretvalue(SecretSlot))
                assert(debug.getstacktaint() == 'LoCInputProbe')
            end
            debug.setobjecttaint(addon, 'LoCInputProbe')
            addon()
            assert(issecure())
        "#,
        )
        .unwrap();
        capture(&env, "SecretSlot");
        assert_payload(&env, if slot == 17.0 { FIRST } else { EMPTY }, false);
    }
}

#[test]
fn strict_public_slot_domain_rejects_nonfinite_fractional_and_range_values() {
    let env = fixture(false);
    for expression in [
        "0",
        "-1",
        "17.5",
        "0/0",
        "math.huge",
        "-math.huge",
        "4294967296",
    ] {
        assert_error(&env, expression);
    }
    check(&env, 17, FIRST, false);
}

#[test]
fn secure_wrong_slot_types_have_contextual_errors_and_leave_state_readable() {
    let env = fixture(false);
    for expression in [
        "nil",
        "true",
        "false",
        "'17'",
        "'private-slot-payload'",
        "{}",
        "CreateFrame('Frame')",
    ] {
        assert_error(&env, expression);
    }
    assert_error(&env, "");
    env.exec(r#"
        local ok, err = pcall(C_ActionBar.GetActionLossOfControlCooldownInfo, 'private-slot-payload')
        assert(not ok)
        assert(not string.find(err, 'private-slot-payload', 1, true))
        assert(issecure())
    "#).unwrap();
    check(&env, 19, SECOND, false);
}

#[test]
fn secure_secret_numeric_domain_is_validated_after_authentication() {
    let env = fixture(false);
    for slot in [0.0, -1.0, 17.5, f64::NAN, f64::INFINITY, 4294967296.0] {
        secret_slot(&env, slot);
        assert_error(&env, "SecretSlot");
        env.exec("assert(issecure()); assert(issecretvalue(SecretSlot))")
            .unwrap();
    }
    secret_slot(&env, 19.0);
    capture(&env, "SecretSlot");
    assert_payload(&env, SECOND, false);
}
