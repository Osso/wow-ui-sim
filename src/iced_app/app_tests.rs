use super::*;
use crate::screen::ScreenKind;
use crate::texture::TextureManager;
use iced::Size;
use std::time::Instant;
use tokio::sync::mpsc;

fn build_env() -> Rc<RefCell<WowLuaEnv>> {
    Rc::new(RefCell::new(
        WowLuaEnv::new().expect("Failed to create Lua environment"),
    ))
}

#[test]
fn gui_configuration_preserves_resolved_class_and_race() {
    let env = build_env();
    {
        let env = env.borrow();
        let mut state = env.state().borrow_mut();
        state.player.class_index = 1;
        state.player.race_index = 1;
    }
    let mut config = crate::config::SimConfig::default();
    config.movement.moving = true;
    assert_eq!(config.player_class, "Paladin");
    assert_eq!(config.player_race, "Human");

    App::apply_config_to_state(&env, &config);

    let env = env.borrow();
    let identity: (String, String) = env
        .eval("local _, class = UnitClass('player'); local _, race = UnitRace('player'); return class, race")
        .unwrap();
    assert_eq!(identity, ("WARRIOR".into(), "Orc".into()));
    assert!(env.state().borrow().player.movement.moving);
}

#[test]
fn gui_selectors_use_resolved_class_and_race() {
    let env = build_env();
    {
        let env = env.borrow();
        let mut state = env.state().borrow_mut();
        state.player.class_index = 8;
        state.player.race_index = 3;
    }
    let config = crate::config::SimConfig::default();
    let selections = {
        let env = env.borrow();
        let state = env.state().borrow();
        AppInitialSelections::from_state_and_config(&state, &config)
    };

    assert_eq!(selections.class, "Mage");
    assert_eq!(selections.race, "Night Elf");
    assert_eq!(selections.xp_level, "33%");
}

pub(super) fn build_test_app(screen_kind: ScreenKind) -> App {
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

pub(super) fn draw_test_canvas(app: &App, size: Size) -> crate::render::WowUiPrimitive {
    iced::widget::shader::Program::draw(
        &app,
        &(),
        iced::mouse::Cursor::Unavailable,
        iced::Rectangle::with_size(size),
    )
}

#[test]
fn reload_ui_shortcut_discards_lua_globals_and_old_frames() {
    let mut app = build_test_app(ScreenKind::Game);
    complete_gui_startup(&mut app);
    app.env
        .borrow()
        .exec(
            "reload_sentinel = true; CreateFrame('Frame', 'ReloadOldFrame'); \
               C_Timer.After(0, function() old_timer_fired = true end)",
        )
        .unwrap();
    let old_lua = Rc::downgrade(&app.env.borrow().lua);
    let textures = Rc::clone(&app.texture_manager);
    let fonts = Rc::clone(&app.font_system);
    {
        let env = app.env.borrow();
        let mut state = env.state().borrow_mut();
        state.player.level = 73;
        state.party_members.pop();
        state.world.zone_name = "Reload preservation zone".to_string();
    }

    let _ = app.update(super::super::Message::ReloadUI);
    complete_gui_startup(&mut app);

    assert!(old_lua.upgrade().is_none(), "old Lua VM must be destroyed");
    assert!(
        app.env
            .borrow()
            .eval::<bool>(
                "return reload_sentinel == nil and ReloadOldFrame == nil and old_timer_fired == nil"
            )
            .unwrap(),
        "ReloadUI must not preserve arbitrary globals, frames or timer callbacks"
    );
    assert!(
        app.saved_vars.is_none(),
        "disabled persistence must stay disabled"
    );
    assert!(Rc::ptr_eq(&textures, &app.texture_manager));
    assert!(Rc::ptr_eq(&fonts, &app.font_system));
    let env = app.env.borrow();
    let state = env.state().borrow();
    assert_eq!(state.player.level, 73);
    assert_eq!(state.party_members.len(), 3);
    assert_eq!(state.world.zone_name, "Reload preservation zone");
}

#[test]
fn reload_ui_saves_declared_variables_before_discarding_the_vm() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = build_test_app(ScreenKind::Game);
    complete_gui_startup(&mut app);
    let mut manager = SavedVariablesManager::with_storage_dir(directory.path().to_path_buf());
    manager
        .init_for_addon(
            app.env.borrow().rilua_mut().state_mut(),
            "ReloadPersistence",
            &["ReloadDB".to_string()],
            &[],
        )
        .unwrap();
    app.saved_vars = Some(manager);
    app.env
        .borrow()
        .exec("ReloadDB = { value = 73 }; reload_sentinel = true; ReloadUI()")
        .unwrap();

    let _ = app.update(super::super::Message::ProcessTimers(Instant::now()));
    complete_gui_startup(&mut app);
    assert!(
        app.env
            .borrow()
            .eval::<bool>("return reload_sentinel == nil")
            .unwrap(),
        "Lua-requested reload must be serviced by the App owner"
    );
    let saved = std::fs::read_to_string(directory.path().join("ReloadPersistence.lua")).unwrap();
    assert!(
        saved.contains("73"),
        "registered value must be written before teardown"
    );
    let mut reader = SavedVariablesManager::with_storage_dir(directory.path().to_path_buf());
    reader
        .init_for_addon(
            app.env.borrow().rilua_mut().state_mut(),
            "ReloadPersistence",
            &["ReloadDB".to_string()],
            &[],
        )
        .unwrap();
    assert_eq!(
        app.env
            .borrow()
            .eval::<i32>("return ReloadDB.value")
            .unwrap(),
        73
    );
}

