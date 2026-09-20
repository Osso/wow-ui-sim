//! Forever's native math surface and its real Blizzard Lua consumers.

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_math_util_uses_native_extensions() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let source = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_SharedXMLBase/MathUtil.lua");
    let code = std::fs::read_to_string(&source).expect("synced Forever MathUtil.lua");
    env.exec(&code).unwrap();
    env.exec(
        r#"
        assert(Round(2.5) == 3)
        assert(Round(-2.5) == -3)
        assert(RoundToSignificantDigits(124, -1) == 120)
        assert(Clamp(15, 0, 10) == 10)
        assert(Lerp(10, 20, 0.25) == 12.5)
        assert(Saturate(-2) == 0)
        assert(Sign(-4) == -1)
        assert(Wrap(0, 5) == 5)
        assert(ClampDegrees(-90) == 270)
        assert(ClampMod(13, 5) == 3)
        assert(ClampedPercentageBetween(15, 0, 10) == 1)
        assert(RoundToNearestMultiple(85, 50) == 100)
        assert(math.normalize(15, 10, 20) == 0.5)
        assert(math.remap(5, 0, 10, 20, 40) == 30)
        assert(math.isfinite(7) and not math.isfinite(math.huge))
        assert(math.isinf(-math.huge) and not math.isinf(7))
        assert(math.isnan(0 / 0) and not math.isnan(7))
        assert(math.wrap(9, 3, 3) == 3)
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(not(any(feature = "retail-12-1-5", feature = "client-wowforever")))]
fn wowforever_math_does_not_leak_into_earlier_profiles() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({"clamp", "isfinite", "isinf", "isnan", "lerp",
            "normalize", "remap", "round", "saturate", "sign", "wrap"}) do
            assert(math[name] == nil, name)
        end
        "#,
    )
    .unwrap();
}
