//! ICU4C search results are modeled behavior, not native WoW equivalence.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_string_matches_strengths_and_canonical_equivalence() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local en = C_Intl.CreateLocaleContext("en-US")
        local function check(text, pattern, strength, expected)
            local actual = en:FindStringMatches(text, pattern, strength)
            assert(#actual == #expected, #actual .. " matches, expected " .. #expected)
            for i, offset in ipairs(expected) do assert(actual[i] == offset) end
            assert(select('#', en:FindStringMatches(text, pattern, strength)) == 1)
        end
        check("café CAFE cafe", "cafe", 0, {0, 6, 11})
        check("café CAFE cafe", "cafe", 1, {6, 11})
        check("café CAFE cafe", "cafe", 2, {11})
        check("café CAFE cafe", "cafe", 3, {11})
        check("é é", "é", 4, {0, 3})
        check("é é", "é", 4, {0, 4})
        local de = C_Intl.CreateLocaleContext("de-DE")
        local matches = de:FindStringMatches("Straße STRASSE", "strasse", 0)
        assert(#matches == 2 and matches[1] == 0 and matches[2] == 8)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_string_matches_byte_offsets_nonoverlap_and_locale_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local context = C_Intl.CreateLocaleContext("enUS")
        local matches = context:FindStringMatches("😀é banana banana", "ana", 2)
        assert(#matches == 2 and matches[1] == 8 and matches[2] == 15)
        matches[1] = -1
        assert(context:FindStringMatches("😀é banana banana", "ana", 2)[1] == 8)
        local zeros = context:FindStringMatches("A\0B A\0B", "A\0B", 2)
        assert(#zeros == 2 and zeros[1] == 0 and zeros[2] == 4)
        local turkish = C_Intl.CreateLocaleContext("tr-TR")
        local dotted = turkish:FindStringMatches("I ı İ i", "i", 0)
        assert(#dotted == 2 and dotted[1] == 5 and dotted[2] == 8)
        assert(turkish:SetLocale("en-US"))
        local english = turkish:FindStringMatches("I ı İ i", "i", 0)
        assert(#english == 3 and english[1] == 0 and english[2] == 5 and english[3] == 8)
        assert(context:GetLocale() == "enUS")
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        local global = C_Intl.FindStringMatches("😀café CAFE", "cafe", 0)
        local local_result = current:FindStringMatches("😀café CAFE", "cafe", 0)
        assert(#global == 2 and global[1] == 4 and global[2] == 10)
        assert(global[1] == local_result[1] and global[2] == local_result[2])
        local large = context:FindStringMatches(string.rep("😀a", 2048), "a", 2)
        assert(#large == 2048)
        for i, offset in ipairs(large) do assert(offset == (i - 1) * 5 + 4) end
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_string_matches_empty_and_invalid_inputs() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local context = C_Intl.CreateLocaleContext("en-US")
        assert(type(rawget(C_Intl, "FindStringMatches")) == "function")
        for strength = 0, 4 do
            assert(#context:FindStringMatches("", "a", strength) == 0)
            assert(#context:FindStringMatches("abc", "", strength) == 0)
            assert(#context:FindStringMatches("", "", strength) == 0)
            assert(#context:FindStringMatches("abc", "xyz", strength) == 0)
        end
        for _, bad in ipairs({-1, 5, 1.5, math.huge, 0/0, "0", false}) do
            assert(not pcall(context.FindStringMatches, context, "abc", "a", bad))
        end
        for _, bad in ipairs({string.char(255), string.char(226, 130)}) do
            assert(not pcall(context.FindStringMatches, context, bad, "a", 0))
            assert(not pcall(context.FindStringMatches, context, "abc", bad, 0))
            assert(not pcall(C_Intl.FindStringMatches, bad, "a", 0))
        end
        assert(not pcall(context.FindStringMatches, {}, "a", "a", 0))
        assert(not pcall(context.FindStringMatches, context, "a", "a"))
        assert(not pcall(context.FindStringMatches, context, false, "a", 0))
        assert(context:SetLocale("not a locale"))
        assert(not pcall(context.FindStringMatches, context, "", "", 0))
        assert(context:SetLocale("en-US"))
        assert(context:FindStringMatches("abc", "b", 2)[1] == 1)
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_string_matches_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)").unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
