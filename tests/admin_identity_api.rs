//! Tests for A_Admin identity API.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SetPlayerName
// ============================================================================

#[test]
fn test_set_player_name() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            A_Admin.SetPlayerName("Aldric")
            local n = UnitName("player")
            return n
            "#,
        )
        .unwrap();
    assert_eq!(name, "Aldric");
}

#[test]
fn test_set_player_name_updates_unit_name() {
    let env = env();
    let name: String = env
        .eval(
            r#"
            A_Admin.SetPlayerName("Zephyros")
            return UnitName("player")
            "#,
        )
        .unwrap();
    assert_eq!(name, "Zephyros");
}

#[test]
fn test_set_player_name_after_startup_has_no_raid_ui_errors() {
    let env = env();
    crate::common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    crate::common::install_error_collector(&env, "__raid_name_errors");
    wow_ui_sim::startup::fire_startup_events(&env);
    crate::common::drain_string_table(&env, "__raid_name_errors");

    for name in ["R88g8 First Name", "R88g8 Second Name"] {
        let actual: String = env
            .eval(&format!("A_Admin.SetPlayerName({name:?}); return UnitName('player')"))
            .expect("player name update");
        assert_eq!(actual, name);
    }
    assert!(
        crate::common::drain_string_table(&env, "__raid_name_errors").is_empty(),
        "name updates must not add Lua errors after roster startup"
    );
}

#[test]
fn test_set_player_name_realm_still_nil() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
            A_Admin.SetPlayerName("Aldric")
            local n, r = UnitName("player")
            return r == nil
            "#,
        )
        .unwrap();
    assert!(is_nil);
}

// ============================================================================
// SetPlayerClass
// ============================================================================

#[test]
fn test_set_player_class_paladin() {
    let env = env();
    let (class_name, class_index): (String, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerClass(2)
            local name, file, idx = UnitClass("player")
            return name, idx
            "#,
        )
        .unwrap();
    assert_eq!(class_name, "Paladin");
    assert_eq!(class_index, 2);
}

#[test]
fn test_set_player_class_warrior() {
    let env = env();
    let (class_name, class_file, class_index): (String, String, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerClass(1)
            return UnitClass("player")
            "#,
        )
        .unwrap();
    assert_eq!(class_name, "Warrior");
    assert_eq!(class_file, "WARRIOR");
    assert_eq!(class_index, 1);
}

#[test]
fn test_set_player_class_mage() {
    let env = env();
    let (class_name, class_file, class_index): (String, String, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerClass(8)
            return UnitClass("player")
            "#,
        )
        .unwrap();
    assert_eq!(class_name, "Mage");
    assert_eq!(class_file, "MAGE");
    assert_eq!(class_index, 8);
}

#[test]
fn test_set_player_class_class_file_matches() {
    let env = env();
    let class_file: String = env
        .eval(
            r#"
            A_Admin.SetPlayerClass(2)
            local _, file, _ = UnitClass("player")
            return file
            "#,
        )
        .unwrap();
    assert_eq!(class_file, "PALADIN");
}

// ============================================================================
// SetPlayerRace
// ============================================================================

#[test]
fn test_set_player_race_human() {
    let env = env();
    // RACE_DATA is 0-indexed: 0=Human, 1=Orc, 2=Dwarf, ...
    let race_name: String = env
        .eval(
            r#"
            A_Admin.SetPlayerRace(0)
            local name, file = UnitRace("player")
            return name
            "#,
        )
        .unwrap();
    assert_eq!(race_name, "Human");
}

#[test]
fn test_set_player_race_orc() {
    let env = env();
    let race_name: String = env
        .eval(
            r#"
            A_Admin.SetPlayerRace(1)
            local name, file = UnitRace("player")
            return name
            "#,
        )
        .unwrap();
    assert_eq!(race_name, "Orc");
}

#[test]
fn test_set_player_race_returns_both_values() {
    let env = env();
    let (race_name, race_file): (String, String) = env
        .eval(
            r#"
            A_Admin.SetPlayerRace(0)
            return UnitRace("player")
            "#,
        )
        .unwrap();
    assert_eq!(race_name, "Human");
    assert_eq!(race_file, "Human");
}

// ============================================================================
// SetPlayerLevel
// ============================================================================

#[test]
fn test_set_player_level() {
    let env = env();
    let level: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerLevel(60)
            return UnitLevel("player")
            "#,
        )
        .unwrap();
    assert_eq!(level, 60);
}

#[test]
fn test_set_player_level_max() {
    let env = env();
    let level: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerLevel(80)
            return UnitLevel("player")
            "#,
        )
        .unwrap();
    assert_eq!(level, 80);
}

#[test]
fn test_set_player_level_low() {
    let env = env();
    let level: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerLevel(1)
            return UnitLevel("player")
            "#,
        )
        .unwrap();
    assert_eq!(level, 1);
}

// ============================================================================
// SetPlayerSex
// ============================================================================

#[test]
fn test_set_player_sex_male() {
    let env = env();
    let sex: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerSex(2)
            return UnitSex("player")
            "#,
        )
        .unwrap();
    assert_eq!(sex, 2);
}

#[test]
fn test_set_player_sex_female() {
    let env = env();
    let sex: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerSex(3)
            return UnitSex("player")
            "#,
        )
        .unwrap();
    assert_eq!(sex, 3);
}
