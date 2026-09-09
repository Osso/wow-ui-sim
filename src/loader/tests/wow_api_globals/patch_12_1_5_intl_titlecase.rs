use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_titlecase_maps_each_word_and_preserves_separators() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "ToTitle")) == "function", "ToTitle missing")
        local en = C_Intl.CreateLocaleContext("enUS")
        for _, case in ipairs({
            {"hELLO wORLD", "Hello World"},
            {"  «hELLO»,\twORLD!\nnext-word", "  «Hello»,\tWorld!\nNext-Word"},
            {"ßETA ﬃSH", "Sseta Ffish"},
            {"", ""},
            {"... \t!", "... \t!"},
        }) do
            assert(en:ToTitle(case[1]) == case[2], case[1])
            assert(select('#', en:ToTitle(case[1])) == 1)
        end
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        assert(C_Intl.ToTitle("hELLO wORLD") == current:ToTitle("hELLO wORLD"))
        local tr = C_Intl.CreateLocaleContext("trTR")
        assert(tr:ToTitle("istanbul izmir") == "İstanbul İzmir")
        local nl = C_Intl.CreateLocaleContext("nl-NL")
        assert(nl:ToTitle("ijssel IJSBERG") == "IJssel IJsberg")
        assert(tr:SetLocale("enUS"))
        assert(tr:ToTitle("istanbul izmir") == "Istanbul Izmir")
        assert(nl:GetLocale() == "nl-NL")
        GetLocale = function() return "trTR" end
        assert(C_Intl.ToTitle("istanbul izmir") == "İstanbul İzmir")
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_titlecase_rejects_invalid_inputs_without_mutating_context() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "ToTitle")) == "function", "ToTitle missing")
        local ctx = C_Intl.CreateLocaleContext("enUS")
        for _, text in ipairs({string.char(255), string.char(192, 175), 42, false}) do
            assert(not pcall(C_Intl.ToTitle, text))
            assert(not pcall(ctx.ToTitle, ctx, text))
        end
        assert(not pcall(C_Intl.ToTitle))
        assert(not pcall(ctx.ToTitle, {}, "hello"))
        assert(ctx:GetLocale() == "enUS")
        assert(ctx:SetLocale("not a locale"))
        assert(not pcall(ctx.ToTitle, ctx, "hello"))
        assert(ctx:GetLocale() == "not a locale")
        GetLocale = function() return "en--US" end
        assert(not pcall(C_Intl.ToTitle, "hello"))
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_titlecase_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
