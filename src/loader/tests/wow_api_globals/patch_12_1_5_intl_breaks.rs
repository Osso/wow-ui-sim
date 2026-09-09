use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_breaks_returns_utf8_boundaries_for_all_modes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "FindBreaks")) == "function", "FindBreaks missing")
        local ctx = C_Intl.CreateLocaleContext("not a locale")
        local function check(text, mode, expected)
            local a = C_Intl.FindBreaks(text, mode)
            local b = ctx:FindBreaks(text, mode)
            assert(#a == #expected and #b == #expected, text)
            for i, value in ipairs(expected) do
                assert(a[i] == value and b[i] == value, text .. ": " .. i)
            end
            assert(select('#', C_Intl.FindBreaks(text, mode)) == 1)
            assert(a ~= b)
        end
        check("abc", 0, {0, 1, 2, 3})
        check("é😀", 0, {0, 3, 7})
        check("👩‍💻", 0, {0, 11})
        check("é hi", 1, {0, 2, 3, 5})
        check("Hi. Bye!", 2, {0, 4, 8})
        check("é\nx", 3, {0, 3, 4})
        for mode = 0, 3 do check("", mode, {0}) end
        assert(ctx:GetLocale() == "not a locale")
        assert(ctx:SetLocale("trTR"))
        check("é hi", 1, {0, 2, 3, 5})
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_breaks_rejects_invalid_inputs_and_receivers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "FindBreaks")) == "function", "FindBreaks missing")
        local ctx = C_Intl.CreateLocaleContext("enUS")
        for _, mode in ipairs({-1, 4, 0.5, false, "0"}) do
            assert(not pcall(C_Intl.FindBreaks, "abc", mode))
            assert(not pcall(ctx.FindBreaks, ctx, "abc", mode))
        end
        for _, text in ipairs({string.char(255), string.char(192,175), false, 42}) do
            assert(not pcall(C_Intl.FindBreaks, text, 0))
            assert(not pcall(ctx.FindBreaks, ctx, text, 0))
        end
        assert(not pcall(C_Intl.FindBreaks, "abc"))
        assert(not pcall(ctx.FindBreaks, {}, "abc", 0))
        assert(ctx:GetLocale() == "enUS")
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_breaks_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
