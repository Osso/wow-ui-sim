//! Numeric publication only, including cumulative 12.0.5 retail additions.

use crate::lua_api::WowLuaEnv;

fn assert_publication(env: &WowLuaEnv, ptr: bool) {
    env.exec(&format!(
        r#"
        local expected = {{ HideUntilCollected = 1, PlayerConditionGrantsOnLogin = 2,
            AllowedRangedShieldsHoldables = 4 }}
        if {ptr} then
            expected.HiddenIllusion = 8
        end
        local count = 0
        for name, value in pairs(expected) do
            assert(Enum.TransmogIllusionFlags[name] == value, name)
            count = count + 1
        end
        local actualCount = 0
        for name, value in pairs(Enum.TransmogIllusionFlags) do
            assert(expected[name] == value, name)
            actualCount = actualCount + 1
        end
        assert(actualCount == count)
        local meta = Enum.TransmogIllusionFlagsMeta
        assert(meta.MinValue == 1)
        assert(meta.MaxValue == ({ptr} and 8 or 4))
        assert(meta.NumValues == count)
        "#
    ))
    .expect("exact transmog illusion flags publication");
}

#[test]
#[cfg(any(feature = "client-ptr", feature = "client-retail"))]
fn patch_12_1_5_transmog_illusion_flags_publication() {
    let env = WowLuaEnv::new().unwrap();
    let ptr = cfg!(feature = "client-ptr");
    // Retail publishes 12.0.5 values; PTR retains its additional HiddenIllusion bit.
    assert_publication(&env, ptr);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_publication(&env, ptr);
}
