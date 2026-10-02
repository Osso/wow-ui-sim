//! Row387 fixtures for an INFERRED explicit association model, not native parity.
//! The API producer is intentionally absent at the inputs/fixtures stage.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

const LINK: &str = "|cff71d5ff|Hspell:101|h[Fixture Cooldown]|h|r";

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create cooldown association environment");
    env.exec(
        r#"
        function CheckCooldown(identifier, expected)
            local function check(...)
                assert(select('#', ...) == 1, 'one nullable numeric result')
                local value = ...
                assert(not issecretvalue(value), 'public result')
                if expected == nil then assert(value == nil, 'association miss')
                else
                    assert(type(value) == 'number', 'number, never AuraData')
                    assert(value == expected, 'declared cooldown spell ID')
                end
            end
            check(C_UnitAuras.GetCooldownAuraBySpellID(identifier))
        end
        function RejectCooldown(...)
            local query = C_UnitAuras.GetCooldownAuraBySpellID
            assert(type(query) == 'function', 'real registered query required')
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string' and #err > 0, 'argument rejection')
        end
    "#,
    )
    .expect("install assertions without replacing API");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state
            .cooldown_aura_associations
            .cooldown_spell_ids
            .extend([(101, 901), (202, 702)]);
        state.spell_id_aliases.clear();
        state
            .spell_id_aliases
            .insert("fixture cooldown".into(), 101);
        state.spell_id_aliases.insert(LINK.to_lowercase(), 202);
    }
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    env.exec("CooldownWrongObject = CreateFrame('Frame')")
        .expect("real frame userdata");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, text) in [
        ("CooldownSecretName", "fixture cooldown"),
        ("CooldownSecretLink", LINK),
        ("CooldownSecretUnknown", "unknown"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret STRING");
    }
    for (name, number) in [
        ("CooldownSecretNumber", 101.0),
        ("CooldownSecretMissing", 303.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret NUMBER");
    }
    let native = lua.get_global_val("CooldownWrongObject");
    assert!(matches!(native, rilua::Val::Userdata(_)));
    lua.state_mut().push(native);
    let wrapper = wrap_secret(lua.state_mut(), native).expect("wrap actual wrong object");
    lua.state_mut().push(wrapper);
    let inserted = lua.set_global_val("CooldownSecretObject", wrapper);
    lua.state_mut().pop();
    lua.state_mut().pop();
    inserted.expect("root wrapper and original userdata");
}

#[test]
fn default_empty_associations_never_manufacture_cooldown_ids() {
    let env = fixture_env();
    assert!(
        env.state()
            .borrow()
            .cooldown_aura_associations
            .cooldown_spell_ids
            .is_empty()
    );
    env.exec("CheckCooldown(101, nil); CheckCooldown(202, nil); CheckCooldown(0, nil); CheckCooldown(4294967295, nil)")
        .expect("empty chosen model has no catalog");
}

#[test]
fn numeric_queries_return_distinct_declared_cooldown_ids() {
    seeded_env()
        .exec("CheckCooldown(101, 901); CheckCooldown(202, 702)")
        .expect("resolved query keys are not returned IDs");
}

#[test]
fn explicit_name_alias_is_case_normalized() {
    seeded_env()
        .exec("CheckCooldown('FIXTURE Cooldown', 901)")
        .expect("existing lowercase alias resolver");
}

#[test]
fn explicit_full_colored_link_alias_is_not_parsed_for_embedded_id() {
    // Link text contains 101, but its explicit alias resolves 202 -> 702.
    seeded_env()
        .exec(&format!("CheckCooldown('{LINK}', 702)"))
        .expect("full link alias, not automatic hyperlink parsing");
}

#[test]
fn unknown_numbers_and_unseeded_strings_return_exactly_one_nil() {
    seeded_env()
        .exec(
            r#"
        for _, identifier in ipairs({303, 'unknown', '', '101',
            '|Hspell:101|h[Fixture Cooldown]|h', 'Fixture Other'}) do
            CheckCooldown(identifier, nil)
        end
    "#,
        )
        .expect("no string numeric coercion or implicit link catalog");
}

#[test]
fn alias_replacement_changes_resolved_key_not_direct_return_value() {
    let env = seeded_env();
    env.exec("CheckCooldown('fixture cooldown', 901)").unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture cooldown".into(), 202);
    env.exec("CheckCooldown('fixture cooldown', 702)").unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture cooldown".into(), 901);
    env.exec("CheckCooldown('fixture cooldown', nil)").unwrap();
}

#[test]
fn numeric_alias_override_and_seeded_numeric_string_use_same_resolved_key() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("101".into(), 202);
    env.exec("CheckCooldown(101, 702); CheckCooldown('101', 702)")
        .unwrap();
    env.state().borrow_mut().spell_id_aliases.remove("101");
    env.exec("CheckCooldown(101, 901); CheckCooldown('101', nil)")
        .unwrap();
}

#[test]
fn association_replace_remove_and_nonrecursive_output_are_immediate() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .cooldown_aura_associations
        .cooldown_spell_ids
        .insert(901, 999);
    env.exec("CheckCooldown(101, 901); CheckCooldown(901, 999)")
        .unwrap();
    env.state()
        .borrow_mut()
        .cooldown_aura_associations
        .cooldown_spell_ids
        .insert(101, 808);
    env.exec("CheckCooldown(101, 808)").unwrap();
    env.state()
        .borrow_mut()
        .cooldown_aura_associations
        .cooldown_spell_ids
        .remove(&101);
    env.exec("CheckCooldown(101, nil); CheckCooldown(202, 702)")
        .unwrap();
}