#[test]
fn reload_ui_round_trips_scalar_and_locally_written_large_variables() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = build_test_app(ScreenKind::Game);
    complete_gui_startup(&mut app);
    let mut manager = SavedVariablesManager::with_storage_dir(directory.path().to_path_buf());
    manager.set_max_load_bytes(Some(16));
    let variables = ["ReloadScalar".to_string(), "ReloadLarge".to_string()];
    let per_character = ["ReloadChar".to_string()];
    let wtf_directory = tempfile::tempdir().unwrap();
    let config = crate::saved_variables::WtfConfig::new(
        wtf_directory.path(),
        "Account",
        "Realm",
        "Character",
    );
    let sources = [
        (
            config.account_saved_vars_file("ReloadPersistence"),
            "ReloadScalar=3\n",
        ),
        (
            config.character_saved_vars_file("ReloadPersistence"),
            "ReloadChar=5\n",
        ),
    ];
    for (path, content) in &sources {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    manager.set_wtf_config(config);
    manager
        .init_for_addon(
            app.env.borrow().rilua_mut().state_mut(),
            "ReloadPersistence",
            &variables,
            &per_character,
        )
        .unwrap();
    app.saved_vars = Some(manager);
    app.env
        .borrow()
        .exec("ReloadScalar = 73; ReloadChar = 77; ReloadLarge = { message = string.rep('saved', 100) }; ReloadUI()")
        .unwrap();
    let _ = app.update(super::super::Message::ProcessTimers(Instant::now()));
    complete_gui_startup(&mut app);
    let manager = app.saved_vars.as_mut().unwrap();
    manager
        .load_wtf_for_addon(
            app.env.borrow().rilua_mut().state_mut(),
            "ReloadPersistence",
        )
        .unwrap();
    manager
        .init_for_addon(
            app.env.borrow().rilua_mut().state_mut(),
            "ReloadPersistence",
            &variables,
            &per_character,
        )
        .unwrap();
    assert!(app.env.borrow().eval::<bool>(
        "return ReloadScalar == 73 and ReloadChar == 77 and ReloadLarge ~= nil and ReloadLarge.message == string.rep('saved', 100)").unwrap(),
        "account/character reload data must beat stale WTF, retain scalars and bypass the import cap");
    for (path, content) in sources {
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            content,
            "imported WTF must stay unchanged"
        );
    }
}

