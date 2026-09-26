use super::*;
use crate::screen::ScreenKind;
use crate::texture::TextureManager;
use iced::Size;
use tokio::sync::mpsc;

fn build_env() -> Rc<RefCell<WowLuaEnv>> {
    Rc::new(RefCell::new(
        WowLuaEnv::new().expect("Failed to create Lua environment"),
    ))
}

fn build_test_app(screen_kind: ScreenKind) -> App {
    let env = build_env();
    env.borrow().set_screen_mode(screen_kind);
    env.borrow().set_screen_size(800.0, 600.0);

    let texture_manager = Rc::new(RefCell::new(TextureManager::new()));
    let font_system = Rc::new(RefCell::new(WowFontSystem::new()));
    let glyph_atlas = Rc::new(RefCell::new(GlyphAtlas::new()));
    let (_cmd_tx, cmd_rx) = mpsc::channel(1);
    let (_lua_tx, lua_rx) = std::sync::mpsc::channel();

    App::build_app(AppInit {
        env,
        log_messages: Vec::new(),
        texture_manager,
        font_system,
        glyph_atlas,
        cmd_rx,
        lua_rx,
        debug_borders: false,
        debug_anchors: false,
        saved_vars: None,
        config: crate::config::SimConfig::default(),
    })
}

#[test]
fn glue_screens_tick_on_update_at_interactive_rate() {
    let app = build_test_app(ScreenKind::CharacterSelect);
    app.strata_dirty.set(0);

    assert_eq!(app.screen_size.get(), Size::new(800.0, 600.0));
    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_millis(33)),
    );
}

#[test]
fn game_screen_keeps_idle_on_update_heartbeat() {
    let app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_secs(1)),
    );
}

#[test]
fn active_cooldown_widget_uses_fast_tick_interval() {
    let app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    app.env
        .borrow()
        .exec(
            r#"
            local cooldown = CreateFrame("Cooldown", "FastTickCooldown", UIParent)
            cooldown:SetCooldown(GetTime(), 30)
        "#,
        )
        .expect("active cooldown should be created");

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_millis(DEFAULT_FAST_TICK_MS)),
    );
}

#[test]
fn slow_mod_rate_cooldown_keeps_fast_tick_after_real_duration() {
    let app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    app.env
        .borrow()
        .exec(
            r#"
            local cooldown = CreateFrame("Cooldown", "SlowModRateCooldown", UIParent)
            cooldown:SetCooldown(GetTime() - 11, 10, 0.5)
        "#,
        )
        .expect("slow mod-rate cooldown should be created");

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_millis(DEFAULT_FAST_TICK_MS)),
    );
}

#[test]
fn fast_mod_rate_completed_cooldown_returns_to_idle_tick() {
    let app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    app.env
        .borrow()
        .exec(
            r#"
            local cooldown = CreateFrame("Cooldown", "FastModRateDoneCooldown", UIParent)
            cooldown:SetCooldown(GetTime() - 5.1, 10, 2)
        "#,
        )
        .expect("fast mod-rate cooldown should be created");

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_secs(1)),
    );
}

#[test]
fn active_cooldown_under_hidden_parent_keeps_idle_tick() {
    let app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    app.env
        .borrow()
        .exec(
            r#"
            local parent = CreateFrame("Frame", "HiddenCooldownParent", UIParent)
            parent:Hide()
            local cooldown = CreateFrame("Cooldown", "HiddenParentCooldown", parent)
            cooldown:SetCooldown(GetTime(), 30)
        "#,
        )
        .expect("hidden cooldown should be created");

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_secs(1)),
    );
}

#[test]
fn tick_marks_active_cooldown_dirty_until_it_finishes() {
    let app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            DirtyTickCooldown = CreateFrame("Cooldown", "DirtyTickCooldown", UIParent)
            DirtyTickCooldown:SetCooldown(GetTime(), 30)
            IdleTickCooldown = CreateFrame("Cooldown", "IdleTickCooldown", UIParent)
        "#,
        )
        .expect("cooldowns should be created");
    let (active_id, idle_id) = {
        let env = app.env.borrow();
        let state = env.state().borrow();
        let _ = state.widgets.take_render_dirty_with_ids();
        (
            state.widgets.get_id_by_name("DirtyTickCooldown").unwrap(),
            state.widgets.get_id_by_name("IdleTickCooldown").unwrap(),
        )
    };
    let take_dirty_ids = || {
        let env = app.env.borrow();
        let state = env.state().borrow();
        state
            .widgets
            .take_render_dirty_with_ids()
            .1
            .unwrap_or_default()
    };

    app.mark_active_cooldown_widgets_dirty();
    let dirty = take_dirty_ids();
    assert!(dirty.contains(&active_id), "active cooldown must redraw");
    assert!(!dirty.contains(&idle_id), "idle cooldown must not redraw");

    app.env
        .borrow()
        .exec("DirtyTickCooldown:Clear()")
        .expect("cooldown should clear");
    let _ = take_dirty_ids();
    app.mark_active_cooldown_widgets_dirty();
    assert!(
        take_dirty_ids().contains(&active_id),
        "just-finished cooldown gets one final redraw"
    );

    app.mark_active_cooldown_widgets_dirty();
    assert!(!take_dirty_ids().contains(&active_id));
}

#[test]
fn registry_tracks_cooldown_ids_across_reregistration() {
    use crate::widget::{Frame, WidgetRegistry, WidgetType};
    let mut registry = WidgetRegistry::new();
    let cooldown = Frame::new(WidgetType::Cooldown, None, None);
    let cooldown_id = cooldown.id;
    registry.register(cooldown);
    registry.register(Frame::new(WidgetType::Frame, None, None));
    assert_eq!(
        registry.cooldown_ids().collect::<Vec<_>>(),
        vec![cooldown_id]
    );

    let mut replacement = Frame::new(WidgetType::Frame, None, None);
    replacement.id = cooldown_id;
    registry.register(replacement);
    assert_eq!(registry.cooldown_ids().count(), 0);
}

