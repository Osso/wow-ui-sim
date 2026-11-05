//! Event-registration surface for `Blizzard_ArrowCalloutFrame`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::load_addon;

const ROOT: &str = "Blizzard_ArrowCalloutFrame";
const MANAGER_FRAME: &str = "ArrowCalloutFrameManager";
const REGISTERED_EVENTS: &[&str] = &[
    "SHOW_ARROW_CALLOUT",
    "HIDE_ARROW_CALLOUT",
    "PLAYER_SOFT_INTERACT_CHANGED",
];

// Wrath has no ArrowCallout vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn arrow_callout_frame_manager_registers_onload_events() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_arrow_callout_frame(env);

        let state = env.state();
        let state = state.borrow();
        let manager = state
            .widgets
            .get_by_name(MANAGER_FRAME)
            .unwrap_or_else(|| panic!("`{MANAGER_FRAME}` must exist after `{ROOT}` loads"));

        for event in REGISTERED_EVENTS {
            assert!(
                manager.registered_events.contains(*event),
                "`{MANAGER_FRAME}` must register `{event}` during `ArrowCalloutMixin:OnLoad`"
            );
        }

        assert_eq!(
            manager.registered_events.len(),
            REGISTERED_EVENTS.len(),
            "`{MANAGER_FRAME}` must register exactly the expected ArrowCalloutFrame events"
        );
    });
}
}

fn load_arrow_callout_frame(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    load_addon(&env.loader_env(), &arrow_callout_toc())
        .expect("Blizzard_ArrowCalloutFrame should load directly from its TOC");
}

fn arrow_callout_toc() -> std::path::PathBuf {
    blizzard_ui_dir()
        .join(ROOT)
        .join("Blizzard_ArrowCalloutFrame.toc")
}
