//! Retail 12.1.5 count widths, reagent traversal and bounded input controls.
//! Property decoding and detailed native item eligibility remain unasserted.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::c_api::{ActionUseCountInfo, charge_state::SpellChargeState};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::{WowLuaEnv, state::BagItem};

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
        local guard = string.find(err, 'Secret values', 1, true)
            or string.find(err, 'requires an untainted caller', 1, true)
        if denial then
            assert(guard)
        else
            assert(not guard)
        end
        assert(debug.getstacktaint() == before)
    end
    SCFrame = CreateFrame('Frame')
    SCFrame:SetAlpha(0.625)
    SCFrame.marker = 67
    SCTable = {
        marker = 'PRIVATE-Count',
        count = 37,
    }
"#;

// INFERRED: PTR uses the Retail R88p contract; no PTR native proof is claimed.
fn uses_retail_native_count_contract() -> bool {
    cfg!(feature = "retail-12-1-5") && matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr)
}

fn fixture(restricted: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("real spell namespace");
    {
        let mut state = env.state().borrow_mut();
        assert!(state.spell_cast_counts.is_empty());
        if uses_retail_native_count_contract() {
            state.spell_cast_context_is_null = Some(false);
            state.bag_items.clear();
            state.spell_reagent_pairs.clear();
            set_reagent_count(&mut state, 19750, 7);
            set_reagent_count(&mut state, 642, 2);
        } else {
            state.spell_cast_counts.extend([(19750, 7), (642, 2)]);
        }
        state.spell_id_aliases.clear();
        state.spell_id_aliases.extend([
            ("fixtureheal".into(), 19750),
            ("flash of light".into(), 19750),
            ("divine shield".into(), 642),
            ("19750".into(), 19750),
            ("642".into(), 642),
        ]);
        populate_action_slots(&mut state);
        state.spell_charges.clear();
        state.cooldowns_restricted = restricted;
    }
    env.exec(ASSERTIONS).unwrap();
    env
}

fn populate_action_slots(state: &mut wow_ui_sim::lua_api::SimState) {
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
}

fn reagent_item_id(spell_id: u32) -> u32 {
    spell_id % i32::MAX as u32 + 1
}

fn set_reagent_count(state: &mut wow_ui_sim::lua_api::SimState, spell_id: u32, count: u32) {
    let item_id = reagent_item_id(spell_id);
    state
        .spell_reagent_pairs
        .insert(spell_id, vec![(item_id as i32, 1)]);
    state.bag_items.retain(|_, item| item.item_id != item_id);

    let mut remaining = count;
    let mut slot = 1;
    while remaining > 0 {
        while state.bag_items.contains_key(&(0, slot)) {
            slot += 1;
        }
        let stack_count = remaining.min(i32::MAX as u32);
        state.bag_items.insert(
            (0, slot),
            BagItem {
                owned_identity: None,
                item_id,
                stack_count: stack_count as i32,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
            },
        );
        remaining -= stack_count;
        slot += 1;
    }
}

fn set_count(env: &WowLuaEnv, spell_id: u32, count: u32) {
    let mut state = env.state().borrow_mut();
    if uses_retail_native_count_contract() {
        set_reagent_count(&mut state, spell_id, count);
    } else {
        state.spell_cast_counts.insert(spell_id, count);
    }
}

fn clear_count(env: &WowLuaEnv, spell_id: u32) {
    let mut state = env.state().borrow_mut();
    if uses_retail_native_count_contract() {
        let item_id = reagent_item_id(spell_id);
        state.spell_reagent_pairs.remove(&spell_id);
        state.bag_items.retain(|_, item| item.item_id != item_id);
    } else {
        state.spell_cast_counts.remove(&spell_id);
    }
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
        "assert(select('#', C_Spell.{api}({arguments})) == 1)\n\
         SCResult = C_Spell.{api}({arguments})"
    ))
    .expect("one actual scalar rooted before host inspection");
}

fn assert_lua_result_domain(env: &WowLuaEnv, number: Option<u32>, restricted: bool) {
    env.exec(&format!(
        "assert(issecretvalue(SCResult) == {restricted})\n\
         if not {restricted} then\n\
             assert(type(SCResult) == '{}')\n\
         end",
        if number.is_some() { "number" } else { "string" }
    ))
    .unwrap();
}