#[test]
fn reload_ui_requested_from_timer_retires_the_callback_environment() {
    let mut app = build_test_app(ScreenKind::Game);
    complete_gui_startup(&mut app);
    app.env
        .borrow()
        .exec("reload_sentinel = true; C_Timer.After(0, function() ReloadUI() end)")
        .unwrap();
    let old_lua = Rc::downgrade(&app.env.borrow().lua);
    let _ = app.update(super::super::Message::ProcessTimers(Instant::now()));
    complete_gui_startup(&mut app);
    assert!(
        old_lua.upgrade().is_none(),
        "timer-originated reload must retire its VM"
    );
    assert!(
        app.env
            .borrow()
            .eval::<bool>("return reload_sentinel == nil")
            .unwrap()
    );
}

pub(super) fn complete_gui_startup(app: &mut App) {
    app.record_canvas_size(app.screen_size.get());
    for _ in 0..200 {
        if app.gui_startup_complete.get() {
            return;
        }
        let _ = app.update(super::super::Message::ProcessTimers(
            std::time::Instant::now(),
        ));
    }
    panic!("bounded GUI startup did not complete");
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
        Some(std::time::Duration::from_micros(16_667)),
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
        Some(std::time::Duration::from_micros(16_667)),
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
fn gui_startup_zero_delay_chain_yields_to_queued_exec() {
    let mut app = build_test_app(ScreenKind::Game);
    let (commands, receiver) = std::sync::mpsc::channel();
    app.lua_rx = Some(receiver);
    app.env
        .borrow()
        .exec(
            r#"
        startup_trace = {}
        startup_ticker_count = 0
        startup_exec_count = 0
        C_Timer.After(0, function()
            table.insert(startup_trace, "A")
            C_Timer.After(0, function() table.insert(startup_trace, "B") end)
        end)
        C_Timer.NewTicker(0, function()
            startup_ticker_count = startup_ticker_count + 1
        end, 3)
    "#,
        )
        .unwrap();

    // Real shader draw records the first canvas; the update turn runs generation A.
    let _ = draw_test_canvas(&app, Size::new(1266.0, 822.0));
    let _ = app.update(super::super::Message::ProcessTimers(
        std::time::Instant::now(),
    ));
    let (trace, ticks): (String, i32) = app
        .env
        .borrow()
        .eval("return table.concat(startup_trace, ','), startup_ticker_count")
        .unwrap();
    assert_eq!(trace, "A", "first generation must return before chained B");
    assert_eq!(ticks, 1, "zero ticker fires once per startup generation");

    // The controller queues real Exec only after the first turn returned.
    let (respond, response) = std::sync::mpsc::channel();
    commands
        .send(crate::lua_server::LuaCommand::Exec {
            code: "startup_exec_count = startup_exec_count + 1; table.insert(startup_trace, 'Exec'); print(table.concat(startup_trace, ','))"
                .into(),
            respond,
        })
        .unwrap();
    let _ = app.update(super::super::Message::ProcessTimers(
        std::time::Instant::now(),
    ));
    match response.try_recv().expect("queued Exec must be answered") {
        crate::lua_server::Response::Output(output) => {
            let lines: Vec<_> = output.lines().collect();
            assert!(!lines.is_empty(), "Exec must return its trace output");
            assert!(lines.iter().all(|line| *line == "A,Exec"), "{output}");
        }
        other => panic!("unexpected Exec response: {other:?}"),
    }
    let (trace, ticks): (String, i32) = app
        .env
        .borrow()
        .eval("return table.concat(startup_trace, ','), startup_ticker_count")
        .unwrap();
    assert_eq!(
        trace, "A,Exec,B",
        "B runs on the later app turn, after Exec"
    );
    assert_eq!(ticks, 2);
    let exec_count: i32 = app.env.borrow().eval("return startup_exec_count").unwrap();
    assert_eq!(exec_count, 1, "queued Exec body must execute exactly once");
}

#[test]
fn gui_startup_uses_first_real_canvas_size_for_display_size_changed() {
    let mut app = build_test_app(ScreenKind::Game);
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

    let _ = draw_test_canvas(&app, Size::new(1266.0, 822.0));
    complete_gui_startup(&mut app);

    let (width, height): (f64, f64) = app
        .env
        .borrow()
        .eval("return __startup_display_width, __startup_display_height")
        .expect("startup size should be readable");
    assert_eq!(width, 1266.0);
    assert_eq!(height, 822.0);
}

#[test]
fn gui_startup_draw_defers_lua_and_keeps_event_order_once() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
        startup_events = {}
        startup_sizes = {}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("DISPLAY_SIZE_CHANGED")
        frame:RegisterEvent("VARIABLES_LOADED")
        frame:RegisterEvent("PLAYER_LOGIN")
        frame:RegisterEvent("PLAYER_ENTERING_WORLD")
        frame:RegisterEvent("FIRST_FRAME_RENDERED")
        frame:SetScript("OnEvent", function(_, event)
            if event == "DISPLAY_SIZE_CHANGED" then
                table.insert(startup_sizes, GetScreenWidth())
            else
                table.insert(startup_events, event)
            end
        end)
    "#,
        )
        .unwrap();
    let size = Size::new(1266.0, 822.0);
    let primitive = draw_test_canvas(&app, size);
    assert!(
        primitive
            .strata_batches
            .iter()
            .flatten()
            .all(|batch| batch.vertices.is_empty())
    );
    let events: String = app
        .env
        .borrow()
        .eval("return table.concat(startup_events, ',')")
        .unwrap();
    assert_eq!(events, "", "draw must not dispatch startup Lua");
    let sizes: i32 = app.env.borrow().eval("return #startup_sizes").unwrap();
    assert_eq!(sizes, 0, "draw must not dispatch resize Lua either");

    complete_gui_startup(&mut app);
    // Further draws and ordinary ticks cannot replay the startup events.
    let _ = draw_test_canvas(&app, size);
    let _ = app.update(super::super::Message::ProcessTimers(
        std::time::Instant::now(),
    ));
    let events: String = app
        .env
        .borrow()
        .eval("return table.concat(startup_events, ',')")
        .unwrap();
    assert_eq!(
        events,
        "VARIABLES_LOADED,PLAYER_LOGIN,PLAYER_ENTERING_WORLD,FIRST_FRAME_RENDERED"
    );
    let widths_match: bool = app
        .env
        .borrow()
        .eval(
            r#"
        if #startup_sizes == 0 then return false end
        for _, width in ipairs(startup_sizes) do
            if width ~= 1266 then return false end
        end
        return true
    "#,
        )
        .unwrap();
    assert!(widths_match);

    let _ = draw_test_canvas(&app, Size::new(1100.0, 700.0));
    let width: f64 = app.env.borrow().eval("return GetScreenWidth()").unwrap();
    assert_eq!(width, 1266.0, "resize draw must not run Lua");
    let _ = app.update(super::super::Message::ProcessTimers(
        std::time::Instant::now(),
    ));
    let width: f64 = app
        .env
        .borrow()
        .eval("return startup_sizes[#startup_sizes]")
        .unwrap();
    assert_eq!(width, 1100.0, "the update path must dispatch the resize");
}

