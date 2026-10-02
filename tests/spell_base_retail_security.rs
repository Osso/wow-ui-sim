//! Row295 input scaffold: real Retail namespace; main owns compiled RED.
//! Relationships are test-owned inputs, not native Paladin override data.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use rilua::table_security::{
    wrap_host_secret_bool, wrap_host_secret_number, wrap_host_secret_string, wrap_secret,
};
use rilua::{LuaApiMut, Val};
use wow_ui_sim::lua_api::WowLuaEnv;

const SECRET_NAMES: [&str; 7] = [
    "BSNumber",
    "BSNil",
    "BSFalse",
    "BSTrue",
    "BSString",
    "BSSecretTable",
    "BSSecretFrame",
];

fn empty_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("base-spell environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = 2;
        state.player.active_spec_index = 2;
    }
    env.exec(
        r#"
        BSFrame = CreateFrame('Frame')
        BSFrame:SetAlpha(0.625)
        BSFrame.marker = 295
        BSInput = {marker = 37, flag = false}
        function BSState()
            assert(BSFrame:GetAlpha() == 0.625 and BSFrame.marker == 295)
            assert(BSInput.marker == 37 and BSInput.flag == false)
        end
        function BSContexts(probe)
            assert(issecure(), 'secure caller')
            probe()
            assert(issecure(), 'secure caller preserved')
            local function addon()
                assert(debug.getstacktaint() == 'BaseSpell295Fixture')
                probe()
                assert(debug.getstacktaint() == 'BaseSpell295Fixture', 'addon taint preserved')
            end
            debug.setobjecttaint(addon, 'BaseSpell295Fixture')
            addon()
            assert(issecure(), 'outer secure caller restored')
        end
        function BSCheck(expected, ...)
            local function result(...)
                assert(select('#', ...) == 1, 'one base result')
                local value = ...
                assert(type(value) == 'number' and not issecretvalue(value), 'public numeric base')
                assert(value == expected, 'exact modeled relationship or identity')
            end
            result(C_Spell.GetBaseSpell(...))
        end
        function BSRecovery()
            BSCheck(642, 19750)
            BSCheck(853, 19750, 70)
            BSCheck(4294967295, 4294967295)
            BSState()
        end
        function BSReject(identifier, spec)
            assert(issecretvalue(spec), 'authentic VM secret arg2')
            local before = debug.getstacktaint()
            local frame, input = BSFrame, BSInput
            local ok, err = pcall(C_Spell.GetBaseSpell, identifier, spec)
            assert(not ok, 'secret spec denied')
            assert(type(err) == 'string', 'public error string')
            assert(#err > 0, 'nonempty rejection')
            assert(string.find(err, 'C_Spell.GetBaseSpell', 1, true), 'API error context')
            assert(string.find(err, 'arg2', 1, true)
                or string.find(err, 'argument 2', 1, true)
                or string.find(err, '#2', 1, true), 'second argument error context')
            assert(not string.find(err, 'PRIVATE-BaseSpell295', 1, true), 'no string payload leak')
            assert(not string.find(err, '987654321', 1, true), 'no number payload leak')
            assert(debug.getstacktaint() == before, 'rejection preserves taint')
            assert(issecretvalue(spec), 'no declassification')
            assert(rawequal(frame, BSFrame) and rawequal(input, BSInput), 'public input identity')
            BSRecovery()
            assert(debug.getstacktaint() == before, 'public recovery preserves taint')
        end
        function BSRejectValid(spec)
            BSReject(19750, spec)
            BSReject('Flash of Light', spec)
            BSReject(4294967295, spec)
        end
        "#,
    )
    .expect("public helpers only; no replacement API callbacks");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = empty_env();
    {
        let mut state = env.state().borrow_mut();
        state.base_spell_relationships.set(66, 19750, 642);
        state.base_spell_relationships.set(70, 19750, 853);
    }
    env
}

fn public_probe(script: &str) {
    seeded_env()
        .exec(&format!("BSContexts(function() {script} end)"))
        .expect("real Retail GetBaseSpell public behavior");
}

fn publish_secret(lua: &mut impl LuaApiMut, name: &str, value: Val) {
    lua.state_mut().push(value);
    let result = lua.set_global_val(name, value);
    lua.state_mut().pop();
    result.expect("root authentic VM wrapper during publication");
}

