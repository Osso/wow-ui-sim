//! R75: PTR rewards override and catalog session validation.

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn override_queries_are_registered_in_game_and_glue_with_neutral_defaults() {
    use wow_ui_sim::screen::ScreenKind;

    for screen in [ScreenKind::Game, ScreenKind::Login] {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(screen);
        let result: (bool, i64, bool, i64, bool, bool) = env
            .eval(
                r#"
            return C_StorePublic.IsRewardsShopOverrideEnabled(),
                select('#', C_StorePublic.IsRewardsShopOverrideEnabled()),
                C_CatalogShop.ValidateOverrideShopInteraction(),
                select('#', C_CatalogShop.ValidateOverrideShopInteraction()),
                C_CatalogShop.ValidateOverrideShopInteraction(nil),
                C_CatalogShop.ValidateOverrideShopInteraction('')
        "#,
            )
            .unwrap();
        assert_eq!(result, (false, 1, false, 1, false, false));
    }
}

#[cfg(not(feature = "client-ptr"))]
#[test]
fn ptr_override_queries_are_not_registered_on_other_profiles() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local store = rawget(_G, 'C_StorePublic')
        local catalog = rawget(_G, 'C_CatalogShop')
        assert(store == nil or rawget(store, 'IsRewardsShopOverrideEnabled') == nil)
        assert(catalog == nil or rawget(catalog, 'ValidateOverrideShopInteraction') == nil)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn override_predicate_obeys_both_mirrors_in_game_and_glue() {
    use wow_ui_sim::screen::ScreenKind;

    for screen in [ScreenKind::Game, ScreenKind::Login] {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(screen);
        for (store, override_flag, expected) in [
            (false, false, false),
            (false, true, true),
            (true, true, false),
            (true, false, false),
        ] {
            env.set_server_mirrored_cvar("bpayStoreEnable", if store { "1" } else { "0" })
                .unwrap();
            env.set_server_mirrored_cvar(
                "rewardsShopOverrideEnabled",
                if override_flag { "1" } else { "0" },
            )
            .unwrap();
            let result: (bool, i64) = env
                .eval("return C_StorePublic.IsRewardsShopOverrideEnabled(), select('#', C_StorePublic.IsRewardsShopOverrideEnabled())")
                .unwrap();
            assert_eq!(result, (expected, 1));
        }
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn override_mirror_changes_notify_without_payload_and_notification_does_not_write_value() {
    use wow_ui_sim::screen::ScreenKind;

    for screen in [ScreenKind::Game, ScreenKind::Login] {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(screen);
        env.set_server_mirrored_cvar("bpayStoreEnable", "0")
            .unwrap();
        env.exec(
            r#"
            overrideEvents = {}
            local listener = CreateFrame('Frame')
            listener:RegisterEvent('STORE_STATUS_CHANGED')
            listener:SetScript('OnEvent', function(_, _, ...)
                overrideEvents[#overrideEvents + 1] = {
                    payload = select('#', ...),
                    enabled = C_StorePublic.IsRewardsShopOverrideEnabled(),
                }
            end)
        "#,
        )
        .unwrap();
        env.set_server_mirrored_cvar("rewardsShopOverrideEnabled", "1")
            .unwrap();
        env.exec(
            r#"
            assert(#overrideEvents == 1)
            assert(overrideEvents[1].payload == 0)
            assert(overrideEvents[1].enabled == true)
        "#,
        )
        .unwrap();
        // The native callback reports the update; it does not supply a value.
        env.fire_event("STORE_STATUS_CHANGED").unwrap();
        env.exec(
            r#"
            assert(#overrideEvents == 2)
            assert(overrideEvents[2].payload == 0)
            assert(overrideEvents[2].enabled == true)
            assert(C_StorePublic.IsRewardsShopOverrideEnabled() == true)
        "#,
        )
        .unwrap();
        env.set_server_mirrored_cvar("rewardsShopOverrideEnabled", "0")
            .unwrap();
        env.exec(
            r#"
            assert(#overrideEvents == 3)
            assert(overrideEvents[3].payload == 0)
            assert(overrideEvents[3].enabled == false)
        "#,
        )
        .unwrap();
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn session_validation_requires_mode_three_and_exact_bytes_in_game_and_glue() {
    use wow_ui_sim::screen::ScreenKind;

    for screen in [ScreenKind::Game, ScreenKind::Login] {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(screen);
        env.state().borrow_mut().catalog_shop_interaction.session = b"session-A".to_vec();
        for (mode, expected) in [(0, false), (2, false), (3, true), (4, false)] {
            env.state().borrow_mut().catalog_shop_interaction.mode = mode;
            let result: (bool, bool, bool, bool) = env
                .eval(
                    r#"
                return C_CatalogShop.ValidateOverrideShopInteraction('session-A'),
                    C_CatalogShop.ValidateOverrideShopInteraction('session-a'),
                    C_CatalogShop.ValidateOverrideShopInteraction('session-A '),
                    C_CatalogShop.ValidateOverrideShopInteraction('session-A\0')
            "#,
                )
                .unwrap();
            assert_eq!(result, (expected, false, false, false));
        }
        env.state().borrow_mut().catalog_shop_interaction.mode = 3;
        env.state().borrow_mut().catalog_shop_interaction.session = vec![255, 0, 128];
        let binary: (bool, bool, bool) = env
            .eval(
                r#"
            return C_CatalogShop.ValidateOverrideShopInteraction(string.char(255, 0, 128)),
                C_CatalogShop.ValidateOverrideShopInteraction(string.char(254, 0, 128)),
                C_CatalogShop.ValidateOverrideShopInteraction(string.char(255))
        "#,
            )
            .unwrap();
        assert_eq!(binary, (true, false, false));
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn session_validation_handles_nil_empty_numbers_and_ignores_override_flag() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().catalog_shop_interaction.mode = 3;
    let empty: (bool, bool, bool, i64) = env
        .eval(
            r#"
        return C_CatalogShop.ValidateOverrideShopInteraction(),
            C_CatalogShop.ValidateOverrideShopInteraction(nil),
            C_CatalogShop.ValidateOverrideShopInteraction(''),
            select('#', C_CatalogShop.ValidateOverrideShopInteraction(''))
    "#,
        )
        .unwrap();
    assert_eq!(empty, (false, false, true, 1));

    env.state().borrow_mut().catalog_shop_interaction.session = b"42".to_vec();
    for flag in ["0", "1"] {
        env.set_server_mirrored_cvar("rewardsShopOverrideEnabled", flag)
            .unwrap();
        let result: (bool, bool, bool, bool) = env
            .eval(
                r#"
            return C_CatalogShop.ValidateOverrideShopInteraction(42),
                C_CatalogShop.ValidateOverrideShopInteraction('42'),
                C_CatalogShop.ValidateOverrideShopInteraction('042'),
                C_CatalogShop.ValidateOverrideShopInteraction('')
        "#,
            )
            .unwrap();
        assert_eq!(result, (true, true, false, false));
    }
}

#[cfg(feature = "client-ptr")]
const VALIDATION_USAGE: &str = "Usage: local isValid = C_CatalogShop.ValidateOverrideShopInteraction([shoppingSessionUUIDStr])";

#[cfg(feature = "client-ptr")]
fn has_exact_validation_usage(error: &str) -> bool {
    let Some(first_line) = error.lines().next() else {
        return false;
    };
    let Some(start) = first_line.find("Usage: ") else {
        return false;
    };
    let reason = &first_line[start..];
    // R75 verifies the Usage reason but explicitly leaves the outer
    // bad-argument envelope unknown. Accept only that exact reason, with
    // an optional single closing parenthesis belonging to the envelope.
    reason == VALIDATION_USAGE || reason.strip_suffix(')') == Some(VALIDATION_USAGE)
}

#[cfg(feature = "client-ptr")]
#[test]
fn validation_usage_matcher_rejects_other_errors_and_suffixes() {
    assert!(has_exact_validation_usage(VALIDATION_USAGE));
    assert!(has_exact_validation_usage(&format!(
        "unknown outer envelope ({VALIDATION_USAGE})"
    )));
    assert!(!has_exact_validation_usage("attempt to call a nil value"));
    assert!(!has_exact_validation_usage(&format!(
        "{VALIDATION_USAGE} unexpected detail"
    )));
    assert!(!has_exact_validation_usage(&format!(
        "{VALIDATION_USAGE}))"
    )));
    assert!(!has_exact_validation_usage("Usage: a different API"));
}

#[cfg(feature = "client-ptr")]
#[test]
fn session_validation_rejects_boolean_table_and_function_arguments_with_usage_reason() {
    let env = WowLuaEnv::new().unwrap();
    for argument in ["true", "false", "{}", "function() end"] {
        let (ok, error): (bool, String) = env
            .eval(&format!(
                r#"
            local ok, err = pcall(function()
                return C_CatalogShop.ValidateOverrideShopInteraction({argument})
            end)
            return ok, tostring(err)
        "#
            ))
            .unwrap();
        assert!(!ok, "invalid argument was accepted: {argument}");
        assert!(
            has_exact_validation_usage(&error),
            "unexpected error for {argument}: {error}"
        );
    }
}
