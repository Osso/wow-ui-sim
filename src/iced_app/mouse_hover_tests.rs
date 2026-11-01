use super::test_support::*;
use super::*;
use crate::screen::ScreenKind;
use iced::widget::shader::Program;

#[test]
fn hit_test_reflects_frames_shown_by_previous_clicks() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            RevealParent = CreateFrame("Button", "RevealParent", UIParent)
            RevealParent:SetSize(100, 100)
            RevealParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            RevealParent:EnableMouse(true)
            RevealParent:SetScript("OnClick", function()
                RevealTarget:Show()
            end)

            RevealTarget = CreateFrame("Button", "RevealTarget", UIParent)
            RevealTarget:SetSize(100, 100)
            RevealTarget:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 300, -100)
            RevealTarget:EnableMouse(true)
            RevealTarget:Hide()
            RevealTarget:SetScript("OnClick", function()
                __reveal_target_clicks = (__reveal_target_clicks or 0) + 1
            end)

            __reveal_target_clicks = 0
            "#,
        )
        .expect("reveal test setup should succeed");
    }

    rebuild_hittable_cache(&app);

    let reveal_pos = Point::new(150.0, 150.0);
    app.handle_mouse_move(reveal_pos);
    app.handle_mouse_down(reveal_pos);
    app.handle_mouse_up(reveal_pos);

    let target_center = {
        let env = app.env.borrow();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let target_id = state
            .widgets
            .iter_ids()
            .find(|&id| {
                state.widgets.get(id).and_then(|f| f.name.as_deref()) == Some("RevealTarget")
            })
            .expect("target frame should exist");
        let rect = state
            .widgets
            .get(target_id)
            .and_then(|f| f.layout_rect)
            .expect("target frame should have a layout rect after being shown");
        Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
    };

    let hit = app.hit_test(target_center);
    assert_eq!(
        hit,
        Some(
            app.env
                .borrow()
                .state()
                .borrow()
                .widgets
                .iter_ids()
                .find(|&id| {
                    app.env
                        .borrow()
                        .state()
                        .borrow()
                        .widgets
                        .get(id)
                        .and_then(|f| f.name.as_deref())
                        == Some("RevealTarget")
                })
                .expect("target frame should exist")
        ),
        "hit-test should see frames revealed by earlier UI mutations"
    );

    app.handle_mouse_down(target_center);
    app.handle_mouse_up(target_center);

    let clicks: f64 = app
        .env
        .borrow()
        .eval("return __reveal_target_clicks")
        .expect("click count should be readable");
    assert_eq!(
        clicks, 1.0,
        "newly revealed frame should be clickable immediately after the revealing click"
    );
}

#[test]
fn click_release_refreshes_hover_when_script_hides_frame() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            SelfHidingHoverButton = CreateFrame("Button", "SelfHidingHoverButton", UIParent)
            SelfHidingHoverButton:SetSize(100, 100)
            SelfHidingHoverButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            SelfHidingHoverButton:EnableMouse(true)
            SelfHidingHoverButton:SetScript("OnEnter", function()
                __self_hide_enter = (__self_hide_enter or 0) + 1
            end)
            SelfHidingHoverButton:SetScript("OnLeave", function()
                __self_hide_leave = (__self_hide_leave or 0) + 1
            end)
            SelfHidingHoverButton:SetScript("OnClick", function(self)
                self:Hide()
            end)
            __self_hide_enter = 0
            __self_hide_leave = 0
            "#,
        )
        .expect("self-hiding hover setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);
    app.handle_mouse_move(click_pos);
    app.handle_mouse_down(click_pos);
    app.handle_mouse_up(click_pos);

    let (enter_count, leave_count): (f64, f64) = app
        .env
        .borrow()
        .eval("return __self_hide_enter, __self_hide_leave")
        .expect("hover counters should be readable");
    let hovered_name = app.hovered_frame.and_then(|id| {
        app.env
            .borrow()
            .state()
            .borrow()
            .widgets
            .get(id)
            .and_then(|frame| frame.name.clone())
    });

    assert_eq!(enter_count, 1.0, "button should enter hover before click");
    assert_eq!(leave_count, 1.0, "hiding on click should fire OnLeave");
    assert_ne!(
        hovered_name.as_deref(),
        Some("SelfHidingHoverButton"),
        "hidden clicked frame must not remain the hover target"
    );
}

