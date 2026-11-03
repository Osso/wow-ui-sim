#![cfg(all(feature = "gui", not(feature = "client-wrath")))]

mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;
use wow_ui_sim::render::font::WowFontSystem;
use wow_ui_sim::widget::AnchorPoint;

#[test]
fn test_world_cursor_tooltip_shows_world_cursor_data() {
    let env = setup_full_env();

    env.exec(
        r#"
        assert(GameTooltip.SetWorldCursor, "GameTooltip:SetWorldCursor should exist")
        GameTooltip:SetWorldCursor(Enum.WorldCursorAnchorType.Cursor)
        "#,
    )
    .expect("Failed to show the world cursor tooltip");

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    let getter_name: String = env
        .eval(
            r#"
            local info = GameTooltip:GetPrimaryTooltipInfo()
            return info and info.getterName or ""
            "#,
        )
        .unwrap();

    assert!(
        visible,
        "GameTooltip should be visible after SetWorldCursor"
    );
    assert!(
        num_lines >= 1,
        "GameTooltip should have world-cursor tooltip lines, got {}",
        num_lines
    );
    assert_eq!(getter_name, "GetWorldCursor");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist after SetWorldCursor");
    assert!(!td.lines[0].left_text.is_empty());
}

#[test]
fn test_world_cursor_nameplate_anchor_positions_tooltip_relative_to_owner() {
    let env = setup_full_env();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "WorldCursorNameplateOwner", UIParent)
        owner:SetSize(120, 20)
        owner:SetPoint("CENTER", UIParent, "CENTER", 80, 40)
        GameTooltip:SetWorldCursor(Enum.WorldCursorAnchorType.Nameplate, owner)
        "#,
    )
    .expect("Failed to show the world cursor tooltip with a nameplate anchor");

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_sys);
    state.ensure_layout_rects();
    drop(state);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let owner_id = state
        .widgets
        .get_id_by_name("WorldCursorNameplateOwner")
        .unwrap();
    let frame = state.widgets.get(gt_id).unwrap();
    let owner = state.widgets.get(owner_id).unwrap();
    let tooltip_rect = frame
        .layout_rect
        .expect("tooltip should have a layout rect");
    let owner_rect = owner.layout_rect.expect("owner should have a layout rect");

    assert_eq!(
        frame.anchors.len(),
        1,
        "Nameplate-anchored world cursor tooltips should create one owner-relative anchor"
    );
    let anchor = &frame.anchors[0];
    assert_eq!(anchor.relative_to_id, Some(owner_id as usize));
    assert_eq!(anchor.point, AnchorPoint::Bottom);
    assert_eq!(anchor.relative_point, AnchorPoint::Top);
    assert!(
        tooltip_rect.y + tooltip_rect.height <= owner_rect.y + 0.1,
        "Tooltip should sit above the owner rect: tooltip={tooltip_rect:?} owner={owner_rect:?}"
    );
}
