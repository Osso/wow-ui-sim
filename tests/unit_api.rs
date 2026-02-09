//! Tests for unit API functions (unit_api.rs).

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// UnitRace
// ============================================================================

#[test]
fn test_unit_race_returns_name_and_file() {
    let env = env();
    let (name, file): (String, String) = env
        .eval("return UnitRace('player')")
        .unwrap();
    assert_eq!(name, "Human");
    assert_eq!(file, "Human");
}

// ============================================================================
// UnitSex
// ============================================================================

#[test]
fn test_unit_sex_returns_male() {
    let env = env();
    let sex: i32 = env.eval("return UnitSex('player')").unwrap();
    assert_eq!(sex, 2);
}

// ============================================================================
// UnitClass
// ============================================================================

#[test]
fn test_unit_class_returns_three_values() {
    let env = env();
    let (name, file, id): (String, String, i32) = env
        .eval("return UnitClass('player')")
        .unwrap();
    assert_eq!(name, "Warrior");
    assert_eq!(file, "WARRIOR");
    assert_eq!(id, 1);
}

// ============================================================================
// UnitClassBase
// ============================================================================

#[test]
fn test_unit_class_base_returns_file_name() {
    let env = env();
    let file: String = env.eval("return UnitClassBase('player')").unwrap();
    assert_eq!(file, "WARRIOR");
}

// ============================================================================
// GetNumClasses
// ============================================================================

#[test]
fn test_get_num_classes() {
    let env = env();
    let num: i32 = env.eval("return GetNumClasses()").unwrap();
    assert_eq!(num, 13);
}

// ============================================================================
// GetClassInfo
// ============================================================================

#[test]
fn test_get_class_info_warrior() {
    let env = env();
    let (name, file, id): (String, String, i32) = env
        .eval("return GetClassInfo(1)")
        .unwrap();
    assert_eq!(name, "Warrior");
    assert_eq!(file, "WARRIOR");
    assert_eq!(id, 1);
}

#[test]
fn test_get_class_info_evoker() {
    let env = env();
    let (name, file, id): (String, String, i32) = env
        .eval("return GetClassInfo(13)")
        .unwrap();
    assert_eq!(name, "Evoker");
    assert_eq!(file, "EVOKER");
    assert_eq!(id, 13);
}

#[test]
fn test_get_class_info_death_knight() {
    let env = env();
    let (name, file, id): (String, String, i32) = env
        .eval("return GetClassInfo(6)")
        .unwrap();
    assert_eq!(name, "Death Knight");
    assert_eq!(file, "DEATHKNIGHT");
    assert_eq!(id, 6);
}

#[test]
fn test_get_class_info_unknown_index() {
    let env = env();
    let (name, file): (String, String) = env
        .eval("return GetClassInfo(99)")
        .unwrap();
    assert_eq!(name, "Unknown");
    assert_eq!(file, "UNKNOWN");
}

#[test]
fn test_get_class_info_all_classes() {
    let env = env();
    let expected = [
        (1, "Warrior", "WARRIOR"),
        (2, "Paladin", "PALADIN"),
        (3, "Hunter", "HUNTER"),
        (4, "Rogue", "ROGUE"),
        (5, "Priest", "PRIEST"),
        (6, "Death Knight", "DEATHKNIGHT"),
        (7, "Shaman", "SHAMAN"),
        (8, "Mage", "MAGE"),
        (9, "Warlock", "WARLOCK"),
        (10, "Monk", "MONK"),
        (11, "Druid", "DRUID"),
        (12, "Demon Hunter", "DEMONHUNTER"),
        (13, "Evoker", "EVOKER"),
    ];
    for (idx, exp_name, exp_file) in expected {
        let (name, file, id): (String, String, i32) = env
            .eval(&format!("return GetClassInfo({})", idx))
            .unwrap();
        assert_eq!(name, exp_name, "class index {}", idx);
        assert_eq!(file, exp_file, "class index {}", idx);
        assert_eq!(id, idx, "class index {}", idx);
    }
}

// ============================================================================
// LocalizedClassList
// ============================================================================

#[test]
fn test_localized_class_list() {
    let env = env();
    let warrior: String = env
        .eval("return LocalizedClassList()['WARRIOR']")
        .unwrap();
    assert_eq!(warrior, "Warrior");
}

#[test]
fn test_localized_class_list_all_entries() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local t = LocalizedClassList()
            local n = 0
            for _ in pairs(t) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 13);
}

