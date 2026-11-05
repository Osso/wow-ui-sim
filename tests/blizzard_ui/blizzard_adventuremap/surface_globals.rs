//! Global-surface probes for `Blizzard_AdventureMap`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_AdventureMap";

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_registers_uipanel_window_entry() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let surface: UIPanelWindowSurface = env
            .eval(
                r#"
                local entry = UIPanelWindows and UIPanelWindows["AdventureMapFrame"]
                return type(entry),
                       entry and entry.area or nil,
                       entry and entry.pushable or nil,
                       entry and entry.allowOtherPanels or nil,
                       type(entry and entry.showFailedFunc),
                       entry and entry.showFailedFunc == C_AdventureMap.Close
                "#,
            )
            .expect("AdventureMap UIPanelWindows surface probe must run cleanly");

        assert_uipanel_window_surface(surface);
    });
}
}

type UIPanelWindowSurface = (String, String, i64, i64, String, bool);

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_exports_quest_choice_result_constants() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let constants: QuestChoiceResultConstants = env
            .eval(
                r#"
                return QUEST_CHOICE_DIALOG_RESULT_ACCEPTED,
                       QUEST_CHOICE_DIALOG_RESULT_DECLINED,
                       QUEST_CHOICE_DIALOG_RESULT_ABSTAIN
                "#,
            )
            .expect("AdventureMap quest-choice result constant probe must run cleanly");

        assert_eq!(
            constants,
            (1, 2, 3),
            "AdventureMap quest-choice result constants must keep their published values"
        );
    });
}
}

type QuestChoiceResultConstants = (i64, i64, i64);

// Wrath has no AdventureMap vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_exports_utility_helper_functions() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        for helper_name in ADVENTURE_MAP_UTILITY_HELPERS {
            let probe = format!("return type(_G[{helper_name:?}])");
            let actual_type: String = env
                .eval(&probe)
                .expect("AdventureMap utility helper type probe must run cleanly");

            assert_eq!(
                actual_type, "function",
                "`{helper_name}` must be exported as a function"
            );
        }
    });
}
}

const ADVENTURE_MAP_UTILITY_HELPERS: &[&str] = &[
    "AdventureMap_IsQuestValid",
    "AdventureMap_IsZoneIDBlockedByZoneChoice",
    "AdventureMap_IsPositionBlockedByZoneChoice",
];

fn assert_uipanel_window_surface(surface: UIPanelWindowSurface) {
    let (
        entry_type,
        area,
        pushable,
        allow_other_panels,
        show_failed_func_type,
        show_failed_func_is_close,
    ) = surface;

    assert_eq!(
        entry_type, "table",
        "`UIPanelWindows[\"AdventureMapFrame\"]` must be registered as a table"
    );
    assert_eq!(
        area, "center",
        "`AdventureMapFrame` panel area must be center"
    );
    assert_eq!(pushable, 0, "`AdventureMapFrame` pushable value must be 0");
    assert_eq!(
        allow_other_panels, 1,
        "`AdventureMapFrame` must allow other panels"
    );
    assert_eq!(
        show_failed_func_type, "function",
        "`AdventureMapFrame.showFailedFunc` must hold a function reference"
    );
    assert!(
        show_failed_func_is_close,
        "`AdventureMapFrame.showFailedFunc` must be the loaded `C_AdventureMap.Close` reference"
    );
}
