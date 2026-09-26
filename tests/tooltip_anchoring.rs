//! Tests for GameTooltip anchor/positioning behavior.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::compute_frame_rect;
use wow_ui_sim::widget::AnchorPoint;

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
    assert_eq!(
        anchor.point,
        AnchorPoint::Left,
        "tooltip point should be Left"
    );
    assert_eq!(
        anchor.relative_point,
        AnchorPoint::Right,
        "owner point should be Right"
    );

    let owner_id = state.widgets.get_id_by_name("AnchorRightOwner").unwrap();
    assert_eq!(anchor.relative_to_id, Some(owner_id as usize));
}

#[test]
fn test_tooltip_topright_anchor_aligns_right_edges() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorTopRightOwner", UIParent)
        owner:SetSize(100, 30)
        owner:SetPoint("CENTER")
        GameTooltip:SetOwner(owner, "ANCHOR_TOPRIGHT")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(
        frame.anchors.len(),
        1,
        "ANCHOR_TOPRIGHT should set one anchor"
    );
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::BottomRight);
    assert_eq!(anchor.relative_point, AnchorPoint::TopRight);
}

#[test]
fn test_tooltip_top_anchor_uses_vertical_edge_points() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorTopOwner", UIParent)
        owner:SetSize(100, 30)
        owner:SetPoint("CENTER")
        GameTooltip:SetOwner(owner, "ANCHOR_TOP")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(frame.anchors.len(), 1, "ANCHOR_TOP should set one anchor");
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::Bottom);
    assert_eq!(anchor.relative_point, AnchorPoint::Top);
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

    assert!(
        frame.anchors.is_empty(),
        "ANCHOR_NONE should not set anchors"
    );
}

#[test]
fn test_tooltip_anchor_preserve_keeps_existing_anchor_when_reowned() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner1 = CreateFrame("Frame", "AnchorPreserveOwner1", UIParent)
        local owner2 = CreateFrame("Frame", "AnchorPreserveOwner2", UIParent)
        owner1:SetSize(100, 30)
        owner2:SetSize(100, 30)
        owner1:SetPoint("CENTER")
        owner2:SetPoint("TOPLEFT")
        GameTooltip:SetOwner(owner1, "ANCHOR_RIGHT", 5, 10)
        GameTooltip:SetOwner(owner2, "ANCHOR_PRESERVE")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let owner1_id = state
        .widgets
        .get_id_by_name("AnchorPreserveOwner1")
        .unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(
        frame.anchors.len(),
        1,
        "ANCHOR_PRESERVE should keep the existing anchor"
    );
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::Left);
    assert_eq!(anchor.relative_point, AnchorPoint::Right);
    assert_eq!(
        anchor.relative_to_id,
        Some(owner1_id as usize),
        "ANCHOR_PRESERVE should keep the existing relative target"
    );
    assert!(
        (anchor.x_offset - 5.0).abs() < 0.1,
        "x_offset should stay preserved"
    );
    assert!(
        (anchor.y_offset - 10.0).abs() < 0.1,
        "y_offset should stay preserved"
    );

    let tooltip_anchor: String = env.eval("return GameTooltip:GetAnchorType()").unwrap();
    assert_eq!(tooltip_anchor, "ANCHOR_PRESERVE");
}

