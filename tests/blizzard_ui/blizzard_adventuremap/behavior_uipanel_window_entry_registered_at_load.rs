//! Load-time `UIPanelWindows` registration behavior for `Blizzard_AdventureMap`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const ROOT: &str = "Blizzard_AdventureMap";

// Wrath has no AdventureMap vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[test]
fn adventure_map_uipanel_window_entry_captures_close_at_load_time() {
    with_blizzard_addon_closure(&["Blizzard_UIParentPanelManager"], &[], |env, _loaded| {
        crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
            env, &crate::common::blizzard_addon_harness::blizzard_ui_dir(), &[ROOT], &[],
        );
        let surface: LoadTimePanelEntry = env
            .eval(
                r#"
                local entry = UIPanelWindows and UIPanelWindows["AdventureMapFrame"]
                local capturedClose = entry and entry.showFailedFunc

                local replacementCalled = false
                C_AdventureMap.Close = function()
                    replacementCalled = true
                end

                capturedClose()

                return type(entry),
                       type(capturedClose),
                       capturedClose == C_AdventureMap.Close,
                       replacementCalled
                "#,
            )
            .expect("AdventureMap UIPanelWindows load-time probe must run cleanly");

        assert_load_time_panel_entry(surface);

        assert!(
            env.state().borrow().adventure_map.last_closed.is_some(),
            "`showFailedFunc` must call the load-time `C_AdventureMap.Close` function reference"
        );
    });
}
}

type LoadTimePanelEntry = (String, String, bool, bool);

fn assert_load_time_panel_entry(surface: LoadTimePanelEntry) {
    let (entry_type, captured_close_type, is_late_bound_close, replacement_called) = surface;

    assert_eq!(
        entry_type, "table",
        "`UIPanelWindows[\"AdventureMapFrame\"]` must be populated while the addon loads"
    );
    assert_eq!(
        captured_close_type, "function",
        "`AdventureMapFrame.showFailedFunc` must store a function at load time"
    );
    assert!(
        !is_late_bound_close,
        "`showFailedFunc` must keep the direct function reference captured during load"
    );
    assert!(
        !replacement_called,
        "`showFailedFunc` must not resolve `C_AdventureMap.Close` late after replacement"
    );
}