#[test]
fn gui_startup_uses_first_real_canvas_size_for_display_size_changed() {
    let app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            __startup_display_width = nil
            __startup_display_height = nil
            local frame = CreateFrame("Frame")
            frame:RegisterEvent("DISPLAY_SIZE_CHANGED")
            frame:SetScript("OnEvent", function()
                __startup_display_width = GetScreenWidth()
                __startup_display_height = GetScreenHeight()
            end)
            "#,
        )
        .expect("startup size recorder should install");

    app.ensure_gui_startup_for_canvas_size(Size::new(1266.0, 822.0));

    let (width, height): (f64, f64) = app
        .env
        .borrow()
        .eval("return __startup_display_width, __startup_display_height")
        .expect("startup size should be readable");
    assert_eq!(width, 1266.0);
    assert_eq!(height, 822.0);
}

#[test]
fn app_screen_size_starts_from_sim_state_size() {
    let app = build_test_app(ScreenKind::Game);

    assert_eq!(app.screen_size.get(), current_env_screen_size(&app.env));
}

#[test]
fn gui_startup_drains_ready_timers_before_interactive_ticks() {
    let env = build_env();
    env.borrow()
        .exec(
            r#"
            __gui_startup_timer_fired = 0
            C_Timer.After(0, function()
                __gui_startup_timer_fired = __gui_startup_timer_fired + 1
            end)
            "#,
        )
        .expect("startup timer setup should succeed");

    App::run_startup_sequence(&env);

    let fired: f64 = env
        .borrow()
        .eval("return __gui_startup_timer_fired")
        .expect("startup timer result should be readable");
    assert_eq!(fired, 1.0, "ready startup timers should be settled");
}

#[test]
fn gui_startup_settles_bounded_on_update_work_before_interactive_ticks() {
    let env = build_env();
    env.borrow()
        .exec(
            r#"
            __gui_startup_on_update_fired = 0
            local frame = CreateFrame("Frame")
            frame:SetScript("OnUpdate", function(self)
                __gui_startup_on_update_fired = __gui_startup_on_update_fired + 1
                if __gui_startup_on_update_fired == 3 then
                    self:SetScript("OnUpdate", nil)
                end
            end)
            "#,
        )
        .expect("startup OnUpdate setup should succeed");

    App::run_startup_sequence(&env);

    let fired: f64 = env
        .borrow()
        .eval("return __gui_startup_on_update_fired")
        .expect("startup OnUpdate result should be readable");
    assert_eq!(
        fired, 3.0,
        "bounded startup OnUpdate work should be settled before GUI ticks"
    );
}

#[test]
fn gui_startup_closes_windows_created_by_startup_timers() {
    let env = build_env();
    env.borrow()
        .exec(
            r#"
            C_Timer.After(0, function()
                Baganator_WelcomeFrame = CreateFrame("Frame", "Baganator_WelcomeFrame", UIParent)
                Baganator_WelcomeFrame:Show()
            end)
            "#,
        )
        .expect("startup timer window setup should succeed");

    App::run_startup_sequence(&env);

    let shown: bool = env
        .borrow()
        .eval("return Baganator_WelcomeFrame and Baganator_WelcomeFrame:IsShown() or false")
        .expect("startup timer window visibility should be readable");
    assert!(
        !shown,
        "startup cleanup should also close windows created by startup timers"
    );
}

#[test]
fn parse_fast_tick_ms_accepts_positive_integers() {
    assert_eq!(parse_fast_tick_ms("1"), Some(1));
    assert_eq!(parse_fast_tick_ms(" 8 "), Some(8));
}

#[test]
fn parse_fast_tick_ms_rejects_zero_and_invalid_values() {
    assert_eq!(parse_fast_tick_ms("0"), None);
    assert_eq!(parse_fast_tick_ms("abc"), None);
    assert_eq!(parse_fast_tick_ms(""), None);
}

/// Plays a looping 0.75s alpha pulse on a texture under a parent with the given alpha.
fn play_pulse_under_parent_alpha(app: &App, parent_alpha: f32) {
    app.env
        .borrow()
        .exec(&format!(
            r#"
            local parent = CreateFrame("Frame", nil, UIParent)
            parent:SetSize(100, 100)
            parent:SetPoint("CENTER")
            parent:SetAlpha({parent_alpha})
            local glow = parent:CreateTexture(nil, "ARTWORK")
            glow:SetAllPoints()
            local group = glow:CreateAnimationGroup()
            group:SetLooping("BOUNCE")
            local fade = group:CreateAnimation("Alpha")
            fade:SetFromAlpha(0.25)
            fade:SetToAlpha(0.5)
            fade:SetDuration(0.75)
            group:Play()
        "#
        ))
        .expect("pulse animation should play");
}

#[test]
fn visible_animation_uses_fast_tick_interval() {
    let app = build_test_app(ScreenKind::Game);
    play_pulse_under_parent_alpha(&app, 1.0);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_millis(DEFAULT_FAST_TICK_MS)),
    );
}

#[test]
fn animation_under_transparent_parent_wakes_at_loop_boundary() {
    let app = build_test_app(ScreenKind::Game);
    play_pulse_under_parent_alpha(&app, 0.0);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    // 0.75s to the next bounce: the widest timer bucket not past the boundary.
    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_millis(250)),
    );
}