#[test]
fn mouse_edge_scripts_imply_mouse_enabled_and_update_hit_grid() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            ImpliedMouseFrame = CreateFrame("Frame", "ImpliedMouseFrame", UIParent)
            ImpliedMouseFrame:SetSize(100, 100)
            ImpliedMouseFrame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            __implied_mouse_down = 0
            "#,
        )
        .expect("implied mouse setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_down(Point::new(150.0, 150.0));

    let before_script: f64 = app
        .env
        .borrow()
        .eval("return __implied_mouse_down")
        .expect("initial mouse counter should be readable");
    assert_eq!(before_script, 0.0);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            ImpliedMouseFrame:SetScript("OnMouseDown", function(_, button)
                if button == "LeftButton" then
                    __implied_mouse_down = __implied_mouse_down + 1
                end
            end)
            "#,
        )
        .expect("setting OnMouseDown should succeed");
    }

    app.handle_mouse_down(Point::new(150.0, 150.0));

    let (mouse_enabled, after_script): (bool, f64) = app
        .env
        .borrow()
        .eval("return ImpliedMouseFrame:IsMouseEnabled(), __implied_mouse_down")
        .expect("implied mouse state should be readable");
    assert!(
        mouse_enabled,
        "setting OnMouseDown should imply EnableMouse(true)"
    );
    assert_eq!(
        after_script, 1.0,
        "hit grid should accept the frame without a full rebuild after SetScript"
    );
}

#[test]
fn disabled_buttons_only_run_hover_motion_scripts_when_opted_in() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            DisabledMotionButton = CreateFrame("Button", "DisabledMotionButton", UIParent)
            DisabledMotionButton:SetSize(100, 100)
            DisabledMotionButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            DisabledMotionButton:EnableMouse(true)
            DisabledMotionButton:Disable()
            DisabledMotionButton:SetScript("OnEnter", function()
                __disabled_motion_enter = (__disabled_motion_enter or 0) + 1
            end)
            DisabledMotionButton:SetScript("OnLeave", function()
                __disabled_motion_leave = (__disabled_motion_leave or 0) + 1
            end)
            __disabled_motion_enter = 0
            __disabled_motion_leave = 0
            "#,
        )
        .expect("disabled motion test button setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let outside_pos = Point::new(20.0, 20.0);
    let inside_pos = Point::new(150.0, 150.0);

    app.handle_mouse_move(outside_pos);
    app.handle_mouse_move(inside_pos);
    app.handle_mouse_move(outside_pos);

    let (blocked_enter, blocked_leave, default_flag): (f64, f64, bool) = app
        .env
        .borrow()
        .eval(
            "return __disabled_motion_enter, __disabled_motion_leave, DisabledMotionButton:GetMotionScriptsWhileDisabled()",
        )
        .expect("default disabled motion state should be readable");
    assert_eq!(
        blocked_enter, 0.0,
        "disabled buttons should not receive OnEnter by default"
    );
    assert_eq!(
        blocked_leave, 0.0,
        "disabled buttons should not receive OnLeave by default"
    );
    assert!(
        !default_flag,
        "disabled buttons should default motion scripts while disabled to false"
    );

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            DisabledMotionButton:SetMotionScriptsWhileDisabled(true)
            __disabled_motion_enter = 0
            __disabled_motion_leave = 0
            "#,
        )
        .expect("enabling motion scripts while disabled should succeed");
    }

    app.handle_mouse_move(outside_pos);
    app.handle_mouse_move(inside_pos);
    app.handle_mouse_move(outside_pos);

    let (allowed_enter, allowed_leave, enabled_flag): (f64, f64, bool) = app
        .env
        .borrow()
        .eval(
            "return __disabled_motion_enter, __disabled_motion_leave, DisabledMotionButton:GetMotionScriptsWhileDisabled()",
        )
        .expect("enabled disabled-motion state should be readable");
    assert_eq!(
        allowed_enter, 1.0,
        "disabled buttons should receive OnEnter after opting into motion scripts"
    );
    assert_eq!(
        allowed_leave, 1.0,
        "disabled buttons should receive OnLeave after opting into motion scripts"
    );
    assert!(
        enabled_flag,
        "GetMotionScriptsWhileDisabled should reflect the opt-in flag"
    );
}

