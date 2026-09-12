use super::test_support::*;
use super::*;
use crate::iced_app::{CanvasMessage, Message};
use crate::screen::ScreenKind;

#[cfg(feature = "retail-12-1-0")]
#[path = "mouse_tests/forbidden_aspects.rs"]
mod forbidden_aspects;

#[test]
fn mouse_focus_order_tracks_gui_hover_after_raise_and_lower() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            MouseFocusLow = CreateFrame("Frame", nil, UIParent)
            MouseFocusLow:SetAllPoints(UIParent)
            MouseFocusLow:SetFrameStrata("DIALOG")
            MouseFocusLow:SetFrameLevel(1)
            MouseFocusLow:EnableMouse(true)
            MouseFocusLow:Show()

            MouseFocusHigh = CreateFrame("Frame", nil, UIParent)
            MouseFocusHigh:SetAllPoints(UIParent)
            MouseFocusHigh:SetFrameStrata("DIALOG")
            MouseFocusHigh:SetFrameLevel(10)
            MouseFocusHigh:EnableMouse(true)
            MouseFocusHigh:Show()
            "#,
        )
        .expect("mouse focus frame setup should succeed");

        let raw_levels: (f64, f64) = env
            .eval("return MouseFocusLow:GetFrameLevel(), MouseFocusHigh:GetFrameLevel()")
            .expect("raw frame levels should be readable");
        assert_eq!(raw_levels, (1.0, 10.0));
    }

    rebuild_hittable_cache(&app);
    let cursor = Point::new(400.0, 300.0);

    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(cursor)));
    let (before_get_mouse_focus, before_get_mouse_foci): (bool, bool) = app
        .env
        .borrow()
        .eval("return GetMouseFocus() == MouseFocusHigh, GetMouseFoci()[1] == MouseFocusHigh")
        .expect("before Raise mouse focus query should succeed");
    assert!(before_get_mouse_focus);
    assert!(before_get_mouse_foci);

    app.env
        .borrow()
        .exec("MouseFocusLow:Raise()")
        .expect("low frame Raise should succeed");
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(cursor)));
    let (after_raise_get_mouse_focus, after_raise_get_mouse_foci): (bool, bool) = app
        .env
        .borrow()
        .eval("return GetMouseFocus() == MouseFocusHigh, GetMouseFoci()[1] == MouseFocusHigh")
        .expect("after low Raise mouse focus query should succeed");
    assert!(after_raise_get_mouse_focus);
    assert!(after_raise_get_mouse_foci);

    app.env
        .borrow()
        .exec("MouseFocusHigh:Lower()")
        .expect("high frame Lower should succeed");
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(cursor)));
    let (after_lower_get_mouse_focus, after_lower_get_mouse_foci): (bool, bool) = app
        .env
        .borrow()
        .eval("return GetMouseFocus() == MouseFocusHigh, GetMouseFoci()[1] == MouseFocusHigh")
        .expect("after high Lower mouse focus query should succeed");
    assert!(after_lower_get_mouse_focus);
    assert!(after_lower_get_mouse_foci);

    app.env
        .borrow()
        .exec("MouseFocusLow:Hide(); MouseFocusHigh:Hide()")
        .expect("mouse focus frames should hide");
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(cursor)));

    let focus_cleared: (bool, bool) = app
        .env
        .borrow()
        .eval("return GetMouseFocus() == nil, GetMouseFoci()[1] == nil")
        .expect("cleared mouse focus should be readable");
    assert_eq!(focus_cleared, (true, true));
}

#[cfg(feature = "client-mists")]
#[test]
fn mists_main_menu_micro_button_click_opens_game_menu() {
    let app = build_test_app(ScreenKind::Game);
    load_blizzard_game_ui_for_mouse_test(&app);
    app.env
        .borrow()
        .exec("MainMenuMicroButton:Click()")
        .expect("MainMenuMicroButton click should run");

    let shown: bool = app
        .env
        .borrow()
        .eval("return GameMenuFrame and GameMenuFrame:IsShown() or false")
        .expect("GameMenuFrame visibility should be readable");
    assert!(shown, "MainMenuMicroButton click should open GameMenuFrame");
}

