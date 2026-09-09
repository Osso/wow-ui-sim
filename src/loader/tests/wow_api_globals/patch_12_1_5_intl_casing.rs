use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_casing_uses_current_and_context_locales() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({"ToLower", "ToUpper", "FoldCase"}) do
            assert(type(rawget(C_Intl, name)) == "function", name .. " missing")
        end
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        assert(C_Intl.ToLower("Iİ ΟΣ") == current:ToLower("Iİ ΟΣ"))
        assert(C_Intl.ToUpper("iı ß") == current:ToUpper("iı ß"))
        local tr = C_Intl.CreateLocaleContext("trTR")
        local en = C_Intl.CreateLocaleContext("en-US")
        assert(tr:ToLower("Iİ") == "ıi")
        assert(tr:ToUpper("iı") == "İI")
        assert(en:ToLower("Iİ") == "ii̇")
        assert(en:ToLower("ΟΣ ΟΣΑ") == "ος οσα")
        assert(en:ToUpper("straße") == "STRASSE")
        assert(tr:GetLocale() == "trTR")
        assert(tr:SetLocale("enUS"))
        assert(tr:ToLower("I") == "i")
        assert(en:SetLocale("tr-TR"))
        assert(en:ToLower("I") == "ı")
        assert(tr:ToLower("I") == "i")
        local script = C_Intl.CreateLocaleContext("zh-Hant-TW")
        assert(script:ToLower("ABC") == "abc")
        assert(script:GetLocale() == "zh-Hant-TW")
        GetLocale = function() return "trTR" end
        assert(C_Intl.ToLower("Iİ") == "ıi")
        assert(C_Intl.ToUpper("iı") == "İI")
        GetLocale = function() return "enUS" end
        assert(C_Intl.ToLower("I") == "i")
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_casing_full_fold_is_locale_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local tr = C_Intl.CreateLocaleContext("tr-TR")
        local en = C_Intl.CreateLocaleContext("enUS")
        for _, case in ipairs({{"Straße ẞ", "strasse ss"}, {"Σσς", "σσσ"},
                               {"Iİı", "ii̇ı"}, {"ﬃ", "ffi"}, {"", ""}}) do
            assert(C_Intl.FoldCase(case[1]) == case[2])
            assert(tr:FoldCase(case[1]) == case[2])
            assert(en:FoldCase(case[1]) == case[2])
            assert(select('#', tr:FoldCase(case[1])) == 1)
        end
        assert(tr:SetLocale("not a locale"))
        assert(tr:FoldCase("Iß") == "iss")
        GetLocale = function() error("fold must not read locale") end
        assert(C_Intl.FoldCase("Iß") == "iss")
        for _, name in ipairs({"ToLower", "ToUpper", "FoldCase"}) do
            assert(en[name](en, "") == "")
        end
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_casing_rejects_invalid_text_locales_and_receivers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local ctx = C_Intl.CreateLocaleContext("enUS")
        for _, name in ipairs({"ToLower", "ToUpper", "FoldCase"}) do
            assert(type(rawget(C_Intl, name)) == "function", name .. " missing")
            for _, text in ipairs({string.char(255), string.char(192, 175), 42, false}) do
                assert(not pcall(C_Intl[name], text))
                assert(not pcall(ctx[name], ctx, text))
            end
            assert(not pcall(C_Intl[name]))
            assert(not pcall(ctx[name], {}, "text"))
        end
        for _, tag in ipairs({"not a locale", "en--US", string.char(255)}) do
            assert(ctx:SetLocale(tag))
            for _, name in ipairs({"ToLower", "ToUpper"}) do
                local ok, err = pcall(ctx[name], ctx, "ABC")
                assert(not ok and tostring(err):find("locale"))
                GetLocale = function() return tag end
                assert(not pcall(C_Intl[name], "ABC"))
            end
            assert(ctx:GetLocale() == tag)
        end
        assert(ctx:SetLocale("trTR"))
        assert(ctx:ToLower("I") == "ı")
        "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_casing_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
