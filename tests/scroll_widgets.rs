//! Tests for ScrollFrame, Slider, and ScrollBar widgets.
//!
//! These tests cover scroll widgets and their templates from Blizzard_SharedXML.

mod common;

use common::env_with_shared_xml;
use wow_ui_sim::lua_api::WowLuaEnv;

// ============================================================================
// Basic ScrollFrame Tests
// ============================================================================

#[test]
fn test_create_scrollframe_basic() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollFrameBasic", UIParent)
        sf:SetSize(200, 300)
        sf:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return TestScrollFrameBasic:GetObjectType()").unwrap();
    assert_eq!(obj_type, "ScrollFrame");
}

// ============================================================================
// FauxScrollFrameTemplate Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_scrollframe_template_creates_scrollbar() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollFrameTemplate", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    let has_scrollbar: bool = env
        .eval("return TestScrollFrameTemplate.ScrollBar ~= nil")
        .unwrap();
    assert!(
        has_scrollbar,
        "ScrollFrame with FauxScrollFrameTemplate should have ScrollBar"
    );
}

#[test]
fn test_scrollbar_has_buttons() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollBarButtons", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    let has_up: bool = env
        .eval("return TestScrollBarButtons.ScrollBar.ScrollUpButton ~= nil")
        .unwrap();
    let has_down: bool = env
        .eval("return TestScrollBarButtons.ScrollBar.ScrollDownButton ~= nil")
        .unwrap();

    assert!(has_up, "ScrollBar should have ScrollUpButton");
    assert!(has_down, "ScrollBar should have ScrollDownButton");
}

#[test]
fn test_scrollbar_has_thumb_texture() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollBarThumb", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    let has_thumb: bool = env
        .eval("return TestScrollBarThumb.ScrollBar.ThumbTexture ~= nil")
        .unwrap();
    assert!(has_thumb, "ScrollBar should have ThumbTexture");
}

// ============================================================================
// ListScrollFrameTemplate Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_scrollbar_track_textures() {
    let env = env_with_shared_xml();

    // ListScrollFrameTemplate (inherits FauxScrollFrameTemplate) adds track textures
    // Note: FauxScrollFrameTemplate itself does NOT have track textures
    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollBarTrack", UIParent, "ListScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    let has_top: bool = env
        .eval("return TestScrollBarTrack.ScrollBarTop ~= nil")
        .unwrap();
    let has_bot: bool = env
        .eval("return TestScrollBarTrack.ScrollBarBottom ~= nil")
        .unwrap();

    assert!(has_top, "ListScrollFrame should have ScrollBarTop texture");
    assert!(has_bot, "ListScrollFrame should have ScrollBarBottom texture");
}

// ============================================================================
// Basic Slider Tests
// ============================================================================

#[test]
fn test_slider_basic() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local slider = CreateFrame("Slider", "TestSliderBasic", UIParent)
        slider:SetSize(200, 20)
        slider:SetPoint("CENTER")
        slider:SetMinMaxValues(0, 100)
        slider:SetValue(50)
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return TestSliderBasic:GetObjectType()").unwrap();
    let min_val: f32 = env.eval("return select(1, TestSliderBasic:GetMinMaxValues())").unwrap();
    let max_val: f32 = env.eval("return select(2, TestSliderBasic:GetMinMaxValues())").unwrap();

    assert_eq!(obj_type, "Slider");
    assert_eq!(min_val, 0.0);
    assert_eq!(max_val, 100.0);
}

#[test]
fn test_slider_has_fontstrings() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local slider = CreateFrame("Slider", "TestSliderFontStrings", UIParent)
        slider:SetSize(200, 20)
    "#,
    )
    .unwrap();

    let has_low: bool = env.eval("return TestSliderFontStrings.Low ~= nil").unwrap();
    let has_high: bool = env.eval("return TestSliderFontStrings.High ~= nil").unwrap();
    let has_text: bool = env.eval("return TestSliderFontStrings.Text ~= nil").unwrap();

    assert!(has_low, "Slider should have Low FontString");
    assert!(has_high, "Slider should have High FontString");
    assert!(has_text, "Slider should have Text FontString");
}

// ============================================================================
// HybridScrollBarTemplate Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_hybrid_scroll_template() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local hsb = CreateFrame("Slider", "TestHybridScrollBar", UIParent, "HybridScrollBarTemplate")
        hsb:SetSize(16, 200)
    "#,
    )
    .unwrap();

    // Should have track textures
    let has_thumb: bool = env.eval("return TestHybridScrollBar.ThumbTexture ~= nil").unwrap();
    let has_top: bool = env.eval("return TestHybridScrollBar.ScrollBarTop ~= nil").unwrap();
    let has_mid: bool = env.eval("return TestHybridScrollBar.ScrollBarMiddle ~= nil").unwrap();
    let has_bot: bool = env.eval("return TestHybridScrollBar.ScrollBarBottom ~= nil").unwrap();

    assert!(has_thumb, "HybridScrollBar should have ThumbTexture");
    assert!(has_top, "HybridScrollBar should have ScrollBarTop");
    assert!(has_mid, "HybridScrollBar should have ScrollBarMiddle");
    assert!(has_bot, "HybridScrollBar should have ScrollBarBottom");

    // Should have scroll buttons
    let has_up: bool = env
        .eval("return TestHybridScrollBar.ScrollUpButton ~= nil")
        .unwrap();
    let has_down: bool = env
        .eval("return TestHybridScrollBar.ScrollDownButton ~= nil")
        .unwrap();

    assert!(has_up, "HybridScrollBar should have ScrollUpButton");
    assert!(has_down, "HybridScrollBar should have ScrollDownButton");
}
