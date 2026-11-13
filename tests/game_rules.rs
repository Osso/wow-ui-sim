//! `C_GameRules` probes — SimState-backed round-trip.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn expected_default_glue_screen() -> Option<String> {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
    match ACTIVE {
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever => None,
        _ => Some("CharacterSelect".into()),
    }
}

#[test]
fn game_rules_hardcore_flag_defaults_to_false() {
    let env = env();
    // WM80: Retail 70688 has no native IsHardcoreActive registration; PTR docs add the boolean API.
    if cfg!(feature = "profile-retail") {
        let absent: bool = env
            .eval("return rawget(C_GameRules, 'IsHardcoreActive') == nil and C_GameRules.IsHardcoreActive == nil")
            .unwrap();
        assert!(absent);
        return;
    }
    if cfg!(feature = "client-ptr") {
        // PTR Blizzard_APIDocumentationGenerated/GameRulesDocumentation.lua:276-281.
        let (result_type, count): (String, i64) = env
            .eval(
                "local value = C_GameRules.IsHardcoreActive(); return type(value), select('#', C_GameRules.IsHardcoreActive())",
            )
            .expect("PTR IsHardcoreActive query succeeds");
        assert_eq!((result_type.as_str(), count), ("boolean", 1));
        return;
    }
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
#[cfg(feature = "client-wowforever")]
fn forever_self_found_allowed_uses_rule_22_and_defaults_false() {
    const SELF_FOUND_ALLOWED_RULE_ID: i32 = 22;

    let env = env();
    assert!(
        !env.state()
            .borrow()
            .game_rules
            .rules_by_id
            .contains_key(&SELF_FOUND_ALLOWED_RULE_ID)
    );

    let (allowed, return_count): (bool, i64) = env
        .eval(
            r##"
            local function pack(...) return { n = select("#", ...), ... } end
            local result = pack(C_GameRules.IsSelfFoundAllowed())
            return result[1], result.n
            "##,
        )
        .unwrap();
    assert_eq!((allowed, return_count), (false, 1));

    env.state()
        .borrow_mut()
        .game_rules
        .rules_by_id
        .insert(SELF_FOUND_ALLOWED_RULE_ID, 1);
    let (allowed, return_count): (bool, i64) = env
        .eval(
            r##"
            local function pack(...) return { n = select("#", ...), ... } end
            local result = pack(C_GameRules.IsSelfFoundAllowed())
            return result[1], result.n
            "##,
        )
        .unwrap();
    assert_eq!((allowed, return_count), (true, 1));
}

#[test]
fn standard_mode_predicate_tracks_supplied_game_mode() {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};

    let env = env();
    if ACTIVE == ClientProfile::Wrath {
        assert!(
            env.eval::<bool>("return C_GameRules.IsStandard == nil")
                .unwrap()
        );
        return;
    }
    for (mode, expected) in [("Standard", true), ("Plunderstorm", false)] {
        env.exec(&format!("A_Admin.SetActiveGameMode(Enum.GameMode.{mode})"))
            .unwrap();
        let (actual, count): (bool, i64) = env
            .eval("return C_GameRules.IsStandard(), select('#', C_GameRules.IsStandard())")
            .unwrap();
        assert_eq!((actual, count), (expected, 1), "{mode}");
    }
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
    assert!(
        plunderstorm,
        "the supplied mode must be Enum.GameMode.Plunderstorm"
    );
}

#[test]
fn defaults_no_rules_standard_mode_with_profile_glue_default() {
    let (active, f, i, s, plunder, mode, glue): (
        bool,
        f64,
        i64,
        String,
        bool,
        i64,
        Option<String>,
    ) = env()
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
    assert_eq!(glue, expected_default_glue_screen());
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
    env.exec(r#"A_Admin.SetActiveGameMode(Enum.GameMode.Standard)"#)
        .unwrap();
    let (mode, plunder, glue): (i64, bool, Option<String>) = env
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
        glue,
        expected_default_glue_screen(),
        "omitting the screen preserves the profile's existing default"
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

#[test]
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
fn empty_host_glue_screen_is_nil_and_nonempty_name_is_preserved() {
    let env = env();
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .rows
        .insert(2, Default::default());
    env.state()
        .borrow_mut()
        .game_rules
        .display
        .set_temporary_override(2);
    assert_eq!(
        env.eval::<Option<String>>("return C_GameRules.GetGameModeGlueScreenName()")
            .unwrap(),
        None
    );
    env.exec(r#"A_Admin.SetActiveGameMode(2, "plunderstorm")"#)
        .unwrap();
    assert_eq!(
        env.eval::<Option<String>>("return C_GameRules.GetGameModeGlueScreenName()")
            .unwrap(),
        Some("plunderstorm".into())
    );
    env.exec(r#"A_Admin.SetActiveGameMode(1, "")"#).unwrap();
    assert_eq!(
        env.eval::<Option<String>>("return C_GameRules.GetGameModeGlueScreenName()")
            .unwrap(),
        None
    );
}

#[test]
#[cfg(any(feature = "client-era", feature = "client-anniversary"))]
fn legacy_glue_screen_retains_state_and_ignores_r60_display_inputs() {
    use wow_ui_sim::c_api::game_mode_display::GameModeRow;
    use wow_ui_sim::screen::ScreenKind;

    let env = env();
    {
        let mut state = env.state().borrow_mut();
        assert_eq!(state.game_rules.glue_screen_name, "CharacterSelect");
        state.game_rules.glue_screen_name = "LegacyHostScreen".into();
        state.game_rules.display.rows.insert(
            7,
            GameModeRow {
                glue_screen_name: "R60HostScreen".into(),
                ..Default::default()
            },
        );
        state.game_rules.display.set_temporary_override(7);
        state.cvars.set_imported("currentGameMode", "7");
    }
    for screen in [ScreenKind::Login, ScreenKind::Game] {
        env.set_screen_mode(screen);
        assert_eq!(
            env.eval::<String>("return C_GameRules.GetGameModeGlueScreenName()")
                .unwrap(),
            "LegacyHostScreen"
        );
    }
    env.exec("A_Admin.SetActiveGameMode(2, 'LegacyAdminScreen')")
        .unwrap();
    assert_eq!(
        env.state().borrow().game_rules.glue_screen_name,
        "LegacyAdminScreen"
    );
    assert_eq!(
        env.state().borrow().game_rules.display.rows[&7].glue_screen_name,
        "R60HostScreen",
        "the legacy admin fixture must not rewrite R60 host rows"
    );
    for args in ["1", "1, nil", "1, ''"] {
        env.exec(&format!("A_Admin.SetActiveGameMode({args})"))
            .unwrap();
        assert_eq!(
            env.eval::<String>("return C_GameRules.GetGameModeGlueScreenName()")
                .unwrap(),
            "CharacterSelect"
        );
    }
}
