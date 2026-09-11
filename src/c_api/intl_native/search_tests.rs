use super::{SearchStrength, find_string_matches};

#[test]
fn icu4c_search_maps_multibyte_ranges_and_expansions() {
    let cases = [
        (
            "en-US",
            "😀é banana banana",
            "ana",
            SearchStrength::Tertiary,
            vec![8, 15],
        ),
        (
            "de-DE",
            "Straße STRASSE",
            "strasse",
            SearchStrength::Primary,
            vec![0, 8],
        ),
        (
            "en-US",
            "é e\u{301}",
            "é",
            SearchStrength::Identical,
            vec![0, 3],
        ),
        (
            "en-US",
            "A\0B A\0B",
            "A\0B",
            SearchStrength::Tertiary,
            vec![0, 4],
        ),
    ];
    for (locale, text, pattern, strength, expected) in cases {
        assert_eq!(
            find_string_matches(locale, text, pattern, strength).unwrap(),
            expected
        );
    }
}

#[test]
fn icu4c_search_empty_and_ignorable_patterns_terminate() {
    for (text, pattern) in [("", ""), ("abc", ""), ("", "abc"), ("abc", "\u{ad}")] {
        assert!(
            find_string_matches("en-US", text, pattern, SearchStrength::Primary)
                .unwrap()
                .is_empty()
        );
    }
    for locale in ["", "not a locale", "en-US\0suffix"] {
        assert!(find_string_matches(locale, "", "", SearchStrength::Primary).is_err());
    }
}

#[test]
fn icu4c_search_repeated_allocations_and_supplementary_matches() {
    let source = "😀x😀x😀";
    for _ in 0..32 {
        assert_eq!(
            find_string_matches("en-US", source, "😀", SearchStrength::Identical).unwrap(),
            vec![0, 5, 10]
        );
        assert_eq!(
            find_string_matches("en-US", source, "absent", SearchStrength::Primary).unwrap(),
            Vec::<usize>::new()
        );
    }
}
