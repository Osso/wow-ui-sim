//! Pin the field set emitted by `build_aura_table` (now split into
//! `write_aura_identity` + `write_aura_flags`) in
//! `src/lua_api/globals/auras.rs`. Exercises `C_UnitAuras.GetAuraDataBySlot`
//! against the fixture BUFF/DEBUFF slots.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AuraInfo, PartyMember};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn buff_slot_table_has_every_documented_field() {
    let env = env();
    let (name, icon, duration, source, spell_id, is_helpful, is_harmful, applications, time_mod): (
        String,
        f64,
        f64,
        String,
        f64,
        bool,
        bool,
        f64,
        f64,
    ) = env
        .eval(
            r#"
            local aura = C_UnitAuras.GetAuraDataBySlot("player", 1)
            return aura.name, aura.icon, aura.duration, aura.sourceUnit,
                   aura.spellId, aura.isHelpful, aura.isHarmful,
                   aura.applications, aura.timeMod
            "#,
        )
        .unwrap();
    assert!(!name.is_empty());
    assert!(icon > 0.0);
    assert!(duration > 0.0);
    assert_eq!(source, "player");
    assert!(spell_id > 0.0);
    assert!(is_helpful);
    assert!(!is_harmful);
    assert!(applications >= 0.0);
    assert_eq!(time_mod, 1.0);
}

#[test]
fn debuff_slot_table_reports_harmful() {
    let env = env();
    let (is_helpful, is_harmful, is_raid): (bool, bool, bool) = env
        .eval(
            r#"
            local aura = C_UnitAuras.GetAuraDataBySlot("target", 2)
            return aura.isHelpful, aura.isHarmful, aura.isRaid
            "#,
        )
        .unwrap();
    assert!(!is_helpful);
    assert!(is_harmful);
    // The seeded Expose Weakness fixture explicitly has no raid classification.
    assert!(!is_raid);
}

#[test]
fn aura_table_exposes_boolean_flag_shape() {
    let env = env();
    // Default buffs select from player and party sources; slot 1 is not always player-cast.
    let expected = env
        .state()
        .borrow()
        .player
        .buffs
        .iter()
        .find(|aura| aura.aura_instance_id == 1)
        .expect("default player aura at slot 1")
        .clone();
    let (
        name,
        source,
        can_apply,
        boss,
        from_player,
        nameplate_all,
        nameplate_personal,
        nameplate_only,
        stealable,
    ): (String, String, bool, bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local aura = C_UnitAuras.GetAuraDataBySlot("player", 1)
            return aura.name, aura.sourceUnit, aura.canApplyAura,
                   aura.isBossAura, aura.isFromPlayerOrPlayerPet,
                   aura.nameplateShowAll, aura.nameplateShowPersonal,
                   aura.isNameplateOnly, aura.isStealable
            "#,
        )
        .unwrap();
    assert_eq!(name, expected.name);
    assert_eq!(source, expected.source_unit);
    assert!(can_apply, "canApplyAura default is true");
    assert!(!boss);
    assert_eq!(from_player, expected.is_from_player_or_player_pet);
    assert!(!nameplate_all);
    assert!(!nameplate_personal);
    assert!(!nameplate_only);
    assert!(!stealable);
}

#[test]
fn aura_applications_charges_stackcount_are_aliased() {
    let env = env();
    let (a, c, s): (f64, f64, f64) = env
        .eval(
            r#"
            local aura = C_UnitAuras.GetAuraDataBySlot("player", 1)
            return aura.applications, aura.charges, aura.stackCount
            "#,
        )
        .unwrap();
    assert_eq!(a, c, "applications and charges must be equal");
    assert_eq!(c, s, "charges and stackCount must be equal");
}

fn helpful_non_raid_fixture() -> AuraInfo {
    AuraInfo {
        name: "Helpful non-raid".into(),
        spell_id: 99001,
        icon: 135841,
        duration: 60.0,
        expiration_time: 60.0,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: true,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: false,
        dispel_type: None,
        aura_instance_id: 101,
    }
}

fn harmful_raid_fixture() -> AuraInfo {
    AuraInfo {
        name: "Harmful raid".into(),
        spell_id: 99002,
        icon: 136039,
        duration: 30.0,
        expiration_time: 30.0,
        applications: 2,
        source_unit: "party1".into(),
        is_helpful: false,
        is_raid: true,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: false,
        is_from_player_or_player_pet: true,
        dispel_type: Some("Magic".into()),
        aura_instance_id: 102,
    }
}

fn classification_env() -> WowLuaEnv {
    let env = env();
    let helpful = helpful_non_raid_fixture();
    let harmful = harmful_raid_fixture();
    let mut party_helpful = helpful.clone();
    party_helpful.aura_instance_id = 201;
    party_helpful.is_nameplate_only = false;
    party_helpful.is_from_player_or_player_pet = true;
    let mut party_harmful = harmful.clone();
    party_harmful.aura_instance_id = 202;
    party_harmful.is_nameplate_only = true;
    party_harmful.is_from_player_or_player_pet = false;
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![helpful, harmful];
        state.party_members = vec![classification_party(party_helpful, party_harmful)];
    }
    env
}

