//! Public mixin methods for `Blizzard_AddOnList`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_AddOnList";
const ADDON_LIST_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnUpdate",
    "GetAddonMetricPercent",
    "GetOverallMetric",
    "UpdateOverallMetric",
    "UpdatePerformance",
    "UpdateAddOnMemoryUsage",
];
const ADDON_LIST_NODE_MIXIN_METHODS: &[&str] = &["OnClick", "SetEnabledAll"];
const ADDON_LIST_ENTRY_MIXIN_METHODS: &[&str] = &["OnLoad", "SetEnabledDependencies"];
const ADDON_CATEGORY_COLLAPSE_EXPAND_MIXIN_METHODS: &[&str] =
    &["SetTreeNode", "OnClick", "ToggleState", "UpdateState"];

prefork_full_ui_case! {
fn addon_list_mixin_exposes_plan_methods(env: &WowLuaEnv) {
        assert_mixin_methods(env, "AddonListMixin", ADDON_LIST_MIXIN_METHODS);
    }
}

#[test]
fn addon_list_node_mixins_expose_plan_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        assert_mixin_methods(env, "AddonListNodeMixin", ADDON_LIST_NODE_MIXIN_METHODS);
        assert_mixin_methods(env, "AddonListEntryMixin", ADDON_LIST_ENTRY_MIXIN_METHODS);
        assert_mixin_methods(
            env,
            "AddonCategoryCollapseExpandMixin",
            ADDON_CATEGORY_COLLAPSE_EXPAND_MIXIN_METHODS,
        );
    });
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