fn assert_result(env: &WowLuaEnv, number: Option<u32>, text: &str, restricted: bool) {
    assert_lua_result_domain(env, number, restricted);
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
    match (payload, number) {
        (Val::Num(actual), Some(expected)) => assert_eq!(actual, f64::from(expected)),
        (Val::Str(actual), None) => assert_eq!(
            lua.state_mut()
                .gc
                .string_arena
                .get(actual)
                .expect("actual Lua string payload")
                .data(),
            text.as_bytes(),
            "actual ValNum/ValStr UTF8 payload; no secret nominal Lua type claim"
        ),
        (actual, expected) => panic!("unexpected payload {actual:?} for {expected:?}"),
    }
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
    publish_secret_scalars(&mut *lua);
    publish_secret_tables(&mut *lua);
    drop(lua);
    publish_secret_metadata(&env);
    env
}

fn publish_secret_metadata(env: &WowLuaEnv) {
    env.exec(
        r#"
        SCSecrets = {
            SCIdentifier,
            SCMax,
            SCReplacement,
            SCNil,
            SCBool,
            SCString,
            SCSecretTable,
            SCSecretFrame,
            SCUnknown,
            SCAlias,
        }
        for _,v in ipairs(SCSecrets) do
            assert(issecretvalue(v))
        end
        "#,
    )
    .unwrap();
}

