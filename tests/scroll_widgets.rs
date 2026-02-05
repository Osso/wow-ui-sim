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
// Button Texture Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_scroll_button_has_textures() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollBtnTex", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    // Check that ScrollUpButton has its textures from UIPanelScrollUpButtonTemplate
    let has_normal: bool = env
        .eval("return TestScrollBtnTex.ScrollBar.ScrollUpButton.Normal ~= nil")
        .unwrap();
    let has_pushed: bool = env
        .eval("return TestScrollBtnTex.ScrollBar.ScrollUpButton.Pushed ~= nil")
        .unwrap();
    let has_disabled: bool = env
        .eval("return TestScrollBtnTex.ScrollBar.ScrollUpButton.Disabled ~= nil")
        .unwrap();
    let has_highlight: bool = env
        .eval("return TestScrollBtnTex.ScrollBar.ScrollUpButton.Highlight ~= nil")
        .unwrap();

    assert!(has_normal, "ScrollUpButton should have Normal texture");
    assert!(has_pushed, "ScrollUpButton should have Pushed texture");
    assert!(has_disabled, "ScrollUpButton should have Disabled texture");
    assert!(has_highlight, "ScrollUpButton should have Highlight texture");
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

// ============================================================================
// TextureKitConstants Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_texture_kit_constants_defined() {
    let env = env_with_shared_xml();

    // TextureKitConstants is defined in TextureUtil.lua (SharedXMLBase).
    // It only loads if Constants.LFG_ROLEConstants is available.
    let defined: bool = env
        .eval("return type(TextureKitConstants) == 'table'")
        .unwrap();
    assert!(defined, "TextureKitConstants should be defined after loading SharedXML");

    let use_atlas_size: bool = env
        .eval("return TextureKitConstants.UseAtlasSize == true")
        .unwrap();
    assert!(use_atlas_size, "TextureKitConstants.UseAtlasSize should be true");
}

// ============================================================================
// WowScrollBoxList Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_scrollboxlist_creates_child_frames() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("Frame", "TestScrollBoxList", UIParent, "WowScrollBoxList")
        sb:SetSize(300, 400)
        sb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    // ScrollBoxBaseTemplate creates DragDelegate, ScrollTarget, and Shadows children
    let has_scroll_target: bool = env
        .eval("return TestScrollBoxList.ScrollTarget ~= nil")
        .unwrap();
    assert!(
        has_scroll_target,
        "WowScrollBoxList should have ScrollTarget child"
    );

    let has_shadows: bool = env
        .eval("return TestScrollBoxList.Shadows ~= nil")
        .unwrap();
    assert!(
        has_shadows,
        "WowScrollBoxList should have Shadows child"
    );

    let has_drag_delegate: bool = env
        .eval("return TestScrollBoxList.DragDelegate ~= nil")
        .unwrap();
    assert!(
        has_drag_delegate,
        "WowScrollBoxList should have DragDelegate child"
    );
}

#[test]
fn test_scrollboxlist_shadows_have_textures() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("Frame", "TestScrollBoxShadows", UIParent, "WowScrollBoxList")
        sb:SetSize(300, 400)
        sb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    // Shadows frame should have Lower and Upper texture children
    let has_lower: bool = env
        .eval("return TestScrollBoxShadows.Shadows.Lower ~= nil")
        .unwrap();
    let has_upper: bool = env
        .eval("return TestScrollBoxShadows.Shadows.Upper ~= nil")
        .unwrap();

    assert!(has_lower, "Shadows should have Lower texture");
    assert!(has_upper, "Shadows should have Upper texture");
}

#[test]
fn test_scrollboxlist_keyvalues() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("Frame", "TestScrollBoxKV", UIParent, "WowScrollBoxList")
        sb:SetSize(300, 400)
    "#,
    )
    .unwrap();

    // ScrollBoxBaseTemplate sets canInterpolateScroll = false
    let can_interpolate: bool = env
        .eval("return TestScrollBoxKV.canInterpolateScroll == false")
        .unwrap();
    assert!(
        can_interpolate,
        "canInterpolateScroll should be false from template KeyValues"
    );
}

#[test]
fn test_scrollboxlist_mixin_methods() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("Frame", "TestScrollBoxMixin", UIParent, "WowScrollBoxList")
        sb:SetSize(300, 400)
    "#,
    )
    .unwrap();

    // ScrollBoxBaseMixin provides GetScrollTarget
    let has_get_scroll_target: bool = env
        .eval("return type(TestScrollBoxMixin.GetScrollTarget) == 'function'")
        .unwrap();
    assert!(
        has_get_scroll_target,
        "WowScrollBoxList should have GetScrollTarget from ScrollBoxBaseMixin"
    );

    // GetScrollTarget should return the ScrollTarget child
    let target_matches: bool = env
        .eval("return TestScrollBoxMixin:GetScrollTarget() == TestScrollBoxMixin.ScrollTarget")
        .unwrap();
    assert!(
        target_matches,
        "GetScrollTarget() should return the ScrollTarget child frame"
    );
}

