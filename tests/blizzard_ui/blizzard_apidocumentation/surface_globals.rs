//! Global surface probes for `Blizzard_APIDocumentation`.

use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, with_blizzard_addon_startup_shape,
};
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, recorded_lua_errors,
};

const ROOT: &str = "Blizzard_APIDocumentation";

// APIDocumentation vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn api_documentation_singleton_exposes_mixin_methods_and_empty_registries() {
    with_blizzard_addon_startup_shape(&[], &[], |env, _loaded| {
        load_api_documentation(env);

        let (has_mixin_methods, tables_empty, functions_empty, events_empty, systems_empty): (
            bool,
            bool,
            bool,
            bool,
            bool,
        ) = env
            .eval(
                r#"
                local methods = {
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
                }

                local hasMixinMethods = true
                for _, methodName in ipairs(methods) do
                    if APIDocumentation[methodName] ~= APIDocumentationMixin[methodName] then
                        hasMixinMethods = false
                        break
                    end
                end

                local function isEmptyTable(tableValue)
                    return type(tableValue) == "table" and next(tableValue) == nil
                end

                return hasMixinMethods,
                       isEmptyTable(APIDocumentation.tables),
                       isEmptyTable(APIDocumentation.functions),
                       isEmptyTable(APIDocumentation.events),
                       isEmptyTable(APIDocumentation.systems)
                "#,
            )
            .expect("APIDocumentation global surface probe must run cleanly");

        assert!(
            has_mixin_methods,
            "APIDocumentation must expose the methods from APIDocumentationMixin"
        );
        assert!(
            tables_empty,
            "APIDocumentation.tables must be empty after load"
        );
        assert!(
            functions_empty,
            "APIDocumentation.functions must be empty after load"
        );
        assert!(
            events_empty,
            "APIDocumentation.events must be empty after load"
        );
        assert!(
            systems_empty,
            "APIDocumentation.systems must be empty after load"
        );
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
