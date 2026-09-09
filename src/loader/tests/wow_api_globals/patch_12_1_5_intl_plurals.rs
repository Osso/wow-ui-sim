use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_plurals_use_cldr_cardinal_and_ordinal_rules() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "SelectPlural")) == "function")
        local en = C_Intl.CreateLocaleContext("enUS")
        local fr = C_Intl.CreateLocaleContext("fr-FR")
        local ru = C_Intl.CreateLocaleContext("ru-RU")
        local ar = C_Intl.CreateLocaleContext("ar")
        local cases = {
            {en, 0, "other"}, {en, 1, "one"}, {en, 2, "other"},
            {en, 1.5, "other"}, {en, -1, "one"}, {en, -0.0, "other"},
            {fr, 0, "one"}, {fr, 1.5, "one"}, {fr, 2, "other"},
            {ru, 1, "one"}, {ru, 2, "few"}, {ru, 5, "many"}, {ru, 1.5, "other"},
            {ar, 0, "zero"}, {ar, 1, "one"}, {ar, 2, "two"},
            {ar, 3, "few"}, {ar, 11, "many"}, {ar, 100, "other"}
        }
        for _, c in ipairs(cases) do assert(c[1]:SelectPlural(c[2], 0) == c[3]) end
        for _, c in ipairs({{1,"one"},{2,"two"},{3,"few"},{4,"other"},{11,"other"},{21,"one"}}) do
            assert(en:SelectPlural(c[1], 1) == c[2])
        end
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        assert(C_Intl.SelectPlural(1, 0) == current:SelectPlural(1, 0))
        assert(select('#', C_Intl.SelectPlural(1, 0)) == 1)
        assert(en:SetLocale("fr-FR"))
        assert(en:SelectPlural(0, 0) == "one")
        assert(ru:GetLocale() == "ru-RU")
        assert(ru:SelectPlural(2, 0) == "few")
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_plurals_reject_invalid_inputs_without_mutating_context() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "SelectPlural")) == "function")
        local c = C_Intl.CreateLocaleContext("enUS")
        for _, n in ipairs({0/0, 1/0, -1/0, "1", false, {}}) do
            assert(not pcall(C_Intl.SelectPlural, n, 0))
            assert(not pcall(c.SelectPlural, c, n, 0))
        end
        for _, t in ipairs({-1, 2, 0.5, "0", false}) do
            assert(not pcall(C_Intl.SelectPlural, 1, t))
            assert(not pcall(c.SelectPlural, c, 1, t))
        end
        assert(not pcall(C_Intl.SelectPlural, 1))
        assert(not pcall(c.SelectPlural, {}, 1, 0))
        assert(c:SetLocale("not_a_locale!"))
        assert(not pcall(c.SelectPlural, c, 1, 0))
        assert(c:GetLocale() == "not_a_locale!")
        assert(c:SetLocale("enUS"))
        assert(c:SelectPlural(1e100, 0) == "other")
        assert(c:SelectPlural(1e-100, 0) == "other")
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_plurals_preserve_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
