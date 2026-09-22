//! Tests for basic CreateFrame functionality.
//!
//! These tests cover frame creation, parent-child relationships, strata inheritance,
//! and widget-type defaults (button textures, slider fontstrings).

#[path = "frame_creation/named_duplicates.rs"]
mod named_duplicates;
#[path = "frame_creation/set_point_overrides.rs"]
mod set_point_overrides;
#[path = "frame_creation/visibility_scripts.rs"]
mod visibility_scripts;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::WidgetType;

// ============================================================================
// Frame Level Defaults
// ============================================================================

#[test]
fn test_frame_level_default_zero() {
    let env = WowLuaEnv::new().unwrap();
    let level: i32 = env
        .eval(
            r#"
            local f = CreateFrame('Frame')
            return f:GetFrameLevel()
            "#,
        )
        .unwrap();
    assert_eq!(level, 0, "newly created frame should have level 0");
}

// ============================================================================
// SetPoint 3-arg form with Lua override (EditMode SetPointOverride pattern)
// ============================================================================

#[test]
fn test_set_point_3arg_with_fixed_override() {
    let env = WowLuaEnv::new().unwrap();
    let (x, y): (f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "TestParent3Arg")
            parent:SetSize(800, 600)
            parent:SetPoint("CENTER")
            local child = CreateFrame("Frame", "TestChild3Arg", parent)
            child:SetSize(100, 50)

            -- Simulate the FIXED SetPointOverride that detects 3-arg form
            child.SetPointBase = child.SetPoint
            child.SetPoint = function(self, point, relativeTo, relativePoint, offsetX, offsetY)
                if type(relativeTo) == "number" then
                    offsetX = relativeTo
                    offsetY = relativePoint
                    relativeTo = nil
                    relativePoint = nil
                end
                self:SetPointBase(point, relativeTo, relativePoint, offsetX, offsetY)
            end

            -- 3-arg form: SetPoint("TOPRIGHT", 10, 20) should work
            child:SetPoint("TOPRIGHT", 10, 20)
            local _, _, _, ox, oy = child:GetPoint(1)
            return ox, oy
        "#,
        )
        .unwrap();
    assert_eq!(
        (x, y),
        (10.0, 20.0),
        "3-arg SetPoint through fixed override preserves offsets"
    );
}

#[test]
fn test_set_frame_level_does_not_fix_level() {
    let env = WowLuaEnv::new().unwrap();
    let level: i32 = env
        .eval(
            r#"
            local f = CreateFrame('Frame')
            local h = CreateFrame('Frame')
            f:SetFrameLevel(5)
            f:SetParent(h)
            return f:GetFrameLevel()
            "#,
        )
        .unwrap();
    assert_eq!(
        level, 1,
        "SetFrameLevel should not prevent level inheritance on reparent"
    );
}

// ============================================================================
// Basic CreateFrame Tests
// ============================================================================

#[test]
fn test_create_frame_basic() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestBasicFrame", UIParent)
        frame:SetSize(100, 50)
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return TestBasicFrame ~= nil").unwrap();
    let width: f32 = env.eval("return TestBasicFrame:GetWidth()").unwrap();
    let height: f32 = env.eval("return TestBasicFrame:GetHeight()").unwrap();
    let obj_type: String = env.eval("return TestBasicFrame:GetObjectType()").unwrap();

    assert!(exists);
    assert_eq!(width, 100.0);
    assert_eq!(height, 50.0);
    assert_eq!(obj_type, "Frame");
}

#[test]
fn test_create_frame_types() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestFrame", UIParent)
        local button = CreateFrame("Button", "TestButton", UIParent)
        local checkbutton = CreateFrame("CheckButton", "TestCheckButton", UIParent)
        local slider = CreateFrame("Slider", "TestSlider", UIParent)
        local editbox = CreateFrame("EditBox", "TestEditBox", UIParent)
        local scrollframe = CreateFrame("ScrollFrame", "TestScrollFrame", UIParent)
        local statusbar = CreateFrame("StatusBar", "TestStatusBar", UIParent)
    "#,
    )
    .unwrap();

    let frame_type: String = env.eval("return TestFrame:GetObjectType()").unwrap();
    let button_type: String = env.eval("return TestButton:GetObjectType()").unwrap();
    let checkbutton_type: String = env.eval("return TestCheckButton:GetObjectType()").unwrap();
    let slider_type: String = env.eval("return TestSlider:GetObjectType()").unwrap();
    let editbox_type: String = env.eval("return TestEditBox:GetObjectType()").unwrap();
    let scrollframe_type: String = env.eval("return TestScrollFrame:GetObjectType()").unwrap();
    let statusbar_type: String = env.eval("return TestStatusBar:GetObjectType()").unwrap();

    assert_eq!(frame_type, "Frame");
    assert_eq!(button_type, "Button");
    assert_eq!(checkbutton_type, "CheckButton");
    assert_eq!(slider_type, "Slider");
    assert_eq!(editbox_type, "EditBox");
    assert_eq!(scrollframe_type, "ScrollFrame");
    assert_eq!(statusbar_type, "StatusBar");
}

