//! Tests for ScrollFrame, Slider, and ScrollBar widgets.
//!
//! These tests cover scroll widgets and their templates from Blizzard_SharedXML.

use crate::common;

use common::env_with_shared_xml;
#[cfg(feature = "gui")]
use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "gui")]
fn scroll_content_quad_bounds(env: &WowLuaEnv) -> (f32, f32, f32, f32) {
    env.set_screen_size(800.0, 600.0);
    let buckets = {
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        state.get_strata_buckets().unwrap().clone()
    };
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        RegistryQuadBatchParams::new(&state.widgets, (800.0, 600.0), &buckets)
            .root_name(Some("ScrollQuadViewport")),
    );
    let solid: Vec<_> = batch
        .vertices
        .iter()
        .filter(|vertex| vertex.tex_index == -1 && vertex.color[0] > 0.9 && vertex.color[1] < 0.1)
        .collect();
    assert!(
        !solid.is_empty(),
        "content color texture should emit visible vertices"
    );
    (
        solid
            .iter()
            .map(|vertex| vertex.position[0])
            .fold(f32::INFINITY, f32::min),
        solid
            .iter()
            .map(|vertex| vertex.position[1])
            .fold(f32::INFINITY, f32::min),
        solid
            .iter()
            .map(|vertex| vertex.position[0])
            .fold(f32::NEG_INFINITY, f32::max),
        solid
            .iter()
            .map(|vertex| vertex.position[1])
            .fold(f32::NEG_INFINITY, f32::max),
    )
}

#[cfg(feature = "gui")]
#[test]
fn nested_scroll_viewport_outside_parent_emits_no_visible_content() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        OuterScrollViewport = CreateFrame("ScrollFrame", "OuterScrollViewport", UIParent)
        OuterScrollViewport:SetSize(100, 100)
        OuterScrollViewport:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
        local outerChild = CreateFrame("Frame", nil, OuterScrollViewport)
        outerChild:SetSize(250, 250)
        outerChild:SetPoint("TOPLEFT", OuterScrollViewport, "TOPLEFT")
        OuterScrollViewport:SetScrollChild(outerChild)
        local inner = CreateFrame("ScrollFrame", nil, outerChild)
        inner:SetSize(30, 30)
        inner:SetPoint("TOPLEFT", outerChild, "TOPLEFT", 70, -70)
        local innerChild = CreateFrame("Frame", nil, inner)
        innerChild:SetSize(30, 30)
        innerChild:SetPoint("TOPLEFT", inner, "TOPLEFT")
        inner:SetScrollChild(innerChild)
        local texture = innerChild:CreateTexture(nil, "ARTWORK")
        texture:SetAllPoints()
        texture:SetColorTexture(0, 0, 1, 1)
    "#,
    )
    .unwrap();
    let blue_vertex_count = || {
        env.set_screen_size(800.0, 600.0);
        let buckets = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            state.get_strata_buckets().unwrap().clone()
        };
        let state = env.state().borrow();
        let batch = build_quad_batch_for_registry(
            RegistryQuadBatchParams::new(&state.widgets, (800.0, 600.0), &buckets)
                .root_name(Some("OuterScrollViewport")),
        );
        batch
            .vertices
            .iter()
            .filter(|vertex| {
                vertex.tex_index == -1 && vertex.color[2] > 0.9 && vertex.color[0] < 0.1
            })
            .count()
    };
    assert!(blue_vertex_count() > 0, "nested content starts visible");
    env.exec("OuterScrollViewport:SetHorizontalScroll(-100); OuterScrollViewport:SetVerticalScroll(-100)").unwrap();
    assert_eq!(
        blue_vertex_count(),
        0,
        "disjoint nested viewport must clip all content"
    );
}

