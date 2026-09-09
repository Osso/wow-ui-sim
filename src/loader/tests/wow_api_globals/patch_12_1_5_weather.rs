//! Simulator weather assumptions, not native weather timing or intensity semantics.
use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_weather_state_and_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(_G, "C_Weather")) == "table", "weather namespace missing")
        local initial = C_Weather.GetCurrentWeather()
        assert(initial.type == Enum.WeatherType.Clear and initial.intensity == 0)
        assert(rawget(_G, "WeatherInfo") == nil)
        A_Admin.SetWeather(Enum.WeatherType.Rain, 0.5)
        local current = C_Weather.GetCurrentWeather()
        assert(current.type == Enum.WeatherType.Rain and current.intensity == 0.5)
        assert(initial.type == Enum.WeatherType.Clear and initial.intensity == 0)
        current.type = 99
        current.intensity = 99
        local fresh = C_Weather.GetCurrentWeather()
        assert(fresh ~= current and fresh.type == Enum.WeatherType.Rain and fresh.intensity == 0.5)
        A_Admin.SetWeather(Enum.WeatherType.Clear, 0)
        assert(C_Weather.GetCurrentWeather().intensity == 0)
    "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    assert_eq!(
        other
            .eval::<f64>("return C_Weather.GetCurrentWeather().intensity")
            .unwrap(),
        0.0
    );
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_weather_explicit_event_observes_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local calls = 0
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("WEATHER_CHANGED")
        frame:SetScript("OnEvent", function(_, event, ...)
            assert(event == "WEATHER_CHANGED" and select('#', ...) == 0)
            local weather = C_Weather.GetCurrentWeather()
            assert(weather.type == Enum.WeatherType.Rain and weather.intensity == 0.5)
            calls = calls + 1
        end)
        A_Admin.SetWeather(Enum.WeatherType.Rain, 0.5)
        assert(calls == 0, "state mutation must not automatically dispatch")
        A_Admin.FireEvent("WEATHER_CHANGED")
        assert(calls == 1)
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_weather_admin_validation_is_atomic() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        A_Admin.SetWeather(Enum.WeatherType.Rain, 0.5)
        for _, args in ipairs({{99, 1}, {1.5, 1}, {"1", 1}, {1, "0.5"}, {1, math.huge}, {1, 0/0}}) do
            assert(not pcall(A_Admin.SetWeather, unpack(args)))
            local weather = C_Weather.GetCurrentWeather()
            assert(weather.type == 1 and weather.intensity == 0.5)
        end
        assert(not pcall(A_Admin.SetWeather, 1))
        -- Finite values are accepted without inventing a native intensity range.
        A_Admin.SetWeather(1, -0.5)
        assert(C_Weather.GetCurrentWeather().intensity == -0.5)
        A_Admin.SetWeather(1, 2)
        assert(C_Weather.GetCurrentWeather().intensity == 2)
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_weather_namespace_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            assert(rawget(_G, "C_Weather") == nil)
            assert(C_Weather == nil, "global namespace fallback must preserve absence")
            assert(rawget(_G, "C_Weather") == nil)
            assert(rawget(A_Admin, "SetWeather") == nil)
            assert(rawget(_G, "WeatherInfo") == nil)
        "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
