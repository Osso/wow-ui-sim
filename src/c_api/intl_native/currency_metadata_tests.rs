use super::{CurrencyNameStyle, currency_fraction_digits, currency_name};

#[test]
fn currency_metadata_maps_all_name_styles_and_locales() {
    for (style, usd, cad) in [
        (CurrencyNameStyle::Symbol, "$", "CA$"),
        (CurrencyNameStyle::NarrowSymbol, "$", "$"),
        (CurrencyNameStyle::Long, "US Dollar", "Canadian Dollar"),
        (CurrencyNameStyle::FormalSymbol, "$", "CA$"),
        (CurrencyNameStyle::VariantSymbol, "$", "CA$"),
    ] {
        assert_eq!(
            currency_name("en-US", "USD", style).unwrap().as_deref(),
            Some(usd)
        );
        assert_eq!(
            currency_name("en-US", "cad", style).unwrap().as_deref(),
            Some(cad)
        );
    }
    for (locale, expected) in [
        ("fr-FR", "dollar des États-Unis"),
        ("de-DE", "US-Dollar"),
        ("en-US-u-nu-arab", "US Dollar"),
    ] {
        assert_eq!(
            currency_name(locale, "USD", CurrencyNameStyle::Long)
                .unwrap()
                .as_deref(),
            Some(expected)
        );
    }
}

#[test]
fn currency_metadata_distinguishes_catalogue_misses_from_zero_digits() {
    for (code, expected) in [("USD", 2), ("jpy", 0), ("KWD", 3)] {
        assert_eq!(currency_fraction_digits(code).unwrap(), Some(expected));
    }
    assert_eq!(currency_fraction_digits("ZZZ").unwrap(), None);
    for style in [
        CurrencyNameStyle::Symbol,
        CurrencyNameStyle::NarrowSymbol,
        CurrencyNameStyle::Long,
        CurrencyNameStyle::FormalSymbol,
        CurrencyNameStyle::VariantSymbol,
    ] {
        assert_eq!(currency_name("en-US", "ZZZ", style).unwrap(), None);
    }
    for code in ["", "US", "USDX", "U1D", "€UR", "US\0"] {
        assert!(currency_name("en-US", code, CurrencyNameStyle::Symbol).is_err());
        assert!(currency_fraction_digits(code).is_err());
    }
}

#[test]
fn currency_metadata_outputs_survive_success_no_match_and_error_cleanup() {
    let retained = currency_name("fr-FR", "USD", CurrencyNameStyle::Long).unwrap();
    for _ in 0..64 {
        let name = currency_name("de-DE", "USD", CurrencyNameStyle::Long).unwrap();
        assert_eq!(name.as_deref(), Some("US-Dollar"));
        drop(name);
        assert_eq!(
            currency_name("en-US", "ZZZ", CurrencyNameStyle::Symbol).unwrap(),
            None
        );
        assert!(currency_name("en-US!", "USD", CurrencyNameStyle::Symbol).is_err());
        assert_eq!(currency_fraction_digits("JPY").unwrap(), Some(0));
        assert_eq!(currency_fraction_digits("ZZZ").unwrap(), None);
    }
    assert_eq!(retained.as_deref(), Some("dollar des États-Unis"));
}
