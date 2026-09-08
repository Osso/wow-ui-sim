//! Ordinary PTR math behavior; native invalid-input and security semantics are not covered.

use crate::lua_api::WowLuaEnv;

fn assert_lua(script: &str) {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.eval::<()>(script)
        .expect("math extension assertions should pass");
}

#[test]
fn clamp_bounds_ordinary_numbers() {
    assert_lua(
        r#"
        assert(math.clamp(-2, 0, 10) == 0)
        assert(math.clamp(4, 0, 10) == 4)
        assert(math.clamp(12, 0, 10) == 10)
        assert(select('#', math.clamp(4, 0, 10)) == 1)
        "#,
    );
}

#[test]
fn isfinite_classifies_numbers() {
    assert_lua(
        r#"
        assert(math.isfinite(0) == true)
        assert(math.isfinite(-1.5) == true)
        assert(math.isfinite(math.huge) == false)
        assert(math.isfinite(-math.huge) == false)
        assert(math.isfinite(0 / 0) == false)
        assert(select('#', math.isfinite(1)) == 1)
        "#,
    );
}

#[test]
fn isinf_classifies_both_infinities() {
    assert_lua(
        r#"
        assert(math.isinf(math.huge) == true)
        assert(math.isinf(-math.huge) == true)
        assert(math.isinf(12) == false)
        assert(math.isinf(0 / 0) == false)
        assert(select('#', math.isinf(math.huge)) == 1)
        "#,
    );
}

#[test]
fn isnan_distinguishes_nan_from_numbers() {
    assert_lua(
        r#"
        assert(math.isnan(0 / 0) == true)
        assert(math.isnan(0) == false)
        assert(math.isnan(-2.5) == false)
        assert(math.isnan(math.huge) == false)
        assert(select('#', math.isnan(0 / 0)) == 1)
        "#,
    );
}

#[test]
fn lerp_interpolates_endpoints_and_fraction() {
    assert_lua(
        r#"
        assert(math.lerp(10, 20, 0) == 10)
        assert(math.lerp(10, 20, 1) == 20)
        assert(math.lerp(10, 20, 0.25) == 12.5)
        assert(select('#', math.lerp(10, 20, 0.25)) == 1)
        "#,
    );
}

#[test]
fn normalize_maps_interval_to_unit_range() {
    assert_lua(
        r#"
        assert(math.normalize(10, 10, 20) == 0)
        assert(math.normalize(20, 10, 20) == 1)
        assert(math.normalize(15, 10, 20) == 0.5)
        assert(select('#', math.normalize(15, 10, 20)) == 1)
        "#,
    );
}

#[test]
fn remap_maps_between_numeric_intervals() {
    assert_lua(
        r#"
        assert(math.remap(0, 0, 10, 100, 200) == 100)
        assert(math.remap(10, 0, 10, 100, 200) == 200)
        assert(math.remap(5, 0, 10, 100, 200) == 150)
        assert(select('#', math.remap(5, 0, 10, 100, 200)) == 1)
        "#,
    );
}

#[test]
fn round_supports_default_and_decimal_precision() {
    assert_lua(
        r#"
        assert(math.round(2.5) == 3)
        assert(math.round(-2.5) == -3)
        assert(math.round(2.25, 1) == 2.3)
        assert(math.round(-2.25, 1) == -2.3)
        assert(math.round(124, -1) == 120)
        assert(select('#', math.round(2.5)) == 1)
        "#,
    );
}

#[test]
fn saturate_clamps_to_unit_interval() {
    assert_lua(
        r#"
        assert(math.saturate(-2) == 0)
        assert(math.saturate(0.25) == 0.25)
        assert(math.saturate(2) == 1)
        assert(select('#', math.saturate(0.25)) == 1)
        "#,
    );
}

#[test]
fn sign_reports_negative_zero_and_positive_numbers() {
    assert_lua(
        r#"
        assert(math.sign(-3) == -1)
        assert(math.sign(0) == 0)
        assert(math.sign(7) == 1)
        assert(select('#', math.sign(7)) == 1)
        "#,
    );
}

#[test]
fn wrap_maps_values_into_nonzero_interval() {
    assert_lua(
        r#"
        assert(math.wrap(7, 5, 15) == 7)
        assert(math.wrap(17, 5, 15) == 7)
        assert(math.wrap(3, 5, 15) == 13)
        assert(select('#', math.wrap(17, 5, 15)) == 1)
        "#,
    );
}