fn classification_party(helpful: AuraInfo, harmful: AuraInfo) -> PartyMember {
    PartyMember {
        name: "Aura fixture party member".into(),
        connected: true,
        class_index: 2,
        level: 80,
        health: 100,
        health_max: 100,
        power: 100,
        power_max: 100,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_leader: false,
        dead_since: None,
        buffs: vec![helpful],
        debuffs: vec![harmful],
    }
}

#[test]
fn explicit_classification_flags_are_public_independent_and_query_consistent() {
    let env = classification_env();
    env.exec(
        r#"
        local fields = {
            "isHelpful", "isHarmful", "isRaid", "isNameplateOnly",
            "isFromPlayerOrPlayerPet",
        }
        local fixtures = {
            {"player", 101, "HELPFUL", {true, false, false, true, false}},
            {"player", 102, "HARMFUL", {false, true, true, false, true}},
            {"party1", 201, "HELPFUL", {true, false, false, false, true}},
            {"party1", 202, "HARMFUL", {false, true, true, true, false}},
            -- Target has fixed Rust fixtures, not a host-populated aura store.
            {"target", 1, "HARMFUL", {false, true, false, false, false}},
        }
        local function addon()
            assert(debug.getstacktaint() == "AuraClassificationFixture")
            for _, fixture in ipairs(fixtures) do
                local unit, id, filter, expected = unpack(fixture)
                local token, slot = C_UnitAuras.GetAuraSlots(unit, filter)
                assert(token == nil and slot == id, unit .. " slot identity")
                local queries = {
                    C_UnitAuras.GetAuraDataBySlot(unit, slot),
                    C_UnitAuras.GetAuraDataByIndex(unit, 1, filter),
                    C_UnitAuras.GetAuraDataByAuraInstanceID(unit, id),
                }
                assert(#queries == 3, unit .. " missing query result")
                for _, aura in ipairs(queries) do
                    assert(aura.auraInstanceID == id, unit .. " instance identity")
                    for i, field in ipairs(fields) do
                        local value = aura[field]
                        assert(type(value) == "boolean", unit .. " " .. field .. " type")
                        assert(not issecretvalue(value), unit .. " " .. field .. " secrecy")
                        assert(value == expected[i], unit .. " " .. field .. " value")
                    end
                    assert(debug.getstacktaint() == "AuraClassificationFixture",
                        "aura query must preserve addon taint")
                end
            end
        end
        debug.setobjecttaint(addon, "AuraClassificationFixture")
        addon()
        "#,
    )
    .unwrap();
}

#[test]
fn classification_query_snapshots_do_not_mutate_host_or_other_results() {
    let env = classification_env();
    env.exec(
        r#"
        local fields = {
            "isHelpful", "isHarmful", "isRaid", "isNameplateOnly",
            "isFromPlayerOrPlayerPet",
        }
        local function addon()
            for _, unit in ipairs({"player", "party1", "target"}) do
                local id = unit == "player" and 101 or unit == "party1" and 201 or 1
                local filter = unit == "target" and "HARMFUL" or "HELPFUL"
                local original = C_UnitAuras.GetAuraDataBySlot(unit, id)
                local independent = C_UnitAuras.GetAuraDataByAuraInstanceID(unit, id)
                assert(original ~= independent, unit .. " independent tables")
                for _, field in ipairs(fields) do
                    original[field] = not original[field]
                end
                local fresh = C_UnitAuras.GetAuraDataByIndex(unit, 1, filter)
                for _, field in ipairs(fields) do
                    assert(fresh[field] == independent[field], unit .. " " .. field .. " snapshot")
                    assert(original[field] ~= fresh[field], unit .. " " .. field .. " mutation")
                    assert(type(fresh[field]) == "boolean" and not issecretvalue(fresh[field]))
                end
                assert(debug.getstacktaint() == "AuraClassificationFixture")
            end
        end
        debug.setobjecttaint(addon, "AuraClassificationFixture")
        addon()
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    let helpful = &state.player.buffs[0];
    assert!(helpful.is_helpful);
    assert!(!helpful.is_raid);
    assert!(helpful.is_nameplate_only);
    assert!(!helpful.is_from_player_or_player_pet);
    let harmful = &state.player.buffs[1];
    assert!(!harmful.is_helpful);
    assert!(harmful.is_raid);
    assert!(!harmful.is_nameplate_only);
    assert!(harmful.is_from_player_or_player_pet);
    let party = &state.party_members[0];
    assert!(!party.buffs[0].is_raid);
    assert!(!party.buffs[0].is_nameplate_only);
    assert!(party.buffs[0].is_from_player_or_player_pet);
    assert!(party.debuffs[0].is_raid);
    assert!(party.debuffs[0].is_nameplate_only);
    assert!(!party.debuffs[0].is_from_player_or_player_pet);
}

#[test]
fn aura_points_field_is_an_empty_table() {
    let env = env();
    let (is_table, count): (bool, i64) = env
        .eval(
            r#"
            local aura = C_UnitAuras.GetAuraDataBySlot("player", 1)
            return type(aura.points) == "table", #aura.points
            "#,
        )
        .unwrap();
    assert!(is_table);
    assert_eq!(count, 0);
}
