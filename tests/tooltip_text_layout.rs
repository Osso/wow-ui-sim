#![cfg(feature = "gui")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::widget::AnchorPoint;

fn update_tooltip_sizes(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_sys);
}

fn update_tooltip_layout(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
}

#[test]
fn test_invalid_anchor_type_warns_and_defaults() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "BadAnchorOwner", UIParent)
        GameTooltip:SetOwner(owner, "INVALID_ANCHOR")
    "#,
    )
    .unwrap();

    let anchor: String = env.eval("return GameTooltip:GetAnchorType()").unwrap();
    assert_eq!(anchor, "ANCHOR_LEFT");

    let state = env.state().borrow();
    let has_warning = state
        .lua_errors
        .iter()
        .any(|e| e.contains("invalid anchor type") && e.contains("INVALID_ANCHOR"));
    assert!(has_warning, "Should warn about invalid anchor type");
}

#[test]
fn test_valid_anchor_type_no_warning() {
    let env = WowLuaEnv::new().unwrap();

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "GoodAnchorOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
    "#,
    )
    .unwrap();

    let anchor: String = env.eval("return GameTooltip:GetAnchorType()").unwrap();
    assert_eq!(anchor, "ANCHOR_RIGHT");

    let state = env.state().borrow();
    let has_anchor_warning = state
        .lua_errors
        .iter()
        .any(|e| e.contains("invalid anchor type"));
    assert!(
        !has_anchor_warning,
        "Valid anchor should not produce a warning"
    );
}

#[test]
fn test_anchor_cursor_custom_offsets() {
    let env = WowLuaEnv::new().unwrap();

    env.state().borrow_mut().mouse_position = Some((100.0, 200.0));
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "CursorOffOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_CURSOR", 10, 30)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(frame.anchors.len(), 1);
    let anchor = &frame.anchors[0];
    assert!((anchor.x_offset - 110.0).abs() < 0.1);
    assert!((anchor.y_offset - 230.0).abs() < 0.1);
}

#[test]
fn test_anchor_cursor_default_offset_when_none_specified() {
    let env = WowLuaEnv::new().unwrap();

    env.state().borrow_mut().mouse_position = Some((50.0, 100.0));
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "CursorDefOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_CURSOR")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    let anchor = &frame.anchors[0];
    assert!((anchor.x_offset - 50.0).abs() < 0.1);
    assert!((anchor.y_offset - 120.0).abs() < 0.1);
}

#[test]
fn test_non_cursor_anchor_uses_offsets() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "OffsetOwner", UIParent)
        owner:SetSize(100, 30)
        owner:SetPoint("CENTER")
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT", 5, 10)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::Left);
    assert_eq!(anchor.relative_point, AnchorPoint::Right);
    assert!((anchor.x_offset - 5.0).abs() < 0.1);
    assert!((anchor.y_offset - 10.0).abs() < 0.1);
}

#[test]
fn test_tooltip_layout_is_clamped_to_viewport_edges() {
    let env = WowLuaEnv::new().unwrap();

    {
        let mut state = env.state().borrow_mut();
        state.screen_width = 400.0;
        state.screen_height = 300.0;
        state.widgets.clear_all_layout_rects();
    }

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "ClampOwner", UIParent)
        owner:SetSize(10, 10)
        owner:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 390, 290)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT")
        GameTooltip:AddLine("A tooltip line wide enough to overflow the viewport")
        GameTooltip:AddLine("Second line to ensure some height")
    "#,
    )
    .unwrap();

    update_tooltip_sizes(&env);
    update_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let rect = state
        .widgets
        .get(gt_id)
        .and_then(|frame| frame.layout_rect)
        .unwrap();

    assert!(
        rect.x >= 0.0,
        "Tooltip left edge is outside viewport: rect={rect:?}, screen={}x{}",
        state.screen_width,
        state.screen_height
    );
    assert!(
        rect.y >= 0.0,
        "Tooltip top edge is outside viewport: rect={rect:?}, screen={}x{}",
        state.screen_width,
        state.screen_height
    );
    assert!(
        rect.x + rect.width <= state.screen_width + 0.1,
        "Tooltip right edge is outside viewport: rect={rect:?}, screen={}x{}, right={}",
        state.screen_width,
        state.screen_height,
        rect.x + rect.width
    );
    assert!(
        rect.y + rect.height <= state.screen_height + 0.1,
        "Tooltip bottom edge is outside viewport: rect={rect:?}, screen={}x{}, bottom={}",
        state.screen_width,
        state.screen_height,
        rect.y + rect.height
    );
}

#[test]
fn test_tooltip_layout_is_clamped_to_top_left_viewport_edge() {
    let env = WowLuaEnv::new().unwrap();

    {
        let mut state = env.state().borrow_mut();
        state.screen_width = 400.0;
        state.screen_height = 300.0;
        state.widgets.clear_all_layout_rects();
    }

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "ClampTopLeftOwner", UIParent)
        owner:SetSize(10, 10)
        owner:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 5, 5)
        GameTooltip:SetOwner(owner, "ANCHOR_TOPLEFT", -20, -15)
        GameTooltip:AddLine("A tooltip line wide enough to overflow the left edge")
        GameTooltip:AddLine("Second line to ensure some height")
    "#,
    )
    .unwrap();

    update_tooltip_sizes(&env);
    update_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let rect = state
        .widgets
        .get(gt_id)
        .and_then(|frame| frame.layout_rect)
        .unwrap();

    assert!(rect.x >= 0.0, "Tooltip should clamp inside the left edge");
    assert!(rect.y >= 0.0, "Tooltip should clamp inside the top edge");
}