#[test]
fn test_widget_type_from_str_preserves_alias_groups() {
    assert_eq!(WidgetType::from_str("BUTTON"), Some(WidgetType::Button));
    assert_eq!(
        WidgetType::from_str("DropdownButton"),
        Some(WidgetType::Button)
    );
    assert_eq!(
        WidgetType::from_str("ScrollingMessageFrame"),
        Some(WidgetType::MessageFrame)
    );
    assert_eq!(
        WidgetType::from_str("DressUpModel"),
        Some(WidgetType::PlayerModel)
    );
    assert_eq!(WidgetType::from_str("EventFrame"), Some(WidgetType::Frame));
    assert_eq!(WidgetType::from_str("WorldFrame"), None);
}

#[test]
fn test_model_type_hierarchy_queries() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local model = CreateFrame("Model", "TypeHierarchyModel", UIParent)
        local playerModel = CreateFrame("PlayerModel", "TypeHierarchyPlayerModel", UIParent)
        local scene = CreateFrame("ModelScene", "TypeHierarchyScene", UIParent)
    "#,
    )
    .unwrap();

    let model_is_model: bool = env
        .eval("return TypeHierarchyModel:IsObjectType('Model')")
        .unwrap();
    let player_model_is_model: bool = env
        .eval("return TypeHierarchyPlayerModel:IsObjectType('Model')")
        .unwrap();
    let player_model_is_player_model: bool = env
        .eval("return TypeHierarchyPlayerModel:IsObjectType('PlayerModel')")
        .unwrap();
    let scene_is_frame: bool = env
        .eval("return TypeHierarchyScene:IsObjectType('Frame')")
        .unwrap();
    let scene_is_model: bool = env
        .eval("return TypeHierarchyScene:IsObjectType('Model')")
        .unwrap();

    assert!(model_is_model, "Model should report Model type");
    assert!(
        player_model_is_model,
        "PlayerModel should inherit Model in IsObjectType"
    );
    assert!(
        player_model_is_player_model,
        "PlayerModel should report PlayerModel type"
    );
    assert!(scene_is_frame, "ModelScene should behave like a frame type");
    assert!(!scene_is_model, "ModelScene should not behave like Model");
}

#[test]
fn test_editbox_mouse_enabled_by_default() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"local eb = CreateFrame("EditBox", "TestEBMouse", UIParent)"#)
        .unwrap();
    let enabled: bool = env.eval("return TestEBMouse:IsMouseEnabled()").unwrap();
    assert!(enabled, "EditBox should have mouse enabled by default");
}

#[test]
fn test_create_frame_anonymous() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", nil, UIParent)
        frame:SetSize(50, 50)
        TestAnonymousFrame = frame
    "#,
    )
    .unwrap();

    let obj_type: String = env
        .eval("return TestAnonymousFrame:GetObjectType()")
        .unwrap();
    let width: f32 = env.eval("return TestAnonymousFrame:GetWidth()").unwrap();

    assert_eq!(obj_type, "Frame");
    assert_eq!(width, 50.0);
}

// ============================================================================
// Parent-Child Relationship Tests
// ============================================================================

#[test]
fn test_create_frame_with_parent() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "TestParentFrame", UIParent)
        parent:SetSize(200, 200)

        local child = CreateFrame("Frame", "TestChildFrame", parent)
        child:SetSize(50, 50)
    "#,
    )
    .unwrap();

    let parent_name: String = env
        .eval("return TestChildFrame:GetParent():GetName()")
        .unwrap();
    assert_eq!(parent_name, "TestParentFrame");

    // Verify child is registered with parent in Rust
    let state = env.state().borrow();
    let parent_id = state
        .widgets
        .get_id_by_name("TestParentFrame")
        .expect("Parent should exist");
    let child_id = state
        .widgets
        .get_id_by_name("TestChildFrame")
        .expect("Child should exist");

    let parent_frame = state.widgets.get(parent_id).unwrap();
    assert!(
        parent_frame.children.contains(&child_id),
        "Parent should have child in children list"
    );
}