#[test]
fn cursor_leaving_canvas_fires_on_leave_for_hovered_frame() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            CanvasLeaveButton = CreateFrame("Button", "CanvasLeaveButton", UIParent)
            CanvasLeaveButton:SetSize(100, 100)
            CanvasLeaveButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            CanvasLeaveButton:EnableMouse(true)
            CanvasLeaveButton:SetNormalTexture("Interface/Buttons/UI-Panel-Button-Up")
            CanvasLeaveButton:SetScript("OnEnter", function(self)
                self:GetNormalTexture():SetAlpha(0)
            end)
            CanvasLeaveButton:SetScript("OnLeave", function(self)
                self:GetNormalTexture():SetAlpha(1)
                __canvas_leave_count = (__canvas_leave_count or 0) + 1
            end)
            __canvas_leave_count = 0
            "#,
        )
        .expect("canvas leave test setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));

    let alpha_after_enter: f64 = app
        .env
        .borrow()
        .eval("return CanvasLeaveButton:GetNormalTexture():GetAlpha()")
        .expect("normal texture alpha should be readable after enter");
    assert_eq!(alpha_after_enter, 0.0);

    app.handle_mouse_leave();

    let (alpha_after_leave, leave_count): (f64, f64) = app
        .env
        .borrow()
        .eval("return CanvasLeaveButton:GetNormalTexture():GetAlpha(), __canvas_leave_count")
        .expect("normal texture alpha should be readable after canvas leave");
    assert_eq!(
        alpha_after_leave, 1.0,
        "cursor leaving the canvas should restore hover-hidden normal textures"
    );
    assert_eq!(leave_count, 1.0, "OnLeave should fire exactly once");
    assert!(
        app.hovered_frame.is_none(),
        "canvas leave should clear the app hover target"
    );
}

#[test]
fn moving_inside_canvas_off_hovered_frame_fires_on_leave() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            CanvasMoveAwayButton = CreateFrame("Button", "CanvasMoveAwayButton", UIParent)
            CanvasMoveAwayButton:SetSize(100, 100)
            CanvasMoveAwayButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            CanvasMoveAwayButton:EnableMouse(true)
            CanvasMoveAwayButton:SetNormalTexture("Interface/Buttons/UI-Panel-Button-Up")
            CanvasMoveAwayButton:SetScript("OnEnter", function(self)
                self:GetNormalTexture():SetAlpha(0)
            end)
            CanvasMoveAwayButton:SetScript("OnLeave", function(self)
                self:GetNormalTexture():SetAlpha(1)
                __canvas_move_leave_count = (__canvas_move_leave_count or 0) + 1
            end)
            __canvas_move_leave_count = 0
            "#,
        )
        .expect("canvas move-away test setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));
    app.handle_mouse_move(Point::new(20.0, 20.0));

    let (alpha_after_leave, leave_count): (f64, f64) = app
        .env
        .borrow()
        .eval(
            "return CanvasMoveAwayButton:GetNormalTexture():GetAlpha(), __canvas_move_leave_count",
        )
        .expect("normal texture alpha should be readable after moving away");
    assert_eq!(
        alpha_after_leave, 1.0,
        "moving off the button inside the canvas should restore normal texture alpha"
    );
    assert_eq!(leave_count, 1.0, "OnLeave should fire exactly once");
    assert!(
        app.hovered_frame.is_none(),
        "moving to empty canvas should clear the app hover target"
    );
}

