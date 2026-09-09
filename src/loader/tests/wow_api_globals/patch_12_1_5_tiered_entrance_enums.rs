//! Exact numeric publication only; no entrance gameplay semantics.

use crate::lua_api::WowLuaEnv;

fn assert_publication(env: &WowLuaEnv, ptr: bool) {
    env.exec(&format!(
        r#"
        local expected = {{ Invalid=0, Delve=1, Sites=2, WorldTier=3, Lairs=4 }}
        if {ptr} then expected.Placeholder_5=5; expected.Placeholder_6=6 end
        local actual = Enum.TieredEntranceType
        for name, value in pairs(expected) do
            assert(actual[name] == value, name .. ": expected " .. value .. ", got " .. tostring(actual[name]))
        end
        local count = 0
        for name, value in pairs(actual) do
            assert(expected[name] == value, "unexpected member " .. name)
            count = count + 1
        end
        local total = {ptr} and 7 or 5
        assert(count == total)
        local meta = Enum.TieredEntranceTypeMeta
        assert(meta.MinValue == 0 and meta.MaxValue == total - 1 and meta.NumValues == total)
        if not {ptr} then
            assert(actual.Placeholder_5 == nil and actual.Placeholder_6 == nil)
        end
        "#
    ))
    .expect("exact entrance enum publication");
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_tiered_entrance_enums_ptr() {
    let env = WowLuaEnv::new().unwrap();
    assert_publication(&env, true);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_publication(&env, true);
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_tiered_entrance_enums_retail() {
    let env = WowLuaEnv::new().unwrap();
    assert_publication(&env, false);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_publication(&env, false);
}