#[test]
fn test_scrollboxlist_rust_children_keys() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("Frame", "TestScrollBoxRust", UIParent, "WowScrollBoxList")
        sb:SetSize(300, 400)
        sb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    // Verify children_keys are synced to Rust side
    let state = env.state().borrow();
    let registry = &state.widgets;

    let sb_id = registry.get_id_by_name("TestScrollBoxRust");
    assert!(sb_id.is_some(), "TestScrollBoxRust should exist in registry");
    let sb_id = sb_id.unwrap();

    let sb = registry.get(sb_id).unwrap();
    assert!(
        sb.children_keys.contains_key("ScrollTarget"),
        "Rust children_keys should have ScrollTarget"
    );
    assert!(
        sb.children_keys.contains_key("Shadows"),
        "Rust children_keys should have Shadows"
    );
    assert!(
        sb.children_keys.contains_key("DragDelegate"),
        "Rust children_keys should have DragDelegate"
    );
}

// ============================================================================
// MinimalScrollBar Structure Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_minimal_scrollbar_child_structure() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinSBStructure", UIParent, "MinimalScrollBar")
        sb:SetSize(8, 200)
    "#,
    )
    .unwrap();

    // Track frame
    let has_track: bool = env
        .eval("return TestMinSBStructure.Track ~= nil")
        .unwrap();
    assert!(has_track, "MinimalScrollBar should have Track child");

    // Track.Thumb (EventButton)
    let has_thumb: bool = env
        .eval("return TestMinSBStructure.Track.Thumb ~= nil")
        .unwrap();
    assert!(has_thumb, "Track should have Thumb child");

    // Back and Forward stepper buttons
    let has_back: bool = env
        .eval("return TestMinSBStructure.Back ~= nil")
        .unwrap();
    let has_forward: bool = env
        .eval("return TestMinSBStructure.Forward ~= nil")
        .unwrap();
    assert!(has_back, "MinimalScrollBar should have Back stepper");
    assert!(has_forward, "MinimalScrollBar should have Forward stepper");
}

#[test]
fn test_minimal_scrollbar_track_textures() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinSBTrackTex", UIParent, "MinimalScrollBar")
        sb:SetSize(8, 200)
    "#,
    )
    .unwrap();

    // Track has Begin/Middle/End atlas textures
    let has_begin: bool = env
        .eval("return TestMinSBTrackTex.Track.Begin ~= nil")
        .unwrap();
    let has_middle: bool = env
        .eval("return TestMinSBTrackTex.Track.Middle ~= nil")
        .unwrap();
    let has_end: bool = env
        .eval("return TestMinSBTrackTex.Track.End ~= nil")
        .unwrap();

    assert!(has_begin, "Track should have Begin texture");
    assert!(has_middle, "Track should have Middle texture");
    assert!(has_end, "Track should have End texture");

    // Verify atlas names
    let begin_atlas: String = env
        .eval("return TestMinSBTrackTex.Track.Begin:GetAtlas() or ''")
        .unwrap();
    assert_eq!(
        begin_atlas, "minimal-scrollbar-track-top",
        "Track.Begin atlas"
    );

    let end_atlas: String = env
        .eval("return TestMinSBTrackTex.Track.End:GetAtlas() or ''")
        .unwrap();
    assert_eq!(
        end_atlas, "minimal-scrollbar-track-bottom",
        "Track.End atlas"
    );
}

#[test]
fn test_minimal_scrollbar_thumb_textures() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinSBThumbTex", UIParent, "MinimalScrollBar")
        sb:SetSize(8, 200)
    "#,
    )
    .unwrap();

    // Thumb has Begin/Middle/End atlas textures
    let has_begin: bool = env
        .eval("return TestMinSBThumbTex.Track.Thumb.Begin ~= nil")
        .unwrap();
    let has_middle: bool = env
        .eval("return TestMinSBThumbTex.Track.Thumb.Middle ~= nil")
        .unwrap();
    let has_end: bool = env
        .eval("return TestMinSBThumbTex.Track.Thumb.End ~= nil")
        .unwrap();

    assert!(has_begin, "Thumb should have Begin texture");
    assert!(has_middle, "Thumb should have Middle texture");
    assert!(has_end, "Thumb should have End texture");

    let begin_atlas: String = env
        .eval("return TestMinSBThumbTex.Track.Thumb.Begin:GetAtlas() or ''")
        .unwrap();
    assert_eq!(
        begin_atlas, "minimal-scrollbar-small-thumb-top",
        "Thumb.Begin atlas"
    );
}

#[test]
fn test_minimal_scrollbar_keyvalues() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinSBKeyValues", UIParent, "MinimalScrollBar")
        sb:SetSize(8, 200)
    "#,
    )
    .unwrap();

    // MinimalScrollBar sets thumbAnchor="TOP" and minThumbExtent=23
    let thumb_anchor: String = env
        .eval("return TestMinSBKeyValues.thumbAnchor or ''")
        .unwrap();
    assert_eq!(thumb_anchor, "TOP", "thumbAnchor should be TOP");

    let min_thumb: f64 = env
        .eval("return TestMinSBKeyValues.minThumbExtent or 0")
        .unwrap();
    assert_eq!(min_thumb, 23.0, "minThumbExtent should be 23");
}

