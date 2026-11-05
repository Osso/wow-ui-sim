//! Public mixin methods for `Blizzard_AdventureMap`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_AdventureMap";
const ADVENTURE_MAP_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "RefreshInsets",
    "IsMapInsetExpanded",
    "SetupTitle",
    "AddStandardDataProviders",
    "ClearAreaTableIDAvailableForInsets",
    "SetAreaTableIDAvailableForInsets",
];
const ADVENTURE_MAP_INSET_MIXIN_METHODS: &[&str] = &[
    "Initialize",
    "OnReleased",
    "BuildDetailTiles",
    "OnCollapseExpandAnimFinished",
    "SyncAnimation",
    "Collapse",
    "Expand",
    "OnAreaEnclosedChanged",
    "OnCanvasScaleChanged",
    "GetMap",
    "SetLocalPinPosition",
    "GetGlobalPosition",
    "OnMouseEnter",
    "OnMouseLeave",
];
const ADVENTURE_MAP_QUEST_CHOICE_DIALOG_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnParentHide",
    "ShowWithQuest",
    "SetPortraitAtlas",
    "OnEvent",
    "OnShow",
    "OnHide",
    "Finalize",
    "Refresh",
    "RefreshRewards",
    "RefreshDetails",
    "AddReward",
    "AcceptQuest",
    "DeclineQuest",
];

// Wrath has no AdventureMap vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_mixin_exposes_plan_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_mixin_methods(env, "AdventureMapMixin", ADVENTURE_MAP_MIXIN_METHODS);
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_inset_mixin_exposes_plan_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_mixin_methods(
            env,
            "AdventureMapInsetMixin",
            ADVENTURE_MAP_INSET_MIXIN_METHODS,
        );
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn adventure_map_quest_choice_dialog_mixin_exposes_plan_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_mixin_methods(
            env,
            "AdventureMapQuestChoiceDialogMixin",
            ADVENTURE_MAP_QUEST_CHOICE_DIALOG_MIXIN_METHODS,
        );
    });
}
}

fn assert_mixin_methods(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    mixin_name: &str,
    method_names: &[&str],
) {
    for method_name in method_names {
        let actual_type = probe_mixin_method_type(env, mixin_name, method_name);

        assert_eq!(
            actual_type, "function",
            "`{mixin_name}.{method_name}` must be exposed as a function"
        );
    }
}

fn probe_mixin_method_type(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    mixin_name: &str,
    method_name: &str,
) -> String {
    env.eval(&format!("return type({mixin_name}[{method_name:?}])"))
        .unwrap_or_else(|err| panic!("failed to probe `{mixin_name}.{method_name}`: {err}"))
}