#[test]
fn test_localized_class_list_specific_entries() {
    let env = env();
    let (dk, dh): (String, String) = env
        .eval("local t = LocalizedClassList(); return t['DEATHKNIGHT'], t['DEMONHUNTER']")
        .unwrap();
    assert_eq!(dk, "Death Knight");
    assert_eq!(dh, "Demon Hunter");
}

// ============================================================================
// UnitName
// ============================================================================

#[test]
fn test_unit_name_player() {
    let env = env();
    let matches: bool = env
        .eval("local n = UnitName('player'); return n == UnitName('player')")
        .unwrap();
    assert!(matches, "UnitName('player') should return a consistent name");
    let name: String = env.eval("return UnitName('player')").unwrap();
    assert!(!name.is_empty(), "Player name should not be empty");
}

#[test]
fn test_unit_name_no_target() {
    let env = env();
    let name: String = env
        .eval("local n, r = UnitName('target'); return n")
        .unwrap();
    assert_eq!(name, "Unknown");
}

#[test]
fn test_unit_name_realm_is_nil() {
    let env = env();
    let is_nil: bool = env
        .eval("local n, r = UnitName('player'); return r == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// UnitNameUnmodified
// ============================================================================

#[test]
fn test_unit_name_unmodified_player() {
    let env = env();
    let same: bool = env
        .eval("return UnitNameUnmodified('player') == UnitName('player')")
        .unwrap();
    assert!(same, "UnitNameUnmodified should match UnitName for player");
}

#[test]
fn test_unit_name_unmodified_no_target() {
    let env = env();
    let name: String = env
        .eval("local n, r = UnitNameUnmodified('target'); return n")
        .unwrap();
    assert_eq!(name, "Unknown");
}

// ============================================================================
// UnitFullName
// ============================================================================

#[test]
fn test_unit_full_name_player() {
    let env = env();
    let (name, realm): (String, String) = env
        .eval("return UnitFullName('player')")
        .unwrap();
    let player_name: String = env.eval("return UnitName('player')").unwrap();
    assert_eq!(name, player_name);
    assert_eq!(realm, "SimRealm");
}

#[test]
fn test_unit_full_name_no_target() {
    let env = env();
    let (name, realm): (String, String) = env
        .eval("return UnitFullName('target')")
        .unwrap();
    assert_eq!(name, "Unknown");
    assert_eq!(realm, "SimRealm");
}

// ============================================================================
// GetUnitName
// ============================================================================

#[test]
fn test_get_unit_name_player() {
    let env = env();
    let name: String = env.eval("return GetUnitName('player')").unwrap();
    let player_name: String = env.eval("return UnitName('player')").unwrap();
    assert_eq!(name, player_name);
}

#[test]
fn test_get_unit_name_no_target() {
    let env = env();
    let name: String = env.eval("return GetUnitName('target', true)").unwrap();
    assert_eq!(name, "Unknown");
}

// ============================================================================
// UnitGUID
// ============================================================================

#[test]
fn test_unit_guid_player() {
    let env = env();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    assert_eq!(guid, "Player-0000-00000001");
}

#[test]
fn test_unit_guid_other() {
    let env = env();
    let guid: String = env.eval("return UnitGUID('target')").unwrap();
    assert_eq!(guid, "Creature-0000-00000000");
}

// ============================================================================
// UnitLevel / UnitEffectiveLevel
// ============================================================================

#[test]
fn test_unit_level() {
    let env = env();
    let level: i32 = env.eval("return UnitLevel('player')").unwrap();
    assert_eq!(level, 70);
}

#[test]
fn test_unit_effective_level() {
    let env = env();
    let level: i32 = env.eval("return UnitEffectiveLevel('player')").unwrap();
    assert_eq!(level, 70);
}

// ============================================================================
// UnitExists
// ============================================================================

#[test]
fn test_unit_exists_player() {
    let env = env();
    let exists: bool = env.eval("return UnitExists('player')").unwrap();
    assert!(exists);
}

#[test]
fn test_unit_exists_no_target() {
    let env = env();
    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(!exists, "No target should exist by default");
}

#[test]
fn test_unit_exists_target_after_targeting() {
    let env = env();
    env.eval::<()>("TargetUnit('player')").unwrap();
    let exists: bool = env.eval("return UnitExists('target')").unwrap();
    assert!(exists, "Target should exist after TargetUnit");
}

#[test]
fn test_unit_exists_pet() {
    let env = env();
    let exists: bool = env.eval("return UnitExists('pet')").unwrap();
    assert!(exists);
}

#[test]
fn test_unit_exists_party_member() {
    let env = env();
    // Default state has 4 party members
    let exists: bool = env.eval("return UnitExists('party1')").unwrap();
    assert!(exists, "party1 should exist with default party");
}

#[test]
fn test_unit_exists_unknown() {
    let env = env();
    let exists: bool = env.eval("return UnitExists('nobody')").unwrap();
    assert!(!exists);
}

// ============================================================================
// UnitFactionGroup
// ============================================================================

#[test]
fn test_unit_faction_group() {
    let env = env();
    let (english, localized): (String, String) = env
        .eval("return UnitFactionGroup('player')")
        .unwrap();
    assert_eq!(english, "Alliance");
    assert_eq!(localized, "Alliance");
}

// ============================================================================
// Unit state functions
// ============================================================================

#[test]
fn test_unit_is_dead_or_ghost() {
    let env = env();
    let val: bool = env.eval("return UnitIsDeadOrGhost('player')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_dead() {
    let env = env();
    let val: bool = env.eval("return UnitIsDead('player')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_ghost() {
    let env = env();
    let val: bool = env.eval("return UnitIsGhost('player')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_afk() {
    let env = env();
    let val: bool = env.eval("return UnitIsAFK('player')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_dnd() {
    let env = env();
    let val: bool = env.eval("return UnitIsDND('player')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_connected() {
    let env = env();
    let val: bool = env.eval("return UnitIsConnected('player')").unwrap();
    assert!(val);
}

#[test]
fn test_unit_is_player_true() {
    let env = env();
    let val: bool = env.eval("return UnitIsPlayer('player')").unwrap();
    assert!(val);
}

#[test]
fn test_unit_is_player_false() {
    let env = env();
    let val: bool = env.eval("return UnitIsPlayer('target')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_player_controlled() {
    let env = env();
    let player: bool = env.eval("return UnitPlayerControlled('player')").unwrap();
    let pet: bool = env.eval("return UnitPlayerControlled('pet')").unwrap();
    let target: bool = env.eval("return UnitPlayerControlled('target')").unwrap();
    assert!(player);
    assert!(pet);
    assert!(!target);
}

#[test]
fn test_unit_is_tap_denied() {
    let env = env();
    let val: bool = env.eval("return UnitIsTapDenied('target')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_enemy() {
    let env = env();
    let val: bool = env.eval("return UnitIsEnemy('player', 'target')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_friend() {
    let env = env();
    let val: bool = env.eval("return UnitIsFriend('player', 'target')").unwrap();
    assert!(val);
}

#[test]
fn test_unit_can_attack() {
    let env = env();
    let val: bool = env.eval("return UnitCanAttack('player', 'target')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_can_assist() {
    let env = env();
    let val: bool = env.eval("return UnitCanAssist('player', 'target')").unwrap();
    assert!(val);
}

#[test]
fn test_unit_is_unit_same() {
    let env = env();
    let val: bool = env.eval("return UnitIsUnit('player', 'player')").unwrap();
    assert!(val);
}

#[test]
fn test_unit_is_unit_different() {
    let env = env();
    let val: bool = env.eval("return UnitIsUnit('player', 'target')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_visible() {
    let env = env();
    let val: bool = env.eval("return UnitIsVisible('player')").unwrap();
    assert!(val);
}

#[test]
fn test_unit_in_range() {
    let env = env();
    let (in_range, checked): (bool, bool) = env
        .eval("return UnitInRange('player')")
        .unwrap();
    assert!(in_range);
    assert!(checked);
}

// ============================================================================
// Group/party functions
// ============================================================================

#[test]
fn test_unit_in_party() {
    let env = env();
    let val: bool = env.eval("return UnitInParty('player')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_in_raid() {
    let env = env();
    let is_nil: bool = env.eval("return UnitInRaid('player') == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_unit_is_group_leader() {
    let env = env();
    let val: bool = env.eval("return UnitIsGroupLeader('player')").unwrap();
    assert!(!val);
}

#[test]
fn test_unit_is_group_assistant() {
    let env = env();
    let val: bool = env.eval("return UnitIsGroupAssistant('player')").unwrap();
    assert!(!val);
}

// ============================================================================
// Health/Power functions
// ============================================================================

#[test]
fn test_unit_health() {
    let env = env();
    let hp: i32 = env.eval("return UnitHealth('player')").unwrap();
    assert_eq!(hp, 100000);
}

#[test]
fn test_unit_health_max() {
    let env = env();
    let hp: i32 = env.eval("return UnitHealthMax('player')").unwrap();
    assert_eq!(hp, 100000);
}

#[test]
fn test_unit_power() {
    let env = env();
    let power: i32 = env.eval("return UnitPower('player')").unwrap();
    assert_eq!(power, 50000);
}

#[test]
fn test_unit_power_with_type() {
    let env = env();
    let power: i32 = env.eval("return UnitPower('player', 0)").unwrap();
    assert_eq!(power, 50000);
}

#[test]
fn test_unit_power_max() {
    let env = env();
    let power: i32 = env.eval("return UnitPowerMax('player')").unwrap();
    assert_eq!(power, 100000);
}

#[test]
fn test_unit_power_type() {
    let env = env();
    let (power_type, token): (i32, String) = env
        .eval("return UnitPowerType('player')")
        .unwrap();
    assert_eq!(power_type, 0);
    assert_eq!(token, "MANA");
}

#[test]
fn test_unit_get_incoming_heals() {
    let env = env();
    let val: i32 = env.eval("return UnitGetIncomingHeals('player')").unwrap();
    assert_eq!(val, 0);
}

#[test]
fn test_unit_get_total_absorbs() {
    let env = env();
    let val: i32 = env.eval("return UnitGetTotalAbsorbs('player')").unwrap();
    assert_eq!(val, 0);
}

#[test]
fn test_unit_get_total_heal_absorbs() {
    let env = env();
    let val: i32 = env.eval("return UnitGetTotalHealAbsorbs('player')").unwrap();
    assert_eq!(val, 0);
}

// ============================================================================
// Threat functions
// ============================================================================

#[test]
fn test_unit_threat_situation() {
    let env = env();
    let is_nil: bool = env
        .eval("return UnitThreatSituation('player', 'target') == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_unit_detailed_threat_situation() {
    let env = env();
    let (is_tanking, status): (bool, i32) = env
        .eval("local a, b = UnitDetailedThreatSituation('player', 'target'); return a, b")
        .unwrap();
    assert!(!is_tanking);
    assert_eq!(status, 0);
}

// ============================================================================
// Classification functions
// ============================================================================

#[test]
fn test_unit_classification() {
    let env = env();
    let val: String = env.eval("return UnitClassification('target')").unwrap();
    assert_eq!(val, "normal");
}

#[test]
fn test_unit_creature_type() {
    let env = env();
    let val: String = env.eval("return UnitCreatureType('target')").unwrap();
    assert_eq!(val, "Humanoid");
}

#[test]
fn test_unit_creature_family() {
    let env = env();
    let is_nil: bool = env
        .eval("return UnitCreatureFamily('target') == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// Casting functions
// ============================================================================

#[test]
fn test_unit_casting_info() {
    let env = env();
    let is_nil: bool = env
        .eval("return UnitCastingInfo('player') == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_unit_channel_info() {
    let env = env();
    let is_nil: bool = env
        .eval("return UnitChannelInfo('player') == nil")
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// GetWeaponEnchantInfo
// ============================================================================

#[test]
fn test_get_weapon_enchant_info() {
    let env = env();
    let (has_mh, mh_exp, mh_charges, mh_id): (bool, i32, i32, i32) = env
        .eval("local a, b, c, d = GetWeaponEnchantInfo(); return a, b, c, d")
        .unwrap();
    assert!(!has_mh);
    assert_eq!(mh_exp, 0);
    assert_eq!(mh_charges, 0);
    assert_eq!(mh_id, 0);
}

#[test]
fn test_get_weapon_enchant_info_offhand() {
    let env = env();
    let (has_oh, oh_exp, oh_charges, oh_id): (bool, i32, i32, i32) = env
        .eval(
            r#"
            local a, b, c, d, e, f, g, h = GetWeaponEnchantInfo()
            return e, f, g, h
            "#,
        )
        .unwrap();
    assert!(!has_oh);
    assert_eq!(oh_exp, 0);
    assert_eq!(oh_charges, 0);
    assert_eq!(oh_id, 0);
}
