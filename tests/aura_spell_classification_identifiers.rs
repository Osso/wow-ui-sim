//! Batch48 rows361/363: INFERRED explicit classifications, not native parity.
//! Inputs/fixtures only; parent owns compiled RED before any producer.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string, wrap_secret};
use wow_ui_sim::c_api::c_unit_aura_classification::AuraSpellClassification;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

const LINK: &str = "|cff71d5ff|Hspell:101|h[Fixture Classification]|h|r";

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("classification environment");
    env.exec(
        r#"
        function CheckClassification(identifier, defensive, private)
            local function check(expected, ...)
                assert(select('#', ...) == 1, 'exactly one result')
                local value = ...
                assert(type(value) == 'boolean', 'required boolean')
                assert(not issecretvalue(value), 'public boolean')
                assert(value == expected, 'declared independent classification')
            end
            check(defensive, C_UnitAuras.AuraIsBigDefensive(identifier))
            check(private, C_UnitAuras.AuraIsPrivate(identifier))
        end
        function RejectClassification(...)
            assert(type(C_UnitAuras.AuraIsBigDefensive) == 'function', 'real defensive API')
            assert(type(C_UnitAuras.AuraIsPrivate) == 'function', 'real private API')
            for _, query in ipairs({C_UnitAuras.AuraIsBigDefensive, C_UnitAuras.AuraIsPrivate}) do
                local ok, err = pcall(query, ...)
                assert(not ok and type(err) == 'string' and #err > 0, 'argument rejection')
            end
        end
    "#,
    )
    .expect("assertion helpers, no API replacement");
    env
}

fn classification(is_big_defensive: bool, is_private: bool) -> AuraSpellClassification {
    AuraSpellClassification {
        is_big_defensive,
        is_private,
    }
}

fn seeded_env() -> WowLuaEnv {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.aura_spell_classifications.spells.extend([
            (101, classification(true, false)),
            (202, classification(false, true)),
            (303, classification(true, true)),
            (404, classification(false, false)),
        ]);
        state.spell_id_aliases.clear();
        state
            .spell_id_aliases
            .insert("fixture classification".into(), 101);
        state.spell_id_aliases.insert(LINK.to_lowercase(), 202);
    }
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    env.exec("ClassificationFrame = CreateFrame('Frame'); assert(ClassificationFrame:GetObjectType() == 'Frame')")
        .expect("actual frame behavior, no userdata representation assumption");
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, text) in [
        ("ClassificationSecretName", "fixture classification"),
        ("ClassificationSecretLink", LINK),
        ("ClassificationSecretUnknown", "unknown"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root actual secret STRING");
    }
    for (name, number) in [
        ("ClassificationSecretNumber", 101.0),
        ("ClassificationSecretMissing", 999.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root actual secret NUMBER");
    }
    let frame = lua.get_global_val("ClassificationFrame");
    assert!(
        !frame.is_nil(),
        "actual backing frame table remains globally rooted"
    );
    lua.state_mut().push(frame);
    let wrapper = wrap_secret(lua.state_mut(), frame).expect("wrap actual wrong frame value");
    lua.state_mut().push(wrapper);
    let inserted = lua.set_global_val("ClassificationSecretFrame", wrapper);
    lua.state_mut().pop();
    lua.state_mut().pop();
    inserted.expect("root wrapper and actual frame");
    drop(lua);
    env.exec(
        r#"
        ClassificationRoots = {ClassificationSecretName, ClassificationSecretLink,
            ClassificationSecretUnknown, ClassificationSecretNumber,
            ClassificationSecretMissing, ClassificationSecretFrame}
        function ProbeClassificationSecrets()
            local before = debug.getstacktaint()
            collectgarbage('collect')
            local globals = {ClassificationSecretName, ClassificationSecretLink,
                ClassificationSecretUnknown, ClassificationSecretNumber,
                ClassificationSecretMissing, ClassificationSecretFrame}
            for index, value in ipairs(ClassificationRoots) do
                assert(rawequal(value, globals[index]), 'rooted identity across GC')
                assert(issecretvalue(value), 'actual VM secret')
                RejectClassification(value)
                assert(rawequal(value, globals[index]) and issecretvalue(value), 'identity and secrecy retained')
                assert(debug.getstacktaint() == before, 'rejection preserves taint')
                CheckClassification(101, true, false)
                CheckClassification('fixture classification', true, false)
                CheckClassification('unknown', false, false)
                assert(debug.getstacktaint() == before, 'public recovery preserves taint')
            end
        end
    "#,
    )
    .expect("GC-rooted secrecy and recovery assertions");
}

#[test]
fn empty_default_has_no_manufactured_classification_catalog() {
    let env = fixture_env();
    assert!(
        env.state()
            .borrow()
            .aura_spell_classifications
            .spells
            .is_empty()
    );
    let default = AuraSpellClassification::default();
    assert!(!default.is_big_defensive && !default.is_private);
    env.exec("CheckClassification(101, false, false); CheckClassification(202, false, false)")
        .unwrap();
}

#[test]
fn numeric_records_keep_defensive_and_private_flags_independent() {
    seeded_env().exec("CheckClassification(101, true, false); CheckClassification(202, false, true); CheckClassification(303, true, true); CheckClassification(404, false, false)")
        .unwrap();
}

#[test]
fn unknown_numbers_and_unseeded_strings_return_one_public_false_per_api() {
    seeded_env().exec("for _, value in ipairs({999, 'unknown', '', '101', 'Fixture Other'}) do CheckClassification(value, false, false) end")
        .unwrap();
}

#[test]
fn uppercase_name_uses_explicit_lowercase_alias() {
    seeded_env()
        .exec("CheckClassification('FIXTURE Classification', true, false)")
        .unwrap();
}

#[test]
fn full_colored_link_alias_overrides_its_embedded_spell_number() {
    seeded_env()
        .exec(&format!("CheckClassification('{LINK}', false, true)"))
        .unwrap();
}

#[test]
fn unseeded_links_are_not_automatically_parsed() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .remove(&LINK.to_lowercase());
    env.exec(&format!("CheckClassification('{LINK}', false, false); CheckClassification('|Hspell:101|h[Fixture Classification]|h', false, false); CheckClassification(101, true, false)")).unwrap();
}

