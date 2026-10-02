//! Batch67 exact301/309. Inferred provider/domain/privacy policies, no native parity.
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
    function SCTainted(probe)
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'SpellCountProbe')
            probe()
            assert(debug.getstacktaint() == 'SpellCountProbe')
        end
        debug.setobjecttaint(addon, 'SpellCountProbe')
        addon()
        assert(issecure())
    end
    function SCReject(api, position, denial, ...)
        local before = debug.getstacktaint()
        local ok, err = pcall(C_Spell[api], ...)
        assert(not ok)
        assert(type(err) == 'string')
        assert(#err > 0)
        assert(string.find(err, 'C_Spell.'..api, 1, true))
        assert(string.find(err, tostring(position), 1, true))
        assert(not string.find(err, 'PRIVATE-Count', 1, true))
        local guard = string.find(err, 'requires an untainted caller', 1, true)
        if denial then assert(guard) else assert(not guard) end
        assert(debug.getstacktaint() == before)
    end
    SCFrame = CreateFrame('Frame')
    SCFrame:SetAlpha(0.625)
    SCFrame.marker = 67
    SCTable = {marker='PRIVATE-Count', count=37}
"#;

fn fixture(restricted: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("real spell namespace");
    {
        let mut state = env.state().borrow_mut();
        assert!(state.spell_cast_counts.is_empty());
        state.spell_cast_counts.extend([(19750, 7), (642, 2)]);
        state.spell_id_aliases.clear();
        state.spell_id_aliases.extend([
            ("fixtureheal".into(), 19750),
            ("Flash of Light".into(), 19750),
            ("Divine Shield".into(), 642),
            ("19750".into(), 19750),
            ("642".into(), 642),
        ]);
        state.action_bars.clear();
        state.action_bars.extend([(17, 19750), (19, 19750)]);
        state.action_use_counts.clear();
        state.action_use_counts.extend([
            (
                17,
                ActionUseCountInfo {
                    spell_id: 19750,
                    count: 99,
                },
            ),
            (
                19,
                ActionUseCountInfo {
                    spell_id: 19750,
                    count: 3,
                },
            ),
        ]);
        state.spell_charges.clear();
        state.cooldowns_restricted = restricted;
    }
    env.exec(ASSERTIONS).unwrap();
    env
}

fn set_count(env: &WowLuaEnv, spell_id: u32, count: u32) {
    env.state()
        .borrow_mut()
        .spell_cast_counts
        .insert(spell_id, count);
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
        "assert(select('#', C_Spell.{api}({arguments})) == 1); \
         SCResult = C_Spell.{api}({arguments})"
    ))
    .expect("one actual scalar rooted before host inspection");
}

fn assert_result(env: &WowLuaEnv, number: Option<u32>, text: &str, restricted: bool) {
    env.exec(&format!(
        "assert(issecretvalue(SCResult) == {restricted}); \
         if not {restricted} then assert(type(SCResult) == '{}') end",
        if number.is_some() { "number" } else { "string" }
    ))
    .unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    assert!(rilua::api::state_is_secure(lua.state_mut()));
    let value = lua.get_global_val("SCResult");
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
    capture(env, "GetSpellDisplayCount", arguments);
    assert_result(env, None, expected, restricted);
}

fn cast_count(env: &WowLuaEnv, arguments: &str, expected: u32, restricted: bool) {
    capture(env, "GetSpellCastCount", arguments);
    assert_result(env, Some(expected), "", restricted);
}

fn positive(env: &WowLuaEnv) {
    cast_count(env, "19750", 7, false);
    display(env, "19750", "7", false);
    cast_count(env, "642", 2, false);
    display(env, "642", "2", false);
}

fn publish(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let result = lua.set_global_val(name, value);
    lua.state_mut().pop();
    result.expect("root authentic VM input");
}