#[test]
fn test_is_visible_ignores_parent_alpha_zero() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "AlphaZeroParent", UIParent)
        local child = CreateFrame("Frame", "AlphaZeroChild", parent)
        parent:SetAlpha(0)
        child:Show()
    "#,
    )
    .unwrap();

    let child_shown: bool = env.eval("return AlphaZeroChild:IsShown()").unwrap();
    let child_visible: bool = env.eval("return AlphaZeroChild:IsVisible()").unwrap();
    let child_effective_alpha: f32 = env
        .eval("return AlphaZeroChild:GetEffectiveAlpha()")
        .unwrap();

    assert!(child_shown, "child should still have its own shown flag");
    assert!(
        child_visible,
        "child should remain visible even when parent effective alpha is zero"
    );
    assert_eq!(
        child_effective_alpha, 0.0,
        "child effective alpha should collapse to zero with a transparent parent"
    );
}

#[test]
fn test_is_visible_remains_true_when_reparented_under_alpha_zero_parent() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local visibleParent = CreateFrame("Frame", "VisibleParent", UIParent)
        local transparentParent = CreateFrame("Frame", "TransparentParent", UIParent)
        transparentParent:SetAlpha(0)

        local child = CreateFrame("Frame", "ReparentedVisibilityChild", visibleParent)
        child:Show()
    "#,
    )
    .unwrap();

    let initially_visible: bool = env
        .eval("return ReparentedVisibilityChild:IsVisible()")
        .unwrap();
    assert!(
        initially_visible,
        "child should start visible under visible parent"
    );

    env.exec(r#"ReparentedVisibilityChild:SetParent(TransparentParent)"#)
        .unwrap();

    let visible_after_reparent: bool = env
        .eval("return ReparentedVisibilityChild:IsVisible()")
        .unwrap();
    let effective_alpha_after_reparent: f32 = env
        .eval("return ReparentedVisibilityChild:GetEffectiveAlpha()")
        .unwrap();

    assert!(
        visible_after_reparent,
        "child should remain visible after reparenting under alpha-zero parent"
    );
    assert_eq!(
        effective_alpha_after_reparent, 0.0,
        "reparenting should recompute child effective alpha"
    );
}

#[test]
fn test_effective_alpha_recovers_after_parent_alpha_restored() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "RestoredAlphaParent", UIParent)
        local child = CreateFrame("Frame", "RestoredAlphaChild", parent)
        parent:SetAlpha(0)
        child:Show()
        parent:SetAlpha(1)
    "#,
    )
    .unwrap();

    let child_visible: bool = env.eval("return RestoredAlphaChild:IsVisible()").unwrap();
    let child_effective_alpha: f32 = env
        .eval("return RestoredAlphaChild:GetEffectiveAlpha()")
        .unwrap();

    assert!(
        child_visible,
        "child should remain visible while parent alpha is restored"
    );
    assert_eq!(
        child_effective_alpha, 1.0,
        "child effective alpha should be recomputed when parent alpha changes"
    );
}

#[test]
fn test_create_frame_default_parent() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestDefaultParent", nil)
        frame:SetSize(50, 50)
    "#,
    )
    .unwrap();

    let parent_name: String = env
        .eval("return TestDefaultParent:GetParent():GetName()")
        .unwrap();
    assert_eq!(parent_name, "UIParent");
}

// ============================================================================
// $parent Name Substitution Tests
// ============================================================================

#[test]
fn test_create_frame_parent_substitution() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "MyAddonFrame", UIParent)
        local child = CreateFrame("Frame", "$parentChild", parent)
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return MyAddonFrameChild ~= nil").unwrap();
    let child_name: String = env.eval("return MyAddonFrameChild:GetName()").unwrap();

    assert!(exists, "Frame with substituted name should exist");
    assert_eq!(child_name, "MyAddonFrameChild");
}

