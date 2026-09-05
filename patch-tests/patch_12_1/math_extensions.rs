use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn math_extensions_publish_documented_numeric_operations() {
    let env = WowLuaEnv::new().expect("create Lua environment");

    let (
        clamp,
        lerp,
        normalized,
        remapped,
        rounded_positive,
        rounded_negative,
        rounded_left_of_decimal,
        saturated_low,
        saturated_high,
        negative_sign,
        zero_sign,
        negative_zero_sign,
        positive_sign,
        wrapped_high,
        wrapped_low,
        equal_endpoint_wrap,
        finite,
        infinite,
        nan,
    ): (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            return math.clamp(15, 0, 10),
                math.lerp(10, 30, 1.5),
                math.normalize(15, 10, 20),
                math.remap(15, 10, 20, 100, 200),
                math.round(1.25, 1),
                math.round(-1.25, 1),
                math.round(150, -2),
                math.saturate(-2),
                math.saturate(2),
                math.sign(-3),
                math.sign(0),
                math.sign(-0.0),
                math.sign(3),
                math.wrap(12, 0, 10),
                math.wrap(-2, 0, 10),
                math.wrap(5, 7, 7),
                math.isfinite(1 / 0),
                math.isinf(1 / 0),
                math.isnan(0 / 0)
            "#,
        )
        .expect("math extensions evaluate");

    assert_eq!(clamp, 10.0);
    assert_eq!(lerp, 40.0);
    assert_eq!(normalized, 0.5);
    assert_eq!(remapped, 150.0);
    assert_eq!(rounded_positive, 1.3);
    assert_eq!(rounded_negative, -1.3);
    assert_eq!(rounded_left_of_decimal, 200.0);
    assert_eq!(saturated_low, 0.0);
    assert_eq!(saturated_high, 1.0);
    assert_eq!(negative_sign, -1.0);
    assert_eq!(zero_sign, 0.0);
    assert_eq!(negative_zero_sign, 0.0);
    assert_eq!(positive_sign, 1.0);
    assert_eq!(wrapped_high, 2.0);
    assert_eq!(wrapped_low, 8.0);
    assert_eq!(equal_endpoint_wrap, 7.0);
    assert!(!finite);
    assert!(infinite);
    assert!(nan);
}
