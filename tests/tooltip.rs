//! Tests for GameTooltip implementation.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::AnchorPoint;

#[test]
fn test_gametooltip_exists_and_has_correct_type() {
    let env = WowLuaEnv::new().unwrap();

    let exists: bool = env.eval("return GameTooltip ~= nil").unwrap();
    assert!(exists);

    let obj_type: String = env.eval("return GameTooltip:GetObjectType()").unwrap();
    assert_eq!(obj_type, "GameTooltip");
}

#[test]
fn test_gametooltip_strata_is_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    let strata: String = env.eval("return GameTooltip:GetFrameStrata()").unwrap();
    assert_eq!(strata, "TOOLTIP");
}

#[test]
fn test_addline_and_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("First line")
        GameTooltip:AddLine("Second line", 1, 0, 0)
        GameTooltip:AddLine("Third line", 0, 1, 0, true)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 3);
}

#[test]
fn test_adddoubleline_and_numlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddDoubleLine("Left", "Right")
        GameTooltip:AddDoubleLine("Name", "Value", 1, 1, 1, 0.5, 0.5, 0.5)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_clearlines_resets_count() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Line 1")
        GameTooltip:AddLine("Line 2")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_settext_clears_and_sets_first_line() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Old line 1")
        GameTooltip:AddLine("Old line 2")
        GameTooltip:SetText("New text")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 1, "SetText should clear existing lines and add one");
}

#[test]
fn test_setowner_and_isowned_and_getowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "TooltipOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let is_owned: bool = env
        .eval("return GameTooltip:IsOwned(TooltipOwner)")
        .unwrap();
    assert!(is_owned, "GameTooltip should be owned by TooltipOwner");

    let owner_name: String = env
        .eval("return GameTooltip:GetOwner():GetName()")
        .unwrap();
    assert_eq!(owner_name, "TooltipOwner");

    // Check that non-owner returns false
    env.exec(r#"local other = CreateFrame("Frame", "OtherFrame", UIParent)"#)
        .unwrap();
    let not_owned: bool = env
        .eval("return GameTooltip:IsOwned(OtherFrame)")
        .unwrap();
    assert!(!not_owned, "GameTooltip should not be owned by OtherFrame");
}

#[test]
fn test_getanchortype_after_setowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_BOTTOMRIGHT")
    "#,
    )
    .unwrap();

    let anchor: String = env.eval("return GameTooltip:GetAnchorType()").unwrap();
    assert_eq!(anchor, "ANCHOR_BOTTOMRIGHT");
}

#[test]
fn test_on_tooltip_cleared_fires_on_clearlines() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.tooltip_cleared_count = 0
        GameTooltip:SetScript("OnTooltipCleared", function()
            _G.tooltip_cleared_count = _G.tooltip_cleared_count + 1
        end)
        GameTooltip:AddLine("Some line")
        GameTooltip:ClearLines()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return _G.tooltip_cleared_count").unwrap();
    assert_eq!(count, 1, "OnTooltipCleared should fire once on ClearLines");
}

#[test]
fn test_on_tooltip_cleared_fires_on_setowner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.cleared_count = 0
        GameTooltip:SetScript("OnTooltipCleared", function()
            _G.cleared_count = _G.cleared_count + 1
        end)
        local owner = CreateFrame("Frame", "ClearedTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return _G.cleared_count").unwrap();
    assert_eq!(count, 1, "OnTooltipCleared should fire on SetOwner");
}

#[test]
fn test_isobjecttype_frame_returns_true_for_gametooltip() {
    let env = WowLuaEnv::new().unwrap();

    let is_frame: bool = env
        .eval("return GameTooltip:IsObjectType('Frame')")
        .unwrap();
    assert!(is_frame, "GameTooltip:IsObjectType('Frame') should return true");

    let is_region: bool = env
        .eval("return GameTooltip:IsObjectType('Region')")
        .unwrap();
    assert!(is_region, "GameTooltip:IsObjectType('Region') should return true");

    let is_tooltip: bool = env
        .eval("return GameTooltip:IsObjectType('GameTooltip')")
        .unwrap();
    assert!(is_tooltip, "GameTooltip:IsObjectType('GameTooltip') should return true");

    let is_button: bool = env
        .eval("return GameTooltip:IsObjectType('Button')")
        .unwrap();
    assert!(!is_button, "GameTooltip:IsObjectType('Button') should return false");
}

#[test]
fn test_isobjecttype_for_other_types() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TypeTestButton", UIParent)
        local cb = CreateFrame("CheckButton", "TypeTestCheckButton", UIParent)
        local frame = CreateFrame("Frame", "TypeTestFrame", UIParent)
    "#,
    )
    .unwrap();

    // Button is a Frame
    let btn_is_frame: bool = env
        .eval("return TypeTestButton:IsObjectType('Frame')")
        .unwrap();
    assert!(btn_is_frame);

    // CheckButton is a Button
    let cb_is_button: bool = env
        .eval("return TypeTestCheckButton:IsObjectType('Button')")
        .unwrap();
    assert!(cb_is_button);

    // CheckButton is a Frame
    let cb_is_frame: bool = env
        .eval("return TypeTestCheckButton:IsObjectType('Frame')")
        .unwrap();
    assert!(cb_is_frame);

    // Frame is NOT a Button
    let frame_is_button: bool = env
        .eval("return TypeTestFrame:IsObjectType('Button')")
        .unwrap();
    assert!(!frame_is_button);
}

