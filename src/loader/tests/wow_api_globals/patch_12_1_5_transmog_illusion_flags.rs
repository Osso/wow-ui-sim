//! Publication only. Retail intentionally retains its two-member simulator drift.

use crate::lua_api::WowLuaEnv;

fn assert_publication(env: &WowLuaEnv, ptr: bool) {
    env.exec(&format!(
        r#"
        local expected = {{ HideUntilCollected = 1, PlayerConditionGrantsOnLogin = 2 }}
        if {ptr} then
            expected.AllowedRangedShieldsHoldables = 4
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
        assert(meta.MaxValue == ({ptr} and 8 or 2))
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
    // Pinned base has value 4 and metadata 1/4/3; preserve actual retail 1/2/2.
    assert_publication(&env, ptr);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_publication(&env, ptr);
}
