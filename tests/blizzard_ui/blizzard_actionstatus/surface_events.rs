//! Event-registration surface for `Blizzard_ActionStatus`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ActionStatus";
const FRAME_NAME: &str = "ActionStatus";
const ONLOAD_LUA_SITE: &str = "ActionStatus.lua:7-15";
const ALTERNATE_PARENT_EVENT: &str = "UI.AlternateTopLevelParentChanged";
const GAME_SCREEN_EVENTS: &[&str] = &[
    "SCREENSHOT_STARTED",
    "SCREENSHOT_SUCCEEDED",
    "SCREENSHOT_FAILED",
];
const GLUE_SCREEN_EVENTS: &[&str] = &[
    "GLUE_SCREENSHOT_STARTED",
    "GLUE_SCREENSHOT_SUCCEEDED",
    "GLUE_SCREENSHOT_FAILED",
];

#[cfg(feature = "optional-tests")]
#[test]
fn action_status_registers_game_screenshot_events_outside_glue() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        assert_in_glue(env, false);
        assert_events_registered(env, GAME_SCREEN_EVENTS);
        assert_events_not_registered(env, GLUE_SCREEN_EVENTS);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn action_status_alternate_top_level_parent_callback_updates_parent() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let adopted_new_parent: bool = env
            .eval(
                r#"
                local parent = CreateFrame("Frame", "ActionStatusAlternateTopLevelProbe", UIParent)
                parent:Show()

                GetAppropriateTopLevelParent = function()
                    return parent
                end

                EventRegistry:TriggerEvent("UI.AlternateTopLevelParentChanged")
                return ActionStatus:GetParent() == parent
                "#,
            )
            .expect("alternate top-level parent callback probe must run cleanly");

        assert!(
            adopted_new_parent,
            "`{ROOT}` OnLoad must register `{ALTERNATE_PARENT_EVENT}` with \
             `EventRegistry:RegisterCallback`, forwarding to `ActionStatus:UpdateParent()`"
        );
    });
}

fn assert_in_glue(env: &wow_ui_sim::lua_api::WowLuaEnv, expected: bool) {
    let in_glue: bool = env
        .eval("return InGlue()")
        .expect("InGlue probe must run cleanly");

    assert_eq!(
        in_glue, expected,
        "`{ROOT}` event probe must run in the game-screen branch"
    );
}

fn assert_events_registered(env: &wow_ui_sim::lua_api::WowLuaEnv, events: &[&str]) {
    for event_name in events {
        let registered = action_status_is_event_registered(env, event_name);

        assert!(
            registered,
            "Expected `{FRAME_NAME}:IsEventRegistered({event_name:?})` to be true after \
             `{ROOT}` loads outside glue. `{ONLOAD_LUA_SITE}` registers the game screenshot \
             event set when `InGlue()` is false."
        );
    }
}

fn assert_events_not_registered(env: &wow_ui_sim::lua_api::WowLuaEnv, events: &[&str]) {
    for event_name in events {
        let registered = action_status_is_event_registered(env, event_name);

        assert!(
            !registered,
            "Expected `{FRAME_NAME}:IsEventRegistered({event_name:?})` to be false after \
             `{ROOT}` loads outside glue. `{ONLOAD_LUA_SITE}` only registers the GLUE_ \
             screenshot event set when `InGlue()` is true."
        );
    }
}

fn action_status_is_event_registered(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    event_name: &str,
) -> bool {
    env.eval(&format!(
        "return _G[{FRAME_NAME:?}]:IsEventRegistered({event_name:?})"
    ))
    .unwrap_or_else(|err| {
        panic!("failed to probe `{FRAME_NAME}:IsEventRegistered({event_name:?})`: {err}")
    })
}