#[test]
fn numeric_u32_endpoints_and_output_endpoints_are_public_numbers() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .cooldown_aura_associations
        .cooldown_spell_ids
        .extend([(0, u32::MAX), (u32::MAX, 0)]);
    env.exec("CheckCooldown(0, 4294967295); CheckCooldown(4294967295, 0)")
        .expect("finite integral inclusive u32 bounds");
}

#[test]
fn invalid_representations_reject_before_unknown_lookup() {
    let env = seeded_env();
    // Even an explicit alias must not bypass validation of a fractional number.
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("101".into(), 202);
    env.exec(
        r#"
        RejectCooldown(); RejectCooldown(nil)
        for _, value in ipairs({false, true, {}, function() end, coroutine.create(function() end),
            CreateFrame('Frame'), 0/0, math.huge, -math.huge, -1, 101.5, 4294967296,
            string.char(255), string.char(192, 175)}) do
            RejectCooldown(value)
        end
        CheckCooldown('unseeded', nil); CheckCooldown(202, 702)
    "#,
    )
    .expect("INFERRED required UTF-8 STRING or finite integral u32 NUMBER");
}

#[test]
fn generic_player_aura_with_same_spell_id_is_not_an_association() {
    let env = fixture_env();
    env.state().borrow_mut().player.buffs = vec![AuraInfo {
        name: "Unrelated generic buff".into(),
        spell_id: 101,
        icon: 134973,
        duration: 30.0,
        expiration_time: 45.0,
        applications: 2,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: 777,
    }];
    env.exec("assert(C_UnitAuras.GetPlayerAuraBySpellID(101).auraInstanceID == 777); CheckCooldown(101, nil)")
        .expect("generic buff cannot fabricate cooldown input");
}

#[test]
fn environments_isolate_maps_and_queries_leave_all_inputs_unchanged() {
    let first = seeded_env();
    let second = seeded_env();
    let aliases = first.state().borrow().spell_id_aliases.clone();
    let records = first
        .state()
        .borrow()
        .cooldown_aura_associations
        .cooldown_spell_ids
        .clone();
    first
        .exec(&format!(
            r#"
        CooldownCaller = {{identifier = '{LINK}', marker = 17}}
        CheckCooldown(CooldownCaller.identifier, 702)
        collectgarbage('collect')
        assert(CooldownCaller.identifier == '{LINK}' and CooldownCaller.marker == 17)
        CheckCooldown(101, 901); CheckCooldown('unknown', nil)
    "#
        ))
        .unwrap();
    assert_eq!(first.state().borrow().spell_id_aliases, aliases);
    assert_eq!(
        first
            .state()
            .borrow()
            .cooldown_aura_associations
            .cooldown_spell_ids,
        records
    );
    first
        .state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture cooldown".into(), 202);
    first
        .state()
        .borrow_mut()
        .cooldown_aura_associations
        .cooldown_spell_ids
        .insert(202, 808);
    first
        .exec("CheckCooldown('fixture cooldown', 808)")
        .unwrap();
    second
        .exec("CheckCooldown('fixture cooldown', 901); CheckCooldown(202, 702)")
        .unwrap();
}

#[test]
fn public_numeric_name_link_queries_work_in_tainted_context_without_clearing_taint() {
    let env = seeded_env();
    env.exec(&format!(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            CheckCooldown(101, 901); CheckCooldown('fixture cooldown', 901)
            CheckCooldown('{LINK}', 702); CheckCooldown('unknown', nil)
            assert(debug.getstacktaint() == before)
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'CooldownPublicFixture')
            probe()
            assert(debug.getstacktaint() == 'CooldownPublicFixture')
        end
        debug.setobjecttaint(addon, 'CooldownPublicFixture')
        addon(); assert(issecure())
    "#
    ))
    .expect("ordinary public calls honor tainted caller context");
}

#[test]
fn actual_vm_secrets_and_wrapped_wrong_object_reject_secure_and_tainted_after_gc() {
    let env = seeded_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local rooted = {CooldownSecretNumber, CooldownSecretName, CooldownSecretLink,
            CooldownSecretUnknown, CooldownSecretMissing, CooldownSecretObject}
        collectgarbage('collect')
        local function probe()
            local before = debug.getstacktaint()
            for _, value in ipairs(rooted) do
                assert(issecretvalue(value), 'actual VM secret, not Lua marker')
                RejectCooldown(value)
                assert(issecretvalue(value), 'no declassification')
                assert(debug.getstacktaint() == before, 'caller taint preserved')
            end
            assert(rawequal(rooted[1], CooldownSecretNumber))
            assert(rawequal(rooted[2], CooldownSecretName))
            assert(rawequal(rooted[6], CooldownSecretObject))
            CheckCooldown(101, 901); CheckCooldown('fixture cooldown', 901)
            assert(debug.getstacktaint() == before, 'public recovery')
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'CooldownSecretFixture')
            probe()
            assert(debug.getstacktaint() == 'CooldownSecretFixture')
        end
        debug.setobjecttaint(addon, 'CooldownSecretFixture')
        addon(); assert(issecure())
    "#,
    )
    .expect("INFERRED conservative secret rejection; native AllowedWhenTainted remains a gap");
}
