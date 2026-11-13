#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn neighborhood_initiative_available_house_xp_defaults_to_zero() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            if type(C_NeighborhoodInitiative.GetAvailableHouseXP) ~= "function" then
                return "missing_get_available_house_xp"
            end

            if C_NeighborhoodInitiative.IsInitiativeEnabled() then
                return "initiative_should_default_disabled"
            end

            if C_NeighborhoodInitiative.GetAvailableHouseXP() ~= 0 then
                return "available_house_xp_should_default_zero"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "C_NeighborhoodInitiative.GetAvailableHouseXP should exist and default to zero"
    );
}