fn secret_env() -> WowLuaEnv {
    let env = seeded_env();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    let number = wrap_host_secret_number(lua.state_mut(), 987654321.0);
    publish_secret(&mut *lua, "BSNumber", number);
    let nil = wrap_secret(lua.state_mut(), Val::Nil).expect("authentic secret nil");
    publish_secret(&mut *lua, "BSNil", nil);
    for (name, payload) in [("BSFalse", false), ("BSTrue", true)] {
        let boolean = wrap_host_secret_bool(lua.state_mut(), payload);
        publish_secret(&mut *lua, name, boolean);
    }
    let string = wrap_host_secret_string(lua.state_mut(), "PRIVATE-BaseSpell295-Payload");
    publish_secret(&mut *lua, "BSString", string);
    for (original_name, secret_name) in [("BSInput", "BSSecretTable"), ("BSFrame", "BSSecretFrame")]
    {
        let original = lua.get_global_val(original_name);
        let Val::Table(reference) = original else {
            panic!("actual frame/table required: {original_name}")
        };
        if original_name == "BSFrame" {
            assert!(
                lua.state_mut()
                    .gc
                    .tables
                    .get(reference)
                    .expect("live frame")
                    .backing()
                    .is_some(),
                "actual frame backing, not fabricated frame-shaped table"
            );
        }
        lua.state_mut().push(original);
        let wrapper = wrap_secret(lua.state_mut(), original).expect("wrap real frame/table");
        publish_secret(&mut *lua, secret_name, wrapper);
        lua.state_mut().pop();
    }
    drop(lua);
    env.exec(
        "BSSecrets = {BSNumber, BSNil, BSFalse, BSTrue, BSString, BSSecretTable, BSSecretFrame}; \
         function BSSecrecy() assert(#BSSecrets == 7); \
         for _, v in ipairs(BSSecrets) do assert(issecretvalue(v)) end end; BSSecrecy()",
    )
    .expect("Lua roots without secret BOOL equality");
    env
}

fn wrapper_metadata(env: &WowLuaEnv) -> Vec<(Val, u64)> {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    SECRET_NAMES
        .iter()
        .map(|name| {
            let value = lua.get_global_val(name);
            let Val::Userdata(reference) = value else {
                panic!("authentic wrapper missing: {name}")
            };
            assert!(rilua::table_security::is_secret_value(
                lua.state_mut(),
                value
            ));
            let sequence = lua
                .state_mut()
                .gc
                .userdata
                .get(reference)
                .expect("rooted wrapper remains live")
                .alloc_seq();
            (value, sequence)
        })
        .collect()
}

fn modeled_bases(env: &WowLuaEnv) -> [u32; 4] {
    let state = env.state().borrow();
    let model = &state.base_spell_relationships;
    [
        model.resolve(66, 19750),
        model.resolve(70, 19750),
        model.resolve(65, 19750),
        model.resolve(66, 4294967295),
    ]
}

fn secret_probe(script: &str) {
    let env = secret_env();
    let roots = wrapper_metadata(&env);
    let bases = modeled_bases(&env);
    let aliases = env.state().borrow().spell_id_aliases.clone();
    env.exec(&format!("BSContexts(function() {script}; BSSecrecy() end)"))
        .expect("authenticated arg2 boundary, caller preservation and recovery");
    assert_eq!(
        wrapper_metadata(&env),
        roots,
        "host identity/allocation/secrecy"
    );
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let Val::Table(list) = lua.get_global_val("BSSecrets") else {
        panic!("Lua wrapper root list missing")
    };
    for (index, (value, _)) in roots.iter().enumerate() {
        assert_eq!(
            lua.state_mut()
                .gc
                .tables
                .get(list)
                .expect("live wrapper list")
                .get_int(index as i64 + 1),
            *value,
            "host root identity; no tainted Lua secret BOOL comparison"
        );
    }
    drop(lua);
    assert_eq!(modeled_bases(&env), bases, "relationship input unchanged");
    assert_eq!(
        env.state().borrow().spell_id_aliases,
        aliases,
        "aliases unchanged"
    );
    let state = env.state().borrow();
    assert_eq!(state.player.class_index, 2);
    assert_eq!(state.player.active_spec_index, 2);
}

#[test]
fn empty_relationship_input_returns_public_identity_through_real_namespace() {
    empty_env()
        .exec("BSContexts(function() BSCheck(19750, 19750); BSCheck(4294967295, 4294967295) end)")
        .expect("identity is model contract, not fallback compatibility");
}

#[test]
fn numeric_identifier_resolves_seeded_current_relationship() {
    public_probe("BSCheck(642, 19750)");
}

#[test]
fn existing_known_name_case_and_numeric_string_resolution_is_preserved() {
    public_probe(
        "BSCheck(642, 'Flash of Light'); BSCheck(642, 'flash of light'); BSCheck(642, '19750')",
    );
}

#[test]
fn omitted_nil_and_zero_public_spec_select_current_66() {
    public_probe("BSCheck(642, 19750); BSCheck(642, 19750, nil); BSCheck(642, 19750, 0)");
}

#[test]
fn explicit_spec_selects_independent_configured_relationship() {
    public_probe("BSCheck(642, 19750, 66); BSCheck(853, 19750, 70)");
}

#[test]
fn positive_integral_u32_specs_accept_unknown_relationship_identity() {
    public_probe(
        "BSCheck(19750, 19750, 1); BSCheck(19750, 19750, 65); BSCheck(19750, 19750, 4294967295); BSCheck(642, 642, 66)",
    );
}

#[test]
fn live_active_specialization_changes_default_but_not_explicit_selection() {
    let env = seeded_env();
    env.exec("BSCheck(642, 19750)").unwrap();
    env.state().borrow_mut().player.active_spec_index = 3;
    env.exec("BSContexts(function() BSCheck(853, 19750); BSCheck(853, 19750, nil); BSCheck(853, 19750, 0); BSCheck(642, 19750, 66) end)").unwrap();
}

