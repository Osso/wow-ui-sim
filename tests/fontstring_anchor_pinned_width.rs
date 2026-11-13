//! Regression tests for FontString width with two horizontal anchors.
//!
//! The default tests cover VERIFIED rectangle geometry. Tests that assert the
//! INFERRED `GetWidth()` result run only with `optional-tests`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1600.0, 1200.0);
    env
}

// VERIFIED (Retail 12.1.5.70688 L1c Q1/Q4): the rectangle path uses the
// resolved horizontal span before the stored width. Read the edges instead of
// asserting the INFERRED public GetWidth() result.
#[test]
fn fontstring_rect_span_overrides_set_width() {
    let env = env();
    let (left, right, parent_left, parent_right): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "SpanPriorityParent", UIParent)
            parent:SetSize(220, 60)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)

            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("LEFT", parent, "LEFT", 50, 0)
            fs:SetPoint("RIGHT", parent, "RIGHT", 0, 0)
            fs:SetWidth(260)

            return fs:GetLeft(), fs:GetRight(), parent:GetLeft(), parent:GetRight()
            "#,
        )
        .unwrap();

    assert!((left - (parent_left + 50.0)).abs() < 1.0);
    assert!((right - parent_right).abs() < 1.0);
    assert!((right - left - 170.0).abs() < 1.0);
}

// VERIFIED (Retail 12.1.5.70688 L1c Q2): a changed size setter invalidates
// region geometry and queues layout. Edge queries avoid the UNKNOWN text-width
// cache behavior after parent resize.
#[test]
fn fontstring_rect_span_tracks_parent_resize() {
    let env = env();
    let (initial_left, initial_right, resized_left, resized_right): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "ResizeAnchorParent", UIParent)
            parent:SetSize(150, 30)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)

            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("TOPLEFT", parent, "TOPLEFT", 5, 0)
            fs:SetPoint("TOPRIGHT", parent, "TOPRIGHT", -5, 0)
            local initialLeft, initialRight = fs:GetLeft(), fs:GetRight()

            parent:SetSize(90, 30)
            return initialLeft, initialRight, fs:GetLeft(), fs:GetRight()
            "#,
        )
        .unwrap();

    assert!((initial_right - initial_left - 140.0).abs() < 1.0);
    assert!((resized_right - resized_left - 80.0).abs() < 1.0);
}

// INFERRED (Retail 12.1.5.70688 L1c Q1/Q4): an ordinary completed GetWidth()
// call reports the resolved span, and the span takes priority over SetWidth.
#[cfg(feature = "optional-tests")]
#[test]
fn inferred_get_width_reports_two_edge_span_over_set_width() {
    let env = env();
    let width: f64 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "GetWidthSpanParent", UIParent)
            parent:SetSize(150, 30)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("TOPLEFT")
            fs:SetPoint("TOPRIGHT")
            fs:SetWidth(260)
            return fs:GetWidth()
            "#,
        )
        .unwrap();

    assert!((width - 150.0).abs() < 1.0);
}

// INFERRED (Retail 12.1.5.70688 L1c Q1/Q4): GetWidth() reports the resolved
// span for two edges with horizontal offsets.
#[cfg(feature = "optional-tests")]
#[test]
fn inferred_get_width_reports_offset_two_edge_span() {
    let env = env();
    let width: f64 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "OffsetSpanParent", UIParent)
            parent:SetSize(220, 60)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("LEFT", parent, "LEFT", 50, 0)
            fs:SetPoint("RIGHT", parent, "RIGHT", 0, 0)
            return fs:GetWidth()
            "#,
        )
        .unwrap();

    assert!((width - 170.0).abs() < 1.0);
}

// INFERRED (Retail 12.1.5.70688 L1c Q3): with one horizontal anchor and zero
// stored width, GetWidth() uses its string-width fallback.
#[cfg(feature = "optional-tests")]
#[test]
fn inferred_single_anchor_get_width_uses_text_width_fallback() {
    let env = env();
    let has_width: bool = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "SingleAnchorParent", UIParent)
            parent:SetSize(500, 60)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("CENTER")
            fs:SetText("Short")
            return fs:GetWidth() > 0
            "#,
        )
        .unwrap();

    assert!(has_width);
}

// INFERRED (Retail 12.1.5.70688 L1c Q1/Q3): GetWidth() uses resolved target
// geometry for two edges, while the exact native point mapping remains unknown.
#[cfg(feature = "optional-tests")]
#[test]
fn inferred_get_width_uses_resolved_span_of_zero_width_target() {
    let env = env();
    let width: f64 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "ZeroWidthTargetParent", UIParent)
            parent:SetSize(260, 50)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local target = CreateFrame("Frame", "ZeroWidthTarget", parent)
            target:SetSize(0, 40)
            target:SetPoint("TOPLEFT", parent, "TOPLEFT", 10, 0)
            target:SetPoint("TOPRIGHT", parent, "TOPRIGHT", -10, 0)
            local fs = target:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("LEFT", target, "LEFT", 20, 0)
            fs:SetPoint("RIGHT", target, "RIGHT", -20, 0)
            return fs:GetWidth()
            "#,
        )
        .unwrap();

    assert!((width - 200.0).abs() < 1.0);
}
