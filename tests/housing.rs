//! `C_Housing.IsHousingServiceEnabled` — SimState-backed round-trip.

use wow_ui_sim::lua_api::WowLuaEnv;

fn probe(env: &WowLuaEnv) -> bool {
    env.eval(r#"return C_Housing.IsHousingServiceEnabled()"#)
        .unwrap()
}

#[cfg(feature = "retail-12-0-0")]
mod freeplace_tests {
    use super::WowLuaEnv;

    #[test]
    fn housing_freeplace_explicit_and_repeated_toggles() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            for index, enabled in ipairs({true, false, false, true, true, false}) do
                C_HousingBasicMode.SetFreePlaceEnabled(enabled)
                assert(C_HousingBasicMode.IsFreePlaceEnabled() == enabled,
                    "free-place state differs after explicit write " .. index)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn housing_freeplace_getter_and_setter_return_arity() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            for _, enabled in ipairs({true, false}) do
                assert(select("#", C_HousingBasicMode.SetFreePlaceEnabled(enabled)) == 0,
                    "free-place setter must return zero values")
                assert(select("#", C_HousingBasicMode.IsFreePlaceEnabled()) == 1,
                    "free-place getter must return one value")
                assert(type(C_HousingBasicMode.IsFreePlaceEnabled()) == "boolean",
                    "free-place getter must return a boolean")
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn housing_freeplace_isolates_lua_environments() {
        let first = WowLuaEnv::new().unwrap();
        first
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(false)")
            .unwrap();
        let second = WowLuaEnv::new().unwrap();
        second
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(true)")
            .unwrap();

        assert!(!first
            .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
            .unwrap());
        assert!(second
            .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
            .unwrap());

        first
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(true)")
            .unwrap();
        second
            .exec("C_HousingBasicMode.SetFreePlaceEnabled(false)")
            .unwrap();
        assert!(first
            .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
            .unwrap());
        assert!(!second
            .eval::<bool>("return C_HousingBasicMode.IsFreePlaceEnabled()")
            .unwrap());
    }

    #[test]
    fn housing_freeplace_preserves_housing_service_state() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            for _, serviceEnabled in ipairs({false, true}) do
                A_Admin.SetHousingServiceEnabled(serviceEnabled)
                for _, freePlaceEnabled in ipairs({true, false}) do
                    C_HousingBasicMode.SetFreePlaceEnabled(freePlaceEnabled)
                    assert(C_Housing.IsHousingServiceEnabled() == serviceEnabled,
                        "free-place write changed housing service availability")
                end
            end
            "#,
        )
        .unwrap();
    }
}

#[test]
fn defaults_to_true() {
    let env = WowLuaEnv::new().unwrap();
    assert!(probe(&env));
}

#[test]
fn admin_set_enables_and_disables() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetHousingServiceEnabled(false)").unwrap();
    assert!(!probe(&env));
    env.exec("A_Admin.SetHousingServiceEnabled(true)").unwrap();
    assert!(probe(&env));
}

#[test]
fn admin_no_arg_defaults_to_true() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetHousingServiceEnabled()").unwrap();
    assert!(probe(&env));
}

#[test]
fn other_c_housing_members_still_resolve_via_metamethod_fallback() {
    // Unimplemented C_Housing.* should return the stub-namespace no-op
    // function (which returns nil), not crash with "attempt to call a nil
    // value".
    let env = WowLuaEnv::new().unwrap();
    let result: String = env
        .eval(
            r#"
            local fn = C_Housing.SomeUnimplementedMember
            if type(fn) ~= "function" then return "missing_function" end
            if fn() ~= nil then return "non_nil_return" end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn dashboard_bootstrap_members_have_safe_defaults() {
    let env = WowLuaEnv::new().unwrap();
    let (max_level, cooldown_is_nil): (i32, bool) = env
        .eval(
            r#"
            return C_Housing.GetMaxHouseLevel(), C_Housing.GetVisitCooldownInfo() == nil
            "#,
        )
        .unwrap();
    assert_eq!(max_level, 0);
    assert!(
        cooldown_is_nil,
        "GetVisitCooldownInfo should default to nil when no cooldown is active"
    );
}