#[test]
fn live_relationship_update_is_visible_without_recreating_environment() {
    let env = seeded_env();
    env.exec("BSCheck(642, 19750)").unwrap();
    env.state()
        .borrow_mut()
        .base_spell_relationships
        .set(66, 19750, 26573);
    env.exec("BSContexts(function() BSCheck(26573, 19750); BSCheck(853, 19750, 70) end)")
        .unwrap();
}

#[test]
fn relationship_inputs_are_isolated_between_environments() {
    let first = seeded_env();
    let second = empty_env();
    first
        .exec("BSContexts(function() BSCheck(642, 19750) end)")
        .unwrap();
    second
        .exec("BSContexts(function() BSCheck(19750, 19750) end)")
        .unwrap();
    second
        .state()
        .borrow_mut()
        .base_spell_relationships
        .set(66, 19750, 853);
    second
        .exec("BSContexts(function() BSCheck(853, 19750) end)")
        .unwrap();
    first
        .exec("BSContexts(function() BSCheck(642, 19750) end)")
        .unwrap();
}

#[test]
fn numeric_relationships_do_not_reverse_identifier_aliases() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("19750".into(), 26573);
    env.exec("BSContexts(function() BSCheck(642, 19750); BSCheck(853, 19750, 70); BSCheck(26573, 26573); BSCheck(642, 642) end)").unwrap();
    assert_eq!(
        env.state().borrow().spell_id_aliases.get("19750"),
        Some(&26573)
    );
}

#[test]
fn public_queries_are_read_only_for_relationships_player_and_inputs() {
    let env = seeded_env();
    let bases = modeled_bases(&env);
    let aliases = env.state().borrow().spell_id_aliases.clone();
    env.exec("BSContexts(function() BSRecovery(); BSCheck(19750, 19750, 65); BSRecovery() end)")
        .unwrap();
    assert_eq!(modeled_bases(&env), bases);
    let state = env.state().borrow();
    assert_eq!(state.spell_id_aliases, aliases);
    assert_eq!(state.player.class_index, 2);
    assert_eq!(state.player.active_spec_index, 2);
}

#[test]
fn invalid_public_spell_identifiers_keep_meaningful_validation() {
    public_probe(
        r#"
        for _, value in ipairs({0, -1, 1.5, math.huge, -math.huge, 0/0,
            4294967296, false, true, BSInput, BSFrame, 'unknown'}) do
            local ok, err = pcall(C_Spell.GetBaseSpell, value, 66)
            assert(not ok and type(err) == 'string')
            assert(string.find(err, 'C_Spell.GetBaseSpell', 1, true), 'API validation context')
        end
        local ok, err = pcall(C_Spell.GetBaseSpell, nil, 66)
        assert(not ok and type(err) == 'string')
        assert(string.find(err, 'C_Spell.GetBaseSpell', 1, true))
        BSRecovery()
    "#,
    );
}

#[test]
fn invalid_public_spec_types_and_ranges_keep_meaningful_validation() {
    public_probe(
        r#"
        for _, spec in ipairs({-1, 1.5, math.huge, -math.huge, 0/0,
            4294967296, false, true, '66', BSInput, BSFrame}) do
            local ok, err = pcall(C_Spell.GetBaseSpell, 19750, spec)
            assert(not ok and type(err) == 'string')
            assert(string.find(err, 'C_Spell.GetBaseSpell', 1, true), 'API validation context')
            assert(string.find(err, 'spec', 1, true), 'existing specialization validation')
        end
        BSRecovery()
    "#,
    );
}

#[test]
fn authentic_secret_number_arg2_rejects_before_relationship_or_identity() {
    secret_probe("BSRejectValid(BSNumber)");
}

#[test]
fn authentic_secret_nil_arg2_is_not_public_default_selection() {
    secret_probe("BSRejectValid(BSNil)");
}

#[test]
fn authentic_secret_bools_arg2_reject_without_tainted_bool_comparison() {
    secret_probe("BSRejectValid(BSFalse); BSRejectValid(BSTrue)");
}

#[test]
fn authentic_secret_string_arg2_rejects_without_private_payload_leak() {
    secret_probe("BSRejectValid(BSString)");
}

#[test]
fn authentic_secret_table_and_real_frame_arg2_reject_without_mutation() {
    secret_probe("BSRejectValid(BSSecretTable); BSRejectValid(BSSecretFrame)");
}

#[test]
fn secret_arg2_denial_precedes_each_invalid_public_arg1_validation() {
    secret_probe(
        r#"
        for _, spec in ipairs(BSSecrets) do
            BSReject(false, spec)
            BSReject(nil, spec)
            BSReject('unknown', spec)
        end
    "#,
    );
}

#[test]
fn rooted_secret_metadata_survives_forced_gc_rejection_and_public_recovery() {
    secret_probe(
        r#"
        for _, spec in ipairs(BSSecrets) do
            BSRejectValid(spec)
            collectgarbage('collect')
            BSSecrecy()
            BSRejectValid(spec)
            BSRecovery()
        end
    "#,
    );
}
