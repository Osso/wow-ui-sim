//! ICU4C data and error policies are modeled behavior, not native WoW guarantees.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_display_names_use_argument_as_display_language() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local target = C_Intl.CreateLocaleContext("fr-FR")
        assert(target:GetDisplayName("en-US") == "French (France)")
        assert(target:GetDisplayName("de-DE") == "Französisch (Frankreich)")
        assert(target:GetDisplayName("fr-FR") == "français (France)")
        assert(target:GetLocale() == "fr-FR")
        assert(target:SetLocale("de-DE"))
        assert(target:GetDisplayName("enUS") == "German (Germany)")
        assert(target:GetDisplayName("fr-FR") == "allemand (Allemagne)")
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        for _, display in ipairs({"enUS", "fr-FR", "de-DE"}) do
            assert(C_Intl.GetDisplayName(display) == current:GetDisplayName(display))
            assert(select('#', target:GetDisplayName(display)) == 1)
        end
        assert(not pcall(target.GetDisplayName, target, ""))
        assert(not pcall(target.GetDisplayName, target, "not a locale"))
        assert(not pcall(target.GetDisplayName, target, string.char(255)))
        assert(not pcall(target.GetDisplayName, {}, "en-US"))
        assert(target:SetLocale("not a locale"))
        assert(not pcall(target.GetDisplayName, target, "en-US"))
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_transliteration_handles_growth_and_repeated_calls() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(type(rawget(C_Intl, "Transliterate")) == "function")
        for _ = 1, 8 do
            assert(C_Intl.Transliterate("Crème brûlée", "Latin-ASCII") == "Creme brulee")
            assert(C_Intl.Transliterate("Москва", "Cyrillic-Latin") == "Moskva")
            assert(C_Intl.Transliterate("", "Latin-ASCII") == "")
            assert(C_Intl.Transliterate("A\0B", "Latin-ASCII") == "A\0B")
            local source = string.rep("A😀", 64)
            local expected = string.rep("\\u0041\\uD83D\\uDE00", 64)
            assert(C_Intl.Transliterate(source, "Any-Hex") == expected)
        end
        assert(select('#', C_Intl.Transliterate("é", "Latin-ASCII")) == 1)
        local context = C_Intl.CreateLocaleContext("en-US")
        assert(context.Transliterate == nil)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_display_transliteration_rejects_invalid_input_without_poisoning_calls() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, id in ipairs({"", "Missing-Transliterator", "Latin-ASCII\0", string.char(255)}) do
            assert(not pcall(C_Intl.Transliterate, "abc", id))
        end
        assert(not pcall(C_Intl.Transliterate, string.char(255), "Latin-ASCII"))
        assert(not pcall(C_Intl.Transliterate, "abc"))
        assert(not pcall(C_Intl.Transliterate, false, "Latin-ASCII"))
        assert(not pcall(C_Intl.GetDisplayName, string.char(255)))
        assert(not pcall(C_Intl.GetDisplayName))
        assert(C_Intl.Transliterate("é", "Latin-ASCII") == "e")
        assert(type(C_Intl.GetDisplayName("en-US")) == "string")
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_display_transliteration_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)").unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
