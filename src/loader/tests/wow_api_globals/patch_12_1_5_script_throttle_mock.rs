//! Explicit zero placeholders only; no native values or throttling semantics.
use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn script_throttle_mock_returns_exact_fresh_zero_tables() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            assert(type(rawget(_G, "GetScriptBucketThrottleLimits")) == "function")
            assert(select('#', GetScriptBucketThrottleLimits()) == 1)
            local first = GetScriptBucketThrottleLimits()
            local second = GetScriptBucketThrottleLimits()
            assert(type(first) == "table" and type(second) == "table")
            assert(not rawequal(first, second))
            local expected = {
                luaScriptBucketThrottleMaxMsPerSecondNormal = true,
                luaScriptBucketThrottleMaxMsPerSecondRestricted = true,
                luaScriptBucketThrottleMaxMsBurstNormal = true,
                luaScriptBucketThrottleMaxMsBurstRestricted = true,
            }
            local count = 0
            for key, value in pairs(first) do
                assert(expected[key], "unexpected key: " .. tostring(key))
                assert(type(value) == "number" and value == 0)
                count = count + 1
            end
            assert(count == 4)
            for key in pairs(expected) do
                first[key] = 123
                assert(type(second[key]) == "number" and second[key] == 0)
                assert(GetScriptBucketThrottleLimits()[key] == 0)
            end
            first.extra = true
            assert(second.extra == nil and GetScriptBucketThrottleLimits().extra == nil)
            "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn script_throttle_mock_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            assert(rawget(_G, "GetScriptBucketThrottleLimits") == nil)
            assert(GetScriptBucketThrottleLimits == nil)
            assert(rawget(_G, "GetScriptBucketThrottleLimits") == nil)
            "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
