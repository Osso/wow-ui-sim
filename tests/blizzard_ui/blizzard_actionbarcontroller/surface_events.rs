//! Event-registration surface for `Blizzard_ActionBarController`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ActionBarController";
const FRAME_NAME: &str = "ActionBarController";
const ONLOAD_LUA_SITE: &str = "ActionBarController.lua:8";
const REGISTERED_EVENTS: &[&str] = &[
    "PLAYER_ENTERING_WORLD",
    "ACTIONBAR_PAGE_CHANGED",
    "UPDATE_BONUS_ACTIONBAR",
    "UNIT_DISPLAYPOWER",
    "UPDATE_VEHICLE_ACTIONBAR",
    "UPDATE_OVERRIDE_ACTIONBAR",
    "UPDATE_SHAPESHIFT_FORM",
    "UPDATE_SHAPESHIFT_FORMS",
    "UPDATE_SHAPESHIFT_USABLE",
    "UPDATE_INVENTORY_ALERTS",
    "UPDATE_POSSESS_BAR",
    "UPDATE_EXTRA_ACTIONBAR",
    "ACTIONBAR_SHOW_BOTTOMLEFT",
    "PET_BATTLE_CLOSE",
    "PET_BATTLE_OPENING_START",
    "SETTINGS_LOADED",
];

#[cfg(feature = "optional-tests")]
#[test]
fn action_bar_controller_registers_all_onload_events() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for event in REGISTERED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("ActionBarController:IsEventRegistered must run cleanly");

            assert!(
                registered,
                "Expected `{FRAME_NAME}:IsEventRegistered({event:?})` to be true after \
                 `{ROOT}` loads. `{ONLOAD_LUA_SITE}` registers the controller event set; \
                 false means the XML OnLoad did not run, RegisterEvent regressed, or \
                 vendor source changed."
            );
        }
    });
}
