use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_transform_operations_preserve_context() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, 'TransformLocale')) == 'function')
        local function check(tag, operation, expected)
            local c = C_Intl.CreateLocaleContext(tag)
            assert(c:TransformLocale(operation) == expected, tag .. ':' .. operation)
            assert(c:GetLocale() == tag)
        end
        check('iw-IL', 0, 'he-IL')
        check('enUS', 0, 'en-US')
        check('zh-TW', 1, 'zh-Hant-TW')
        check('en-Latn-US', 2, 'en')
        check('zh-Hant-TW', 3, 'zh')
        check('zh-Hant-TW', 4, 'Hant')
        check('zh-Hant-TW', 5, 'TW')
        check('sl-rozaj', 6, 'rozaj')
        check('en', 4, '')
        check('en', 5, '')
        check('en', 6, '')
        check('zh-Hant-TW', 7, 'zh-Hant')
        check('zh-Hant', 7, 'zh')
        check('en', 7, 'und')
        check('und', 7, 'und')
        check('en-US-u-ca-gregory-x-private', 7, 'en-US-u-ca-gregory')
        check('en-US-u-ca-gregory', 7, 'en-US')
        check('sl-rozaj', 7, 'sl')
        check('sl-rozaj-biske', 6, 'biske-rozaj')
        check('sl-rozaj-biske', 7, 'sl-biske')
        check('en-US-u-ca-gregory-x-a-b', 7, 'en-US-u-ca-gregory')
        check('en-US-u-ca-gregory', 1, 'en-Latn-US-u-ca-gregory')
        local a = C_Intl.CreateLocaleContext('fr-FR')
        local b = C_Intl.CreateLocaleContext('enUS')
        assert(a:SetLocale('zh-Hant-TW'))
        assert(b:GetLocale() == 'enUS')
        assert(a:TransformLocale(5) == 'TW')
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        for i = 0, 7 do
            assert(C_Intl.TransformLocale(i) == current:TransformLocale(i))
        end
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_transform_rejects_invalid_input() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local c = C_Intl.CreateLocaleContext('enUS')
        assert(type(c.TransformLocale) == 'function')
        for _, op in ipairs({-1, 8, 1.5, '0', false}) do
            assert(not pcall(C_Intl.TransformLocale, op))
            assert(not pcall(c.TransformLocale, c, op))
        end
        assert(not pcall(C_Intl.TransformLocale))
        assert(not pcall(c.TransformLocale, {}, 0))
        local bad = C_Intl.CreateLocaleContext('not_a_locale!')
        assert(not pcall(bad.TransformLocale, bad, 0))
        assert(bad:GetLocale() == 'not_a_locale!')
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_transform_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil)").unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
