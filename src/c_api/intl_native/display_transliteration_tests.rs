use super::{display_name, transliterate};

#[test]
fn icu4c_display_names_preserve_target_and_display_direction() {
    assert_eq!(display_name("fr-FR", "en-US").unwrap(), "French (France)");
    assert_eq!(
        display_name("fr-FR", "de-DE").unwrap(),
        "Französisch (Frankreich)"
    );
    assert_eq!(
        display_name("de-DE", "fr-FR").unwrap(),
        "allemand (Allemagne)"
    );
    assert_eq!(
        display_name("en_US", "de-DE").unwrap(),
        "Englisch (Vereinigte Staaten)"
    );
    assert!(display_name("", "en-US").is_err());
    assert!(display_name("en-US", "not a locale").is_err());
    assert!(display_name("en-US", "en\0US").is_err());
}

#[test]
fn icu4c_transliteration_retries_expansion_from_original_utf16() {
    for _ in 0..8 {
        assert_eq!(
            transliterate("Crème brûlée", "Latin-ASCII").unwrap(),
            "Creme brulee"
        );
        assert_eq!(transliterate("Москва", "Cyrillic-Latin").unwrap(), "Moskva");
        let input = "A😀".repeat(64);
        let expected = "\\u0041\\uD83D\\uDE00".repeat(64);
        assert_eq!(transliterate(&input, "Any-Hex").unwrap(), expected);
        assert_eq!(transliterate("", "Latin-ASCII").unwrap(), "");
        assert_eq!(transliterate("A\0B", "Latin-ASCII").unwrap(), "A\0B");
    }
}

#[test]
fn icu4c_invalid_transliterators_fail_without_poisoning_later_calls() {
    for id in ["", "Missing-Transliterator", "Latin-ASCII\0"] {
        assert!(transliterate("abc", id).is_err(), "{id:?}");
        assert!(transliterate("", id).is_err(), "empty input: {id:?}");
    }
    assert_eq!(transliterate("é", "Latin-ASCII").unwrap(), "e");
}
