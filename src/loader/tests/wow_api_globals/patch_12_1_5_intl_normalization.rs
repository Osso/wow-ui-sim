use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_normalization_forms() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, "Normalize")) == "function")
        assert(type(rawget(C_Intl, "IsNormalized")) == "function")
        local cases = {
            {"é", "é", 0}, {"é", "é", 1},
            {"ﬃ", "ffi", 2}, {"ﬃ", "ffi", 3},
            {"ạ́", "ạ́", 1},
            {"가", "가", 0}, {"가", "가", 1},
        }
        for _, case in ipairs(cases) do
            local input, expected, form = unpack(case)
            assert(C_Intl.IsNormalized(input, form) == false)
            assert(C_Intl.Normalize(input, form) == expected)
            assert(C_Intl.IsNormalized(expected, form) == true)
            assert(C_Intl.Normalize(expected, form) == expected)
            assert(select('#', C_Intl.Normalize(input, form)) == 1)
        end
        for form = 0, 3 do
            assert(C_Intl.Normalize("", form) == "")
            assert(C_Intl.IsNormalized("", form))
        end
        local context = C_Intl.CreateLocaleContext("fr-FR")
        assert(context:GetLocale() == "fr-FR")
        assert(C_Intl.GetCurrentLocale() == GetLocale())
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_normalization_rejects_invalid_input() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, name in ipairs({"Normalize", "IsNormalized"}) do
            local fn = rawget(C_Intl, name)
            assert(type(fn) == "function")
            for _, text in ipairs({string.char(255), string.char(192, 175), string.char(237, 160, 128)}) do
                local ok, message = pcall(fn, text, 0)
                assert(not ok and tostring(message):find("valid UTF%-8"))
            end
            for _, form in ipairs({-1, 4, 0.5, "NFC", false, {}}) do
                local ok, message = pcall(fn, "abc", form)
                assert(not ok and tostring(message):find("normalization form"))
            end
            assert(not pcall(fn, "abc"))
            assert(not pcall(fn, {}, 0))
            assert(not pcall(fn, nil, 0))
        end
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_normalization_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(GetLocale() == 'enUS')")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
