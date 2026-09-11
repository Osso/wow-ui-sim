use super::{DateTimeStyle as Style, format_date_time};

#[test]
fn icu4c_dates_epoch_fraction_and_timezone_boundaries() {
    let format = |seconds, zone| {
        format_date_time("en-GB", seconds, Style::None, Style::Medium, zone).unwrap()
    };
    assert_eq!(format(0.0, "UTC"), "00:00:00");
    assert_eq!(format(-0.001, "UTC"), "23:59:59");
    assert_eq!(format(0.999, "UTC"), "00:00:00");
    assert_eq!(format(1.001, "UTC"), "00:00:01");
    assert_eq!(format(0.0, ""), format(0.0, "UTC"));
    assert_eq!(format(0.0, "GMT+05:30"), "05:30:00");
    assert_eq!(format(1710053999.0, "America/New_York"), "01:59:59");
    assert_eq!(format(1710054000.0, "America/New_York"), "03:00:00");
    assert_eq!(
        format_date_time("en-GB", 0.0, Style::Short, Style::None, "America/New_York").unwrap(),
        "31/12/1969"
    );
}

#[test]
fn icu4c_dates_styles_and_extensions() {
    for style in [
        Style::None,
        Style::Short,
        Style::Medium,
        Style::Long,
        Style::Full,
    ] {
        let result = format_date_time("en-GB", 0.0, style, style, "UTC").unwrap();
        assert_eq!(result.is_empty(), matches!(style, Style::None));
    }
    let latin = format_date_time("en-GB", 0.0, Style::Short, Style::None, "UTC").unwrap();
    let arabic =
        format_date_time("en-GB-u-nu-arab", 0.0, Style::Short, Style::None, "UTC").unwrap();
    assert_ne!(latin, arabic);
    assert!(
        format_date_time("en-GB", 0.0, Style::None, Style::None, "UTC")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn icu4c_dates_validate_even_when_both_styles_are_none() {
    for zone in ["Not/A_Zone", "UTC trailing", "UTC\0", "GMT+25:00"] {
        for style in [Style::None, Style::Short] {
            assert!(
                format_date_time("en-GB", 0.0, style, style, zone).is_err(),
                "{zone:?}"
            );
        }
    }
    for seconds in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        -f64::MAX,
    ] {
        assert!(format_date_time("en-GB", seconds, Style::Short, Style::Short, "UTC").is_err());
    }
}
