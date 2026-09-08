//! Enum publication only; no bag behavior or forbidden-aspect enforcement.

use crate::lua_api::WowLuaEnv;

#[test]
fn patch_12_1_5_bag_flag_publication() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let actual: (Option<f64>, f64, f64, f64) = env
        .eval("return Enum.BagFlag.IgnoreSoulbound, Enum.BagFlagMeta.MinValue, Enum.BagFlagMeta.MaxValue, Enum.BagFlagMeta.NumValues")
        .expect("BagFlag query should evaluate");
    #[cfg(feature = "retail-12-1-5")]
    assert_eq!(actual, (None, 1.0, 134217728.0, 27.0));
    #[cfg(not(feature = "retail-12-1-5"))]
    assert_eq!(actual, (Some(4194304.0), 1.0, 134217728.0, 28.0));
}

#[test]
fn patch_12_1_5_forbidden_aspect_publication() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    let actual: (Option<f64>, Option<f64>, f64, f64, f64) = env
        .eval("return Enum.ForbiddenAspect.QueryAnimationProgress, Enum.ForbiddenAspect.AddAnimations, Enum.ForbiddenAspectMeta.MinValue, Enum.ForbiddenAspectMeta.MaxValue, Enum.ForbiddenAspectMeta.NumValues")
        .expect("ForbiddenAspect query should evaluate");
    #[cfg(feature = "retail-12-1-5")]
    assert_eq!(actual, (Some(2048.0), Some(4096.0), 1.0, 4096.0, 13.0));
    #[cfg(not(feature = "retail-12-1-5"))]
    assert_eq!(actual, (None, None, 1.0, 1024.0, 11.0));
}