#[test]
fn hover_after_gui_resize_uses_resized_screen_coordinates() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            ResizeAnchoredButton = CreateFrame("Button", "ResizeAnchoredButton", UIParent)
            ResizeAnchoredButton:SetSize(50, 50)
            ResizeAnchoredButton:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", -20, 20)
            ResizeAnchoredButton:EnableMouse(true)
            ResizeAnchoredButton:SetScript("OnEnter", function()
                __resize_anchor_enter = (__resize_anchor_enter or 0) + 1
            end)
            __resize_anchor_enter = 0
            "#,
        )
        .expect("resize anchored hover setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.sync_screen_size_to_state(iced::Size::new(1024.0, 768.0));
    app.screen_size.set(iced::Size::new(1024.0, 768.0));

    let resized_center = {
        let env = app.env.borrow();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let button_id = state
            .widgets
            .get_id_by_name("ResizeAnchoredButton")
            .expect("ResizeAnchoredButton should exist");
        let rect = state
            .widgets
            .get(button_id)
            .and_then(|frame| frame.layout_rect)
            .expect("ResizeAnchoredButton should have a resized layout rect");
        Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
    };

    app.handle_mouse_move(resized_center);

    let enter_count: f64 = app
        .env
        .borrow()
        .eval("return __resize_anchor_enter")
        .expect("resize hover counter should be readable");
    assert_eq!(
        enter_count, 1.0,
        "hover after GUI resize should use resized frame coordinates"
    );
}

#[test]
fn first_draw_at_actual_canvas_size_rebuilds_resize_dependent_hover_rects() {
    let mut app = build_test_app(ScreenKind::Game);
    let startup_size = iced::Size::new(1024.0, 768.0);
    let actual_canvas_size = iced::Size::new(920.0, 640.0);

    app.env
        .borrow()
        .set_screen_size(startup_size.width, startup_size.height);
    app.screen_size.set(startup_size);
    {
        let env = app.env.borrow();
        env.exec(
            r#"
            StartupResizeAnchoredButton = CreateFrame("Button", "StartupResizeAnchoredButton", UIParent)
            StartupResizeAnchoredButton:SetSize(50, 50)
            StartupResizeAnchoredButton:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOMRIGHT", -20, 20)
            StartupResizeAnchoredButton:EnableMouse(true)
            StartupResizeAnchoredButton:SetScript("OnEnter", function()
                __startup_resize_enter = (__startup_resize_enter or 0) + 1
            end)
            __startup_resize_enter = 0
            "#,
        )
        .expect("startup resize hover setup should succeed");
        env.state().borrow_mut().ensure_layout_rects();
    }

    let _primitive = <&crate::iced_app::App as Program<crate::iced_app::Message>>::draw(
        &&app,
        &(),
        iced::mouse::Cursor::Unavailable,
        iced::Rectangle::new(iced::Point::ORIGIN, actual_canvas_size),
    );

    // Startup and resize processing moved from draw to update in 77eaac46d.
    for _ in 0..200 {
        if app.gui_startup_complete.get() {
            break;
        }
        let _ = app.update(crate::iced_app::Message::ProcessTimers(
            std::time::Instant::now(),
        ));
    }
    assert!(app.gui_startup_complete.get(), "GUI startup must complete");

    let resized_center = {
        let env = app.env.borrow();
        let mut state = env.state().borrow_mut();
        state.ensure_layout_rects();
        let button_id = state
            .widgets
            .get_id_by_name("StartupResizeAnchoredButton")
            .expect("StartupResizeAnchoredButton should exist");
        let rect = state
            .widgets
            .get(button_id)
            .and_then(|frame| frame.layout_rect)
            .expect("StartupResizeAnchoredButton should have a resized layout rect");
        Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
    };

    app.handle_mouse_move(resized_center);

    let enter_count: f64 = app
        .env
        .borrow()
        .eval("return __startup_resize_enter")
        .expect("startup resize hover counter should be readable");
    assert_eq!(
        enter_count, 1.0,
        "first GUI draw at actual canvas size should make bottom-right hit testing use resized coordinates"
    );
}

#[test]
fn mouse_hover_sizes_tooltip_before_next_resize_or_draw() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            TooltipHoverButton = CreateFrame("Button", "TooltipHoverButton", UIParent)
            TooltipHoverButton:SetSize(100, 40)
            TooltipHoverButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            TooltipHoverButton:EnableMouse(true)
            TooltipHoverButton:SetScript("OnEnter", function(self)
                GameTooltip:SetOwner(self, "ANCHOR_RIGHT")
                GameTooltip:ClearLines()
                GameTooltip:AddLine("Tooltip hover should size before draw")
                GameTooltip:Show()
            end)
            "#,
        )
        .expect("tooltip hover setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 120.0));

    let (shown, width, height): (bool, f64, f64) = app
        .env
        .borrow()
        .eval("return GameTooltip:IsShown(), GameTooltip:GetWidth(), GameTooltip:GetHeight()")
        .expect("tooltip geometry should be readable after hover");
    assert!(shown, "tooltip should be shown by the hover script");
    assert!(
        width > 0.0,
        "tooltip width should be resolved by the hover event, got {width}"
    );
    assert!(
        height > 0.0,
        "tooltip height should be resolved by the hover event, got {height}"
    );
}

