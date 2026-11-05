//! Global surface probes for `Blizzard_ArrowCalloutFrame`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use crate::common::panel_fixtures::blizzard_ui_dir;
use wow_ui_sim::loader::load_addon;

const ROOT: &str = "Blizzard_ArrowCalloutFrame";
const ARROW_CALLOUT_MIXIN_METHODS: &[&str] =
    &["OnLoad", "OnEvent", "HideCallout", "Setup", "AnchorCallout"];
const CONTAINER_MIXIN_METHODS: &[&str] = &["OnLoad", "Setup"];
const CLOSE_BUTTON_MIXIN_METHODS: &[&str] = &["OnClick"];
const WIDGET_CONTAINER_MIXIN_METHODS: &[&str] = &["Setup"];

// Wrath has no ArrowCallout vendor widget/handler input; mixed and modeled probes remain enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn arrow_callout_frame_exports_expected_mixin_globals() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_arrow_callout_frame(env);

        assert_mixin_methods(env, "ArrowCalloutMixin", ARROW_CALLOUT_MIXIN_METHODS);
        assert_mixin_methods(env, "ArrowCalloutContainerMixin", CONTAINER_MIXIN_METHODS);
        assert_mixin_methods(
            env,
            "ArrowCalloutCloseButtonMixin",
            CLOSE_BUTTON_MIXIN_METHODS,
        );
        assert_mixin_methods(
            env,
            "WidgetContainerCalloutTemplateMixin",
            WIDGET_CONTAINER_MIXIN_METHODS,
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

fn assert_mixin_methods(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    mixin_name: &str,
    method_names: &[&str],
) {
    let probe = mixin_surface_probe(mixin_name, method_names);
    let surface: MixinSurface = env
        .eval(&probe)
        .unwrap_or_else(|error| panic!("{mixin_name} surface probe must run cleanly: {error}"));
    let (mixin_type, missing_methods) = surface;

    assert_eq!(mixin_type, "table", "`{mixin_name}` must be a table");
    assert!(
        missing_methods.is_empty(),
        "`{mixin_name}` must expose methods {method_names:?}; missing or wrong types: {:?}",
        missing_methods
    );
}

fn mixin_surface_probe(mixin_name: &str, method_names: &[&str]) -> String {
    let method_list = method_names
        .iter()
        .map(|method_name| format!("{method_name:?}"))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
        local mixin = _G[{mixin_name:?}]
        local methods = {{{method_list}}}
        local missing = {{}}

        for _, methodName in ipairs(methods) do
            if type(mixin) ~= "table" or type(mixin[methodName]) ~= "function" then
                local methodType = type(mixin) == "table" and type(mixin[methodName]) or "nil"
                table.insert(missing, methodName .. ":" .. methodType)
            end
        end

        return type(mixin), missing
        "#
    )
}

type MixinSurface = (String, Vec<String>);
