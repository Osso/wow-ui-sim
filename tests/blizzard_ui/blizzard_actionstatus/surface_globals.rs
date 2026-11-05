//! Surface-global probes for `Blizzard_ActionStatus`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ActionStatus";
const ACTION_STATUS_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "OnUpdate",
    "SetAlternateParentFrame",
    "ClearAlternateParentFrame",
    "DisplayMessage",
    "GetBestParent",
    "UpdateParent",
];

#[cfg(feature = "optional-tests")]
#[test]
fn action_status_mixin_surface_global_exposes_expected_methods() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let is_table: bool = env
            .eval(r#"return type(ActionStatusMixin) == "table""#)
            .expect("ActionStatusMixin type probe must run cleanly");

        assert!(
            is_table,
            "`ActionStatusMixin` must be a table after `{ROOT}` loads"
        );

        for method_name in ACTION_STATUS_MIXIN_METHODS {
            let is_function = action_status_mixin_method_is_function(env, method_name);

            assert!(
                is_function,
                "`ActionStatusMixin.{method_name}` must be a function after `{ROOT}` loads"
            );
        }
    });
}

fn action_status_mixin_method_is_function(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    method_name: &str,
) -> bool {
    env.eval(&format!(
        r#"return type(ActionStatusMixin["{method_name}"]) == "function""#
    ))
    .unwrap_or_else(|err| panic!("ActionStatusMixin.{method_name} type probe failed: {err}"))
}
