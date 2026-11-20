use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn dispatcher_toc() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
    .join("Blizzard_Dispatcher/Blizzard_Dispatcher.toc")
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn boost_tutorial_toc() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
    .join("Blizzard_BoostTutorial/Blizzard_BoostTutorial.toc")
}

fn env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    load_addon(&env.loader_env(), &dispatcher_toc()).expect("Failed to load Blizzard_Dispatcher");
    env.exec("Dispatcher:Initialize()")
        .expect("Failed to initialize Dispatcher");
    env
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn dispatcher_routes_cinematic_event_to_blizzard_tutorial_consumer() {
    let env = env();
    load_addon(&env.loader_env(), &boost_tutorial_toc())
        .expect("Failed to load Blizzard_BoostTutorial");
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "DispatcherConsumerFrame", UIParent)
        frame:Hide()
        NPE_TutorialMainFrame:Initialize(frame)
        frame:Show()
        DispatcherConsumerShownBefore = frame:IsShown()
        Dispatcher:OnEvent("CINEMATIC_START")
        DispatcherConsumerHidden = not frame:IsShown()
        "#,
    )
    .expect("Blizzard tutorial consumer event should dispatch");

    let (shown_before, hidden_after): (bool, bool) = env
        .eval("return DispatcherConsumerShownBefore, DispatcherConsumerHidden")
        .expect("consumer effect query should succeed");
    assert!(
        shown_before,
        "consumer frame must be shown before event dispatch"
    );
    assert!(
        hidden_after,
        "CINEMATIC_START must hide the tutorial consumer frame"
    );
}

#[test]
fn dispatcher_event_supports_object_methods_and_unregister_all() {
    let env = env();
    env.exec(
        r#"
        DispatcherEventTest = {
            count = 0,
            PLAYER_LOGIN = function(self)
                self.count = self.count + 1
            end,
        }

        Dispatcher:RegisterEvent("PLAYER_LOGIN", DispatcherEventTest)
        "#,
    )
    .unwrap();

    env.exec("Dispatcher:OnEvent(\"PLAYER_LOGIN\")").unwrap();
    let count: i32 = env.eval("return DispatcherEventTest.count").unwrap();
    assert_eq!(count, 1, "object event method should run when fired");

    env.exec("Dispatcher:UnregisterAll(DispatcherEventTest)")
        .unwrap();
    env.exec("Dispatcher:OnEvent(\"PLAYER_LOGIN\")").unwrap();
    let count: i32 = env.eval("return DispatcherEventTest.count").unwrap();
    assert_eq!(
        count, 1,
        "UnregisterAll should remove the object's event registrations"
    );
}

#[test]
fn dispatcher_function_hooks_global_and_restores_on_unregister_all() {
    let env = env();
    let (before, during, after): (i32, i32, i32) = env
        .eval(
            r#"
            DispatcherFunctionTest = {
                count = 0,
                ToggleBackpack = function(self)
                    self.count = self.count + 1
                end,
            }

            function ToggleBackpack()
                DISPATCHER_FUNCTION_BASE = (DISPATCHER_FUNCTION_BASE or 0) + 10
            end

            ToggleBackpack()
            local before = DISPATCHER_FUNCTION_BASE

            Dispatcher:RegisterFunction("ToggleBackpack", DispatcherFunctionTest)
            ToggleBackpack()
            local during = DISPATCHER_FUNCTION_BASE + DispatcherFunctionTest.count

            Dispatcher:UnregisterAll(DispatcherFunctionTest)
            ToggleBackpack()
            local after = DISPATCHER_FUNCTION_BASE + DispatcherFunctionTest.count

            return before, during, after
            "#,
        )
        .unwrap();
    assert_eq!(before, 10, "original function should run before hooking");
    assert_eq!(
        during, 21,
        "hooked function should call the original and then the object method"
    );
    assert_eq!(
        after, 31,
        "after unregistering, only the original function should continue running"
    );
}

#[test]
fn dispatcher_script_hooks_frame_and_once_unhooks_after_first_run() {
    let env = env();
    let (first, second): (i32, i32) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "DispatcherScriptFrame", UIParent)
            frame:Hide()

            DispatcherScriptTest = {
                count = 0,
                OnShow = function(self)
                    self.count = self.count + 1
                end,
            }

            Dispatcher:RegisterScript(frame, "OnShow", DispatcherScriptTest, true)
            frame:Show()
            local first = DispatcherScriptTest.count
            local handler = frame:GetScript("OnShow")
            handler(frame)
            frame:Hide()
            handler(frame)
            return first, DispatcherScriptTest.count
            "#,
        )
        .unwrap();
    assert_eq!(first, 1, "script hook should fire on first show");
    assert_eq!(second, 1, "one-shot script hook should remove itself");
}

#[test]
fn dispatcher_on_update_once_runs_only_once() {
    let env = env();
    env.exec(
        r#"
        DispatcherOnUpdateTest = {
            count = 0,
            OnUpdate = function(self, elapsed)
                self.count = self.count + 1
            end,
        }

        Dispatcher:RegisterEvent("OnUpdate", DispatcherOnUpdateTest, true)
        "#,
    )
    .unwrap();

    env.exec("Dispatcher:OnEvent(\"OnUpdate\", 0.016)").unwrap();
    env.exec("Dispatcher:OnEvent(\"OnUpdate\", 0.016)").unwrap();

    let count: i32 = env.eval("return DispatcherOnUpdateTest.count").unwrap();
    assert_eq!(
        count, 1,
        "OnUpdate one-shot dispatcher callbacks should unhook after first tick"
    );
}