#[cfg(feature = "client-mists")]
fn load_blizzard_game_ui_for_mouse_test(app: &App) {
    use crate::loader::{discover_blizzard_addons_for_screen, load_addon};

    let env = app.env.borrow();
    env.set_screen_mode(ScreenKind::Game);

    if let Some(framexml_toc) = crate::client_profile::blizzard_ui_framexml_toc() {
        load_addon(&env.loader_env(), &framexml_toc).expect("FrameXML should load");
    }

    let addons_dir = crate::client_profile::blizzard_ui_addons_dir();
    let addons = discover_blizzard_addons_for_screen(&addons_dir, ScreenKind::Game);
    for (name, toc_path) in addons {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|err| panic!("{name} should load: {err}"));
        if name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }

    env.apply_post_load_workarounds();
    crate::startup::settle_headless_startup(&env);
}
#[test]
fn mouse_wheel_dispatch_requires_frame_mouse_wheel_enabled() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            MouseWheelDispatchFrame = CreateFrame("Frame", "MouseWheelDispatchFrame", UIParent)
            MouseWheelDispatchFrame:SetSize(100, 100)
            MouseWheelDispatchFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            MouseWheelDispatchFrame:EnableMouse(true)
            MouseWheelDispatchFrame:SetScript("OnMouseWheel", function(_, delta)
                __wheel_delta = (__wheel_delta or 0) + delta
            end)
            __wheel_delta = 0
            "#,
        )
        .expect("test frame setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));
    app.handle_scroll(0.0, -1.0);

    let not_delivered: f64 = app
        .env
        .borrow()
        .eval("return __wheel_delta")
        .expect("wheel delta query should succeed");
    assert_eq!(
        not_delivered, 0.0,
        "mouse wheel scripts should not fire while mouse wheel is disabled"
    );

    {
        let env = app.env.borrow();
        env.exec("MouseWheelDispatchFrame:EnableMouseWheel(true)")
            .expect("enabling mouse wheel should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));
    app.handle_scroll(0.0, -1.0);

    let delivered: f64 = app
        .env
        .borrow()
        .eval("return __wheel_delta")
        .expect("wheel delta query should succeed");
    assert_eq!(
        delivered, -1.0,
        "mouse wheel scripts should fire once the frame enables mouse wheel"
    );
}
#[test]
fn drag_start_can_transfer_to_delegate_and_abort_before_mouse_up() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            DragSourceFrame = CreateFrame("Frame", "DragSourceFrame", UIParent)
            DragSourceFrame:SetSize(100, 100)
            DragSourceFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            DragSourceFrame:EnableMouse(true)
            DragSourceFrame:SetScript("OnDragStart", function(self)
                self:InterceptStartDrag(DragDelegateFrame)
                DragDelegateFrame:AbortDrag()
            end)

            DragDelegateFrame = CreateFrame("Frame", "DragDelegateFrame", UIParent)
            DragDelegateFrame:SetScript("OnDragStop", function()
                __drag_stop_calls = (__drag_stop_calls or 0) + 1
            end)

            __drag_stop_calls = 0
            "#,
        )
        .expect("drag test frame setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));
    app.handle_mouse_down(Point::new(150.0, 150.0));
    app.handle_mouse_move(Point::new(170.0, 170.0));

    let active_drag_frame = app.env.borrow().state().borrow().active_drag_frame;
    assert_eq!(
        active_drag_frame, None,
        "AbortDrag during OnDragStart should clear the active drag frame immediately"
    );

    let delegate_dragging: bool = app
        .env
        .borrow()
        .eval("return DragDelegateFrame:IsDragging()")
        .expect("delegate dragging query should succeed");
    assert!(
        !delegate_dragging,
        "delegate should not report dragging after AbortDrag clears the transfer"
    );

    app.handle_mouse_up(Point::new(170.0, 170.0));

    let drag_stop_calls: f64 = app
        .env
        .borrow()
        .eval("return __drag_stop_calls")
        .expect("drag stop count query should succeed");
    assert_eq!(
        drag_stop_calls, 0.0,
        "mouse up should not fire OnDragStop after AbortDrag already cleared the drag"
    );
}
#[test]
fn moving_drag_updates_frame_anchor_to_follow_mouse() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            MovingDragFrame = CreateFrame("Frame", "MovingDragFrame", UIParent)
            MovingDragFrame:SetSize(100, 100)
            MovingDragFrame:SetPoint("CENTER", UIParent, "TOPLEFT", 150, -150)
            MovingDragFrame:SetMovable(true)
            MovingDragFrame:EnableMouse(true)
            MovingDragFrame:RegisterForDrag("LeftButton")
            MovingDragFrame:SetScript("OnDragStart", function(self)
                self:StartMoving()
            end)
            "#,
        )
        .expect("moving drag frame setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));
    app.handle_mouse_down(Point::new(150.0, 150.0));
    app.handle_mouse_move(Point::new(170.0, 170.0));

    let (num_points, point, relative_to, relative_point, x, y): (
        i64,
        String,
        String,
        String,
        f64,
        f64,
    ) = app
        .env
        .borrow()
        .eval(
            r#"
            local point, relativeTo, relativePoint, x, y = MovingDragFrame:GetPoint(1)
            local relativeName = relativeTo and relativeTo:GetName() or "nil"
            return MovingDragFrame:GetNumPoints(), point, relativeName, relativePoint, x, y
        "#,
        )
        .expect("moving drag anchor query should succeed");

    assert_eq!(
        num_points, 1,
        "moving drag should collapse anchors to one TOPLEFT point"
    );
    assert_eq!(
        point, "TOPLEFT",
        "moving drag should re-anchor the frame by TOPLEFT"
    );
    assert_eq!(
        relative_to, "UIParent",
        "moving drag should anchor relative to the parent"
    );
    assert_eq!(
        relative_point, "TOPLEFT",
        "moving drag should use the parent's TOPLEFT as its target"
    );
    assert_eq!(x, 120.0, "mouse delta should advance the frame x offset");
    assert_eq!(y, -120.0, "mouse delta should advance the frame y offset");
}
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
fn drag_start_on_child_moves_parent_started_by_handler() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            DelegatedDragParent = CreateFrame("Frame", "DelegatedDragParent", UIParent)
            DelegatedDragParent:SetSize(100, 100)
            DelegatedDragParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            DelegatedDragParent:SetMovable(true)

            DelegatedDragSelection = CreateFrame("Frame", "DelegatedDragSelection", DelegatedDragParent)
            DelegatedDragSelection:SetAllPoints()
            DelegatedDragSelection:EnableMouse(true)
            DelegatedDragSelection:RegisterForDrag("LeftButton")
            DelegatedDragSelection:SetScript("OnDragStart", function()
                DelegatedDragParent:StartMoving()
            end)
            DelegatedDragSelection:SetScript("OnDragStop", function()
                DelegatedDragParent:StopMovingOrSizing()
            end)
            "#,
        )
        .expect("delegated drag frame setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(120.0, 120.0));
    app.handle_mouse_down(Point::new(120.0, 120.0));
    app.handle_mouse_move(Point::new(140.0, 140.0));

    let (point, relative_to, relative_point, x, y): (String, String, String, f64, f64) = app
        .env
        .borrow()
        .eval(
            r#"
            local point, relativeTo, relativePoint, x, y = DelegatedDragParent:GetPoint(1)
            local relativeName = relativeTo and relativeTo:GetName() or "nil"
            return point, relativeName, relativePoint, x, y
            "#,
        )
        .expect("delegated moving drag anchor query should succeed");

    assert_eq!(point, "TOPLEFT");
    assert_eq!(relative_to, "UIParent");
    assert_eq!(relative_point, "TOPLEFT");
    assert_eq!(x, 120.0, "parent started by child drag should move in x");
    assert_eq!(y, -120.0, "parent started by child drag should move in y");
}