#[test]
fn test_tooltip_anchor_preserve_keeps_existing_position_when_reowned() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner1 = CreateFrame("Frame", "AnchorPreserveLayoutOwner1", UIParent)
        local owner2 = CreateFrame("Frame", "AnchorPreserveLayoutOwner2", UIParent)
        owner1:SetSize(100, 30)
        owner2:SetSize(100, 30)
        owner1:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 120, -140)
        owner2:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", -220, 180)
        GameTooltip:SetOwner(owner1, "ANCHOR_RIGHT", 12, -8)
    "#,
    )
    .unwrap();

    let (tooltip_id, screen_width, screen_height, before_rect) = {
        let state = env.state().borrow();
        let tooltip_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let before_rect = compute_frame_rect(
            &state.widgets,
            tooltip_id,
            state.screen_width,
            state.screen_height,
        );
        (
            tooltip_id,
            state.screen_width,
            state.screen_height,
            before_rect,
        )
    };

    env.exec(
        r#"
        GameTooltip:SetOwner(AnchorPreserveLayoutOwner2, "ANCHOR_PRESERVE")
    "#,
    )
    .unwrap();

    let after_rect = {
        let state = env.state().borrow();
        compute_frame_rect(&state.widgets, tooltip_id, screen_width, screen_height)
    };
    let owner_name: String = env.eval("return GameTooltip:GetOwner():GetName()").unwrap();

    assert_eq!(owner_name, "AnchorPreserveLayoutOwner2");
    assert!(
        (after_rect.x - before_rect.x).abs() < 0.1,
        "ANCHOR_PRESERVE should keep the tooltip x position when re-owned"
    );
    assert!(
        (after_rect.y - before_rect.y).abs() < 0.1,
        "ANCHOR_PRESERVE should keep the tooltip y position when re-owned"
    );
    assert!(
        (after_rect.width - before_rect.width).abs() < 0.1,
        "tooltip width should stay preserved"
    );
    assert!(
        (after_rect.height - before_rect.height).abs() < 0.1,
        "tooltip height should stay preserved"
    );
}

#[test]
fn test_set_anchor_type_reanchors_existing_owned_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "SetAnchorTypeOwner", UIParent)
        owner:SetSize(100, 30)
        owner:SetPoint("CENTER")
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT", 5, 10)
        GameTooltip:SetAnchorType("ANCHOR_TOPLEFT", 7, 9)
    "#,
    )
    .unwrap();

    let tooltip_anchor: String = env.eval("return GameTooltip:GetAnchorType()").unwrap();
    assert_eq!(tooltip_anchor, "ANCHOR_TOPLEFT");

    let is_owned: bool = env
        .eval("return GameTooltip:IsOwned(SetAnchorTypeOwner)")
        .unwrap();
    assert!(is_owned, "SetAnchorType should not change the owner");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let owner_id = state.widgets.get_id_by_name("SetAnchorTypeOwner").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(
        frame.anchors.len(),
        1,
        "SetAnchorType should rebuild the tooltip anchor"
    );
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::BottomLeft);
    assert_eq!(anchor.relative_point, AnchorPoint::TopLeft);
    assert_eq!(anchor.relative_to_id, Some(owner_id as usize));
    assert!((anchor.x_offset - 7.0).abs() < 0.1);
    assert!((anchor.y_offset - 9.0).abs() < 0.1);
}

#[test]
fn test_set_anchor_type_repositions_existing_owned_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "SetAnchorTypeLayoutOwner", UIParent)
        owner:SetSize(100, 30)
        owner:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 260, -180)
        GameTooltip:SetOwner(owner, "ANCHOR_RIGHT", 5, 10)
    "#,
    )
    .unwrap();

    let (tooltip_id, screen_width, screen_height, before_rect) = {
        let state = env.state().borrow();
        let tooltip_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let before_rect = compute_frame_rect(
            &state.widgets,
            tooltip_id,
            state.screen_width,
            state.screen_height,
        );
        (
            tooltip_id,
            state.screen_width,
            state.screen_height,
            before_rect,
        )
    };

    env.exec(r#"GameTooltip:SetAnchorType("ANCHOR_TOPLEFT", 7, 9)"#)
        .unwrap();

    let (after_rect, owner_rect) = {
        let state = env.state().borrow();
        let owner_id = state
            .widgets
            .get_id_by_name("SetAnchorTypeLayoutOwner")
            .unwrap();
        let after_rect =
            compute_frame_rect(&state.widgets, tooltip_id, screen_width, screen_height);
        let owner_rect = compute_frame_rect(&state.widgets, owner_id, screen_width, screen_height);
        (after_rect, owner_rect)
    };

    let is_owned: bool = env
        .eval("return GameTooltip:IsOwned(SetAnchorTypeLayoutOwner)")
        .unwrap();
    assert!(is_owned, "SetAnchorType should keep the existing owner");
    assert!(
        (after_rect.x - before_rect.x).abs() > 0.1 || (after_rect.y - before_rect.y).abs() > 0.1,
        "SetAnchorType should move the tooltip to the new anchor position"
    );
    assert!(
        (after_rect.x - (owner_rect.x + 7.0)).abs() < 0.1,
        "ANCHOR_TOPLEFT should align tooltip left edge with the owner left edge"
    );
    assert!(
        (after_rect.y + after_rect.height - (owner_rect.y - 9.0)).abs() < 0.1,
        "ANCHOR_TOPLEFT should place the tooltip above the owner using the provided y offset"
    );
}

