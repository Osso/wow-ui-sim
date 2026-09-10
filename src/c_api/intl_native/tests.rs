use super::{NumberStyle, format_currency, format_number, parse_currency, parse_number, version};

#[test]
fn formats_locale_numbers_percent_and_integer_rounding() {
    assert_eq!(
        format_number("en_US", 1234.5, NumberStyle::Decimal).unwrap(),
        "1,234.5"
    );
    assert_eq!(
        format_number("de_DE", 1234.5, NumberStyle::Decimal).unwrap(),
        "1.234,5"
    );
    assert_eq!(
        format_number("fr_FR", 1234.5, NumberStyle::Decimal).unwrap(),
        "1\u{202f}234,5"
    );
    assert_eq!(
        format_number("en-US", 0.125, NumberStyle::Percent).unwrap(),
        "12%"
    );
    assert_eq!(
        format_number("en-US", 2.5, NumberStyle::Integer).unwrap(),
        "2"
    );
    assert_eq!(
        format_number("en-US", 3.5, NumberStyle::Integer).unwrap(),
        "4"
    );
    assert_eq!(
        format_number("en-US", -2.5, NumberStyle::Integer).unwrap(),
        "-2"
    );
}

#[test]
fn formats_explicit_and_locale_default_currencies() {
    assert_eq!(
        format_currency("en-US", 1234.5, "usd").unwrap(),
        "$1,234.50"
    );
    assert_eq!(
        format_currency("de-DE", 1234.5, "EUR").unwrap(),
        "1.234,50\u{a0}€"
    );
    assert_eq!(
        format_currency("fr-FR", 1234.5, "EUR").unwrap(),
        "1\u{202f}234,50\u{a0}€"
    );
    assert_eq!(format_currency("en-US", 1234.5, "JPY").unwrap(), "¥1,234");
    assert_eq!(
        format_number("en-US", 12.5, NumberStyle::Currency).unwrap(),
        "$12.50"
    );
}

#[test]
fn parses_entire_localized_numbers_and_percentages() {
    assert_eq!(
        parse_number("en-US", "1,234.5", NumberStyle::Decimal).unwrap(),
        Some(1234.5)
    );
    assert_eq!(
        parse_number("de-DE", "1.234,5", NumberStyle::Decimal).unwrap(),
        Some(1234.5)
    );
    assert_eq!(
        parse_number("fr-FR", "1\u{202f}234,5", NumberStyle::Decimal).unwrap(),
        Some(1234.5)
    );
    assert_eq!(
        parse_number("en-US", "12.5%", NumberStyle::Percent).unwrap(),
        Some(0.125)
    );
    assert_eq!(
        parse_number("en-US", "123", NumberStyle::Integer).unwrap(),
        Some(123.0)
    );
    assert_eq!(
        parse_number("en-US", "123.5", NumberStyle::Integer).unwrap(),
        None
    );
    assert_eq!(
        parse_number("en-US", "$12.50", NumberStyle::Currency).unwrap(),
        Some(12.5)
    );
}

#[test]
fn parses_currency_amount_and_iso_code() {
    for (locale, text, amount, code) in [
        ("en-US", "$1,234.50", 1234.5, "USD"),
        ("de-DE", "1.234,50\u{a0}€", 1234.5, "EUR"),
        ("fr-FR", "1\u{202f}234,50\u{a0}€", 1234.5, "EUR"),
    ] {
        let result = parse_currency(locale, text).unwrap().unwrap();
        assert_eq!(result.amount, amount);
        assert_eq!(result.currency_code, code);
    }
}

#[test]
fn rejects_partial_empty_nonfinite_and_embedded_nul_parses() {
    for text in [
        "",
        "not a number",
        "123junk",
        "123\0",
        "123\0junk",
        "123 ",
        "NaN",
        "∞",
        "1e9999",
    ] {
        assert_eq!(
            parse_number("en-US", text, NumberStyle::Decimal).unwrap(),
            None,
            "{text:?}"
        );
    }
    for text in ["", "not currency", "$12junk", "$12\0", "$12 ", "$∞"] {
        assert_eq!(parse_currency("en-US", text).unwrap(), None, "{text:?}");
    }
}

#[test]
fn preserves_bcp47_extensions_and_consumes_the_full_locale_tag() {
    let text = format_number("en-US-u-nu-arab", 1234.5, NumberStyle::Decimal).unwrap();
    assert_eq!(text, "١٬٢٣٤٫٥");
    assert_eq!(
        parse_number("en-US-u-nu-arab", &text, NumberStyle::Decimal).unwrap(),
        Some(1234.5)
    );
    for locale in ["", "en-US!", "en-US-", "en-US\0junk"] {
        assert!(
            format_number(locale, 1.0, NumberStyle::Decimal).is_err(),
            "{locale:?}"
        );
    }
}

#[test]
fn rejects_invalid_currency_codes_and_nonfinite_values() {
    for code in ["", "US", "USDX", "U1D", "€UR", "US\0"] {
        assert!(format_currency("en-US", 1.0, code).is_err(), "{code:?}");
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(format_number("en-US", value, NumberStyle::Decimal).is_err());
        assert!(format_currency("en-US", value, "USD").is_err());
    }
}

#[test]
fn reports_linked_icu_version() {
    let parts: Vec<u32> = version()
        .split('.')
        .map(|part| part.parse().unwrap())
        .collect();
    assert_eq!(parts.len(), 4);
    assert!(parts[0] >= 72);
}