#[test]
fn alias_mutation_and_removal_take_effect_immediately() {
    let env = seeded_env();
    env.exec("CheckClassification('fixture classification', true, false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture classification".into(), 202);
    env.exec("CheckClassification('fixture classification', false, true)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture classification".into(), 999);
    env.exec("CheckClassification('fixture classification', false, false)")
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .remove("fixture classification");
    env.exec("CheckClassification('fixture classification', false, false); CheckClassification(101, true, false)").unwrap();
}

#[test]
fn numeric_alias_override_and_seeded_numeric_string_share_resolution() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("101".into(), 202);
    env.exec("CheckClassification(101, false, true); CheckClassification('101', false, true)")
        .unwrap();
    env.state().borrow_mut().spell_id_aliases.remove("101");
    env.exec("CheckClassification(101, true, false); CheckClassification('101', false, false)")
        .unwrap();
}

#[test]
fn classification_replacement_and_removal_are_immediate() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .aura_spell_classifications
        .spells
        .insert(101, classification(false, true));
    env.exec("CheckClassification(101, false, true); CheckClassification('fixture classification', false, true)").unwrap();
    env.state()
        .borrow_mut()
        .aura_spell_classifications
        .spells
        .remove(&101);
    env.exec("CheckClassification(101, false, false); CheckClassification(202, false, true)")
        .unwrap();
}

#[test]
fn finite_integral_u32_endpoints_are_valid_identifiers() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .aura_spell_classifications
        .spells
        .extend([
            (0, classification(true, false)),
            (u32::MAX, classification(false, true)),
        ]);
    env.exec("CheckClassification(0, true, false); CheckClassification(4294967295, false, true)")
        .unwrap();
}

#[test]
fn invalid_public_representations_reject_before_alias_lookup() {
    let env = seeded_env();
    env.state().borrow_mut().spell_id_aliases.extend([
        ("101".into(), 202),
        ("-1".into(), 101),
        ("4294967296".into(), 303),
    ]);
    env.exec(
        r#"
        RejectClassification(); RejectClassification(nil)
        local frame = CreateFrame('Frame')
        assert(frame:GetObjectType() == 'Frame')
        for _, value in ipairs({false, true, {}, frame, function() end,
            coroutine.create(function() end), 0/0, math.huge, -math.huge,
            -1, 101.5, 4294967296, string.char(255), string.char(192, 175)}) do
            RejectClassification(value)
        end
        CheckClassification(202, false, true)
    "#,
    )
    .expect("INFERRED actual UTF-8 STRING or finite integral u32 NUMBER only");
}

