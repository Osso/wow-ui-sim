//! Enum publication only; no weather-state or rendering semantics.

use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn assert_weather_type(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(type(Enum.WeatherType) == "table", "WeatherType missing")
        local expected = { Clear = 0, Rain = 1, Snow = 2, Sandstorm = 3, Miscellaneous = 4 }
        local count = 0
        for name, value in pairs(Enum.WeatherType) do
            assert(expected[name] == value, "unexpected WeatherType member: " .. name)
            count = count + 1
        end
        assert(count == 5)
        for name, value in pairs(expected) do
            assert(Enum.WeatherType[name] == value, name)
        end
        assert(Enum.WeatherTypeMeta.MinValue == 0)
        assert(Enum.WeatherTypeMeta.MaxValue == 4)
        assert(Enum.WeatherTypeMeta.NumValues == 5)
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_weather_type_enums_ptr() {
    let env = WowLuaEnv::new().unwrap();
    assert_weather_type(&env);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_weather_type(&env);
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_weather_type_enums_preserves_retail() {
    let env = WowLuaEnv::new().unwrap();
    let absence = "assert(Enum.WeatherType == nil); assert(Enum.WeatherTypeMeta == nil)";
    env.exec(absence).unwrap();
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    env.exec(absence).unwrap();
}