const SECRETS: [&str; 10] = [
    "SCIdentifier",
    "SCMax",
    "SCReplacement",
    "SCNil",
    "SCBool",
    "SCString",
    "SCSecretTable",
    "SCSecretFrame",
    "SCUnknown",
    "SCAlias",
];

fn secret_fixture() -> WowLuaEnv {
    let env = fixture(false);
    positive(&env); // Meaningful provider proof before authentication/guard assertions.
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    for (name, number) in [
        ("SCIdentifier", 19750.0),
        ("SCMax", 6.5),
        ("SCUnknown", 880001.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        publish(&mut *lua, name, value);
    }
    for (name, text) in [
        ("SCReplacement", "é雪"),
        ("SCString", "PRIVATE-Count"),
        ("SCAlias", "fixtureheal"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        publish(&mut *lua, name, value);
    }
    let value = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish(&mut *lua, "SCNil", value);
    let value = wrap_host_secret_bool(lua.state_mut(), false);
    publish(&mut *lua, "SCBool", value);
    for (source, name) in [("SCTable", "SCSecretTable"), ("SCFrame", "SCSecretFrame")] {
        let value = lua.get_global_val(source);
        let Val::Table(reference) = value else {
            panic!("real table/frame")
        };
        if source == "SCFrame" {
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
    env.exec("SCSecrets = {SCIdentifier,SCMax,SCReplacement,SCNil,SCBool,SCString,SCSecretTable,SCSecretFrame,SCUnknown,SCAlias}; for _,v in ipairs(SCSecrets) do assert(issecretvalue(v)) end").unwrap();
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
fn empty_default_inputs_produce_two_positive_spell_keyed_scalar_domains() {
    positive(&fixture(false));
}

#[test]
fn actual_names_and_explicit_aliases_use_same_spell_counts() {
    let env = fixture(false);
    for id in ["19750", "'19750'", "'fixtureheal'", "'Flash of Light'"] {
        cast_count(&env, id, 7, false);
        display(&env, id, "7", false);
    }
    cast_count(&env, "'Divine Shield'", 2, false);
    display(&env, "'642'", "2", false);
}

#[test]
fn alias_first_numeric_and_string_overrides_follow_caller_identity() {
    let env = fixture(false);
    positive(&env);
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("19750".into(), 642);
    for id in ["19750", "'19750'"] {
        cast_count(&env, id, 2, false);
        display(&env, id, "2", false);
    }
    cast_count(&env, "'fixtureheal'", 7, false);
    display(&env, "'fixtureheal'", "7", false);
}

#[test]
fn public_numeric_identity_can_read_host_count_without_catalog_acquisition() {
    let env = fixture(false);
    set_count(&env, 880001, 11);
    set_count(&env, 0, 4);
    for (id, count, text) in [("880001", 11, "11"), ("0", 4, "4")] {
        cast_count(&env, id, count, false);
        display(&env, id, text, false);
    }
}

#[test]
fn unresolved_strings_never_reverse_numeric_keys_or_alias_values() {
    let env = fixture(false);
    set_count(&env, 880001, 11);
    for id in ["'880001'", "'unknown-spell'", "''"] {
        cast_count(&env, id, 0, false);
        display(&env, id, "", false);
    }
    env.state().borrow_mut().spell_id_aliases.remove("19750");
    cast_count(&env, "'19750'", 0, false);
    display(&env, "'19750'", "", false);
    cast_count(&env, "19750", 7, false);
}

#[test]
fn same_spell_two_slots_with_99_and_3_cannot_supply_spell_count_seven() {
    let env = fixture(false);
    positive(&env);
    env.state().borrow_mut().spell_cast_counts.remove(&19750);
    cast_count(&env, "19750", 0, false);
    display(&env, "19750", "", false);
    set_count(&env, 19750, 7);
    env.state().borrow_mut().action_bars.clear();
    env.state().borrow_mut().action_use_counts.clear();
    positive(&env);
}

#[test]
fn missing_known_and_unknown_numeric_sources_return_zero_and_empty() {
    let env = fixture(false);
    positive(&env);
    env.state().borrow_mut().spell_cast_counts.remove(&19750);
    for id in ["19750", "880001", "4294967295"] {
        cast_count(&env, id, 0, false);
        display(&env, id, "", false);
    }
}

#[test]
fn explicit_zero_displays_zero_not_absence_even_with_negative_threshold() {
    let env = fixture(false);
    set_count(&env, 19750, 0);
    cast_count(&env, "19750", 0, false);
    display(&env, "19750", "0", false);
    display(&env, "19750,-1", "*", false);
    display(&env, "880001,-1", "", false);
}

#[test]
fn count_replace_and_clear_are_live_without_cast_consumption() {
    let env = fixture(false);
    positive(&env);
    set_count(&env, 19750, 12);
    cast_count(&env, "19750", 12, false);
    display(&env, "19750", "12", false);
    env.state().borrow_mut().spell_cast_counts.clear();
    cast_count(&env, "19750", 0, false);
    display(&env, "642", "", false);
    set_count(&env, 642, 8);
    cast_count(&env, "642", 8, false);
    display(&env, "642", "8", false);
}

#[test]
fn typed_valid_charges_override_display_only_cast_stays_seven() {
    let env = fixture(false);
    set_charge(&env, 3, 5);
    cast_count(&env, "19750", 7, false);
    display(&env, "19750", "3", false);
    display(&env, "642", "2", false);
}

#[test]
fn zero_max_is_ignored_and_charge_zero_updates_and_clear_are_live() {
    let env = fixture(false);
    set_charge(&env, 3, 0);
    display(&env, "19750", "7", false);
    set_charge(&env, 0, 5);
    display(&env, "19750", "0", false);
    set_charge(&env, 4, 5);
    display(&env, "19750", "4", false);
    env.state().borrow_mut().spell_charges.clear();
    display(&env, "19750", "7", false);
    cast_count(&env, "19750", 7, false);
}

#[test]
fn charges_alone_never_supply_cast_counts() {
    let env = fixture(false);
    env.state().borrow_mut().spell_cast_counts.remove(&19750);
    set_charge(&env, 3, 5);
    cast_count(&env, "19750", 0, false);
    display(&env, "19750", "3", false);
}

#[test]
fn default_threshold_9999_keeps_digits_10000_replaces() {
    let env = fixture(false);
    set_count(&env, 19750, 9999);
    display(&env, "19750", "9999", false);
    set_count(&env, 19750, 10000);
    cast_count(&env, "19750", 10000, false);
    display(&env, "19750", "*", false);
}

#[test]
fn strict_beyond_threshold_equality_fractional_negative_and_charge_quantity() {
    let env = fixture(false);
    for (args, expected) in [
        ("19750,6", "*"),
        ("19750,7", "7"),
        ("19750,6.5", "*"),
        ("19750,-1", "*"),
    ] {
        display(&env, args, expected, false);
    }
    set_charge(&env, 3, 5);
    display(&env, "19750,3", "3", false);
    display(&env, "19750,2.5", "*", false);
}

#[test]
fn omitted_and_nil_format_arguments_use_documented_defaults() {
    let env = fixture(false);
    for args in ["19750", "19750,nil", "19750,nil,nil"] {
        display(&env, args, "7", false);
    }
    display(&env, "19750,6,nil", "*", false);
}

#[test]
fn decimal_ascii_and_empty_unicode_replacement_preserve_exact_bytes() {
    let env = fixture(false);
    display(&env, "19750,6,''", "", false);
    display(&env, "19750,6,'é雪'", "é雪", false);
    display(&env, "19750,7,'é雪'", "7", false);
    set_count(&env, 19750, 1234);
    display(&env, "19750", "1234", false);
}

#[test]
fn local_u32_width_preserves_max_quantity_and_exact_decimal() {
    let env = fixture(false);
    set_count(&env, 19750, u32::MAX);
    cast_count(&env, "19750", u32::MAX, false);
    display(&env, "19750,4294967295", "4294967295", false);
    display(&env, "19750,4294967294.5,'limit'", "limit", false);
    display(&env, "19750", "*", false);
}

#[test]
fn queries_are_read_only_and_environments_and_result_replacements_isolated() {
    let env = fixture(false);
    let other = fixture(false);
    set_charge(&env, 3, 5);
    let (counts, charges, aliases, bars, slots) = {
        let state = env.state().borrow();
        (
            state.spell_cast_counts.clone(),
            state.spell_charges.clone(),
            state.spell_id_aliases.clone(),
            state.action_bars.clone(),
            state.action_use_counts.clone(),
        )
    };
    env.exec("local c=C_Spell.GetSpellCastCount; local d=C_Spell.GetSpellDisplayCount; for i=1,8 do assert(c('fixtureheal')==7); assert(d('fixtureheal')=='3') end; SCResult='replaced'; assert(d(19750)=='3')").unwrap();
    {
        let state = env.state().borrow();
        assert_eq!(state.spell_cast_counts, counts);
        assert_eq!(state.spell_charges, charges);
        assert_eq!(state.spell_id_aliases, aliases);
        assert_eq!(state.action_bars, bars);
        assert_eq!(state.action_use_counts, slots);
    }
    set_count(&env, 642, 9);
    positive(&other);
}

#[test]
fn strict_public_identifier_utf8_u32_domain_in_secure_and_tainted_frames() {
    let env = fixture(false);
    positive(&env);
    env.exec(r#"
        local function probe()
            for _,api in ipairs({'GetSpellCastCount','GetSpellDisplayCount'}) do
                SCReject(api,1,false)
                SCReject(api,1,false,nil)
                for _,v in ipairs({false,{},SCFrame,-1,19750.5,0/0,math.huge,-math.huge,4294967296,string.char(255)}) do
                    SCReject(api,1,false,v)
                end
            end
        end
        probe(); SCTainted(probe)
    "#).unwrap();
}

#[test]
fn finite_threshold_and_utf8_nul_free_cstring_validate_without_source() {
    let env = fixture(false);
    positive(&env);
    env.exec(
        r#"
        local function probe()
            for _,id in ipairs({19750,880001,'unknown-spell'}) do
                for _,v in ipairs({false,'6',{},SCFrame,0/0,math.huge,-math.huge}) do
                    SCReject('GetSpellDisplayCount',2,false,id,v)
                end
                for _,v in ipairs({false,7,{},SCFrame,'a\000b',string.char(255)}) do
                    SCReject('GetSpellDisplayCount',3,false,id,6,v)
                end
            end
        end
        probe(); SCTainted(probe)
    "#,
    )
    .unwrap();
}

#[test]
fn public_tainted_callers_are_not_blanket_denied_and_recover_trust() {
    let env = fixture(false);
    positive(&env);
    env.exec("SCTainted(function() assert(C_Spell.GetSpellCastCount('fixtureheal')==7); assert(C_Spell.GetSpellDisplayCount(19750,6,'é雪')=='é雪'); assert(C_Spell.GetSpellDisplayCount(642)=='2') end)").unwrap();
}

#[test]
fn secure_display_authenticates_secret_numeric_string_identifier_and_defaults() {
    let env = secret_fixture();
    display(&env, "SCIdentifier,SCMax,SCReplacement", "é雪", false);
    display(&env, "SCAlias", "7", false);
    display(&env, "SCUnknown,SCMax,SCReplacement", "", false);
    display(&env, "SCString", "", false);
    display(&env, "SCIdentifier,SCNil,SCNil", "7", false);
}

#[test]
fn secure_display_wrong_authenticated_types_report_positions_without_payload() {
    let env = secret_fixture();
    env.exec(
        r#"
        SCReject('GetSpellDisplayCount',1,false,SCMax)
        for _,v in ipairs({SCNil,SCBool,SCSecretTable,SCSecretFrame}) do
            SCReject('GetSpellDisplayCount',1,false,v)
        end
        for _,v in ipairs({SCBool,SCString,SCSecretTable,SCSecretFrame}) do
            SCReject('GetSpellDisplayCount',2,false,19750,v)
        end
        for _,v in ipairs({SCBool,SCIdentifier,SCSecretTable,SCSecretFrame}) do
            SCReject('GetSpellDisplayCount',3,false,19750,6,v)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn cast_conservatively_rejects_all_six_authentic_secret_kinds_in_both_frames() {
    let env = secret_fixture();
    let before = metadata(&env, &SECRETS);
    env.exec(
        r#"
        local function probe()
            for _,v in ipairs(SCSecrets) do
                local taint=debug.getstacktaint()
                local ok,err=pcall(C_Spell.GetSpellCastCount,v)
                assert(not ok); assert(type(err)=='string')
                assert(string.find(err,'C_Spell.GetSpellCastCount',1,true))
                assert(string.find(err,'secret spell identifier access is not modeled',1,true))
                assert(not string.find(err,'PRIVATE-Count',1,true))
                assert(debug.getstacktaint()==taint)
            end
        end
        probe(); SCTainted(probe)
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &SECRETS), before);
    positive(&env);
}

#[test]
fn display_authenticates_all_original_args_before_types_lookup_or_no_source() {
    let env = secret_fixture();
    env.exec(
        r#"
        SCTainted(function()
            for _,v in ipairs(SCSecrets) do
                SCReject('GetSpellDisplayCount',1,true,v)
                for _,id in ipairs({19750,880001,'unknown-spell',false}) do
                    SCReject('GetSpellDisplayCount',2,true,id,v)
                    SCReject('GetSpellDisplayCount',3,true,id,nil,v)
                    SCReject('GetSpellDisplayCount',3,true,id,'bad',v)
                end
                SCReject('GetSpellDisplayCount',2,true,nil,v)
                SCReject('GetSpellDisplayCount',3,true,nil,nil,v)
            end
            SCReject('GetSpellDisplayCount',1,true,SCNil,SCMax,SCReplacement)
            SCReject('GetSpellDisplayCount',2,true,'unknown-spell',SCBool,SCReplacement)
        end)
    "#,
    )
    .unwrap();
    display(&env, "SCAlias,SCMax,SCReplacement", "é雪", false);
}

#[test]
fn rooted_input_copies_and_gc_preserve_allocation_secrecy_and_frame_state() {
    let env = secret_fixture();
    let before = metadata(&env, &SECRETS);
    env.exec(
        r#"
        SCTainted(function()
            SCInputCopy={}
            for i,v in ipairs(SCSecrets) do
                SCInputCopy[i]=v
                assert(issecretvalue(v)); assert(not canaccessvalue(v))
                SCReject('GetSpellDisplayCount',1,true,v)
            end
        end)
        collectgarbage('collect')
        assert(SCFrame:GetAlpha()==0.625); assert(SCFrame.marker==67)
        assert(SCTable.marker=='PRIVATE-Count'); assert(SCTable.count==37)
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &SECRETS), before);
    assert_copies(&env, "SCInputCopy", &before);
    display(&env, "SCAlias", "7", false);
    positive(&env);
}

fn assert_copies(env: &WowLuaEnv, name: &str, originals: &[(Val, u64)]) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    assert!(rilua::api::state_is_secure(lua.state_mut()));
    let Val::Table(copy) = lua.get_global_val(name) else {
        panic!("rooted copy")
    };
    for (index, (value, _)) in originals.iter().enumerate() {
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
}

#[test]
fn restricted_positive_cast_and_charge_display_have_real_num_str_payloads() {
    let env = fixture(false);
    restrict_after_positive(&env);
    cast_count(&env, "19750", 7, true);
    display(&env, "19750", "7", true);
    set_charge(&env, 3, 5);
    cast_count(&env, "19750", 7, true);
    display(&env, "19750", "3", true);
    display(&env, "19750,2,'é雪'", "é雪", true);
}

#[test]
fn restricted_missing_zero_and_empty_replacement_remain_typed_secret_scalars() {
    let env = fixture(false);
    restrict_after_positive(&env);
    for id in ["880001", "'unknown-spell'"] {
        cast_count(&env, id, 0, true);
        display(&env, id, "", true);
    }
    set_count(&env, 19750, 0);
    cast_count(&env, "19750", 0, true);
    display(&env, "19750", "0", true);
    display(&env, "642,1,''", "", true);
}

#[test]
fn tainted_restricted_outputs_deny_math_concat_len_and_opaque_payload_leaks() {
    let env = fixture(false);
    restrict_after_positive(&env);
    env.exec(
        r#"
        SCTainted(function()
            SCCast=C_Spell.GetSpellCastCount(19750)
            SCDisplay=C_Spell.GetSpellDisplayCount(19750,6,'PRIVATE-Count')
            for _,v in ipairs({SCCast,SCDisplay}) do
                assert(issecretvalue(v)); assert(not canaccessvalue(v))
            end
            for _,op in ipairs({function() return SCCast+1 end,
                function() return SCDisplay..'x' end,
                function() return string.len(SCDisplay) end}) do
                local ok,err=pcall(op)
                assert(not ok); assert(type(err)=='string'); assert(#err>0)
                assert(not string.find(err,'PRIVATE-Count',1,true))
                assert(debug.getstacktaint()=='SpellCountProbe')
            end
            assert(not string.find(tostring(SCDisplay),'PRIVATE-Count',1,true))
            assert(tostring(SCCast)~='7')
        end)
        SCResult=SCCast
    "#,
    )
    .unwrap();
    assert_result(&env, Some(7), "", true);
    env.exec("SCResult=SCDisplay").unwrap();
    assert_result(&env, None, "PRIVATE-Count", true);
}

#[test]
fn output_copy_gc_and_flag_off_keep_old_wrappers_private_and_new_public() {
    let env = fixture(false);
    restrict_after_positive(&env);
    env.exec(
        "SCCast=C_Spell.GetSpellCastCount(19750); SCDisplay=C_Spell.GetSpellDisplayCount(19750)",
    )
    .unwrap();
    let before = metadata(&env, &["SCCast", "SCDisplay"]);
    env.exec(
        r#"
        SCTainted(function() SCOutputCopy={SCCast,SCDisplay} end)
        for i=1,40 do
            SCFreshCast=C_Spell.GetSpellCastCount(19750)
            SCFreshDisplay=C_Spell.GetSpellDisplayCount(19750)
            assert(issecretvalue(SCFreshCast)); assert(issecretvalue(SCFreshDisplay))
            collectgarbage('collect')
        end
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &["SCCast", "SCDisplay"]), before);
    assert_copies(&env, "SCOutputCopy", &before);
    env.exec("SCResult=SCFreshCast").unwrap();
    assert_result(&env, Some(7), "", true);
    env.exec("SCResult=SCFreshDisplay").unwrap();
    assert_result(&env, None, "7", true);
    env.state().borrow_mut().cooldowns_restricted = false;
    positive(&env);
    env.exec(r#"
        SCTainted(function()
            for _,v in ipairs(SCOutputCopy) do assert(issecretvalue(v)); assert(not canaccessvalue(v)) end
            assert(C_Spell.GetSpellCastCount(19750)==7)
            assert(C_Spell.GetSpellDisplayCount(19750)=='7')
        end)
    "#).unwrap();
    assert_eq!(metadata(&env, &["SCCast", "SCDisplay"]), before);
}