#[test]
fn moving_drag_clamps_frame_to_screen_when_enabled() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            ClampedMovingDragFrame = CreateFrame("Frame", "ClampedMovingDragFrame", UIParent)
            ClampedMovingDragFrame:SetSize(100, 100)
            ClampedMovingDragFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 10, -10)
            ClampedMovingDragFrame:SetMovable(true)
            ClampedMovingDragFrame:SetClampedToScreen(true)
            ClampedMovingDragFrame:EnableMouse(true)
            ClampedMovingDragFrame:RegisterForDrag("LeftButton")
            ClampedMovingDragFrame:SetScript("OnDragStart", function(self)
                self:StartMoving()
            end)
            "#,
        )
        .expect("clamped moving drag frame setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(20.0, 20.0));
    app.handle_mouse_down(Point::new(20.0, 20.0));
    app.handle_mouse_move(Point::new(-80.0, -80.0));

    let (point, relative_to, relative_point, x, y): (String, String, String, f64, f64) = app
        .env
        .borrow()
        .eval(
            r#"
            local point, relativeTo, relativePoint, x, y = ClampedMovingDragFrame:GetPoint(1)
            local relativeName = relativeTo and relativeTo:GetName() or "nil"
            return point, relativeName, relativePoint, x, y
        "#,
        )
        .expect("clamped moving drag anchor query should succeed");

    assert_eq!(point, "TOPLEFT");
    assert_eq!(relative_to, "UIParent");
    assert_eq!(relative_point, "TOPLEFT");
    assert_eq!(
        x, 0.0,
        "clamped moving drag should not move past the left edge"
    );
    assert_eq!(
        y, 0.0,
        "clamped moving drag should not move past the top edge"
    );
}