#[test]
fn test_create_frame_parent_case_insensitive() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "ParentFrame", UIParent)
        local child1 = CreateFrame("Frame", "$parentButton", parent)
        local child2 = CreateFrame("Frame", "$ParentText", parent)
    "#,
    )
    .unwrap();

    let exists1: bool = env.eval("return ParentFrameButton ~= nil").unwrap();
    let exists2: bool = env.eval("return ParentFrameText ~= nil").unwrap();

    assert!(exists1, "$parent substitution should work");
    assert!(exists2, "$Parent substitution should work");
}

// ============================================================================
// Strata and Level Inheritance Tests
// ============================================================================

#[test]
fn test_create_frame_strata_inheritance() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "HighStrataParent", UIParent)
        parent:SetFrameStrata("DIALOG")
        parent:SetFrameLevel(10)

        local child = CreateFrame("Frame", "HighStrataChild", parent)
    "#,
    )
    .unwrap();

    let child_strata: String = env.eval("return HighStrataChild:GetFrameStrata()").unwrap();
    let child_level: i32 = env.eval("return HighStrataChild:GetFrameLevel()").unwrap();

    assert_eq!(
        child_strata, "DIALOG",
        "Child should inherit parent's strata"
    );
    assert_eq!(child_level, 11, "Child level should be parent level + 1");
}

// ============================================================================
// Button Child Element Tests
// ============================================================================

#[test]
fn test_create_button_no_default_textures() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestButtonTextures", UIParent)
        btn:SetSize(100, 30)
    "#,
    )
    .unwrap();

    let has_normal: bool = env
        .eval("return TestButtonTextures:GetNormalTexture() ~= nil")
        .unwrap();
    let has_pushed: bool = env
        .eval("return TestButtonTextures:GetPushedTexture() ~= nil")
        .unwrap();
    let has_highlight: bool = env
        .eval("return TestButtonTextures:GetHighlightTexture() ~= nil")
        .unwrap();

    assert!(!has_normal, "Fresh button should not have NormalTexture");
    assert!(!has_pushed, "Fresh button should not have PushedTexture");
    assert!(
        !has_highlight,
        "Fresh button should not have HighlightTexture"
    );
}

#[test]
fn test_create_button_has_text_fontstring() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestButtonText", UIParent)
        btn:SetText("Click Me")
    "#,
    )
    .unwrap();

    let has_text: bool = env.eval("return TestButtonText.Text ~= nil").unwrap();
    let text_content: String = env.eval("return TestButtonText:GetText()").unwrap();

    assert!(has_text, "Button should have Text FontString");
    assert_eq!(text_content, "Click Me");
}

// ============================================================================
// Slider Child Element Tests
// ============================================================================

#[test]
fn test_create_slider_has_children() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local slider = CreateFrame("Slider", "TestSliderChildren", UIParent)
        slider:SetSize(200, 20)
    "#,
    )
    .unwrap();

    let has_low: bool = env.eval("return TestSliderChildren.Low ~= nil").unwrap();
    let has_high: bool = env.eval("return TestSliderChildren.High ~= nil").unwrap();
    let has_text: bool = env.eval("return TestSliderChildren.Text ~= nil").unwrap();
    let has_thumb: bool = env
        .eval("return TestSliderChildren.ThumbTexture ~= nil")
        .unwrap();

    assert!(has_low, "Slider should have Low fontstring");
    assert!(has_high, "Slider should have High fontstring");
    assert!(has_text, "Slider should have Text fontstring");
    assert!(has_thumb, "Slider should have ThumbTexture");
}

#[test]
fn test_create_slider_default_fontstrings_are_anchored() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local slider = CreateFrame("Slider", "TestSliderAnchoredChildren", UIParent)
        slider:SetSize(200, 20)
    "#,
    )
    .unwrap();

    let anchored: bool = env
        .eval(
            r#"
            local lowPoint, _, lowRelative = TestSliderAnchoredChildren.Low:GetPoint()
            local highPoint, _, highRelative = TestSliderAnchoredChildren.High:GetPoint()
            local textPoint, _, textRelative = TestSliderAnchoredChildren.Text:GetPoint()
            return lowPoint == "TOPLEFT" and lowRelative == "BOTTOMLEFT"
               and highPoint == "TOPRIGHT" and highRelative == "BOTTOMRIGHT"
               and textPoint == "BOTTOM" and textRelative == "TOP"
            "#,
        )
        .unwrap();

    assert!(anchored, "Slider label fontstrings should have points");
}

// ============================================================================
// CreateTexture and CreateFontString Tests
// ============================================================================