#[cfg(feature = "gui")]
#[test]
fn scroll_offsets_translate_emitted_content_quads_on_both_axes_without_drift() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ScrollQuadViewport = CreateFrame("ScrollFrame", "ScrollQuadViewport", UIParent)
        ScrollQuadViewport:SetSize(100, 100)
        ScrollQuadViewport:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 20, -20)
        local child = CreateFrame("Frame", nil, ScrollQuadViewport)
        child:SetSize(200, 200)
        child:SetPoint("TOPLEFT", ScrollQuadViewport, "TOPLEFT")
        ScrollQuadViewport:SetScrollChild(child)
        local external = CreateFrame("Frame", nil, child)
        external:SetSize(20, 20)
        external:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 70, -70)
        local texture = external:CreateTexture(nil, "ARTWORK")
        texture:SetAllPoints()
        texture:SetColorTexture(1, 0, 0, 1)
    "#,
    )
    .unwrap();

    assert_eq!(scroll_content_quad_bounds(&env), (70.0, 70.0, 90.0, 90.0));
    env.exec(
        "ScrollQuadViewport:SetHorizontalScroll(40); ScrollQuadViewport:SetVerticalScroll(40)",
    )
    .unwrap();
    assert_eq!(scroll_content_quad_bounds(&env), (30.0, 30.0, 50.0, 50.0));
    env.exec(
        "ScrollQuadViewport:SetHorizontalScroll(40); ScrollQuadViewport:SetVerticalScroll(40)",
    )
    .unwrap();
    assert_eq!(scroll_content_quad_bounds(&env), (30.0, 30.0, 50.0, 50.0));
    env.exec("ScrollQuadViewport:SetHorizontalScroll(0); ScrollQuadViewport:SetVerticalScroll(0)")
        .unwrap();
    assert_eq!(scroll_content_quad_bounds(&env), (70.0, 70.0, 90.0, 90.0));
}

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

    let obj_type: String = env
        .eval("return TestScrollFrameBasic:GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "ScrollFrame");
}

#[test]
fn test_scrollframe_update_scroll_child_rect_uses_resolved_subtree_bounds() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollFrameRectRefresh", UIParent)
        sf:SetSize(100, 100)
        sf:SetPoint("CENTER")

        local child = CreateFrame("Frame", nil, sf)
        child:SetSize(100, 100)
        child:SetPoint("TOPLEFT", sf, "TOPLEFT", 0, 0)
        sf:SetScrollChild(child)

        local content = CreateFrame("Frame", nil, child)
        content:SetSize(180, 220)
        content:SetPoint("TOPLEFT", child, "TOPLEFT", 0, 0)
    "#,
    )
    .unwrap();

    let before: (f64, f64) = env
        .eval(
            "return TestScrollFrameRectRefresh:GetHorizontalScrollRange(), \
             TestScrollFrameRectRefresh:GetVerticalScrollRange()",
        )
        .unwrap();
    assert_eq!(before, (0.0, 0.0));

    env.exec("TestScrollFrameRectRefresh:UpdateScrollChildRect()")
        .unwrap();

    let after: (f64, f64) = env
        .eval(
            "return TestScrollFrameRectRefresh:GetHorizontalScrollRange(), \
             TestScrollFrameRectRefresh:GetVerticalScrollRange()",
        )
        .unwrap();
    assert_eq!(after, (80.0, 120.0));
}

