//! Event-registration surface for `Blizzard_ArchaeologyUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ArchaeologyUI";
const FRAME_NAME: &str = "ArchaeologyFrame";
const REGISTERED_EVENTS: &[&str] = &[
    "RESEARCH_ARTIFACT_UPDATE",
    "RESEARCH_ARTIFACT_COMPLETE",
    "RESEARCH_ARTIFACT_DIG_SITE_UPDATED",
    "CURRENCY_DISPLAY_UPDATE",
    "SKILL_LINES_CHANGED",
    "BAG_UPDATE_DELAYED",
    "GET_ITEM_INFO_RECEIVED",
];

#[cfg(feature = "optional-tests")]
#[test]
fn archaeology_frame_registers_onload_events() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for event in REGISTERED_EVENTS {
            let registered: bool = env
                .eval(&format!(
                    "return _G[{FRAME_NAME:?}]:IsEventRegistered({event:?})"
                ))
                .expect("ArchaeologyFrame:IsEventRegistered probe must run cleanly");

            assert!(
                registered,
                "`{FRAME_NAME}:IsEventRegistered({event:?})` must be true after `{ROOT}` OnLoad"
            );
        }
    });
}
