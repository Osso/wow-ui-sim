use super::app_tests::build_test_app;
use super::*;
use crate::iced_app::Message;
use crate::screen::ScreenKind;
use iced_runtime::Action;
use iced_runtime::futures::futures::StreamExt;
use rilua::LuaApiMut;

fn assert_exit(task: Task<Message>) {
    let action = pollster::block_on(async {
        iced_runtime::task::into_stream(task)
            .expect("finish must request exit")
            .next()
            .await
            .unwrap()
    });
    assert!(matches!(action, Action::Exit), "{action:?}");
}

fn persisted_app(directory: &std::path::Path) -> App {
    let mut app = build_test_app(ScreenKind::Game);
    app.gui_startup_complete.set(true);
    app.gui_initialization_complete.set(true);
    let mut manager = SavedVariablesManager::with_storage_dir(directory.to_owned());
    manager
        .init_for_addon(
            app.env.borrow().rilua_mut().state_mut(),
            "SessionProbe",
            &["SessionDB".into()],
            &[],
        )
        .unwrap();
    app.env.borrow().exec("SessionDB = 73").unwrap();
    app.saved_vars = Some(manager);
    app
}

#[test]
fn close_quit_and_ctrl_q_finalize_once_before_exiting() {
    for route in ["close", "quit", "ctrl-q"] {
        let directory = tempfile::tempdir().unwrap();
        let mut app = persisted_app(directory.path());
        let message = match route {
            "close" => Message::CloseRequested,
            "quit" => {
                app.env.borrow().exec("Quit()").unwrap();
                Message::FpsTick
            }
            _ => Message::KeyPress("CTRL-Q".into(), None, std::time::Instant::now()),
        };
        assert_exit(app.update(message));
        assert_eq!(*app.completion.borrow(), Some(Ok(())), "{route}");
        assert_eq!(super::super::ui_completion_result(&app.completion), Ok(()));
        let path = directory.path().join("SessionProbe.lua");
        let saved = std::fs::read(&path).unwrap();
        let mut readback = rilua::Lua::new().unwrap();
        readback.exec_bytes(&saved, "@session-readback").unwrap();
        assert_eq!(readback.get_global_val("SessionDB"), rilua::Val::Num(73.0));

        app.env.borrow().exec("SessionDB = 99").unwrap();
        assert_exit(app.update(Message::CloseRequested));
        drop(app);
        assert_eq!(std::fs::read(path).unwrap(), saved, "{route}");
    }
}

#[test]
fn failed_close_reaches_the_caller_and_drop_does_not_retry() {
    let directory = tempfile::tempdir().unwrap();
    let blocked = directory.path().join("not-a-directory");
    let mut app = persisted_app(&blocked);
    std::fs::write(&blocked, b"preserve").unwrap();

    assert_exit(app.update(Message::CloseRequested));
    let failure = super::super::ui_completion_result(&app.completion).unwrap_err();
    assert!(!failure.is_empty());
    assert_eq!(*app.completion.borrow(), Some(Err(failure)));

    std::fs::remove_file(&blocked).unwrap();
    drop(app);
    assert!(!blocked.exists(), "Drop must not retry a failed save");
}

#[test]
fn close_during_startup_rejects_incomplete_state_without_saving() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = persisted_app(directory.path());
    app.gui_startup_complete.set(false);
    app.gui_initialization_complete.set(false);
    assert_exit(app.update(Message::CloseRequested));
    assert!(
        super::super::ui_completion_result(&app.completion)
            .unwrap_err()
            .contains("before startup initialization completed")
    );
    assert!(!directory.path().join("SessionProbe.lua").exists());
}