#[test]
fn test_scrollframe_offsets_round_trip_outside_explicit_ranges_and_notify_after_commit() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollFrameOutOfRangeOffsets", UIParent)
        sf:SetSize(100, 100)
        sf:SetPoint("CENTER")
        local child = CreateFrame("Frame", nil, sf)
        child:SetSize(200, 300)
        child:SetPoint("TOPLEFT", sf, "TOPLEFT", 0, 0)
        sf:SetScrollChild(child)
        sf:UpdateScrollChildRect()

        ScrollOffsetEvents = {}
        sf:SetScript("OnHorizontalScroll", function(self, offset)
            table.insert(ScrollOffsetEvents, {"h", offset, self:GetHorizontalScroll(), self:GetVerticalScroll()})
        end)
        sf:SetScript("OnVerticalScroll", function(self, offset)
            table.insert(ScrollOffsetEvents, {"v", offset, self:GetHorizontalScroll(), self:GetVerticalScroll()})
        end)
    "#,
    )
    .unwrap();

    let ranges: (f64, f64) = env
        .eval(
            "return TestScrollFrameOutOfRangeOffsets:GetHorizontalScrollRange(), \
             TestScrollFrameOutOfRangeOffsets:GetVerticalScrollRange()",
        )
        .unwrap();
    assert_eq!(ranges, (100.0, 200.0));

    env.exec(
        r#"
        local sf = TestScrollFrameOutOfRangeOffsets
        sf:SetVerticalScroll(-50)
        sf:SetVerticalScroll(-50)
        sf:SetHorizontalScroll(999)
        sf:SetHorizontalScroll(999)
        sf:SetVerticalScroll(999)
        sf:SetVerticalScroll(999)
        sf:SetHorizontalScroll(-50)
        sf:SetHorizontalScroll(-50)
    "#,
    )
    .unwrap();

    let (horizontal, vertical, notifications): (f64, f64, String) = env
        .eval(
            r#"
            local events = {}
            for _, event in ipairs(ScrollOffsetEvents) do
                table.insert(events, table.concat(event, ":"))
            end
            local sf = TestScrollFrameOutOfRangeOffsets
            return sf:GetHorizontalScroll(), sf:GetVerticalScroll(), table.concat(events, "|")
        "#,
        )
        .unwrap();
    assert_eq!((horizontal, vertical), (-50.0, 999.0));
    assert_eq!(
        notifications,
        "v:-50:0:-50|h:999:999:-50|v:999:999:999|h:-50:-50:999"
    );
}

#[test]
fn test_scrollframe_offsets_round_trip_with_zero_cached_ranges() {
    let env = WowLuaEnv::new().unwrap();

    let (horizontal_range, vertical_range, horizontal, vertical): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local sf = CreateFrame("ScrollFrame", "TestScrollFrameZeroRangeOffsets", UIParent)
            sf:SetSize(100, 100)
            sf:SetHorizontalScroll(999)
            sf:SetVerticalScroll(-50)
            return sf:GetHorizontalScrollRange(), sf:GetVerticalScrollRange(),
                sf:GetHorizontalScroll(), sf:GetVerticalScroll()
        "#,
        )
        .unwrap();
    assert_eq!((horizontal_range, vertical_range), (0.0, 0.0));
    assert_eq!((horizontal, vertical), (999.0, -50.0));
}

#[test]
fn test_scrollframe_same_offsets_do_not_dirty_render_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollFrameNoRenderDirty", UIParent)
        sf:SetSize(100, 100)
        sf:SetHorizontalScroll(25)
        sf:SetVerticalScroll(30)
    "#,
    )
    .unwrap();

    {
        let state = env.state().borrow();
        let _ = state.widgets.take_render_dirty_with_ids();
    }

    env.exec(
        r#"
        TestScrollFrameNoRenderDirty:SetHorizontalScroll(25)
        TestScrollFrameNoRenderDirty:SetVerticalScroll(30)
    "#,
    )
    .unwrap();

    let (dirty_mask, dirty_ids) = {
        let state = env.state().borrow();
        state.widgets.take_render_dirty_with_ids()
    };

    assert_eq!(
        dirty_mask, 0,
        "same scroll offsets should not dirty render state"
    );
    assert!(
        dirty_ids.is_some_and(|ids| ids.is_empty()),
        "same scroll offsets should not enqueue dirty frame IDs"
    );
}