#[test]
fn hover_after_lua_shows_hidden_subtree_rebuilds_hit_state_before_resize() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            RuntimeShownParent = CreateFrame("Frame", "RuntimeShownParent", UIParent)
            RuntimeShownParent:SetSize(140, 80)
            RuntimeShownParent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 200, -160)
            RuntimeShownParent:Hide()

            RuntimeShownChild = CreateFrame("Button", "RuntimeShownChild", RuntimeShownParent)
            RuntimeShownChild:SetSize(60, 30)
            RuntimeShownChild:SetPoint("TOPLEFT", RuntimeShownParent, "TOPLEFT", 20, -20)
            RuntimeShownChild:EnableMouse(true)
            RuntimeShownChild:SetScript("OnEnter", function()
                __runtime_shown_child_enter = (__runtime_shown_child_enter or 0) + 1
            end)
            __runtime_shown_child_enter = 0
            "#,
        )
        .expect("runtime shown subtree setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.strata_dirty.set(0);

    app.env
        .borrow()
        .exec("RuntimeShownParent:Show()")
        .expect("showing hidden subtree should succeed");
    app.invalidate_after_lua_mutation();

    app.handle_mouse_move(Point::new(230.0, 195.0));

    let enter_count: f64 = app
        .env
        .borrow()
        .eval("return __runtime_shown_child_enter")
        .expect("runtime shown child counter should be readable");
    assert_eq!(
        enter_count, 1.0,
        "newly shown child should receive hover before the next GUI resize or draw"
    );
}

#[test]
fn syncing_same_gui_size_does_not_trigger_resize_refresh() {
    let app = build_test_app(ScreenKind::Game);
    rebuild_hittable_cache(&app);
    app.strata_dirty.set(0);

    app.sync_screen_size_to_state(iced::Size::new(800.0, 600.0));

    assert_eq!(
        app.strata_dirty.get(),
        0,
        "matching startup size should not dirty render state as a resize"
    );
    assert!(
        app.cached_hittable.borrow().is_some(),
        "matching startup size should keep the existing hit grid"
    );
}

#[test]
fn moving_inside_same_hovered_frame_does_not_dirty_render_state() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            SameHoverButton = CreateFrame("Button", "SameHoverButton", UIParent)
            SameHoverButton:SetSize(100, 100)
            SameHoverButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            SameHoverButton:EnableMouse(true)
            SameHoverButton:SetScript("OnEnter", function()
                __same_hover_enter = (__same_hover_enter or 0) + 1
            end)
            SameHoverButton:SetScript("OnLeave", function()
                __same_hover_leave = (__same_hover_leave or 0) + 1
            end)
            __same_hover_enter = 0
            __same_hover_leave = 0
            "#,
        )
        .expect("same-hover test setup should succeed");
    }

    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));
    drain_mouse_move_dirty(&app);

    app.handle_mouse_move(Point::new(160.0, 160.0));
    let (dirty_mask, dirty_ids) = drain_mouse_move_dirty(&app);
    let (enter_count, leave_count): (f64, f64) = app
        .env
        .borrow()
        .eval("return __same_hover_enter, __same_hover_leave")
        .expect("same-hover counters should be readable");

    assert_eq!(enter_count, 1.0, "OnEnter should only fire once");
    assert_eq!(
        leave_count, 0.0,
        "OnLeave should not fire inside same frame"
    );
    assert_eq!(dirty_mask, 0, "same-hover movement should not dirty strata");
    assert!(
        dirty_ids.is_some_and(|ids| ids.is_empty()),
        "same-hover movement should not dirty frame IDs"
    );
}

fn drain_mouse_move_dirty(app: &App) -> (u16, Option<rustc_hash::FxHashSet<u64>>) {
    app.env
        .borrow()
        .state()
        .borrow()
        .widgets
        .take_render_dirty_with_ids()
}
