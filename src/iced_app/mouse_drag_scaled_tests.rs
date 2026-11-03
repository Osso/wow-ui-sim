use super::test_support::*;
use super::*;
use crate::screen::ScreenKind;

#[test]
fn scaled_moving_drag_converts_screen_delta_to_anchor_offset() {
    let mut app = build_test_app(ScreenKind::Game);
    let env = app.env.borrow();
    env.exec(
        r#"
        ScaledMovingDragFrame = CreateFrame("Frame", "ScaledMovingDragFrame", UIParent)
        ScaledMovingDragFrame:SetSize(100, 100)
        ScaledMovingDragFrame:SetScale(0.5)
        ScaledMovingDragFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        ScaledMovingDragFrame:SetMovable(true)
        ScaledMovingDragFrame:EnableMouse(true)
        ScaledMovingDragFrame:RegisterForDrag("LeftButton")
        ScaledMovingDragFrame:SetScript("OnDragStart", function(self) self:StartMoving() end)
        "#,
    )
    .expect("scaled moving drag frame setup should succeed");
    drop(env);

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(75.0, 75.0));
    app.handle_mouse_down(Point::new(75.0, 75.0));
    app.handle_mouse_move(Point::new(95.0, 95.0));

    let (x, y): (f64, f64) = app
        .env
        .borrow()
        .eval("local _, _, _, x, y = ScaledMovingDragFrame:GetPoint(1); return x, y")
        .expect("scaled moving drag anchor query should succeed");
    assert_eq!(x, 140.0, "20 screen pixels should become 40 UI units");
    assert_eq!(y, -140.0, "20 screen pixels should become 40 UI units");
}

#[test]
fn scaled_sizing_drag_tracks_mouse_until_frame_unit_bound() {
    let mut app = build_test_app(ScreenKind::Game);
    let env = app.env.borrow();
    env.exec(
        r#"
        ScaledSizingFrame = CreateFrame("Frame", "ScaledSizingFrame", UIParent)
        ScaledSizingFrame:SetSize(100, 100)
        ScaledSizingFrame:SetScale(0.5)
        ScaledSizingFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        ScaledSizingFrame:SetResizable(true)
        ScaledSizingFrame:SetResizeBounds(80, 80, 140, 120)
        ScaledSizingFrame:EnableMouse(true)
        ScaledSizingFrame:SetScript("OnMouseDown", function(self) self:StartSizing("BOTTOMRIGHT") end)
        "#,
    )
    .expect("scaled sizing drag frame setup should succeed");
    let initial_top: f64 = env.eval("return ScaledSizingFrame:GetTop()").unwrap();
    drop(env);

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(95.0, 95.0));
    app.handle_mouse_down(Point::new(95.0, 95.0));

    for (mouse_y, expected_height) in [(100.0, 110.0), (105.0, 120.0), (110.0, 120.0)] {
        app.handle_mouse_move(Point::new(95.0, mouse_y));
        let geometry: (f64, f64, f64) = app
            .env
            .borrow()
            .eval("return ScaledSizingFrame:GetWidth(), ScaledSizingFrame:GetHeight(), ScaledSizingFrame:GetTop()")
            .expect("scaled sizing geometry query should succeed");
        assert_eq!(geometry, (100.0, expected_height, initial_top));
    }
}
