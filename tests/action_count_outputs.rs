//! Batch66 exact237/241. Inferred provider/domain/privacy policies, no native parity.
//! Scaffold only: compiled RED must precede any getter or publication change.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::c_api::{ActionUseCountInfo, charge_state::SpellChargeState};
use wow_ui_sim::lua_api::WowLuaEnv;

const ASSERTIONS: &str = r#"
    function ACTainted(probe)
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ActionCountProbe')
            probe()
            assert(debug.getstacktaint() == 'ActionCountProbe')
        end
        debug.setobjecttaint(addon, 'ActionCountProbe')
        addon()
        assert(issecure())
    end
    function ACReject(api, position, denial, ...)
        local before = debug.getstacktaint()
        local ok, err = pcall(C_ActionBar[api], ...)
        assert(not ok)
        assert(type(err) == 'string')
        assert(#err > 0)
        assert(string.find(err, 'C_ActionBar.'..api, 1, true))
        assert(string.find(err, tostring(position), 1, true))
        assert(not string.find(err, 'PRIVATE-Count', 1, true))
        local guard = string.find(err, 'requires an untainted caller', 1, true)
        if denial then assert(guard) else assert(not guard) end
        assert(debug.getstacktaint() == before)
    end
    ACFrame = CreateFrame('Frame')
    ACFrame:SetAlpha(0.625)
    ACFrame.marker = 66
    ACTable = {marker='PRIVATE-Count', count=37}
"#;

fn fixture(restricted: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("real action namespace");
    {
        let mut state = env.state().borrow_mut();
        assert!(state.action_use_counts.is_empty());
        state.action_bars.clear();
        state.action_bars.extend([(17, 19750), (19, 642)]);
        state.spell_charges.clear();
        state.cooldowns_restricted = restricted;
        state.action_use_counts.extend([
            (
                17,
                ActionUseCountInfo {
                    spell_id: 19750,
                    count: 7,
                },
            ),
            (
                19,
                ActionUseCountInfo {
                    spell_id: 642,
                    count: 2,
                },
            ),
        ]);
    }
    env.exec(ASSERTIONS).unwrap();
    env
}

fn set_count(env: &WowLuaEnv, slot: u32, spell_id: u32, count: u32) {
    env.state()
        .borrow_mut()
        .action_use_counts
        .insert(slot, ActionUseCountInfo { spell_id, count });
}

fn set_charge(env: &WowLuaEnv, current: u32, max: u32) {
    env.state().borrow_mut().spell_charges.insert(
        19750,
        SpellChargeState {
            current_charges: current,
            max_charges: max,
            recharge_start: 312.0,
            recharge_duration: 237.0,
            charge_mod_rate: 1.25,
        },
    );
}

fn capture(env: &WowLuaEnv, api: &str, arguments: &str) {
    env.exec(&format!(
        "assert(select('#', C_ActionBar.{api}({arguments})) == 1); \
         ACResult = C_ActionBar.{api}({arguments})"
    ))
    .expect("one actual scalar rooted before host inspection");
}

fn assert_result(env: &WowLuaEnv, number: Option<u32>, text: &str, restricted: bool) {
    env.exec(&format!(
        "assert(issecretvalue(ACResult) == {restricted}); \
         if not {restricted} then assert(type(ACResult) == '{}') end",
        if number.is_some() { "number" } else { "string" }
    ))
    .unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    assert!(rilua::api::state_is_secure(lua.state_mut()));
    let value = lua.get_global_val("ACResult");
    assert_eq!(
        rilua::table_security::is_secret_value(lua.state_mut(), value),
        restricted
    );
    let payload = rilua::table_security::unwrap_secret(lua.state_mut(), value)
        .expect("trusted host authentication");
    let expected = match number {
        Some(count) => Val::Num(f64::from(count)),
        None => Val::Str(lua.state_mut().gc.intern_string(text.as_bytes())),
    };
    assert_eq!(
        payload, expected,
        "actual ValNum/ValStr UTF8 payload; no secret nominal Lua type claim"
    );
}

fn display(env: &WowLuaEnv, arguments: &str, expected: &str, restricted: bool) {
    capture(env, "GetActionDisplayCount", arguments);
    assert_result(env, None, expected, restricted);
}

fn use_count(env: &WowLuaEnv, arguments: &str, expected: u32, restricted: bool) {
    capture(env, "GetActionUseCount", arguments);
    assert_result(env, Some(expected), "", restricted);
}

fn positive(env: &WowLuaEnv) {
    use_count(env, "17", 7, false);
    display(env, "17", "7", false);
    use_count(env, "19", 2, false);
    display(env, "19", "2", false);
}

fn publish(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let result = lua.set_global_val(name, value);
    lua.state_mut().pop();
    result.expect("root authentic VM input");
}

const SECRETS: [&str; 9] = [
    "ACSlot",
    "ACMax",
    "ACReplacement",
    "ACNil",
    "ACBool",
    "ACString",
    "ACSecretTable",
    "ACSecretFrame",
    "ACUnknown",
];

fn secret_fixture() -> WowLuaEnv {
    let env = fixture(false);
    positive(&env); // Meaningful provider proof before authentication/guard assertions.
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    for (name, number) in [("ACSlot", 17.0), ("ACMax", 6.5), ("ACUnknown", 18.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        publish(&mut *lua, name, value);
    }
    for (name, text) in [("ACReplacement", "é雪"), ("ACString", "PRIVATE-Count")] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        publish(&mut *lua, name, value);
    }
    let value = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish(&mut *lua, "ACNil", value);
    let value = wrap_host_secret_bool(lua.state_mut(), false);
    publish(&mut *lua, "ACBool", value);
    for (source, name) in [("ACTable", "ACSecretTable"), ("ACFrame", "ACSecretFrame")] {
        let value = lua.get_global_val(source);
        let Val::Table(reference) = value else {
            panic!("real table/frame")
        };
        if source == "ACFrame" {
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
        let wrapped = wrap_secret(lua.state_mut(), value).unwrap();
        publish(&mut *lua, name, wrapped);
        lua.state_mut().pop();
    }
    drop(lua);
    env.exec(
        r#"
        ACSecrets = {
            ACSlot, ACMax, ACReplacement, ACNil, ACBool,
            ACString, ACSecretTable, ACSecretFrame, ACUnknown,
        }
        for _, value in ipairs(ACSecrets) do
            assert(issecretvalue(value))
        end
        "#,
    )
    .unwrap();
    env
}

fn metadata(env: &WowLuaEnv, names: &[&str]) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    names
        .iter()
        .map(|name| {
            let value = lua.get_global_val(name);
            assert!(rilua::table_security::is_secret_value(
                lua.state_mut(),
                value
            ));
            let Val::Userdata(reference) = value else {
                panic!("actual opaque VM wrapper")
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

fn restrict_after_positive(env: &WowLuaEnv) {
    positive(env);
    env.state().borrow_mut().cooldowns_restricted = true;
}

#[test]
fn explicit_inputs_are_empty_by_default_and_independent_per_slot() {
    positive(&fixture(false));
}

#[test]
fn same_spell_bindings_retain_independent_slot_counts_and_one_scalar() {
    let env = fixture(false);
    positive(&env);
    env.state().borrow_mut().action_bars.insert(19, 19750);
    set_count(&env, 19, 19750, 2);
    use_count(&env, "17", 7, false);
    use_count(&env, "19", 2, false);
    display(&env, "17", "7", false);
    display(&env, "19", "2", false);
}

#[test]
fn no_assignment_and_missing_provider_have_distinct_zero_and_empty_domains() {
    let env = fixture(false);
    positive(&env);
    env.state().borrow_mut().action_use_counts.remove(&17);
    for slot in ["17", "18", "4294967295"] {
        use_count(&env, slot, 0, false);
        display(&env, slot, "", false);
    }
}

#[test]
fn supplied_zero_is_display_string_zero_not_absence() {
    let env = fixture(false);
    set_count(&env, 17, 19750, 0);
    use_count(&env, "17", 0, false);
    display(&env, "17", "0", false);
    display(&env, "17,-1", "*", false);
}

#[test]
fn stale_binding_is_ignored_without_mutation_and_replacement_is_immediate() {
    let env = fixture(false);
    positive(&env);
    env.state().borrow_mut().action_bars.insert(17, 642);
    use_count(&env, "17", 0, false);
    display(&env, "17", "", false);
    assert_eq!(env.state().borrow().action_use_counts[&17].spell_id, 19750);
    set_count(&env, 17, 642, 9);
    use_count(&env, "17", 9, false);
    display(&env, "17", "9", false);
    use_count(&env, "19", 2, false);
    display(&env, "19", "2", false);
}

#[test]
fn binding_and_provider_clears_are_live_without_consumption() {
    let env = fixture(false);
    positive(&env);
    env.state().borrow_mut().action_bars.remove(&17);
    use_count(&env, "17", 0, false);
    display(&env, "17", "", false);
    env.state().borrow_mut().action_bars.insert(17, 19750);
    display(&env, "17", "7", false);
    env.state().borrow_mut().action_use_counts.clear();
    use_count(&env, "19", 0, false);
    display(&env, "19", "", false);
}

#[test]
fn valid_charges_override_display_only_separate_use_seven_display_three() {
    let env = fixture(false);
    positive(&env);
    set_charge(&env, 3, 5);
    use_count(&env, "17", 7, false);
    display(&env, "17", "3", false);
    display(&env, "19", "2", false);
}

#[test]
fn zero_max_charges_are_ignored_and_live_charge_updates_are_visible() {
    let env = fixture(false);
    set_charge(&env, 3, 0);
    display(&env, "17", "7", false);
    set_charge(&env, 0, 5);
    display(&env, "17", "0", false);
    set_charge(&env, 4, 5);
    display(&env, "17", "4", false);
    env.state().borrow_mut().spell_charges.clear();
    display(&env, "17", "7", false);
}

#[test]
fn charges_do_not_supply_use_count_and_require_current_assignment() {
    let env = fixture(false);
    set_charge(&env, 3, 5);
    env.state().borrow_mut().action_use_counts.remove(&17);
    use_count(&env, "17", 0, false);
    display(&env, "17", "3", false);
    env.state().borrow_mut().action_bars.insert(17, 642);
    use_count(&env, "17", 0, false);
    display(&env, "17", "", false);
}

#[test]
fn default_threshold_distinguishes_9999_from_10000() {
    let env = fixture(false);
    set_count(&env, 17, 19750, 9999);
    display(&env, "17", "9999", false);
    set_count(&env, 17, 19750, 10000);
    use_count(&env, "17", 10000, false);
    display(&env, "17", "*", false);
}

#[test]
fn strict_threshold_equality_fractional_and_negative_are_numeric() {
    let env = fixture(false);
    display(&env, "17,6", "*", false);
    display(&env, "17,7", "7", false);
    display(&env, "17,6.5", "*", false);
    display(&env, "17,-1", "*", false);
    display(&env, "18,-1", "", false);
}

#[test]
fn nil_and_omitted_formatter_arguments_use_defaults() {
    let env = fixture(false);
    for args in ["17", "17,nil", "17,nil,nil", "17,6,nil"] {
        display(
            &env,
            args,
            if args == "17,6,nil" { "*" } else { "7" },
            false,
        );
    }
}

#[test]
fn empty_and_utf8_replacements_preserve_exact_bytes_unlocalized_digits() {
    let env = fixture(false);
    display(&env, "17,6,''", "", false);
    display(&env, "17,6,'é雪'", "é雪", false);
    display(&env, "17,7,'é雪'", "7", false);
    set_count(&env, 17, 19750, 1234);
    display(&env, "17", "1234", false);
}

#[test]
fn u32_max_quantity_has_exact_public_number_and_decimal_thresholds() {
    let env = fixture(false);
    set_count(&env, 17, 19750, u32::MAX);
    use_count(&env, "17", u32::MAX, false);
    display(&env, "17,4294967295", "4294967295", false);
    display(&env, "17,4294967294.5,'limit'", "limit", false);
    display(&env, "17", "*", false);
}

#[test]
fn reads_aliases_and_result_replacement_preserve_maps_and_environment_isolation() {
    let env = fixture(false);
    let independent = fixture(false);
    positive(&env);
    set_charge(&env, 3, 5);
    let (bars, counts, charges) = {
        let state = env.state().borrow();
        (
            state.action_bars.clone(),
            state.action_use_counts.clone(),
            state.spell_charges.clone(),
        )
    };
    env.exec(
        r#"
        local display = C_ActionBar.GetActionDisplayCount
        local use = C_ActionBar.GetActionUseCount
        for index = 1, 8 do
            assert(display(17) == '3')
            assert(use(17) == 7)
        end
        ACResult = 'changed'
        assert(display(17) == '3')
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(state.action_bars, bars);
    assert_eq!(state.action_use_counts, counts);
    assert_eq!(state.spell_charges, charges);
    drop(state);
    set_count(&env, 19, 642, 9);
    use_count(&independent, "19", 2, false);
    display(&independent, "17", "7", false);
}

#[test]
fn strict_slot_domain_rejects_invalid_public_inputs_in_both_caller_frames() {
    let env = fixture(false);
    positive(&env);
    env.exec(r#"
        local function probe()
            for _,api in ipairs({'GetActionDisplayCount','GetActionUseCount'}) do
                ACReject(api,1,false)
                ACReject(api,1,false,nil)
                for _,v in ipairs({false,'17',{},ACFrame,0,-1,17.5,0/0,math.huge,-math.huge,4294967296}) do
                    ACReject(api,1,false,v)
                end
            end
        end
        probe(); ACTainted(probe)
    "#).unwrap();
}

#[test]
fn finite_threshold_and_cstring_representability_validate_even_without_source() {
    let env = fixture(false);
    positive(&env);
    env.exec(
        r#"
        local function probe()
            for _,slot in ipairs({17,18}) do
                for _,v in ipairs({false,'6',{},ACFrame,0/0,math.huge,-math.huge}) do
                    ACReject('GetActionDisplayCount',2,false,slot,v)
                end
                for _,v in ipairs({false,7,{},ACFrame,'a\000b',string.char(255)}) do
                    ACReject('GetActionDisplayCount',3,false,slot,6,v)
                end
            end
        end
        probe(); ACTainted(probe)
    "#,
    )
    .unwrap();
}

#[test]
fn public_tainted_calls_preserve_exact_scalars_and_trust() {
    let env = fixture(false);
    positive(&env);
    env.exec(
        r#"
        ACTainted(function()
            assert(C_ActionBar.GetActionUseCount(17) == 7)
            assert(C_ActionBar.GetActionDisplayCount(17, 6, 'é雪') == 'é雪')
            assert(C_ActionBar.GetActionDisplayCount(19) == '2')
        end)
        "#,
    )
    .unwrap();
}

#[test]
fn secure_secret_numeric_slot_threshold_and_string_replacement_are_accepted() {
    let env = secret_fixture();
    use_count(&env, "ACSlot", 7, false);
    display(&env, "ACSlot,ACMax,ACReplacement", "é雪", false);
    display(&env, "ACUnknown,ACMax,ACReplacement", "", false);
    display(&env, "ACSlot,ACNil,ACNil", "7", false);
}

#[test]
fn secure_authenticated_wrong_types_report_argument_positions() {
    let env = secret_fixture();
    env.exec(
        r#"
        ACReject('GetActionUseCount',1,false,ACMax)
        ACReject('GetActionDisplayCount',1,false,ACMax)
        for _,v in ipairs({ACNil,ACBool,ACString,ACSecretTable,ACSecretFrame}) do
            ACReject('GetActionUseCount',1,false,v)
            ACReject('GetActionDisplayCount',1,false,v)
        end
        for _,v in ipairs({ACBool,ACString,ACSecretTable,ACSecretFrame}) do
            ACReject('GetActionDisplayCount',2,false,17,v)
        end
        for _,v in ipairs({ACBool,ACSlot,ACSecretTable,ACSecretFrame}) do
            ACReject('GetActionDisplayCount',3,false,17,6,v)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn all_six_secret_kinds_deny_tainted_slot_before_known_or_missing_lookup() {
    let env = secret_fixture();
    let before = metadata(&env, &SECRETS);
    env.exec(
        r#"
        ACTainted(function()
            for _,v in ipairs(ACSecrets) do
                ACReject('GetActionUseCount',1,true,v)
                ACReject('GetActionDisplayCount',1,true,v)
            end
        end)
        assert(ACFrame:GetAlpha()==0.625); assert(ACFrame.marker==66)
        assert(ACTable.marker=='PRIVATE-Count'); assert(ACTable.count==37)
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &SECRETS), before);
    use_count(&env, "ACSlot", 7, false);
}

#[test]
fn formatter_authentication_precedes_invalid_missing_and_unknown_public_inputs() {
    let env = secret_fixture();
    env.exec(
        r#"
        ACTainted(function()
            for _,v in ipairs(ACSecrets) do
                for _,slot in ipairs({17,18,'bad',false}) do
                    ACReject('GetActionDisplayCount',2,true,slot,v)
                    ACReject('GetActionDisplayCount',3,true,slot,nil,v)
                    ACReject('GetActionDisplayCount',3,true,slot,'bad',v)
                end
                ACReject('GetActionDisplayCount',2,true,nil,v)
                ACReject('GetActionDisplayCount',3,true,nil,nil,v)
            end
        end)
    "#,
    )
    .unwrap();
    display(&env, "ACSlot,ACMax,ACReplacement", "é雪", false);
}

#[test]
fn rooted_secret_inputs_retain_metadata_identity_after_denial_copy_and_gc() {
    let env = secret_fixture();
    let before = metadata(&env, &SECRETS);
    env.exec(
        r#"
        ACTainted(function()
            ACInputCopy={}
            for i,v in ipairs(ACSecrets) do
                ACInputCopy[i]=v
                assert(issecretvalue(v)); assert(not canaccessvalue(v))
                ACReject('GetActionUseCount',1,true,v)
            end
        end)
        collectgarbage('collect')
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &SECRETS), before);
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(copy) = lua.get_global_val("ACInputCopy") else {
        panic!("copy root")
    };
    for (index, (value, _)) in before.iter().enumerate() {
        assert_eq!(
            lua.state_mut()
                .gc
                .tables
                .get(copy)
                .unwrap()
                .get_int(index as i64 + 1),
            *value
        );
    }
    drop(lua);
    display(&env, "ACSlot,ACMax,ACReplacement", "é雪", false);
}

#[test]
fn restricted_outputs_are_actual_typed_string_and_number_with_charge_priority() {
    let env = fixture(false);
    restrict_after_positive(&env);
    set_charge(&env, 3, 5);
    use_count(&env, "17", 7, true);
    display(&env, "17", "3", true);
    display(&env, "17,2,'é雪'", "é雪", true);
}

#[test]
fn restricted_zero_and_no_source_are_secret_scalars_not_nil() {
    let env = fixture(false);
    restrict_after_positive(&env);
    use_count(&env, "18", 0, true);
    display(&env, "18,-1", "", true);
    set_count(&env, 17, 19750, 0);
    use_count(&env, "17", 0, true);
    display(&env, "17", "0", true);
}

#[test]
fn restricted_tainted_public_callers_observe_metadata_not_payloads() {
    let env = fixture(false);
    restrict_after_positive(&env);
    env.exec(
        r#"
        ACTainted(function()
            ACUse=C_ActionBar.GetActionUseCount(17)
            ACDisplay=C_ActionBar.GetActionDisplayCount(17,6,'PRIVATE-Count')
            for _,v in ipairs({ACUse,ACDisplay}) do
                assert(issecretvalue(v)); assert(not canaccessvalue(v))
            end
        end)
        ACResult=ACUse
    "#,
    )
    .unwrap();
    assert_result(&env, Some(7), "", true);
    env.exec("ACResult=ACDisplay").unwrap();
    assert_result(&env, None, "PRIVATE-Count", true);
}

#[test]
fn tainted_math_concatenation_and_scalar_observation_deny_without_private_leaks() {
    let env = fixture(false);
    restrict_after_positive(&env);
    env.exec(
        r#"
        ACUse=C_ActionBar.GetActionUseCount(17)
        ACDisplay=C_ActionBar.GetActionDisplayCount(17,6,'PRIVATE-Count')
        ACTainted(function()
            local operations={
                function() return ACUse+1 end,
                function() return ACDisplay..'x' end,
                function() return string.len(ACDisplay) end,
            }
            for _,operation in ipairs(operations) do
                local ok,err=pcall(operation)
                assert(not ok); assert(type(err)=='string'); assert(#err>0)
                assert(not string.find(err,'PRIVATE-Count',1,true))
                assert(issecretvalue(ACUse)); assert(issecretvalue(ACDisplay))
                assert(debug.getstacktaint()=='ActionCountProbe')
            end
            -- Opaque userdata observations are not native scalar-type parity.
            assert(not string.find(tostring(ACDisplay),'PRIVATE-Count',1,true))
            assert(tostring(ACUse)~='7')
        end)
        ACResult=ACUse
    "#,
    )
    .unwrap();
    assert_result(&env, Some(7), "", true);
    env.exec("ACResult=ACDisplay").unwrap();
    assert_result(&env, None, "PRIVATE-Count", true);
}

#[test]
fn output_copy_gc_and_live_flag_off_preserve_old_wrapper_privacy() {
    let env = fixture(false);
    restrict_after_positive(&env);
    env.exec(
        "ACUse=C_ActionBar.GetActionUseCount(17); ACDisplay=C_ActionBar.GetActionDisplayCount(17)",
    )
    .unwrap();
    let before = metadata(&env, &["ACUse", "ACDisplay"]);
    env.exec(
        r#"
        ACTainted(function() ACCopy={ACUse,ACDisplay} end)
        for i=1,40 do
            ACFreshUse=C_ActionBar.GetActionUseCount(17)
            ACFreshDisplay=C_ActionBar.GetActionDisplayCount(17)
            assert(issecretvalue(ACFreshUse)); assert(issecretvalue(ACFreshDisplay))
            collectgarbage('collect')
        end
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &["ACUse", "ACDisplay"]), before);
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(copy) = lua.get_global_val("ACCopy") else {
        panic!("rooted copy")
    };
    for (index, (value, _)) in before.iter().enumerate() {
        assert_eq!(
            lua.state_mut()
                .gc
                .tables
                .get(copy)
                .unwrap()
                .get_int(index as i64 + 1),
            *value
        );
    }
    drop(lua);
    env.exec("ACResult=ACFreshUse").unwrap();
    assert_result(&env, Some(7), "", true);
    env.exec("ACResult=ACFreshDisplay").unwrap();
    assert_result(&env, None, "7", true);
    env.state().borrow_mut().cooldowns_restricted = false;
    positive(&env);
    env.exec(
        r#"
        ACTainted(function()
            for _,v in ipairs(ACCopy) do assert(issecretvalue(v)); assert(not canaccessvalue(v)) end
            assert(C_ActionBar.GetActionUseCount(17)==7)
            assert(C_ActionBar.GetActionDisplayCount(17)=='7')
        end)
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &["ACUse", "ACDisplay"]), before);
}