#[test]
fn gui_startup_branching_zero_delay_chain_spans_turns() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
        branches = {}
        local function branch(name, depth)
            C_Timer.After(0, function()
                table.insert(branches, name)
                if depth > 0 then
                    branch(name .. "L", depth - 1)
                    branch(name .. "R", depth - 1)
                end
            end)
        end
        branch("root", 2)
    "#,
        )
        .unwrap();
    let _ = draw_test_canvas(&app, Size::new(800.0, 600.0));
    for expected in [
        "root",
        "root,rootL,rootR",
        "root,rootL,rootR,rootLL,rootLR,rootRL,rootRR",
    ] {
        let _ = app.update(super::super::Message::ProcessTimers(
            std::time::Instant::now(),
        ));
        let trace: String = app
            .env
            .borrow()
            .eval("return table.concat(branches, ',')")
            .unwrap();
        assert_eq!(trace, expected);
    }
    complete_gui_startup(&mut app);
    let count: i32 = app.env.borrow().eval("return #branches").unwrap();
    assert_eq!(count, 7);
}

#[test]
fn gui_startup_waits_for_canvas_before_running_lua() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            "canvas_wait_fired = false; C_Timer.After(0, function() canvas_wait_fired = true end)",
        )
        .unwrap();
    let _ = app.update(super::super::Message::ProcessTimers(
        std::time::Instant::now(),
    ));
    let fired: bool = app.env.borrow().eval("return canvas_wait_fired").unwrap();
    assert!(!fired);
    assert!(!app.gui_startup_complete.get());
    complete_gui_startup(&mut app);
    let fired: bool = app.env.borrow().eval("return canvas_wait_fired").unwrap();
    assert!(fired);
}