#[test]
fn test_create_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestTextureFrame", UIParent)
        frame:SetSize(100, 100)

        local tex = frame:CreateTexture("TestTexture", "BACKGROUND")
        tex:SetAllPoints()
        tex:SetColorTexture(1, 0, 0, 1)
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return TestTexture ~= nil").unwrap();
    let obj_type: String = env.eval("return TestTexture:GetObjectType()").unwrap();
    let parent: String = env
        .eval("return TestTexture:GetParent():GetName()")
        .unwrap();

    assert!(exists);
    assert_eq!(obj_type, "Texture");
    assert_eq!(parent, "TestTextureFrame");
}

#[test]
fn test_create_fontstring() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestFontFrame", UIParent)
        frame:SetSize(200, 50)

        local fs = frame:CreateFontString("TestFS", "OVERLAY")
        fs:SetPoint("CENTER")
        fs:SetText("Hello World")
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return TestFS ~= nil").unwrap();
    let obj_type: String = env.eval("return TestFS:GetObjectType()").unwrap();
    let text: String = env.eval("return TestFS:GetText()").unwrap();

    assert!(exists);
    assert_eq!(obj_type, "FontString");
    assert_eq!(text, "Hello World");
}

// ============================================================================
// Integration: Addon-style frame creation
// ============================================================================

#[test]
fn test_addon_style_frame_creation() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local AddonFrame = CreateFrame("Frame", "MyAddon", UIParent)
        AddonFrame:SetSize(400, 300)
        AddonFrame:SetPoint("CENTER")
        AddonFrame:SetFrameStrata("HIGH")

        local TitleBar = CreateFrame("Frame", "$parentTitleBar", AddonFrame)
        TitleBar:SetSize(400, 30)
        TitleBar:SetPoint("TOP")

        local Title = TitleBar:CreateFontString("$parentTitle", "OVERLAY")
        Title:SetPoint("CENTER")
        Title:SetText("My Addon")

        local CloseBtn = CreateFrame("Button", "$parentCloseButton", TitleBar)
        CloseBtn:SetSize(24, 24)
        CloseBtn:SetPoint("RIGHT", -5, 0)

        local Content = CreateFrame("ScrollFrame", "$parentContent", AddonFrame)
        Content:SetSize(380, 250)
        Content:SetPoint("BOTTOM", 0, 10)
    "#,
    )
    .unwrap();

    let main_exists: bool = env.eval("return MyAddon ~= nil").unwrap();
    let titlebar_exists: bool = env.eval("return MyAddonTitleBar ~= nil").unwrap();
    let title_exists: bool = env.eval("return MyAddonTitleBarTitle ~= nil").unwrap();
    let close_exists: bool = env
        .eval("return MyAddonTitleBarCloseButton ~= nil")
        .unwrap();
    let content_exists: bool = env.eval("return MyAddonContent ~= nil").unwrap();

    assert!(main_exists);
    assert!(titlebar_exists);
    assert!(title_exists);
    assert!(close_exists);
    assert!(content_exists);

    let titlebar_parent: String = env
        .eval("return MyAddonTitleBar:GetParent():GetName()")
        .unwrap();
    let title_parent: String = env
        .eval("return MyAddonTitleBarTitle:GetParent():GetName()")
        .unwrap();
    let close_parent: String = env
        .eval("return MyAddonTitleBarCloseButton:GetParent():GetName()")
        .unwrap();

    assert_eq!(titlebar_parent, "MyAddon");
    assert_eq!(title_parent, "MyAddonTitleBar");
    assert_eq!(close_parent, "MyAddonTitleBar");

    let titlebar_strata: String = env.eval("return MyAddonTitleBar:GetFrameStrata()").unwrap();
    assert_eq!(titlebar_strata, "HIGH");
}

// ============================================================================
// CreateFrame with frame in name position
// ============================================================================

#[test]
fn test_create_frame_with_frame_in_name_position() {
    let env = WowLuaEnv::new().unwrap();
    let (name_nil, parent_nil): (bool, bool) = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        local g = CreateFrame("Frame", f)
        return g:GetName() == nil, g:GetParent() == nil
    "#,
        )
        .unwrap();
    assert!(
        name_nil,
        "CreateFrame with frame as name should have nil name"
    );
    assert!(
        parent_nil,
        "CreateFrame with frame as name should have nil parent"
    );
}