#[test]
fn stop_moving_or_sizing_marks_dragged_frame_user_placed() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            UserPlacedDragFrame = CreateFrame("Frame", "UserPlacedDragFrame", UIParent)
            UserPlacedDragFrame:SetSize(100, 100)
            UserPlacedDragFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 50, -50)
            UserPlacedDragFrame:SetMovable(true)
            UserPlacedDragFrame:EnableMouse(true)
            UserPlacedDragFrame:RegisterForDrag("LeftButton")
            UserPlacedDragFrame:SetScript("OnDragStart", function(self)
                self:StartMoving()
            end)
            UserPlacedDragFrame:SetScript("OnDragStop", function(self)
                self:StopMovingOrSizing()
            end)
            "#,
        )
        .expect("user placed drag frame setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(60.0, 60.0));
    app.handle_mouse_down(Point::new(60.0, 60.0));
    app.handle_mouse_move(Point::new(90.0, 90.0));
    app.handle_mouse_up(Point::new(90.0, 90.0));

    let user_placed: bool = app
        .env
        .borrow()
        .eval(
            r#"
            return UserPlacedDragFrame:IsUserPlaced()
        "#,
        )
        .expect("user placed drag state query should succeed");
    let is_moving = app
        .env
        .borrow()
        .state()
        .borrow()
        .widgets
        .get_by_name("UserPlacedDragFrame")
        .map(|frame| frame.is_moving)
        .expect("user placed drag frame should exist");

    assert!(
        !is_moving,
        "StopMovingOrSizing should clear the moving state after drag stop"
    );
    assert!(
        user_placed,
        "StopMovingOrSizing should mark a dragged frame as user placed"
    );
}

#[test]
fn drag_start_requires_registered_button_match() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            __mismatched_drag_start_calls = 0
            MismatchedDragButtonFrame = CreateFrame("Frame", "MismatchedDragButtonFrame", UIParent)
            MismatchedDragButtonFrame:SetSize(100, 100)
            MismatchedDragButtonFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 50, -50)
            MismatchedDragButtonFrame:EnableMouse(true)
            MismatchedDragButtonFrame:RegisterForDrag("RightButton")
            MismatchedDragButtonFrame:SetScript("OnDragStart", function()
                __mismatched_drag_start_calls = __mismatched_drag_start_calls + 1
            end)
            "#,
        )
        .expect("mismatched drag button frame setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(60.0, 60.0));
    app.handle_mouse_down(Point::new(60.0, 60.0));
    app.handle_mouse_move(Point::new(90.0, 90.0));

    let drag_start_calls: f64 = app
        .env
        .borrow()
        .eval("return __mismatched_drag_start_calls")
        .expect("drag start call count query should succeed");
    let active_drag_frame = app.env.borrow().state().borrow().active_drag_frame;

    assert_eq!(
        drag_start_calls, 0.0,
        "left-button drag should not fire OnDragStart when only RightButton is registered"
    );
    assert_eq!(
        active_drag_frame, None,
        "mismatched drag button should not activate a drag frame"
    );
}

