//! `C_GameRules` probes — SimState-backed round-trip.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn game_rules_hardcore_flag_defaults_to_false() {
    // Legacy coverage — IsHardcoreActive is still installed from the
    // namespace-stub pass even though the PLAN-listed rule getters moved
    // to SimState.
    let env = env();
    let result: String = env
        .eval(
            r#"
            if type(C_GameRules.IsHardcoreActive) ~= "function" then
                return "missing_is_hardcore_active"
            end
            if C_GameRules.IsHardcoreActive() then
                return "hardcore_should_default_false"
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn active_game_mode_matches_published_enum() {
    let env = env();
    let standard: bool = env
        .eval("return C_GameRules.GetActiveGameMode() == Enum.GameMode.Standard")
        .unwrap();
    env.exec("A_Admin.SetActiveGameMode(Enum.GameMode.Plunderstorm)")
        .unwrap();
    let plunderstorm: bool = env
        .eval("return C_GameRules.GetActiveGameMode() == Enum.GameMode.Plunderstorm and C_GameRules.IsPlunderstorm()")
        .unwrap();
    assert!(standard, "the default mode must be Enum.GameMode.Standard");
    assert!(plunderstorm, "the supplied mode must be Enum.GameMode.Plunderstorm");
}

#[test]
fn defaults_no_rules_standard_mode_character_select() {
    let (active, f, i, s, plunder, mode, glue): (bool, f64, i64, String, bool, i64, String) = env()
        .eval(
            r#"
            return C_GameRules.IsGameRuleActive("Anything"),
                   C_GameRules.GetGameRuleAsFloat("Anything"),
                   C_GameRules.GetGameRuleAsInt("Anything"),
                   C_GameRules.GetGameRuleAsString("Anything"),
                   C_GameRules.IsPlunderstorm(),
                   C_GameRules.GetActiveGameMode(),
                   C_GameRules.GetGameModeGlueScreenName()
            "#,
        )
        .unwrap();
    assert!(!active);
    assert_eq!(f, 0.0);
    assert_eq!(i, 0);
    assert_eq!(s, "");
    assert!(!plunder);
    assert_eq!(mode, 1);
    assert_eq!(glue, "CharacterSelect");
}

#[test]
fn number_rule_round_trips_as_float_int_and_string() {
    let env = env();
    env.exec(r#"A_Admin.SetGameRule("MAX_PLAYERS", 42)"#)
        .unwrap();
    let (active, f, i, s): (bool, f64, i64, String) = env
        .eval(
            r#"
            return C_GameRules.IsGameRuleActive("MAX_PLAYERS"),
                   C_GameRules.GetGameRuleAsFloat("MAX_PLAYERS"),
                   C_GameRules.GetGameRuleAsInt("MAX_PLAYERS"),
                   C_GameRules.GetGameRuleAsString("MAX_PLAYERS")
            "#,
        )
        .unwrap();
    assert!(active);
    assert_eq!(f, 42.0);
    assert_eq!(i, 42);
    assert_eq!(s, "42");
}

#[test]
fn string_rule_parses_numeric_form_when_possible() {
    let env = env();
    env.exec(r#"A_Admin.SetGameRule("LEVEL_CAP", "80")"#)
        .unwrap();
    let (f, i, s): (f64, i64, String) = env
        .eval(
            r#"
            return C_GameRules.GetGameRuleAsFloat("LEVEL_CAP"),
                   C_GameRules.GetGameRuleAsInt("LEVEL_CAP"),
                   C_GameRules.GetGameRuleAsString("LEVEL_CAP")
            "#,
        )
        .unwrap();
    assert_eq!(f, 80.0);
    assert_eq!(i, 80);
    assert_eq!(s, "80");
}

#[test]
fn non_numeric_string_keeps_string_form_but_numbers_default_to_zero() {
    let env = env();
    env.exec(r#"A_Admin.SetGameRule("SEASON", "DragonFlight")"#)
        .unwrap();
    let (f, i, s): (f64, i64, String) = env
        .eval(
            r#"
            return C_GameRules.GetGameRuleAsFloat("SEASON"),
                   C_GameRules.GetGameRuleAsInt("SEASON"),
                   C_GameRules.GetGameRuleAsString("SEASON")
            "#,
        )
        .unwrap();
    assert_eq!(f, 0.0);
    assert_eq!(i, 0);
    assert_eq!(s, "DragonFlight");
}

#[test]
fn bool_true_is_active_false_removes() {
    let env = env();
    env.exec(r#"A_Admin.SetGameRule("FEATURE_FOO", true)"#)
        .unwrap();
    let active: bool = env
        .eval(r#"return C_GameRules.IsGameRuleActive("FEATURE_FOO")"#)
        .unwrap();
    assert!(active);
    env.exec(r#"A_Admin.SetGameRule("FEATURE_FOO", false)"#)
        .unwrap();
    let active: bool = env
        .eval(r#"return C_GameRules.IsGameRuleActive("FEATURE_FOO")"#)
        .unwrap();
    assert!(!active, "SetGameRule(name, false) removes the rule");
}

#[test]
fn nil_removes_rule() {
    let env = env();
    env.exec(r#"A_Admin.SetGameRule("X", 5)"#).unwrap();
    env.exec(r#"A_Admin.SetGameRule("X", nil)"#).unwrap();
    let active: bool = env
        .eval(r#"return C_GameRules.IsGameRuleActive("X")"#)
        .unwrap();
    assert!(!active);
}

#[test]
fn set_active_game_mode_flips_is_plunderstorm() {
    let env = env();
    env.exec(r#"A_Admin.SetActiveGameMode(Enum.GameMode.Plunderstorm, "PlunderstormLogin")"#)
        .unwrap();
    let (mode, plunder, glue): (i64, bool, String) = env
        .eval(
            r#"
            return C_GameRules.GetActiveGameMode(),
                   C_GameRules.IsPlunderstorm(),
                   C_GameRules.GetGameModeGlueScreenName()
            "#,
        )
        .unwrap();
    assert_eq!(mode, 2);
    assert!(plunder);
    assert_eq!(glue, "PlunderstormLogin");
}

#[test]
fn set_active_game_mode_back_to_standard_resets_glue_default() {
    let env = env();
    env.exec(r#"A_Admin.SetActiveGameMode(Enum.GameMode.Plunderstorm, "PlunderstormLogin")"#)
        .unwrap();
    env.exec(r#"A_Admin.SetActiveGameMode(Enum.GameMode.Standard)"#).unwrap();
    let (mode, plunder, glue): (i64, bool, String) = env
        .eval(
            r#"
            return C_GameRules.GetActiveGameMode(),
                   C_GameRules.IsPlunderstorm(),
                   C_GameRules.GetGameModeGlueScreenName()
            "#,
        )
        .unwrap();
    assert_eq!(mode, 1);
    assert!(!plunder);
    assert_eq!(
        glue, "CharacterSelect",
        "no-glue arg falls back to CharacterSelect",
    );
}

#[test]
fn multiple_rules_coexist() {
    let env = env();
    env.exec(r#"A_Admin.SetGameRule("A", 1)"#).unwrap();
    env.exec(r#"A_Admin.SetGameRule("B", "bee")"#).unwrap();
    env.exec(r#"A_Admin.SetGameRule("C", true)"#).unwrap();
    let (a, b, c): (f64, String, bool) = env
        .eval(
            r#"
            return C_GameRules.GetGameRuleAsFloat("A"),
                   C_GameRules.GetGameRuleAsString("B"),
                   C_GameRules.IsGameRuleActive("C")
            "#,
        )
        .unwrap();
    assert_eq!(a, 1.0);
    assert_eq!(b, "bee");
    assert!(c);
}

#[test]
fn numeric_enum_game_rule_ids_resolve_to_rule_names() {
    let env = env();
    env.exec(r#"A_Admin.SetGameRule("ForcedPartyFrameScale", 125)"#)
        .unwrap();
    let (active, value): (bool, f64) = env
        .eval(
            r#"
            return C_GameRules.IsGameRuleActive(Enum.GameRule.ForcedPartyFrameScale),
                   C_GameRules.GetGameRuleAsFloat(Enum.GameRule.ForcedPartyFrameScale, 2)
            "#,
        )
        .unwrap();
    assert!(active, "Enum.GameRule id should resolve to the named rule");
    assert_eq!(value, 1.25);
}