#[test]
fn failed_reload_preserves_previous_save_and_allows_repair() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = persisted_app(directory.path());
    let path = directory.path().join("SessionProbe.lua");
    let prior = b"SessionDB = 37\n";
    std::fs::write(&path, prior).unwrap();
    app.env
        .borrow()
        .exec("SessionDB = function() end; ReloadUI()")
        .unwrap();

    let _ = app.update(Message::FpsTick);
    assert!(!app.is_finishing());
    assert!(
        app.log_messages
            .iter()
            .any(|message| message.contains("ReloadUI failed"))
    );
    assert_eq!(std::fs::read(&path).unwrap(), prior);
    assert!(!app.env.borrow().state().borrow().ui_reload_requested);
    app.env.borrow_mut().set_reload_loader(|env, manager| {
        manager.as_mut().unwrap().init_for_addon(
            env.rilua_mut().state_mut(),
            "SessionProbe",
            &["SessionDB".into()],
            &[],
        )
    });
    app.env.borrow().exec("SessionDB = 89; ReloadUI()").unwrap();
    let _ = app.update(Message::FpsTick);
    super::app_tests::complete_gui_startup(&mut app);
    assert_eq!(
        app.env.borrow().eval::<i32>("return SessionDB").unwrap(),
        89
    );
    assert_exit(app.update(Message::CloseRequested));
    assert_eq!(super::super::ui_completion_result(&app.completion), Ok(()));
}

#[test]
fn runtime_exit_without_finalization_is_not_successful_completion() {
    let completion = UiCompletion::default();
    assert!(
        super::super::ui_completion_result(&completion)
            .unwrap_err()
            .contains("without completing session finalization")
    );
}

#[test]
fn quit_from_ipc_stops_later_commands_before_finalization() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = persisted_app(directory.path());
    let (commands, receiver) = std::sync::mpsc::channel();
    app.lua_rx = Some(receiver);
    let (first_response, first) = std::sync::mpsc::channel();
    let (late_response, late) = std::sync::mpsc::channel();
    for (code, respond) in [
        ("Quit()", first_response),
        ("SessionDB = 99", late_response),
    ] {
        commands
            .send(crate::lua_server::LuaCommand::Exec {
                code: code.into(),
                respond,
            })
            .unwrap();
    }

    assert_exit(app.update(Message::FpsTick));
    assert!(matches!(
        first.try_recv(),
        Ok(crate::lua_server::Response::Output(_))
    ));
    assert!(matches!(
        late.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Disconnected)
    ));
    assert_eq!(
        app.env.borrow().eval::<i32>("return SessionDB").unwrap(),
        73
    );
    assert_eq!(*app.completion.borrow(), Some(Ok(())));
}

#[test]
fn startup_callback_failure_prevents_successful_completion() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = persisted_app(directory.path());
    app.gui_startup_complete.set(false);
    app.gui_initialization_complete.set(false);
    app.env
        .borrow()
        .exec(
            "local f = CreateFrame('Frame'); f:RegisterEvent('PLAYER_LOGIN'); \
         f:SetScript('OnEvent', function() error('startup callback probe') end)",
        )
        .unwrap();
    app.record_canvas_size(app.screen_size.get());
    let _ = app.update(Message::ProcessTimers(std::time::Instant::now()));

    assert_exit(app.update(Message::CloseRequested));
    assert!(
        super::super::ui_completion_result(&app.completion)
            .unwrap_err()
            .contains("startup callback probe")
    );
    assert!(!directory.path().join("SessionProbe.lua").exists());
}

#[test]
fn quit_after_essential_startup_does_not_wait_for_gui_settling() {
    for trigger in ["login", "timer"] {
        let directory = tempfile::tempdir().unwrap();
        let mut app = persisted_app(directory.path());
        app.gui_startup_complete.set(false);
        app.gui_initialization_complete.set(false);
        let setup = if trigger == "login" {
            "local f = CreateFrame('Frame'); f:RegisterEvent('PLAYER_LOGIN'); \
             f:SetScript('OnEvent', function() \
                 SessionDB = 89; \
                 local group = f:CreateAnimationGroup(); \
                 group:CreateAnimation('Alpha'):SetDuration(0.1); \
                 group:SetScript('OnFinished', function() SessionDB = 999 end); \
                 group:Play(); Quit() \
             end)"
        } else {
            "C_Timer.After(0, function() SessionDB = 89; Quit() end)"
        };
        app.env.borrow().exec(setup).unwrap();
        app.record_canvas_size(app.screen_size.get());

        assert_exit(app.update(Message::ProcessTimers(std::time::Instant::now())));
        assert!(app.gui_initialization_complete.get(), "{trigger}");
        assert!(!app.gui_startup_complete.get(), "{trigger}");
        assert_eq!(super::super::ui_completion_result(&app.completion), Ok(()));
        let mut readback = rilua::Lua::new().unwrap();
        readback
            .exec(&std::fs::read_to_string(directory.path().join("SessionProbe.lua")).unwrap())
            .unwrap();
        assert_eq!(readback.get_global_val("SessionDB"), rilua::Val::Num(89.0));
    }
}