#[test]
fn generic_buff_and_cooldown_association_do_not_classify_a_spell() {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![AuraInfo {
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
        state
            .cooldown_aura_associations
            .cooldown_spell_ids
            .insert(101, 202);
    }
    env.exec("assert(C_UnitAuras.GetPlayerAuraBySpellID(101).auraInstanceID == 777); CheckClassification(101, false, false); CheckClassification(202, false, false); assert(C_UnitAuras.GetPlayerAuraBySpellID(101).auraInstanceID == 777)").unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .cooldown_aura_associations
            .cooldown_spell_ids
            .get(&101),
        Some(&202)
    );
}

#[test]
fn private_instance_state_does_not_supply_or_change_spell_classifications() {
    let env = seeded_env();
    env.exec(
        r#"
        local record = {spellID = 101, auraInstanceID = 777, name = 'Private instance'}
        C_UnitAurasPrivate._state.privateAurasByUnit.player = {record}
        C_UnitAurasPrivate._state.auraDataByUnit.player = {[777] = record}
        assert(C_UnitAurasPrivate.GetAuraDataByAuraInstanceIDPrivate('player', 777).spellID == 101)
        CheckClassification(101, true, false); CheckClassification(777, false, false)
        CheckClassification(202, false, true)
        assert(C_UnitAurasPrivate._state.privateAurasByUnit.player[1] == record)
        assert(record.spellID == 101 and record.auraInstanceID == 777 and record.name == 'Private instance')
    "#,
    ).expect("existing temporary private-instance fixture, not native classification acquisition");
}

#[test]
fn queries_are_read_only_and_environments_isolate_inputs() {
    let first = seeded_env();
    let second = seeded_env();
    let aliases = first.state().borrow().spell_id_aliases.clone();
    let records: std::collections::HashMap<_, _> = first
        .state()
        .borrow()
        .aura_spell_classifications
        .spells
        .iter()
        .map(|(&id, value)| (id, (value.is_big_defensive, value.is_private)))
        .collect();
    first.exec(&format!("ClassificationCaller = {{identifier = '{LINK}', marker = 17}}; CheckClassification(ClassificationCaller.identifier, false, true); collectgarbage('collect'); assert(ClassificationCaller.identifier == '{LINK}' and ClassificationCaller.marker == 17); CheckClassification(101, true, false); CheckClassification('unknown', false, false)")).unwrap();
    assert_eq!(first.state().borrow().spell_id_aliases, aliases);
    let after: std::collections::HashMap<_, _> = first
        .state()
        .borrow()
        .aura_spell_classifications
        .spells
        .iter()
        .map(|(&id, value)| (id, (value.is_big_defensive, value.is_private)))
        .collect();
    assert_eq!(after, records);
    first
        .state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture classification".into(), 303);
    first
        .state()
        .borrow_mut()
        .aura_spell_classifications
        .spells
        .insert(303, classification(false, false));
    first
        .exec("CheckClassification('fixture classification', false, false)")
        .unwrap();
    second.exec("CheckClassification('fixture classification', true, false); CheckClassification(303, true, true)").unwrap();
}

#[test]
fn public_queries_allow_tainted_callers_without_changing_taint() {
    seeded_env()
        .exec(&format!(
            r#"
        local function probe()
            local before = debug.getstacktaint()
            CheckClassification(101, true, false)
            CheckClassification('fixture classification', true, false)
            CheckClassification('{LINK}', false, true)
            CheckClassification('unknown', false, false)
            assert(debug.getstacktaint() == before)
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ClassificationPublicFixture')
            probe()
            assert(debug.getstacktaint() == 'ClassificationPublicFixture')
        end
        debug.setobjecttaint(addon, 'ClassificationPublicFixture')
        addon(); assert(issecure())
    "#
        ))
        .unwrap();
}

#[test]
fn actual_secret_values_reject_after_gc_with_identity_taint_and_public_recovery() {
    let env = seeded_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(issecure()); ProbeClassificationSecrets(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'ClassificationSecretFixture')
            ProbeClassificationSecrets()
            assert(debug.getstacktaint() == 'ClassificationSecretFixture')
        end
        debug.setobjecttaint(addon, 'ClassificationSecretFixture')
        addon(); assert(issecure())
    "#,
    )
    .expect("INFERRED conservative secret rejection in both caller contexts");
}