#[test]
fn app_screen_size_starts_from_sim_state_size() {
    let app = build_test_app(ScreenKind::Game);

    assert_eq!(app.screen_size.get(), current_env_screen_size(&app.env));
}

#[test]
fn gui_startup_settles_ready_timers_at_completion() {
    let mut app = build_test_app(ScreenKind::Game);
    let env = Rc::clone(&app.env);
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

    complete_gui_startup(&mut app);

    let fired: f64 = env
        .borrow()
        .eval("return __gui_startup_timer_fired")
        .expect("startup timer result should be readable");
    assert_eq!(fired, 1.0, "ready startup timers should be settled");
}

#[test]
fn gui_startup_settles_bounded_on_update_work_at_completion() {
    let mut app = build_test_app(ScreenKind::Game);
    let env = Rc::clone(&app.env);
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

    complete_gui_startup(&mut app);

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
    let mut app = build_test_app(ScreenKind::Game);
    let env = Rc::clone(&app.env);
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

    complete_gui_startup(&mut app);
    let created: bool = env
        .borrow()
        .eval("return Baganator_WelcomeFrame ~= nil")
        .unwrap();
    assert!(created, "startup timer must actually create the window");

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
    play_pulse_with_setup(app, parent_alpha, "");
}

/// Same pulse, running `setup` (with `group` and `fade` in scope) before Play.
fn play_pulse_with_setup(app: &App, parent_alpha: f32, setup: &str) {
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
            {setup}
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
        Some(std::time::Duration::from_micros(16_667)),
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

#[test]
fn unseen_animation_with_on_update_handler_keeps_fast_tick() {
    for setup in [
        r#"fade:SetScript("OnUpdate", function() end)"#,
        r#"group:SetScript("OnUpdate", function() end)"#,
    ] {
        let app = build_test_app(ScreenKind::Game);
        play_pulse_with_setup(&app, 0.0, setup);
        app.strata_dirty.set(0);
        app.textures_pending.set(false);

        // OnUpdate runs every tick, so its handler needs the frame rate.
        assert_eq!(
            app.compute_tick_interval(),
            Some(std::time::Duration::from_micros(16_667)),
            "{setup}",
        );
    }
}

/// Plays a parentless 0.5s REPEAT timer made of a plain Animation (the
/// DandersFrames range-poll pattern), running `setup` before Play.
fn play_plain_timer_animation(app: &App, setup: &str) {
    app.env
        .borrow()
        .exec(&format!(
            r#"
            local timerFrame = CreateFrame("Frame")
            local group = timerFrame:CreateAnimationGroup()
            local step = group:CreateAnimation()
            step:SetDuration(0.5)
            group:SetLooping("REPEAT")
            group:SetScript("OnLoop", function() end)
            {setup}
            group:Play()
        "#
        ))
        .expect("plain timer animation should play");
}

#[test]
fn plain_animation_timer_wakes_at_loop_boundary() {
    let app = build_test_app(ScreenKind::Game);
    play_plain_timer_animation(&app, "");
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    // 0.5s to the next loop: the widest timer bucket not past the boundary.
    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_millis(250)),
    );
}

#[test]
fn plain_animation_timer_with_on_update_keeps_fast_tick() {
    let app = build_test_app(ScreenKind::Game);
    play_plain_timer_animation(&app, r#"step:SetScript("OnUpdate", function() end)"#);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_micros(16_667)),
    );
}

#[test]
fn per_frame_ticker_keeps_sixty_hertz_tick() {
    let app = build_test_app(ScreenKind::Game);
    app.strata_dirty.set(0);
    app.textures_pending.set(false);

    // Blizzard's SmoothStatusBar pattern: a 0-interval ticker for the session.
    app.env
        .borrow()
        .exec("C_Timer.NewTicker(0, function() end)")
        .expect("ticker should be created");

    assert_eq!(
        app.compute_tick_interval(),
        Some(std::time::Duration::from_micros(16_667)),
    );
}