#[test]
fn quit_before_startup_without_persisted_data_is_a_normal_exit() {
    let mut app = build_test_app(ScreenKind::Game);
    assert!(!app.gui_initialization_complete.get());
    app.env.borrow().exec("Quit()").unwrap();
    assert_exit(app.update(Message::FpsTick));
    assert_eq!(super::super::ui_completion_result(&app.completion), Ok(()));
}

#[test]
fn pending_exec_lua_quit_skips_later_tick_callbacks() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = persisted_app(directory.path());
    app.last_on_update_time = std::time::Instant::now() - std::time::Duration::from_millis(20);
    app.env.borrow().exec(
        "local f = CreateFrame('Frame'); f:SetScript('OnUpdate', function() SessionDB = 99 end); \
         C_Timer.After(0, function() SessionDB = 100 end)",
    ).unwrap();
    app.pending_exec_lua = Some(("Quit()".into(), false));
    assert_exit(app.update(Message::ProcessTimers(std::time::Instant::now())));
    assert_eq!(
        app.env.borrow().eval::<i32>("return SessionDB").unwrap(),
        73
    );
    assert_eq!(super::super::ui_completion_result(&app.completion), Ok(()));
}

#[test]
fn failed_rebuild_stays_interactive_for_reload_without_running_partial_callbacks() {
    let directory = tempfile::tempdir().unwrap();
    let mut app = persisted_app(directory.path());
    let attempts = Rc::new(std::cell::Cell::new(0));
    app.env.borrow_mut().set_reload_loader({
        let attempts = attempts.clone();
        move |env, manager| {
            attempts.set(attempts.get() + 1);
            manager.as_mut().unwrap().init_for_addon(
                env.rilua_mut().state_mut(), "SessionProbe", &["SessionDB".into()], &[],
            )?;
            if attempts.get() == 1 {
                env.exec(
                    "SessionDB = 999; PartialUpdateRan = false; \
                     local f = CreateFrame('Frame'); f:SetScript('OnUpdate', function() PartialUpdateRan = true end)",
                )?;
                return Err(crate::Error::Other("rebuild probe".into()));
            }
            Ok(())
        }
    });
    app.env.borrow().exec("ReloadUI()").unwrap();
    let _ = app.update(Message::FpsTick);
    assert!(!app.is_finishing());
    assert!(!app.env.borrow().is_ui_environment_usable());
    let path = directory.path().join("SessionProbe.lua");
    let preceding = std::fs::read(&path).unwrap();
    let _ = app.update(Message::ProcessTimers(std::time::Instant::now()));
    assert!(
        !app.env
            .borrow()
            .eval::<bool>("return PartialUpdateRan")
            .unwrap()
    );
    assert!(
        app.env
            .borrow_mut()
            .finalize_session(&mut app.saved_vars)
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), preceding);

    let _ = app.update(Message::ReloadUI);
    assert_eq!(attempts.get(), 2);
    super::app_tests::complete_gui_startup(&mut app);
    assert!(app.env.borrow().is_ui_environment_usable());
    assert_eq!(
        app.env.borrow().eval::<i32>("return SessionDB").unwrap(),
        73
    );
    assert_exit(app.update(Message::CloseRequested));
    assert_eq!(super::super::ui_completion_result(&app.completion), Ok(()));
}
