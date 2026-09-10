//! ICU4C formatting is simulator behavior, not a native WoW output guarantee.
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn intl_numbers_formats_and_parses_all_styles() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local en = C_Intl.CreateLocaleContext("en-US")
        local styles = Enum.NumberStyle
        local cases = {
            {1234.5, styles.Decimal, "1,234.5", 1234.5},
            {1234.5, styles.Integer, "1,234", 1234},
            {0.125, styles.Percent, "12%", 0.12},
            {12.5, styles.Currency, "$12.50", 12.5},
        }
        for _, case in ipairs(cases) do
            local value, style, expected, parsed = unpack(case)
            local formatted = en:FormatNumber(value, style)
            assert(formatted == expected, tostring(formatted))
            assert(math.abs(en:ParseNumber(formatted, style) - parsed) < 0.000001)
            assert(select('#', en:FormatNumber(value, style)) == 1)
            assert(select('#', en:ParseNumber(formatted, style)) == 1)
        end
        local current = C_Intl.CreateLocaleContext(C_Intl.GetCurrentLocale())
        assert(C_Intl.FormatNumber(1234.5, styles.Decimal) == current:FormatNumber(1234.5, styles.Decimal))
        assert(C_Intl.ParseNumber("42", styles.Decimal) == 42)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_numbers_currency_results_and_context_changes_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local en = C_Intl.CreateLocaleContext("enUS")
        local de = C_Intl.CreateLocaleContext("de-DE")
        assert(en:FormatCurrency(1234.5, "EUR") == "€1,234.50")
        assert(de:FormatCurrency(1234.5, "EUR") == "1.234,50 €")
        for _, context in ipairs({en, de, C_Intl.CreateLocaleContext("fr-FR")}) do
            local text = context:FormatCurrency(1234.5, "EUR")
            local result = context:ParseCurrency(text)
            assert(result.amount == 1234.5 and result.currencyCode == "EUR")
            result.amount = -1
            assert(context:ParseCurrency(text).amount == 1234.5)
        end
        assert(de:SetLocale("en-US"))
        assert(de:FormatNumber(1234.5, 0) == en:FormatNumber(1234.5, 0))
        assert(en:GetLocale() == "enUS")
        local result = C_Intl.ParseCurrency(C_Intl.FormatCurrency(12.5, "USD"))
        assert(result.amount == 12.5 and result.currencyCode == "USD")
        assert(CurrencyParseResult == nil)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn intl_numbers_reject_invalid_arguments_and_incomplete_parses() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local context = C_Intl.CreateLocaleContext("en-US")
        for _, text in ipairs({"", "not a number", "12 trailing", "12\0"}) do
            assert(select('#', context:ParseNumber(text, 0)) == 0)
        end
        assert(select('#', context:ParseCurrency("not currency")) == 0)
        assert(select('#', C_Intl.ParseNumber("not a number", 0)) == 0)
        assert(select('#', C_Intl.ParseCurrency("not currency")) == 0)
        for _, value in ipairs({math.huge, -math.huge, 0/0}) do
            assert(not pcall(context.FormatNumber, context, value, 0))
            assert(not pcall(context.FormatCurrency, context, value, "USD"))
        end
        assert(not pcall(context.FormatNumber, context, 1, 99))
        assert(not pcall(context.ParseNumber, context, "1", 99))
        assert(not pcall(context.FormatCurrency, context, 1, "US"))
        assert(not pcall(context.ParseNumber, context, string.char(255), 0))
        assert(not pcall(context.ParseCurrency, context, string.char(255)))
        assert(not pcall(context.FormatNumber, {}, 1, 0))
        assert(context:FormatNumber(42, 0) == "42")
    "#).unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn intl_numbers_preserves_retail_namespace_absence() {
    let env = WowLuaEnv::new().unwrap();
    for _ in 0..2 {
        env.exec("assert(C_Intl == nil); assert(CurrencyParseResult == nil)").unwrap();
        wow_ui_sim::ptr::compat_bootstrap::apply_post_load(&env);
    }
}