#[test]
fn slider_reports_thumb_drag_state_while_mouse_is_held() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            SliderThumbDragFrame = CreateFrame("Slider", "SliderThumbDragFrame", UIParent)
            SliderThumbDragFrame:SetSize(120, 20)
            SliderThumbDragFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            SliderThumbDragFrame:EnableMouse(true)
            "#,
        )
        .expect("slider setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let drag_pos = Point::new(150.0, 110.0);

    let before_mouse_down: bool = app
        .env
        .borrow()
        .eval("return SliderThumbDragFrame:IsDraggingThumb()")
        .expect("initial thumb drag query should succeed");
    assert!(
        !before_mouse_down,
        "fresh sliders should not report thumb dragging"
    );

    app.handle_mouse_move(drag_pos);
    app.handle_mouse_down(drag_pos);

    let during_mouse_down: bool = app
        .env
        .borrow()
        .eval("return SliderThumbDragFrame:IsDraggingThumb()")
        .expect("active thumb drag query should succeed");
    assert!(
        during_mouse_down,
        "slider should report thumb dragging while left mouse is held on it"
    );

    app.handle_mouse_up(drag_pos);

    let after_mouse_up: bool = app
        .env
        .borrow()
        .eval("return SliderThumbDragFrame:IsDraggingThumb()")
        .expect("post mouse up thumb drag query should succeed");
    assert!(
        !after_mouse_up,
        "slider thumb drag state should clear on mouse up"
    );
}

#[test]
fn pass_through_buttons_reroute_clicks_and_can_be_cleared() {
    let mut app = build_test_app(ScreenKind::Game);
    setup_pass_through_test_frames(&app);

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);

    app.handle_mouse_down(click_pos);
    app.handle_mouse_up(click_pos);
    app.handle_right_mouse_down(click_pos);
    app.handle_right_mouse_up(click_pos);

    let (parent_left, parent_right, child_left, child_right) = read_pass_through_counters(&app);
    assert_eq!(parent_left, 0.0, "left click should not pass through");
    assert_eq!(
        parent_right, 1.0,
        "right click should pass through to parent"
    );
    assert_eq!(
        child_left, 1.0,
        "child should still receive non-passthrough left clicks"
    );
    assert_eq!(
        child_right, 0.0,
        "child should not receive passthrough right clicks"
    );

    clear_pass_through_buttons(&app);
    rebuild_hittable_cache(&app);
    app.handle_right_mouse_down(click_pos);
    app.handle_right_mouse_up(click_pos);

    let (_, cleared_parent_right, _, cleared_child_right) = read_pass_through_counters(&app);
    assert_eq!(
        cleared_parent_right, 0.0,
        "clearing pass-through buttons should stop rerouting right clicks"
    );
    assert_eq!(
        cleared_child_right, 1.0,
        "child should receive right clicks again after passthrough is cleared"
    );
}

