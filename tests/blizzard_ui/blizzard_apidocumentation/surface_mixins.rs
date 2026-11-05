//! Mixin surface probes for `Blizzard_APIDocumentation`.

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, with_blizzard_addon_startup_shape,
};
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors,
};

const ROOT: &str = "Blizzard_APIDocumentation";

const API_DOCUMENTATION_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "HandleSlashCommand",
    "HandleAPILink",
    "HandleDefaultCommand",
    "HandleCopyAPI",
    "HandleOpenDump",
    "FindAPIByName",
    "GetAPITableByTypeName",
    "OutputUsage",
    "OutputStats",
    "OutputAllSystems",
    "TryHandlingSystemSearchCommand",
    "OutputAPIMatches",
    "OutputAllAPIMatches",
    "OutputAllSystemAPIMatches",
    "OutputAllSystemAPI",
    "AddAllMatches",
    "FindAllAPIMatches",
    "FindSystemByName",
    "AddDocumentationTable",
    "WriteLine",
    "WriteLineF",
    "WriteAllLines",
    "GetIndentString",
    "AddTable",
    "AddFunction",
    "AddEvent",
    "AddField",
    "AddSystem",
];

const BASE_API_MIXIN_METHODS: &[&str] = &[
    "GetType",
    "GetPrettyType",
    "GetLinkHexColor",
    "GetName",
    "GetFullName",
    "GetParentName",
    "GetLoweredParentName",
    "GetLoweredName",
    "GetClipboardString",
    "GenerateAPILink",
    "GetSingleOutputLine",
    "GetDetailedOutputLines",
    "MatchesSearchString",
    "MatchesName",
    "MatchesNameCaseInsenstive",
    "MatchesAnyAPI",
    "MatchesAnyDocumentation",
    "AddDocumentationTags",
    "AddSystemTag",
];

const SYSTEMS_API_MIXIN_OVERRIDES: &[&str] = &[
    "GetType",
    "GetLinkHexColor",
    "GetNamespaceName",
    "GetLoweredNamespaceName",
    "MatchesSearchString",
    "GetSingleOutputLine",
    "GetDetailedOutputLines",
    "MatchesName",
    "MatchesNameCaseInsenstive",
    "FindAllAPIMatches",
    "ListAllAPI",
    "GetNumTables",
    "GetNumFunctions",
    "GetNumEvents",
];

const API_KIND_MIXIN_TYPES: &[(&str, &str)] = &[
    ("FunctionsAPIMixin", "function"),
    ("EventsAPIMixin", "event"),
    ("FieldsAPIMixin", "field"),
    ("TablesAPIMixin", "table"),
];

// APIDocumentation vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn api_documentation_mixin_exposes_expected_methods() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);

        for method_name in API_DOCUMENTATION_MIXIN_METHODS {
            let method_type: String = env
                .eval(&format!(
                    r#"return type(APIDocumentationMixin["{method_name}"])"#
                ))
                .expect("APIDocumentationMixin method type probe must run cleanly");

            assert_eq!(
                "function", method_type,
                "APIDocumentationMixin.{method_name} must be a function"
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn base_api_mixin_exposes_expected_methods() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);

        for method_name in BASE_API_MIXIN_METHODS {
            let method_type: String = env
                .eval(&format!(r#"return type(BaseAPIMixin["{method_name}"])"#))
                .expect("BaseAPIMixin method type probe must run cleanly");

            assert_eq!(
                "function", method_type,
                "BaseAPIMixin.{method_name} must be a function"
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn systems_api_mixin_inherits_base_and_exposes_expected_overrides() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);

        let inherited_pretty_type: bool = env
            .eval("return SystemsAPIMixin.GetPrettyType == BaseAPIMixin.GetPrettyType")
            .expect("SystemsAPIMixin inheritance probe must run cleanly");

        assert!(
            inherited_pretty_type,
            "SystemsAPIMixin must inherit methods from BaseAPIMixin"
        );

        let system_type: String = env
            .eval("return SystemsAPIMixin:GetType()")
            .expect("SystemsAPIMixin:GetType probe must run cleanly");

        assert_eq!(
            "system", system_type,
            "SystemsAPIMixin:GetType() must return the API kind string"
        );

        for method_name in SYSTEMS_API_MIXIN_OVERRIDES {
            let method_type: String = env
                .eval(&format!(r#"return type(SystemsAPIMixin["{method_name}"])"#))
                .expect("SystemsAPIMixin method type probe must run cleanly");

            assert_eq!(
                "function", method_type,
                "SystemsAPIMixin.{method_name} must be a function"
            );
        }
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn api_kind_mixins_return_dispatch_type_strings() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);

        for (mixin_name, expected_type) in API_KIND_MIXIN_TYPES {
            let actual_type: String = env
                .eval(&format!(r#"return {mixin_name}:GetType()"#))
                .expect("API kind mixin GetType probe must run cleanly");

            assert_eq!(
                *expected_type, actual_type,
                "{mixin_name}:GetType() must return the slash-dispatcher API kind"
            );
        }
    });
}
}

fn load_api_documentation(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    clear_recorded_lua_errors(env);
    let ui_dir = blizzard_ui_dir();
    let loaded = load_blizzard_addon_closure_into_env(env, &ui_dir, &[ROOT], &[]);

    assert!(
        loaded.iter().any(|addon| addon == ROOT),
        "{ROOT} must be included in the loaded addon closure; loaded={loaded:?}"
    );

    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "{ROOT} must settle without recorded Lua errors:\n  {}",
        errors.join("\n  ")
    );
}