#[test]
fn test_tooltip_anchor_cursor_uses_absolute_position() {
    let env = WowLuaEnv::new().unwrap();

    // Set mouse position before SetOwner
    env.state()
        .borrow_mut()
        .set_mouse_position(Some((200.0, 300.0)));

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

    assert_eq!(
        frame.anchors.len(),
        1,
        "ANCHOR_CURSOR should set one anchor"
    );
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::TopLeft);
    assert!(
        anchor.relative_to_id.is_none(),
        "ANCHOR_CURSOR should not reference owner"
    );
    assert!(
        (anchor.x_offset - 200.0).abs() < 0.1,
        "x_offset should be mouse x"
    );
    assert!(
        (anchor.y_offset - 320.0).abs() < 0.1,
        "y_offset should be mouse y + 20"
    );
}

#[test]
fn test_tooltip_side_cursor_anchors_use_absolute_position() {
    for anchor_kind in ["ANCHOR_CURSOR_RIGHT", "ANCHOR_CURSOR_LEFT"] {
        let env = WowLuaEnv::new().unwrap();

        env.state()
            .borrow_mut()
            .set_mouse_position(Some((200.0, 300.0)));

        env.exec(&format!(
            r#"
            local owner = CreateFrame("Frame", "AnchorCursorSideOwner", UIParent)
            GameTooltip:SetOwner(owner, "{anchor_kind}")
        "#
        ))
        .unwrap();

        let state = env.state().borrow();
        let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
        let frame = state.widgets.get(gt_id).unwrap();

        assert_eq!(
            frame.anchors.len(),
            1,
            "{anchor_kind} should set one anchor"
        );
        let anchor = &frame.anchors[0];
        assert_eq!(anchor.point, AnchorPoint::TopLeft);
        assert!(anchor.relative_to_id.is_none());
        assert!(
            (anchor.x_offset - 200.0).abs() < 0.1,
            "{anchor_kind} x_offset should be mouse x"
        );
        assert!(
            (anchor.y_offset - 320.0).abs() < 0.1,
            "{anchor_kind} y_offset should be mouse y + 20"
        );
    }
}

#[test]
fn test_tooltip_anchor_cursor_tracks_mouse_movement() {
    let env = WowLuaEnv::new().unwrap();

    env.state()
        .borrow_mut()
        .set_mouse_position(Some((100.0, 200.0)));
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "AnchorCursorFollowOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_CURSOR", 10, 30)
        GameTooltip:SetText("Cursor-follow tooltip")
        assert(GameTooltip:IsShown())
    "#,
    )
    .unwrap();

    env.state()
        .borrow_mut()
        .set_mouse_position(Some((300.0, 450.0)));

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert_eq!(frame.anchors.len(), 1);
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.point, AnchorPoint::TopLeft);
    assert!(anchor.relative_to_id.is_none());
    assert!(
        (anchor.x_offset - 310.0).abs() < 0.1,
        "x_offset should follow the updated mouse position"
    );
    assert!(
        (anchor.y_offset - 480.0).abs() < 0.1,
        "y_offset should follow the updated mouse position"
    );
}