#[test]
fn test_minimal_scrollbar_mixin_accessors() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinSBAccessors", UIParent, "MinimalScrollBar")
        sb:SetSize(8, 200)
    "#,
    )
    .unwrap();

    // ScrollBarMixin accessor methods should work
    let track_matches: bool = env
        .eval("return TestMinSBAccessors:GetTrack() == TestMinSBAccessors.Track")
        .unwrap();
    assert!(track_matches, "GetTrack() should return Track child");

    let thumb_matches: bool = env
        .eval("return TestMinSBAccessors:GetThumb() == TestMinSBAccessors.Track.Thumb")
        .unwrap();
    assert!(thumb_matches, "GetThumb() should return Track.Thumb");

    let back_matches: bool = env
        .eval("return TestMinSBAccessors:GetBackStepper() == TestMinSBAccessors.Back")
        .unwrap();
    assert!(back_matches, "GetBackStepper() should return Back");

    let forward_matches: bool = env
        .eval("return TestMinSBAccessors:GetForwardStepper() == TestMinSBAccessors.Forward")
        .unwrap();
    assert!(forward_matches, "GetForwardStepper() should return Forward");
}

#[test]
fn test_minimal_scrollbar_stepper_sizes() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinSBSizes", UIParent, "MinimalScrollBar")
        sb:SetSize(8, 200)
    "#,
    )
    .unwrap();

    // Back and Forward buttons should be 17x11 per the XML
    let back_w: f64 = env.eval("return TestMinSBSizes.Back:GetWidth()").unwrap();
    let back_h: f64 = env.eval("return TestMinSBSizes.Back:GetHeight()").unwrap();
    assert_eq!(back_w, 17.0, "Back button width");
    assert_eq!(back_h, 11.0, "Back button height");

    let fwd_w: f64 = env.eval("return TestMinSBSizes.Forward:GetWidth()").unwrap();
    let fwd_h: f64 = env.eval("return TestMinSBSizes.Forward:GetHeight()").unwrap();
    assert_eq!(fwd_w, 17.0, "Forward button width");
    assert_eq!(fwd_h, 11.0, "Forward button height");
}

#[test]
fn test_minimal_scrollbar_rust_children_keys() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinSBRust", UIParent, "MinimalScrollBar")
        sb:SetSize(8, 200)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let registry = &state.widgets;

    let sb_id = registry.get_id_by_name("TestMinSBRust");
    assert!(sb_id.is_some(), "TestMinSBRust should exist in registry");
    let sb_id = sb_id.unwrap();

    let sb = registry.get(sb_id).unwrap();
    assert!(
        sb.children_keys.contains_key("Track"),
        "Rust children_keys should have Track"
    );
    assert!(
        sb.children_keys.contains_key("Back"),
        "Rust children_keys should have Back"
    );
    assert!(
        sb.children_keys.contains_key("Forward"),
        "Rust children_keys should have Forward"
    );

    // Verify Track has Thumb in its children_keys
    let track_id = *sb.children_keys.get("Track").unwrap();
    let track = registry.get(track_id).unwrap();
    assert!(
        track.children_keys.contains_key("Thumb"),
        "Track's Rust children_keys should have Thumb"
    );
}

// ============================================================================
// MinimalScrollBar Atlas Texture Tests (requires SharedXML)
// ============================================================================

#[test]
fn test_minimal_scrollbar_atlas_textures() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sb = CreateFrame("EventFrame", "TestMinScrollBarAtlas", UIParent, "MinimalScrollBar")
        sb:SetSize(16, 200)
    "#,
    )
    .unwrap();

    // Back button should have Texture child with atlas set via OnLoad
    let back_atlas: String = env
        .eval(
            r#"
        local tex = TestMinScrollBarAtlas.Back.Texture
        return tex:GetAtlas() or ""
    "#,
        )
        .unwrap();
    assert_eq!(
        back_atlas, "minimal-scrollbar-arrow-top",
        "Back button texture should have atlas set via OnLoad"
    );

    // GetTexture should return the resolved file path from the atlas database
    let back_file: String = env
        .eval(
            r#"
        local tex = TestMinScrollBarAtlas.Back.Texture
        return tex:GetTexture() or ""
    "#,
        )
        .unwrap();
    assert!(
        back_file.contains("minimalscrollbarproportional"),
        "Back texture file should be resolved from atlas: got '{}'",
        back_file
    );

    // Forward button should also have its atlas set
    let forward_atlas: String = env
        .eval(
            r#"
        local tex = TestMinScrollBarAtlas.Forward.Texture
        return tex:GetAtlas() or ""
    "#,
        )
        .unwrap();
    assert_eq!(
        forward_atlas, "minimal-scrollbar-arrow-bottom",
        "Forward button texture should have atlas set via OnLoad"
    );
}
