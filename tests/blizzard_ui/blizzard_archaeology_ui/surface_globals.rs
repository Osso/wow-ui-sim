//! Public globals for `Blizzard_ArchaeologyUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_ArchaeologyUI";
const GLOBAL_FUNCTIONS: &[&str] = &[
    "ArchaeologyFrame_Show",
    "ArchaeologyFrame_Hide",
    "ArchaeologyFrame_OnLoad",
    "ArchaeologyFrame_OnEvent",
    "ArchaeologyFrame_ShowFailed",
    "ArchaeologyFrame_ShowArtifact",
    "ArchaeologyFrame_UpdateSummary",
    "ArchaeologyFrame_UpdateComplete",
    "ArchaeologyFrame_CurrentArtifactUpdate",
];
const MODULE_CONSTANTS: &[(&str, i32)] = &[
    ("ARCHAEOLOGY_BUTTON_HEIGHT", 59),
    ("ARCHAEOLOGY_MAX_RACES", 12),
    ("ARCHAEOLOGY_MAX_STONES", 4),
    ("ARCHAEOLOGY_MAX_COMPLETED_SHOWN", 12),
    ("ARCHAEOLOGY_HELP_TAB", 0),
    ("ARCHAEOLOGY_SUMMARY_TAB", 1),
    ("ARCHAEOLOGY_COMPLETED_TAB", 2),
    ("ARCHAEOLOGY_SUMMARY_PAGE", 1),
    ("ARCHAEOLOGY_COMPLETED_PAGE", 2),
    ("ARCHAEOLOGY_CURRENT_PAGE", 3),
];

#[cfg(feature = "optional-tests")]
#[test]
fn archaeology_ui_publishes_expected_global_functions() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for global_name in GLOBAL_FUNCTIONS {
            let actual_type = global_type(env, global_name);

            assert_eq!(
                actual_type, "function",
                "`{global_name}` must be published as a global function after `{ROOT}` loads"
            );
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn archaeology_ui_publishes_module_constants() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for (global_name, expected_value) in MODULE_CONSTANTS {
            let actual_value = global_i32(env, global_name);

            assert_eq!(
                actual_value, *expected_value,
                "`{global_name}` must match Blizzard_ArchaeologyUI.lua's module constant"
            );
        }
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn archaeology_ui_registers_uipanel_window_entry() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let surface: UIPanelWindowSurface = env
            .eval(
                r#"
                local entry = UIPanelWindows and UIPanelWindows["ArchaeologyFrame"]
                return type(entry),
                       entry and entry.area or nil,
                       entry and entry.pushable or nil,
                       type(entry and entry.showFailedFunc),
                       entry and entry.showFailedFunc == ArchaeologyFrame_ShowFailed
                "#,
            )
            .expect("ArchaeologyFrame UIPanelWindows surface probe must run cleanly");

        assert_uipanel_window_surface(surface);
    });
}

type UIPanelWindowSurface = (String, String, i32, String, bool);

fn global_type(env: &WowLuaEnv, global_name: &str) -> String {
    env.eval(&format!("return type(_G[{global_name:?}])"))
        .unwrap_or_else(|err| panic!("failed to probe global type for `{global_name}`: {err}"))
}

fn global_i32(env: &WowLuaEnv, global_name: &str) -> i32 {
    env.eval(&format!("return _G[{global_name:?}]"))
        .unwrap_or_else(|err| panic!("failed to probe integer global `{global_name}`: {err}"))
}

fn assert_uipanel_window_surface(surface: UIPanelWindowSurface) {
    let (entry_type, area, pushable, show_failed_func_type, show_failed_func_is_expected) = surface;

    assert_eq!(
        entry_type, "table",
        "`UIPanelWindows[\"ArchaeologyFrame\"]` must be registered as a table"
    );
    assert_eq!(area, "left", "`ArchaeologyFrame` panel area must be left");
    assert_eq!(pushable, 3, "`ArchaeologyFrame` pushable value must be 3");
    assert_eq!(
        show_failed_func_type, "function",
        "`ArchaeologyFrame.showFailedFunc` must hold a function reference"
    );
    assert!(
        show_failed_func_is_expected,
        "`ArchaeologyFrame.showFailedFunc` must be `ArchaeologyFrame_ShowFailed`"
    );
}