#[test]
fn hover_and_click_use_child_render_order_when_parent_wins_phase_one() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            HitOrderParent = CreateFrame("Button", "HitOrderParent", UIParent)
            HitOrderParent:SetSize(100, 100)
            HitOrderParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            HitOrderParent:SetFrameLevel(10)
            HitOrderParent:EnableMouse(true)

            HitOrderHigh = CreateFrame("Button", "HitOrderHigh", HitOrderParent)
            HitOrderHigh:SetAllPoints(HitOrderParent)
            HitOrderHigh:SetFrameLevel(3)
            HitOrderHigh:EnableMouse(true)
            HitOrderHigh:SetScript("OnClick", function()
                __hit_order_high_clicks = (__hit_order_high_clicks or 0) + 1
            end)

            HitOrderLow = CreateFrame("Button", "HitOrderLow", HitOrderParent)
            HitOrderLow:SetAllPoints(HitOrderParent)
            HitOrderLow:SetFrameLevel(1)
            HitOrderLow:EnableMouse(true)
            HitOrderLow:SetScript("OnClick", function()
                __hit_order_low_clicks = (__hit_order_low_clicks or 0) + 1
            end)

            __hit_order_high_clicks = 0
            __hit_order_low_clicks = 0
            "#,
        )
        .expect("hit-order test setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let hover_pos = Point::new(150.0, 150.0);
    app.handle_mouse_move(hover_pos);

    let hovered_name = {
        let env = app.env.borrow();
        let state = env.state().borrow();
        let hovered_id = state.hovered_frame.expect("a child should be hovered");
        state
            .widgets
            .get(hovered_id)
            .and_then(|frame| frame.name.clone())
            .expect("hovered frame should have a name")
    };
    assert_eq!(
        hovered_name, "HitOrderHigh",
        "hover should resolve to the highest-rendered child, not the last-created child"
    );

    app.handle_mouse_down(hover_pos);
    app.handle_mouse_up(hover_pos);

    let (high_clicks, low_clicks): (f64, f64) = app
        .env
        .borrow()
        .eval("return __hit_order_high_clicks, __hit_order_low_clicks")
        .expect("click counters should be readable");
    assert_eq!(
        high_clicks, 1.0,
        "click should go to the highest-rendered child when the parent wins phase one"
    );
    assert_eq!(
        low_clicks, 0.0,
        "lower-rendered sibling should not steal the click by creation order"
    );
}

#[test]
fn alpha_zero_mouse_enabled_frames_remain_clickable() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            AlphaZeroClickButton = CreateFrame("Button", "AlphaZeroClickButton", UIParent)
            AlphaZeroClickButton:SetSize(100, 100)
            AlphaZeroClickButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            AlphaZeroClickButton:EnableMouse(true)
            AlphaZeroClickButton:SetAlpha(0)
            AlphaZeroClickButton:SetScript("OnClick", function()
                __alpha_zero_clicks = (__alpha_zero_clicks or 0) + 1
            end)
            __alpha_zero_clicks = 0
            "#,
        )
        .expect("alpha-zero click setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);
    app.handle_mouse_down(click_pos);
    app.handle_mouse_up(click_pos);

    let clicks: f64 = app
        .env
        .borrow()
        .eval("return __alpha_zero_clicks")
        .expect("alpha-zero click count should be readable");
    assert_eq!(
        clicks, 1.0,
        "alpha controls rendering, not mouse hit-test eligibility"
    );
}

#[test]
fn propagated_mouse_clicks_fire_parent_mouse_handlers() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            PropagatingDropdownParent = CreateFrame("Button", "PropagatingDropdownParent", UIParent)
            PropagatingDropdownParent:SetSize(100, 100)
            PropagatingDropdownParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            PropagatingDropdownParent:SetScript("OnMouseDown", function(_, button)
                __propagated_parent_down = (__propagated_parent_down or "") .. button .. ";"
            end)

            PropagatingDropdownChild = CreateFrame("Button", "PropagatingDropdownChild", PropagatingDropdownParent)
            PropagatingDropdownChild:SetAllPoints(PropagatingDropdownParent)
            PropagatingDropdownChild:SetPropagateMouseClicks(true)
            PropagatingDropdownChild:SetScript("OnMouseDown", function(_, button)
                __propagated_child_down = (__propagated_child_down or "") .. button .. ";"
            end)

            __propagated_parent_down = ""
            __propagated_child_down = ""
            "#,
        )
        .expect("propagating click setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_down(Point::new(150.0, 150.0));

    let (parent_down, child_down): (String, String) = app
        .env
        .borrow()
        .eval("return __propagated_parent_down, __propagated_child_down")
        .expect("propagated click counters should be readable");
    assert_eq!(child_down, "LeftButton;");
    assert_eq!(parent_down, "LeftButton;");
}
