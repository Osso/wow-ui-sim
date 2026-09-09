use crate::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_collation_strengths_and_binary_keys() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(rawget(C_Intl, 'CompareStrings')) == 'function')
        assert(type(rawget(C_Intl, 'GetSortKey')) == 'function')
        local c = C_Intl.CreateLocaleContext('enUS')
        assert(c:CompareStrings('e', 'é', 0) == 0)
        assert(c:CompareStrings('e', 'é', 1) ~= 0)
        assert(c:CompareStrings('a', 'A', 1) == 0)
        assert(c:CompareStrings('a', 'A', 2) ~= 0)
        local values = {'', 'a', 'A', 'á', 'b', 'é', 'é', 'z', 'ä', '😀'}
        local function byte_compare(a, b)
            for i = 1, math.min(#a, #b) do
                if a:byte(i) ~= b:byte(i) then
                    return a:byte(i) < b:byte(i) and -1 or 1
                end
            end
            return #a == #b and 0 or (#a < #b and -1 or 1)
        end
        for strength = 0, 4 do
            assert(c:CompareStrings('é', 'é', strength) == 0)
            assert(c:CompareStrings('', '', strength) == 0)
            for _, a in ipairs(values) do
                local key = c:GetSortKey(a, strength)
                assert(type(key) == 'string')
                assert(key == C_Intl.GetSortKey(a, strength))
                assert(select('#', c:GetSortKey(a, strength)) == 1)
                for _, b in ipairs(values) do
                    local comparison = c:CompareStrings(a, b, strength)
                    assert(comparison == -1 or comparison == 0 or comparison == 1)
                    assert(comparison == C_Intl.CompareStrings(a, b, strength))
                    assert(byte_compare(key, c:GetSortKey(b, strength)) == comparison)
                end
            end
        end
        -- ICU keys are binary weights, not hex/base64 or UTF-8 text.
        local key = c:GetSortKey('a', 2)
        assert(key:find(string.char(1), 1, true))
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_collation_locales_and_validation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local de = C_Intl.CreateLocaleContext('de-DE')
        local sv = C_Intl.CreateLocaleContext('sv-SE')
        assert(de:CompareStrings('ä', 'z', 2) == -1)
        assert(sv:CompareStrings('ä', 'z', 2) == 1)
        assert(de:SetLocale('svSE'))
        assert(de:CompareStrings('ä', 'z', 2) == 1)
        assert(sv:GetLocale() == 'sv-SE')
        assert(de:SetLocale('bad locale!'))
        assert(not pcall(de.CompareStrings, de, 'a', 'b', 2))
        assert(not pcall(de.GetSortKey, de, 'a', 2))
        assert(de:GetLocale() == 'bad locale!')
        for _, strength in ipairs({-1, 5, 1.5, '2', false}) do
            assert(not pcall(C_Intl.CompareStrings, 'a', 'b', strength))
            assert(not pcall(C_Intl.GetSortKey, 'a', strength))
            assert(not pcall(sv.CompareStrings, sv, 'a', 'b', strength))
            assert(not pcall(sv.GetSortKey, sv, 'a', strength))
        end
        assert(not pcall(C_Intl.CompareStrings, 'a', 'b'))
        assert(not pcall(C_Intl.GetSortKey, 'a'))
        for _, text in ipairs({string.char(255), string.char(192, 175)}) do
            assert(not pcall(C_Intl.CompareStrings, text, 'a', 2))
            assert(not pcall(C_Intl.CompareStrings, 'a', text, 2))
            assert(not pcall(C_Intl.GetSortKey, text, 2))
            assert(not pcall(sv.CompareStrings, sv, text, 'a', 2))
            assert(not pcall(sv.GetSortKey, sv, text, 2))
        end
        assert(not pcall(sv.CompareStrings, {}, 'a', 'b', 2))
        assert(not pcall(sv.GetSortKey, {}, 'a', 2))
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_collation_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)")
            .unwrap();
        crate::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