#[test]
fn test_setminimumwidth_and_getminimumwidth() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetMinimumWidth(150)").unwrap();

    let width: f32 = env.eval("return GameTooltip:GetMinimumWidth()").unwrap();
    assert_eq!(width, 150.0);
}

#[test]
fn test_setpadding_and_getpadding() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetPadding(8)").unwrap();

    let padding: f32 = env.eval("return GameTooltip:GetPadding()").unwrap();
    assert_eq!(padding, 8.0);
}

#[test]
fn test_fadeout_hides_and_clears_owner() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "FadeOutOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
        GameTooltip:FadeOut()
    "#,
    )
    .unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(!visible, "FadeOut should hide the tooltip");

    let has_owner: bool = env
        .eval("return GameTooltip:GetOwner() ~= nil")
        .unwrap();
    assert!(!has_owner, "FadeOut should clear the owner");
}

#[test]
fn test_appendtext() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameTooltip:AddLine("Hello")
        GameTooltip:AppendText(" World")
    "#,
    )
    .unwrap();

    // Verify through tooltip data
    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines.len(), 1);
    assert_eq!(td.lines[0].left_text, "Hello World");
}

#[test]
fn test_setowner_makes_tooltip_visible() {
    let env = WowLuaEnv::new().unwrap();

    // GameTooltip starts hidden
    let initially_visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(!initially_visible, "GameTooltip should start hidden");

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "VisOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let now_visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(now_visible, "SetOwner should make tooltip visible");
}

#[test]
fn test_createframe_gametooltip_type() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local tt = CreateFrame("GameTooltip", "CustomTooltip", UIParent)
        tt:AddLine("Test")
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return CustomTooltip:GetObjectType()").unwrap();
    assert_eq!(obj_type, "GameTooltip");

    let count: i32 = env.eval("return CustomTooltip:NumLines()").unwrap();
    assert_eq!(count, 1);

    let strata: String = env.eval("return CustomTooltip:GetFrameStrata()").unwrap();
    assert_eq!(strata, "TOOLTIP");
}

#[test]
fn test_other_tooltip_frames_exist() {
    let env = WowLuaEnv::new().unwrap();

    let item_ref: bool = env.eval("return ItemRefTooltip ~= nil").unwrap();
    let shopping1: bool = env.eval("return ShoppingTooltip1 ~= nil").unwrap();
    let shopping2: bool = env.eval("return ShoppingTooltip2 ~= nil").unwrap();
    let friends: bool = env.eval("return FriendsTooltip ~= nil").unwrap();

    assert!(item_ref, "ItemRefTooltip should exist");
    assert!(shopping1, "ShoppingTooltip1 should exist");
    assert!(shopping2, "ShoppingTooltip2 should exist");
    assert!(friends, "FriendsTooltip should exist");

    // All should be GameTooltip type
    let item_type: String = env
        .eval("return ItemRefTooltip:GetObjectType()")
        .unwrap();
    assert_eq!(item_type, "GameTooltip");
}

#[test]
fn test_tooltip_anchor_right_sets_anchors() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorRightOwner", UIParent)
        owner:SetSize(100, 30)
        owner:SetPoint("CENTER")
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(frame.anchors.len(), 1, "ANCHOR_RIGHT should set one anchor");
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::TopLeft, "tooltip point should be TopLeft");
    assert_eq!(anchor.relative_point, AnchorPoint::TopRight, "owner point should be TopRight");

    let owner_id = state.widgets.get_id_by_name("AnchorRightOwner").unwrap();
    assert_eq!(anchor.relative_to_id, Some(owner_id as usize));
}

#[test]
fn test_tooltip_anchor_none_no_anchors() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorNoneOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert!(frame.anchors.is_empty(), "ANCHOR_NONE should not set anchors");
}

#[test]
fn test_tooltip_anchor_cursor_uses_absolute_position() {
    let env = WowLuaEnv::new().unwrap();

    // Set mouse position before SetOwner
    env.state().borrow_mut().mouse_position = Some((200.0, 300.0));

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorCursorOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_CURSOR")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(frame.anchors.len(), 1, "ANCHOR_CURSOR should set one anchor");
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::TopLeft);
    assert!(anchor.relative_to_id.is_none(), "ANCHOR_CURSOR should not reference owner");
    assert!((anchor.x_offset - 200.0).abs() < 0.1, "x_offset should be mouse x");
    assert!((anchor.y_offset - 320.0).abs() < 0.1, "y_offset should be mouse y + 20");
}
