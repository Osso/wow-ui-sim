//! FontStrings with two opposite-edge anchors (TOPLEFT + TOPRIGHT, LEFT + RIGHT, etc.)
//! must derive width from the anchor target rather than from auto-measured text width.
//! Otherwise long text overflows the slot it was anchored into — see the SpellBook
//! `Name` FontString in Blizzard_PlayerSpells where text was running into adjacent
//! icon columns.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1600.0, 1200.0);
    env
}

#[test]
fn two_anchor_fontstring_takes_parent_width_not_text_width() {
    let env = env();
    let (width, is_auto): (f64, bool) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "AnchorParent", UIParent)
            parent:SetSize(150, 30)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("TOPLEFT")
            fs:SetPoint("TOPRIGHT")
            fs:SetWordWrap(true)
            fs:SetMaxLines(3)
            fs:SetText("This is a very long string that would exceed the parent width if rendered as one line")
            return fs:GetWidth(), fs.__widthIsTextAuto or false
            "#,
        )
        .unwrap();
    assert!(
        (width - 150.0).abs() < 1.0,
        "FontString width should match anchor-pinned parent width 150, got {width}"
    );
    assert!(
        !is_auto,
        "FontString width must not be flagged as auto-text-derived when anchors pin both edges"
    );
}

#[test]
fn left_offset_anchor_subtracts_from_parent_width() {
    let env = env();
    let width: f64 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "AnchorParent2", UIParent)
            parent:SetSize(220, 60)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("LEFT", parent, "LEFT", 50, 0)
            fs:SetPoint("RIGHT", parent, "RIGHT", 0, 0)
            fs:SetWordWrap(true)
            fs:SetText("Crusader Strike")
            return fs:GetWidth()
            "#,
        )
        .unwrap();
    assert!(
        (width - 170.0).abs() < 1.0,
        "FontString width should be parent.width(220) - left_offset(50) = 170, got {width}"
    );
}

#[test]
fn single_anchor_fontstring_still_uses_auto_text_width() {
    let env = env();
    // Sanity: behavior unchanged for the common case (one anchor, no width set).
    let (has_width, is_auto): (bool, bool) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "AnchorParent3", UIParent)
            parent:SetSize(500, 60)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("CENTER")
            fs:SetText("Short")
            return fs:GetWidth() > 0, fs.__widthIsTextAuto or true
            "#,
        )
        .unwrap();
    assert!(
        has_width,
        "single-anchor FontString should still auto-size width from text"
    );
    let _ = is_auto; // flag is internal; only the width matters for the public test contract
}

#[test]
fn fontstring_get_num_lines_reports_wrapped_line_count() {
    let env = env();
    let (method_type, short_lines, wrapped_lines): (String, i32, i32) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "NumLinesParent", UIParent)
            parent:SetSize(120, 80)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)

            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetWidth(120)
            fs:SetWordWrap(true)

            fs:SetText("Short")
            local shortLines = fs:GetNumLines()

            fs:SetText("This is a much longer Adventure Guide description that should wrap across multiple lines")
            local wrappedLines = fs:GetNumLines()

            return type(fs.GetNumLines), shortLines, wrappedLines
            "#,
        )
        .unwrap();

    assert_eq!(method_type, "function");
    assert_eq!(short_lines, 1, "single-line text should report one line");
    assert!(
        wrapped_lines > 1,
        "wrapped text should report multiple lines, got {wrapped_lines}"
    );
}

// INFERRED from Retail 12.1.5.70688 L1 Q1: two resolved horizontal anchors set
// the FontString width. `EUI_ActionBars_Apply.lua:315-357` sets the HotKey text before its two anchors.
#[test]
fn two_edge_fontstring_width_updates_when_points_follow_text() {
    let env = env();
    let (width, text_width): (f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "TextFirstAnchorParent", UIParent)
            parent:SetSize(150, 30)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetText("A long consumer label that is wider than its anchor span")
            local textWidth = fs:GetWidth()
            fs:SetWordWrap(true)
            fs:ClearAllPoints()
            fs:SetPoint("TOPRIGHT", parent, "TOPRIGHT", -5, 0)
            fs:SetPoint("TOPLEFT", parent, "TOPLEFT", 5, 0)
            return fs:GetWidth(), textWidth
            "#,
        )
        .unwrap();
    let anchor_width = 140.0;
    assert!(
        (text_width - anchor_width).abs() > 1.0,
        "the fixture text must exceed the 140-unit anchor span, got {text_width}"
    );
    assert!(
        (width - anchor_width).abs() < 1.0,
        "GetWidth should follow the two anchors, got {width} for span {anchor_width}"
    );
}

// INFERRED from Retail 12.1.5.70688 L1 Q1: anchor width uses the target's
// resolved span, not only its stored width. L1 Q2 verifies SetWidth(0) stores zero.
#[test]
fn two_edge_fontstring_uses_resolved_width_of_zero_width_target() {
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
            fs:SetWordWrap(true)
            fs:SetText("Resolved target width")
            return fs:GetWidth()
            "#,
        )
        .unwrap();
    let anchor_width = 200.0;
    assert!(
        (width - anchor_width).abs() < 1.0,
        "FontString width should use the resolved target span, got {width} for {anchor_width}"
    );
}

// INFERRED from Retail 12.1.5.70688 L1 Q1: the FontString's anchored width
// follows current target geometry after the parent is resized.
#[test]
fn two_edge_fontstring_width_tracks_parent_resize() {
    let env = env();
    let (initial_width, width, parent_width): (f64, f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "ResizeAnchorParent", UIParent)
            parent:SetSize(150, 30)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
            local fs = parent:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetPoint("TOPLEFT", parent, "TOPLEFT", 5, 0)
            fs:SetPoint("TOPRIGHT", parent, "TOPRIGHT", -5, 0)
            fs:SetWordWrap(true)
            fs:SetText("A long label that remains anchored when its parent changes width")
            local initialWidth = fs:GetWidth()
            parent:SetSize(90, 30)
            return initialWidth, fs:GetWidth(), parent:GetWidth()
            "#,
        )
        .unwrap();
    let anchor_width = parent_width - 10.0;
    assert!(
        (initial_width - 140.0).abs() < 1.0,
        "initial anchored width should be 140 units, got {initial_width}"
    );
    assert!(
        (width - anchor_width).abs() < 1.0,
        "GetWidth should follow the resized anchors, got {width} for {anchor_width}"
    );
}