#[test]
fn test_scrollframe_metatable_does_not_advertise_set_max_lines() {
    let env = WowLuaEnv::new().unwrap();

    let has_set_max_lines: bool = env
        .eval(
            r#"
            local sf = CreateFrame("ScrollFrame", "TestScrollFrameSetMaxLinesLeak", UIParent)
            local mt = getmetatable(sf)
            return mt ~= nil and mt.__index ~= nil and mt.__index.SetMaxLines ~= nil
        "#,
        )
        .unwrap();

    assert!(
        !has_set_max_lines,
        "ScrollFrame metatable should not advertise SetMaxLines"
    );
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

#[test]
fn test_legacy_scrollbar_supports_scrollutil_percentage_api() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "LegacyScrollUtilFrame", UIParent, "UIPanelScrollFrameTemplate")
        sf:SetSize(200, 100)

        local child = CreateFrame("Frame", nil, sf)
        child:SetSize(200, 200)
        sf:SetScrollChild(child)

        ScrollUtil.InitScrollFrameWithScrollBar(sf, sf.ScrollBar)
        child:SetHeight(240)
        sf.ScrollBar:SetScrollPercentage(0.5, ScrollBoxConstants.NoScrollInterpolation)
        "#,
    )
    .unwrap();

    let scroll_percentage: f64 = env
        .eval("return LegacyScrollUtilFrame.ScrollBar:GetScrollPercentage()")
        .unwrap();
    assert_eq!(scroll_percentage, 0.5);
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
    assert!(
        has_bot,
        "ListScrollFrame should have ScrollBarBottom texture"
    );
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
    let min_val: f32 = env
        .eval("return select(1, TestSliderBasic:GetMinMaxValues())")
        .unwrap();
    let max_val: f32 = env
        .eval("return select(2, TestSliderBasic:GetMinMaxValues())")
        .unwrap();

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
    let has_high: bool = env
        .eval("return TestSliderFontStrings.High ~= nil")
        .unwrap();
    let has_text: bool = env
        .eval("return TestSliderFontStrings.Text ~= nil")
        .unwrap();

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
    assert!(
        has_highlight,
        "ScrollUpButton should have Highlight texture"
    );
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
    let has_thumb: bool = env
        .eval("return TestHybridScrollBar.ThumbTexture ~= nil")
        .unwrap();
    let has_global_thumb: bool = env
        .eval("return TestHybridScrollBarThumbTexture ~= nil")
        .unwrap();
    let has_top: bool = env
        .eval("return TestHybridScrollBar.ScrollBarTop ~= nil")
        .unwrap();
    let has_mid: bool = env
        .eval("return TestHybridScrollBar.ScrollBarMiddle ~= nil")
        .unwrap();
    let has_bot: bool = env
        .eval("return TestHybridScrollBar.ScrollBarBottom ~= nil")
        .unwrap();

    assert!(has_thumb, "HybridScrollBar should have ThumbTexture");
    assert!(
        has_global_thumb,
        "HybridScrollBar ThumbTexture should be published as a named global"
    );
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

