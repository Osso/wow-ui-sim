use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn locale_context_storage_ptr() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "CreateLocaleContext")) == "function")
        local a = C_Intl.CreateLocaleContext("enUS")
        local b = C_Intl.CreateLocaleContext("fr-FR")
        assert(type(a) == "userdata" and a ~= b)
        assert(a:GetLocale() == "enUS" and b:GetLocale() == "fr-FR")
        assert(a:SetLocale("zh-Hant-TW") == true)
        assert(a:GetLocale() == "zh-Hant-TW" and b:GetLocale() == "fr-FR")
        for _, invalid in ipairs({"", "en\000US"}) do
            assert(a:SetLocale(invalid) == false)
            assert(a:GetLocale() == "zh-Hant-TW")
            assert(not pcall(C_Intl.CreateLocaleContext, invalid))
        end
        assert(not pcall(C_Intl.CreateLocaleContext))
        assert(not pcall(C_Intl.CreateLocaleContext, {}))
        assert(not pcall(a.SetLocale, a, {}))
        assert(a:GetLocale() == "zh-Hant-TW")
        assert(C_Intl.GetCurrentLocale() == GetLocale())
        assert(rawget(_G, "LuaLocaleContext") == nil)
        collectgarbage("collect")
        assert(a:GetLocale() == "zh-Hant-TW")
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn locale_context_storage_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec(
            r#"
            assert(C_Intl == nil)
            assert(rawget(_G, "LuaLocaleContext") == nil)
            assert(GetLocale() == "enUS")
        "#,
        )
        .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
