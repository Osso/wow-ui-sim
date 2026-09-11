//! ICU currency metadata is modeled behavior, not a native WoW data guarantee.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_currency_metadata_names_cover_styles_and_context_updates() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local en = C_Intl.CreateLocaleContext("en-US")
        local fr = C_Intl.CreateLocaleContext("fr-FR")
        local de = C_Intl.CreateLocaleContext("de-DE")
        local styles = Enum.CurrencyNameStyle
        local cases = {
            {styles.Symbol, "$", "CA$"},
            {styles.NarrowSymbol, "$", "$"},
            {styles.Long, "US Dollar", "Canadian Dollar"},
            {styles.FormalSymbol, "$", "CA$"},
            {styles.VariantSymbol, "$", "CA$"},
        }
        for _, case in ipairs(cases) do
            local style, usd, cad = unpack(case)
            assert(en:GetCurrencyName("USD", style) == usd)
            assert(en:GetCurrencyName("cad", style) == cad)
            assert(select('#', en:GetCurrencyName("USD", style)) == 1)
        end
        assert(fr:GetCurrencyName("USD", styles.Long) == "dollar des États-Unis")
        assert(de:GetCurrencyName("USD", styles.Long) == "US-Dollar")
        assert(fr:SetLocale("de-DE"))
        assert(fr:GetCurrencyName("USD", styles.Long) == de:GetCurrencyName("USD", styles.Long))
        assert(en:GetLocale() == "en-US")
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        for _, style in ipairs({0, 1, 2, 3, 4}) do
            assert(C_Intl.GetCurrencyName("USD", style) == current:GetCurrencyName("USD", style))
            assert(select('#', C_Intl.GetCurrencyName("USD", style)) == 1)
        end
        local extended = C_Intl.CreateLocaleContext("en-US-u-nu-arab")
        assert(extended:GetCurrencyName("USD", styles.Long) == "US Dollar")
        assert(extended:GetLocale() == "en-US-u-nu-arab")
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_currency_metadata_fraction_digits_distinguish_zero_and_unknown() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, case in ipairs({{"USD", 2}, {"jpy", 0}, {"KWD", 3}}) do
            assert(C_Intl.GetCurrencyFractionDigits(case[1]) == case[2])
            assert(select('#', C_Intl.GetCurrencyFractionDigits(case[1])) == 1)
        end
        assert(select('#', C_Intl.GetCurrencyFractionDigits("ZZZ")) == 0)
        local en = C_Intl.CreateLocaleContext("en-US")
        for style = 0, 4 do
            assert(select('#', C_Intl.GetCurrencyName("ZZZ", style)) == 0)
            assert(select('#', en:GetCurrencyName("ZZZ", style)) == 0)
        end
        assert(en.GetCurrencyFractionDigits == nil)
        assert(C_Intl.GetCurrencyFractionDigits("JPY") == 0)
        assert(en:GetCurrencyName("USD", 0) == "$")
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_currency_metadata_rejects_malformed_inputs_without_state_changes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local en = C_Intl.CreateLocaleContext("en-US")
        for _, code in ipairs({"", "US", "USDX", "U1D", "US\0", "€UR", string.char(255)}) do
            assert(not pcall(C_Intl.GetCurrencyName, code, 0))
            assert(not pcall(en.GetCurrencyName, en, code, 0))
            assert(not pcall(C_Intl.GetCurrencyFractionDigits, code))
        end
        for _, style in ipairs({-1, 5, 0.5, "0", false, math.huge, 0/0}) do
            assert(not pcall(en.GetCurrencyName, en, "USD", style))
            assert(not pcall(C_Intl.GetCurrencyName, "USD", style))
        end
        assert(not pcall(C_Intl.GetCurrencyName, "USD"))
        assert(not pcall(C_Intl.GetCurrencyFractionDigits))
        assert(not pcall(C_Intl.GetCurrencyFractionDigits, 123))
        assert(not pcall(en.GetCurrencyName, {}, "USD", 0))
        assert(en:GetLocale() == "en-US")
        assert(en:SetLocale("invalid!"))
        assert(not pcall(en.GetCurrencyName, en, "USD", 0))
        assert(en:SetLocale("en-US"))
        assert(en:GetCurrencyName("USD", 0) == "$")
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_currency_metadata_preserves_retail_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(LuaLocaleContext == nil)").unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