fn publish_secret_scalars(lua: &mut impl LuaApiMut) {
    for (name, number) in [
        ("SCIdentifier", 19750.0),
        ("SCMax", 6.5),
        ("SCUnknown", 880001.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        publish(lua, name, value);
    }
    for (name, text) in [
        ("SCReplacement", "é雪"),
        ("SCString", "PRIVATE-Count"),
        ("SCAlias", "fixtureheal"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        publish(lua, name, value);
    }
    let value = wrap_secret(lua.state_mut(), Val::Nil).unwrap();
    publish(lua, "SCNil", value);
    let value = wrap_host_secret_bool(lua.state_mut(), false);
    publish(lua, "SCBool", value);
}

fn publish_secret_tables(lua: &mut impl LuaApiMut) {
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
        publish(lua, name, wrapped);
        lua.state_mut().pop();
    }
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
fn r88r_numeric_and_spell_text_identifiers_bypass_name_replacement() {
    // VERIFIED Retail R88r.r2, lt-67d2 #1239; PTR applicability remains INFERRED.
    let env = fixture(false);
    if !uses_retail_native_count_contract() {
        return;
    }
    for (id, wrong) in [(19750, 642), (642, 19750), (880001, 642)] {
        for text in [
            id.to_string(),
            format!("spell:{id}"),
            format!("|Hspell:{id}:0|h[x]|h"),
        ] {
            env.state()
                .borrow_mut()
                .spell_id_aliases
                .insert(text, wrong);
        }
    }
    for (id, count, text) in [(19750, 7, "7"), (642, 2, "2"), (880001, 0, "")] {
        for input in [
            id.to_string(),
            format!("'{id}'"),
            format!("'spell:{id}'"),
            format!("'|Hspell:{id}:0|h[x]|h'"),
        ] {
            cast_count(&env, &input, count, false);
            display(&env, &input, text, false);
        }
    }
}

#[test]
fn retail_cast_count_requires_spell_store_membership() {
    let env = fixture(false);
    set_count(&env, 880001, 11);
    set_count(&env, 0, 4);
    if uses_retail_native_count_contract() {
        env.state()
            .borrow_mut()
            .spell_cast_counts
            .insert(880001, 11);
        for id in ["880001", "0"] {
            cast_count(&env, id, 0, false);
            display(&env, id, "0", false);
        }
    } else {
        for (id, count, text) in [("880001", 11, "11"), ("0", 4, "4")] {
            cast_count(&env, id, count, false);
            display(&env, id, text, false);
        }
    }
}

#[test]
fn unresolved_strings_never_reverse_numeric_keys_or_alias_values() {
    let env = fixture(false);
    set_count(&env, 880001, 11);
    if uses_retail_native_count_contract() {
        cast_count(&env, "'880001'", 0, false);
        display(&env, "'880001'", "0", false);
    } else {
        cast_count(&env, "'880001'", 0, false);
        display(&env, "'880001'", "", false);
    }
    for id in ["'unknown-spell'", "''"] {
        cast_count(&env, id, 0, false);
        display(&env, id, "", false);
    }
    env.state().borrow_mut().spell_id_aliases.remove("19750");
    if uses_retail_native_count_contract() {
        cast_count(&env, "'19750'", 7, false);
        display(&env, "'19750'", "7", false);
    } else {
        cast_count(&env, "'19750'", 0, false);
        display(&env, "'19750'", "", false);
    }
    cast_count(&env, "19750", 7, false);
}

#[test]
fn same_spell_two_slots_with_99_and_3_cannot_supply_spell_count_seven() {
    let env = fixture(false);
    positive(&env);
    clear_count(&env, 19750);
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
    clear_count(&env, 19750);
    for id in ["19750", "880001"] {
        cast_count(&env, id, 0, false);
        display(&env, id, "", false);
    }
    // UNKNOWN R88k.5 line 11623 / R88p.p2 (#1231): do not assert 4294967295
    // conversion outcomes or the old maximum-style range error for identifiers.
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
    clear_count(&env, 19750);
    clear_count(&env, 642);
    cast_count(&env, "19750", 0, false);
    display(&env, "642", "", false);
    set_count(&env, 642, 8);
    cast_count(&env, "642", 8, false);
    display(&env, "642", "8", false);
}

#[test]
fn retail_reagent_mode_precedes_charge_mode_without_changing_cast_count() {
    let env = fixture(false);
    set_charge(&env, 3, 5);
    cast_count(&env, "19750", 7, false);
    if uses_retail_native_count_contract() {
        display(&env, "19750", "7", false);
        clear_count(&env, 19750);
        display(&env, "19750", "3", false);
    } else {
        display(&env, "19750", "3", false);
    }
    display(&env, "642", "2", false);
}

#[test]
fn zero_max_is_ignored_and_charge_zero_updates_and_clear_are_live() {
    let env = fixture(false);
    if uses_retail_native_count_contract() {
        clear_count(&env, 19750);
    }
    set_charge(&env, 3, 0);
    let no_usable_charge = if uses_retail_native_count_contract() {
        ""
    } else {
        "7"
    };
    display(&env, "19750", no_usable_charge, false);
    set_charge(&env, 0, 5);
    display(&env, "19750", "0", false);
    set_charge(&env, 4, 5);
    display(&env, "19750", "4", false);
    env.state().borrow_mut().spell_charges.clear();
    display(&env, "19750", no_usable_charge, false);
    let cast_count_without_reagent = if uses_retail_native_count_contract() {
        0
    } else {
        7
    };
    cast_count(&env, "19750", cast_count_without_reagent, false);
}

#[test]
fn charges_alone_never_supply_cast_counts() {
    let env = fixture(false);
    clear_count(&env, 19750);
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
    if uses_retail_native_count_contract() {
        clear_count(&env, 19750);
        display(&env, "19750,3", "3", false);
        display(&env, "19750,2.5", "3", false);
    } else {
        display(&env, "19750,3", "3", false);
        display(&env, "19750,2.5", "*", false);
    }
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

// INFERRED PTR applicability, isolated from the Retail evidence commit.
#[cfg(feature = "client-ptr")]
#[test]
fn inferred_ptr_signed_maximum_preserves_valid_count_decimal() {
    let env = fixture(false);
    set_count(&env, 19750, u32::MAX);
    cast_count(&env, "19750", u32::MAX, false);
    env.exec("SCReject('GetSpellDisplayCount',2,false,19750,4294967295)")
        .unwrap();
    env.exec("SCReject('GetSpellDisplayCount',2,false,19750,4294967294.5)")
        .unwrap();
    set_count(&env, 19750, i32::MAX as u32);
    display(&env, "19750,2147483647", "2147483647", false);
    display(&env, "642,2147483647", "2", false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[test]
fn queries_are_read_only_and_environments_and_result_replacements_isolated() {
    let env = fixture(false);
    let other = fixture(false);
    set_charge(&env, 3, 5);
    let (counts, charges, aliases, bars, slots, reagents, bags) = {
        let state = env.state().borrow();
        (
            state.spell_cast_counts.clone(),
            state.spell_charges.clone(),
            state.spell_id_aliases.clone(),
            state.action_bars.clone(),
            state.action_use_counts.clone(),
            state.spell_reagent_pairs.clone(),
            state
                .bag_items
                .iter()
                .map(|(slot, item)| (*slot, (item.item_id, item.stack_count)))
                .collect::<std::collections::BTreeMap<_, _>>(),
        )
    };
    let display_count = if uses_retail_native_count_contract() {
        "7"
    } else {
        "3"
    };
    env.exec(&format!(
        r#"
        local c = C_Spell.GetSpellCastCount
        local d = C_Spell.GetSpellDisplayCount
        for i=1,8 do
            assert(c('fixtureheal')==7)
            assert(d('fixtureheal')=='{display_count}')
        end
        SCResult = 'replaced'
        assert(d(19750)=='{display_count}')
    "#
    ))
    .unwrap();
    {
        let state = env.state().borrow();
        assert_eq!(state.spell_cast_counts, counts);
        assert_eq!(state.spell_charges, charges);
        assert_eq!(state.spell_id_aliases, aliases);
        assert_eq!(state.action_bars, bars);
        assert_eq!(state.action_use_counts, slots);
        assert_eq!(state.spell_reagent_pairs, reagents);
        let current_bags = state
            .bag_items
            .iter()
            .map(|(slot, item)| (*slot, (item.item_id, item.stack_count)))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(current_bags, bags);
    }
    set_count(&env, 642, 9);
    positive(&other);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn retail_identifier_conversion_matches_the_signed32_subset() {
    // R88k.5 line 11623 / R88p.p2: exceptional outcomes depend on UNKNOWN FP configuration.
    // Keep only VERIFIED represented numbers and ordinary type errors.
    let env = fixture(false);
    cast_count(&env, "19750.5", 7, false);
    cast_count(&env, "642.5", 2, false);
    cast_count(&env, "-1", 0, false);
    display(&env, "19750.5", "7", false);
    display(&env, "642.5", "2", false);
    cast_count(&env, "'19750.5'", 7, false);
    display(&env, "'19750.5'", "7", false);
    env.exec(
        r#"
        for _,api in ipairs({'GetSpellCastCount','GetSpellDisplayCount'}) do
            SCReject(api,1,false)
            SCReject(api,1,false,nil)
            for _,value in ipairs({false, {}, SCFrame}) do
                SCReject(api,1,false,value)
            end
        end
    "#,
    )
    .unwrap();
}

// INFERRED PTR applicability of R88p.p2; old strict-u32 rejection contradicted it.
#[cfg(feature = "client-ptr")]
#[test]
fn inferred_ptr_signed_identifiers_truncate_and_accept_negative_absence() {
    let env = fixture(false);
    cast_count(&env, "19750.9", 7, false);
    cast_count(&env, "642.9", 2, false);
    cast_count(&env, "-1", 0, false);
    display(&env, "19750.9", "7", false);
    display(&env, "642.9", "2", false);
}

// VERIFIED R88k.5 line 11622 (lt-c3ed #1056, confirmed lt-67d2 #1226):
// numeric maximum strings and ordinary numeric replacement text are accepted.
const INVALID_SPELL_COUNT_FORMAT_ARGUMENTS: &str = r#"
        local function probe()
            for _,id in ipairs({
                19750,
                880001,
                'unknown-spell',
            }) do
                for _,v in ipairs({
                    false,
                    'not-a-number',
                    {},
                    SCFrame,
                    0/0,
                    math.huge,
                    -math.huge,
                }) do
                    SCReject('GetSpellDisplayCount',2,false,id,v)
                end
                for _,v in ipairs({
                    false,
                    {},
                    SCFrame,
                    'a\000b',
                    string.char(255),
                }) do
                    SCReject('GetSpellDisplayCount',3,false,id,6,v)
                end
            end
        end
        probe()
        SCTainted(probe)
    "#;

#[test]
fn finite_threshold_and_utf8_nul_free_cstring_validate_without_source() {
    let env = fixture(false);
    positive(&env);
    display(&env, "19750,'6'", "*", false);
    display(&env, "642,'2'", "2", false);
    display(&env, "880001,'6'", "", false);
    env.exec(INVALID_SPELL_COUNT_FORMAT_ARGUMENTS).unwrap();
    env.exec(r#"
        for _,maximum in ipairs({4294967295, -2147483649, 2147483647.5, -2147483648.5}) do
            local ok,err = pcall(C_Spell.GetSpellDisplayCount, 19750, maximum)
            assert(not ok)
            assert(string.find(err, 'outside of expected range -2147483648 to 2147483647 - Usage:', 1, true))
        end
    "#).unwrap();
}

#[test]
fn public_tainted_callers_are_not_blanket_denied_and_recover_trust() {
    let env = fixture(false);
    positive(&env);
    env.exec(
        r#"
        SCTainted(function()
            assert(C_Spell.GetSpellCastCount('fixtureheal')==7)
            assert(C_Spell.GetSpellDisplayCount(19750,6,'é雪')=='é雪')
            assert(C_Spell.GetSpellDisplayCount(642)=='2')
        end)
    "#,
    )
    .unwrap();
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

// R88k.5 line 11623 and R88p line 12450 (lt-67d2 #1226): access is
// VERIFIED; authenticated fractional conversion remains UNKNOWN.
const WRONG_AUTHENTICATED_DISPLAY_TYPES: &str = r#"
        -- UNKNOWN R88k.5 line 11623: authenticated fractional conversion.
        for _,v in ipairs({
            SCNil,
            SCBool,
            SCSecretTable,
            SCSecretFrame,
        }) do
            SCReject('GetSpellDisplayCount',1,false,v)
        end
        for _,v in ipairs({
            SCBool,
            SCString,
            SCSecretTable,
            SCSecretFrame,
        }) do
            SCReject('GetSpellDisplayCount',2,false,19750,v)
        end
        for _,v in ipairs({
            SCBool,
            SCIdentifier,
            SCSecretTable,
            SCSecretFrame,
        }) do
            SCReject('GetSpellDisplayCount',3,false,19750,6,v)
        end
    "#;

#[test]
fn secure_display_wrong_authenticated_types_report_positions_without_payload() {
    let env = secret_fixture();
    env.exec(WRONG_AUTHENTICATED_DISPLAY_TYPES).unwrap();
}

const NUMERIC_SECRET_CAST_ACCESS: &str = r#"
        local function probe()
            local taint=debug.getstacktaint()
            assert(select('#',C_Spell.GetSpellCastCount(SCIdentifier))==1)
            assert(C_Spell.GetSpellCastCount(SCIdentifier)==7)
            assert(C_Spell.GetSpellCastCount(SCUnknown)==0)
            -- UNKNOWN R88k.5 line 11623: no fractional-secret conversion claim.
            for _,v in ipairs({SCReplacement,SCNil,SCBool,SCString,SCSecretTable,SCSecretFrame,SCAlias}) do
                local ok,err=pcall(C_Spell.GetSpellCastCount,v)
                assert(not ok)
                assert(type(err)=='string')
                assert(string.find(err,'C_Spell.GetSpellCastCount',1,true))
                assert(string.find(err,'secret spell identifier access is not modeled',1,true))
                assert(not string.find(err,'PRIVATE-Count',1,true))
                assert(debug.getstacktaint()==taint)
            end
        end
        probe()
        SCTainted(probe)
    "#;

#[test]
fn cast_accepts_numeric_secrets_but_preserves_other_kind_rejection() {
    let env = secret_fixture();
    let before = metadata(&env, &SECRETS);
    env.exec(NUMERIC_SECRET_CAST_ACCESS).unwrap();
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
                for _,id in ipairs({
                    19750,
                    880001,
                    'unknown-spell',
                    false,
                }) do
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
                assert(issecretvalue(v))
                assert(not canaccessvalue(v))
                SCReject('GetSpellDisplayCount',1,true,v)
            end
        end)
        collectgarbage('collect')
        assert(SCFrame:GetAlpha()==0.625)
        assert(SCFrame.marker==67)
        assert(SCTable.marker=='PRIVATE-Count')
        assert(SCTable.count==37)
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
    let expected_display = if uses_retail_native_count_contract() {
        "7" // Retail reagent mode takes precedence over charge display.
    } else {
        "3"
    };
    display(&env, "19750", expected_display, true);
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
            for _,v in ipairs({
                SCCast,
                SCDisplay,
            }) do
                assert(issecretvalue(v))
                assert(not canaccessvalue(v))
            end
            for _,op in ipairs({
                function()
                    return SCCast+1
                end,
                function()
                    return SCDisplay..'x'
                end,
                function()
                    return string.len(SCDisplay)
                end,
            }) do
                local ok,err=pcall(op)
                assert(not ok)
                assert(type(err)=='string')
                assert(#err>0)
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
        r#"
        SCCast = C_Spell.GetSpellCastCount(19750)
        SCDisplay = C_Spell.GetSpellDisplayCount(19750)
        "#,
    )
    .unwrap();
    let before = metadata(&env, &["SCCast", "SCDisplay"]);
    env.exec(
        r#"
        SCTainted(function()
            SCOutputCopy = {
                SCCast,
                SCDisplay,
            }
        end)
        for i=1,40 do
            SCFreshCast=C_Spell.GetSpellCastCount(19750)
            SCFreshDisplay=C_Spell.GetSpellDisplayCount(19750)
            assert(issecretvalue(SCFreshCast))
            assert(issecretvalue(SCFreshDisplay))
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
    env.exec(
        r#"
        SCTainted(function()
            for _,v in ipairs(SCOutputCopy) do
                assert(issecretvalue(v))
                assert(not canaccessvalue(v))
            end
            assert(C_Spell.GetSpellCastCount(19750)==7)
            assert(C_Spell.GetSpellDisplayCount(19750)=='7')
        end)
    "#,
    )
    .unwrap();
    assert_eq!(metadata(&env, &["SCCast", "SCDisplay"]), before);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_signed_reagent_quantities_use_eight_eligible_pairs() {
    // VERIFIED R88p.p3: quantities narrow to i16 before eligibility and division.
    let env = fixture(false);
    set_count(&env, 19750, 15);
    set_count(&env, 642, 8);
    {
        let mut sim = env.state().borrow_mut();
        sim.spell_reagent_pairs.insert(
            19750,
            [
                [
                    (0, 1),
                    (-2, 1),
                    (123, 32768),
                    (reagent_item_id(19750) as i32, 65539),
                ]
                .as_slice(),
                [(0, 1); 4].as_slice(),
                [(123, 1)].as_slice(),
            ]
            .concat(),
        );
        sim.spell_reagent_pairs
            .insert(642, vec![(reagent_item_id(642) as i32, 65538)]);
    }
    cast_count(&env, "19750", 5, false);
    cast_count(&env, "642", 4, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_default_reagent_traversal_excludes_bank_tabs() {
    // VERIFIED R88p.p3: mask 32 is required for both bank-tab groups.
    let env = fixture(false);
    {
        let mut sim = env.state().borrow_mut();
        let first = sim
            .bag_items
            .values()
            .find(|i| i.item_id == reagent_item_id(19750))
            .unwrap()
            .clone();
        let second = sim
            .bag_items
            .values()
            .find(|i| i.item_id == reagent_item_id(642))
            .unwrap()
            .clone();
        sim.bag_items.insert((6, 1), first);
        sim.bag_items.insert((12, 1), second);
    }
    cast_count(&env, "19750", 7, false);
    cast_count(&env, "642", 2, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_numeric_replacements_convert_without_replacing_count_output() {
    // VERIFIED R88k.5 publication/arguments #1056 bullet 3: numbers become text.
    let env = fixture(false);
    display(&env, "19750,6,17", "17", false);
    display(&env, "642,1,29", "29", false);
    clear_count(&env, 19750);
    display(&env, "19750,6,17", "", false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_cast_traversal_counts_carried_root_items() {
    let env = fixture(false);
    let mut first = wow_ui_sim::lua_api::state::EquippedItem {
        owned_identity: None,
        item_id: 0,
        enchant_id: 0,
        gem_ids: [0; 3],
        applied_transmog: None,
    };
    let mut second = first.clone();
    first.item_id = reagent_item_id(19750);
    second.item_id = reagent_item_id(642);
    {
        let mut sim = env.state().borrow_mut();
        sim.player.equipped_items.clear();
        sim.player.equipped_items.insert(1, first);
        sim.player.equipped_items.insert(2, second);
    }
    cast_count(&env, "19750", 8, false);
    cast_count(&env, "642", 3, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
fn copy_reagent_stack(env: &WowLuaEnv, spell: u32, location: (i32, i32), count: i32) {
    let mut sim = env.state().borrow_mut();
    let mut item = sim
        .bag_items
        .values()
        .find(|i| i.item_id == reagent_item_id(spell))
        .unwrap()
        .clone();
    item.stack_count = count;
    sim.bag_items.insert(location, item);
}

#[cfg(feature = "retail-12-1-5")]
fn reagent_masks(env: &WowLuaEnv, mask: u32) {
    let mut sim = env.state().borrow_mut();
    for spell in [19750, 642] {
        sim.spell_reagent_inventory
            .metadata_masks
            .insert(reagent_item_id(spell) as i32, mask);
    }
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_mask32_admits_bank_tabs_and_recursive_children_live() {
    let env = fixture(false);
    copy_reagent_stack(&env, 19750, (6, 1), 7);
    copy_reagent_stack(&env, 642, (12, 1), 2);
    copy_reagent_stack(&env, 19750, (50, 1), 5);
    env.state()
        .borrow_mut()
        .spell_reagent_inventory
        .container_parents
        .insert(50, (6, 1));
    cast_count(&env, "19750", 7, false);
    cast_count(&env, "642", 2, false);
    reagent_masks(&env, 32);
    cast_count(&env, "19750", 19, false);
    cast_count(&env, "642", 4, false);
    env.state().borrow_mut().bag_items.remove(&(6, 1));
    cast_count(&env, "19750", 7, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_mask2_without_properties_keeps_sum_then_divide() {
    let env = fixture(false);
    set_count(&env, 19750, 2);
    set_count(&env, 642, 3);
    copy_reagent_stack(&env, 19750, (0, 90), 2);
    copy_reagent_stack(&env, 642, (0, 91), 3);
    {
        let mut sim = env.state().borrow_mut();
        sim.spell_reagent_pairs.get_mut(&19750).unwrap()[0].1 = 3;
        sim.spell_reagent_pairs.get_mut(&642).unwrap()[0].1 = 2;
    }
    reagent_masks(&env, 2);
    cast_count(&env, "19750", 1, false);
    cast_count(&env, "642", 3, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_signed_maximum_fraction_defaults_and_trailing_cast_arguments() {
    let env = fixture(false);
    cast_count(&env, "19750.9, {}, 'ignored'", 7, false);
    cast_count(&env, "642.9, false", 2, false);
    display(&env, "19750,7.9", "7", false);
    display(&env, "19750,2147483647", "7", false);
    display(&env, "642,-2147483648", "*", false);
    display(&env, "642,1.9", "*", false);
    set_count(&env, 19750, 0);
    display(&env, "19750,-1", "*", false);
    display(&env, "19750,-0.5", "0", false);
    display(&env, "19750,nil", "0", false);
    cast_count(&env, "-1", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
fn reagent_container_info(spell: u32, slot: Option<i32>) -> wow_ui_sim::c_api::bag_info::BagInfo {
    wow_ui_sim::c_api::bag_info::BagInfo {
        num_slots: 16,
        family: 0,
        name: None,
        inventory_slot: slot,
        item_id: Some(reagent_item_id(spell)),
        hyperlink: None,
        owned_identity: None,
    }
}

#[cfg(feature = "retail-12-1-5")]
fn equip_reagent_container(env: &WowLuaEnv, spell: u32, bag: i32, slot: i32) {
    let mut sim = env.state().borrow_mut();
    sim.bag_info
        .insert(bag, reagent_container_info(spell, Some(slot)));
    sim.player.equipped_items.insert(
        slot,
        wow_ui_sim::lua_api::state::EquippedItem {
            owned_identity: None,
            item_id: reagent_item_id(spell),
            enchant_id: 0,
            gem_ids: [0; 3],
            applied_transmog: None,
        },
    );
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_bank_container_roots_are_filtered_and_not_double_counted() {
    let env = fixture(false);
    equip_reagent_container(&env, 19750, 6, 24);
    equip_reagent_container(&env, 642, 12, 25);
    cast_count(&env, "19750", 7, false);
    cast_count(&env, "642", 2, false);
    reagent_masks(&env, 32);
    cast_count(&env, "19750", 8, false);
    cast_count(&env, "642", 3, false);
    env.state().borrow_mut().player.equipped_items.remove(&24);
    cast_count(&env, "19750", 8, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_nested_containers_are_items_not_additional_roots() {
    let env = fixture(false);
    copy_reagent_stack(&env, 19750, (6, 1), 1);
    copy_reagent_stack(&env, 642, (1, 1), 2);
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_info.insert(1, reagent_container_info(19750, None));
        sim.spell_reagent_inventory
            .container_parents
            .insert(1, (6, 1));
    }
    cast_count(&env, "19750", 7, false);
    cast_count(&env, "642", 2, false);
    reagent_masks(&env, 32);
    cast_count(&env, "19750", 8, false);
    cast_count(&env, "642", 4, false);
    env.state().borrow_mut().bag_items.remove(&(6, 1));
    cast_count(&env, "19750", 7, false);
    cast_count(&env, "642", 2, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_cyclic_container_inputs_cannot_supply_reagent_counts() {
    // Simulator robustness only: native does not claim cyclic physical inventory.
    let env = fixture(false);
    copy_reagent_stack(&env, 19750, (1, 90), 1);
    copy_reagent_stack(&env, 642, (2, 90), 2);
    env.state()
        .borrow_mut()
        .spell_reagent_inventory
        .container_parents
        .extend([(1, (2, 90)), (2, (1, 90))]);
    cast_count(&env, "19750", 7, false);
    cast_count(&env, "642", 2, false);
    clear_count(&env, 19750);
    cast_count(&env, "19750", 0, false);
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88p_mode1_denial_uses_native_reason_before_maximum_conversion() {
    let env = secret_fixture();
    positive(&env);
    cast_count(&env, "880001", 0, false);
    env.exec(
        r#"
        SCTainted(function()
            local ok,err = pcall(C_Spell.GetSpellDisplayCount, SCIdentifier, 4294967295)
            assert(not ok)
            -- R88p.p2: decoration/truncation is context-dependent; inspect a reason fragment.
            assert(string.find(err, 'Secret values', 1, true))
            assert(not string.find(err, 'outside of expected range', 1, true))
        end)
    "#,
    )
    .unwrap();
}

// VERIFIED Retail R88r.r1, lt-67d2 #1239. PTR applicability remains INFERRED.

#[cfg(feature = "retail-12-1-5")]
fn publish_r88r_numbers(env: &WowLuaEnv, numbers: &[(&str, f64)]) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    for &(name, number) in numbers {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        publish(&mut *lua, name, value);
    }
}

#[cfg(feature = "retail-12-1-5")]
#[test]
fn r88r_accepted_secret_identifiers_truncate_to_distinct_backed_spells() {
    let env = secret_fixture();
    publish_r88r_numbers(
        &env,
        &[
            ("SRFirst", 19750.5),
            ("SRSecond", 642.5),
            ("SRAbsent", 880001.5),
            ("SRSix", 6.5),
        ],
    );
    // A host reagent row discriminates ID6 via display-mode selection, without a Spell row.
    env.state()
        .borrow_mut()
        .spell_reagent_pairs
        .insert(6, vec![(31003, 1)]);
    assert!(wow_ui_sim::spells::get_spell(6).is_none());
    cast_count(&env, "SRFirst", 7, false);
    cast_count(&env, "SRSecond", 2, false);
    cast_count(&env, "SRAbsent", 0, false);
    display(&env, "SRFirst", "7", false);
    display(&env, "SRSecond", "2", false);
    display(&env, "SRAbsent", "", false);
    display(&env, "SRSix", "0", false);
    // No secret <=1 ID case: native Lua metadata-source condition is unmodeled.
}
