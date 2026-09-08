//! Application-bar option processing only, not aura rendering or security.

use crate::lua_api::WowLuaEnv;

#[test]
fn patch_12_1_5_aura_application_options_default() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let actual: (Option<f64>, f64, f64) = env
        .eval(
            r#"
            local result = C_AuraContainerUtil.ProcessCustomAuraButtonApplicationBarOptions({
                maxApplications = 5, interpolation = 1,
            })
            return result.minApplications, result.maxApplications, result.interpolation
        "#,
        )
        .expect("application-bar options should process");
    #[cfg(feature = "retail-12-1-5")]
    assert_eq!(actual, (Some(0.0), 5.0, 1.0));
    #[cfg(not(feature = "retail-12-1-5"))]
    assert_eq!(actual, (None, 5.0, 1.0));
}

#[test]
fn patch_12_1_5_aura_application_options_explicit() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let actual: (Option<f64>, f64, f64) = env
        .eval(
            r#"
            local result = C_AuraContainerUtil.ProcessCustomAuraButtonApplicationBarOptions({
                minApplications = 2, maxApplications = 7, interpolation = 0,
            })
            return result.minApplications, result.maxApplications, result.interpolation
        "#,
        )
        .expect("application-bar options should process");
    #[cfg(feature = "retail-12-1-5")]
    assert_eq!(actual, (Some(2.0), 7.0, 0.0));
    #[cfg(not(feature = "retail-12-1-5"))]
    assert_eq!(actual, (None, 7.0, 0.0));
}