#[test]
fn test_hybrid_scroll_template_applies_declared_thumb_texture() {
    let env = env_with_shared_xml();
    assert!(
        wow_ui_sim::xml::get_template_chain("HybridScrollBarTemplate")
            .iter()
            .any(|entry| entry.frame.thumb_texture().is_some()),
        "SharedXML should register HybridScrollBarTemplate's inherited ThumbTexture"
    );

    env.exec(
        r#"
        local hsb = CreateFrame("Slider", "TestHybridScrollBarThumbXml", UIParent, "HybridScrollBarTemplate")
        hsb:SetSize(16, 200)
    "#,
    )
    .unwrap();

    let applied: (bool, bool, f32, f32, String) = env
        .eval(
            r#"
        local thumb = TestHybridScrollBarThumbXml:GetThumbTexture()
        return thumb == TestHybridScrollBarThumbXmlThumbTexture,
            TestHybridScrollBarThumbXml.thumbTexture == thumb,
            thumb:GetWidth(),
            thumb:GetHeight(),
            thumb:GetTextureFilePath() or ""
    "#,
        )
        .unwrap();

    assert_eq!(
        applied,
        (
            true,
            true,
            18.0,
            24.0,
            "Interface\\Buttons\\UI-ScrollBar-Knob".to_string()
        )
    );
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
    assert!(
        defined,
        "TextureKitConstants should be defined after loading SharedXML"
    );

    let use_atlas_size: bool = env
        .eval("return TextureKitConstants.UseAtlasSize == true")
        .unwrap();
    assert!(
        use_atlas_size,
        "TextureKitConstants.UseAtlasSize should be true"
    );
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

    let has_shadows: bool = env.eval("return TestScrollBoxList.Shadows ~= nil").unwrap();
    assert!(has_shadows, "WowScrollBoxList should have Shadows child");

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
    assert!(
        sb_id.is_some(),
        "TestScrollBoxRust should exist in registry"
    );
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

#[test]
fn test_scrollboxlist_callbacks_and_foreach_frame_are_stateful() {
    let env = env_with_shared_xml();

    let result: String = env
        .eval(
            r#"
            local sb = CreateFrame("Frame", "TestScrollBoxCallbacks", UIParent, "WowScrollBoxList")
            sb:SetSize(300, 400)

            local callbackCalls = 0
            local owner = {}
            local ownerOk = false
            local argsOk = false

            sb:RegisterCallback(ScrollBoxListMixin.Event.OnScroll, function(self, scrollPercentage, visibleExtentPercentage, panExtentPercentage)
                callbackCalls = callbackCalls + 1
                ownerOk = self == owner
                argsOk = scrollPercentage == 0.25 and visibleExtentPercentage == 0.5 and panExtentPercentage == 0.75
            end, owner)

            sb:TriggerEvent(ScrollBoxListMixin.Event.OnScroll, 0.25, 0.5, 0.75)
            sb:UnregisterCallback(ScrollBoxListMixin.Event.OnScroll, owner)
            sb:TriggerEvent(ScrollBoxListMixin.Event.OnScroll, 1, 1, 1)

            local first = CreateFrame("Button", nil, sb.ScrollTarget)
            local second = CreateFrame("Button", nil, sb.ScrollTarget)
            first.GetElementData = function() return { label = "First" } end
            second.GetElementData = function() return { label = "Second" } end

            sb.view = {
                GetFrames = function(self)
                    return { first, second }
                end,
                ForEachFrame = function(self, func)
                    for _, frame in ipairs(self:GetFrames()) do
                        local result = func(frame, frame:GetElementData())
                        if result then
                            return result
                        end
                    end
                end,
            }

            local seen = {}
            local found = sb:ForEachFrame(function(frame, elementData)
                table.insert(seen, elementData.label)
                if elementData.label == "Second" then
                    return frame
                end
            end)

            return table.concat({
                tostring(callbackCalls),
                tostring(ownerOk),
                tostring(argsOk),
                table.concat(seen, ","),
                tostring(found == second),
            }, "|")
        "#,
        )
        .unwrap();

    assert_eq!(result, "1|true|true|First,Second|true");
}

#[test]
fn test_scrollboxlist_interpolate_scroll_round_trip() {
    let env = env_with_shared_xml();

    let result: String = env
        .eval(
            r#"
            local sb = CreateFrame("Frame", "TestScrollBoxInterpolate", UIParent, "WowScrollBoxList")
            sb:SetSize(300, 400)

            local initial = tostring(sb:CanInterpolateScroll())
            sb:SetInterpolateScroll(true)
            local enabled = tostring(sb:CanInterpolateScroll())
            sb:SetInterpolateScroll(false)
            local disabled = tostring(sb:CanInterpolateScroll())

            return table.concat({ initial, enabled, disabled, tostring(sb.canInterpolateScroll) }, "|")
        "#,
        )
        .unwrap();

    assert_eq!(result, "false|true|false|false");
}
