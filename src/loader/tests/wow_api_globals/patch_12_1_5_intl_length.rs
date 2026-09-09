use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_length_counts_scalars_independent_of_locale() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "Length")) == "function")
        local a = C_Intl.CreateLocaleContext("enUS")
        local b = C_Intl.CreateLocaleContext("ja-JP")
        local cases = {{"", 0}, {"Hello", 5}, {"é", 1}, {"é", 2},
                       {"😀", 1}, {"👩‍💻", 3}, {"👨‍👩‍👧‍👦", 7}}
        for _, case in ipairs(cases) do
            local text, expected = unpack(case)
            assert(C_Intl.Length(text) == expected)
            assert(a:Length(text) == expected and b:Length(text) == expected)
            assert(select('#', C_Intl.Length(text)) == 1)
            assert(select('#', a:Length(text)) == 1)
        end
        assert(a:SetLocale("fr-FR"))
        assert(b:GetLocale() == "ja-JP")
        assert(a:Length("é") == 2 and b:Length("é") == 2)
        assert(C_Intl.Normalize("é", 0) == "é")
        assert(C_Intl.IsNormalized("é", 0))
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_length_rejects_invalid_text_and_receivers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local context = C_Intl.CreateLocaleContext("enUS")
        assert(type(rawget(C_Intl, "Length")) == "function")
        assert(type(context.Length) == "function")
        for _, text in ipairs({string.char(255), string.char(192, 175),
                               string.char(237, 160, 128), string.char(226, 130)}) do
            local ok, err = pcall(C_Intl.Length, text)
            assert(not ok and tostring(err):find("valid UTF%-8"))
            ok, err = pcall(context.Length, context, text)
            assert(not ok and tostring(err):find("valid UTF%-8"))
        end
        for _, text in ipairs({false, 42, {}}) do
            assert(not pcall(C_Intl.Length, text))
            assert(not pcall(context.Length, context, text))
        end
        assert(not pcall(C_Intl.Length))
        assert(not pcall(context.Length, context))
        assert(not pcall(context.Length, {}, "abc"))
        assert(context:GetLocale() == "enUS")
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_length_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
