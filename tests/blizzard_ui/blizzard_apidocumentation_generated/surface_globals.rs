//! Global surface probes for `Blizzard_APIDocumentationGenerated`.

use crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env;
use crate::common::blizzard_addon_harness::new_blizzard_addon_env;
use crate::common::panel_fixtures::{
    blizzard_ui_dir, clear_recorded_lua_errors, load_panel_addons, recorded_lua_errors,
};

const ROOT: &str = "Blizzard_APIDocumentationGenerated";
const MIN_SYSTEMS: i64 = 300;
const MIN_TABLES: i64 = 1_500;
const MIN_FUNCTIONS: i64 = 5_000;
const MIN_EVENTS: i64 = 1_500;

#[cfg(feature = "optional-tests")]
#[test]
fn generated_api_documentation_populates_corpus_above_lower_bounds() {
    let env = load_generated_api_documentation();

    let counts: DocumentationCorpusCounts = env
        .eval::<(i64, i64, i64, i64)>(
            r#"
            return #APIDocumentation.systems,
                   #APIDocumentation.tables,
                   #APIDocumentation.functions,
                   #APIDocumentation.events
            "#,
        )
        .expect("generated APIDocumentation corpus counts must be readable")
        .into();

    assert!(
        counts.systems >= MIN_SYSTEMS,
        "expected at least {MIN_SYSTEMS} documented systems, got {}",
        counts.systems
    );
    assert!(
        counts.tables >= MIN_TABLES,
        "expected at least {MIN_TABLES} documented tables, got {}",
        counts.tables
    );
    assert!(
        counts.functions >= MIN_FUNCTIONS,
        "expected at least {MIN_FUNCTIONS} documented functions, got {}",
        counts.functions
    );
    assert!(
        counts.events >= MIN_EVENTS,
        "expected at least {MIN_EVENTS} documented events, got {}",
        counts.events
    );
}

#[cfg(feature = "optional-tests")]
#[test]
fn generated_api_documentation_entries_have_expected_mixin_shape() {
    let env = load_generated_api_documentation();

    let failure: String = env
        .eval(
            r#"
            local function validateCollection(collectionName, expectedType)
                local collection = APIDocumentation[collectionName]
                for index, entry in ipairs(collection) do
                    local okType, actualType = pcall(function()
                        return entry:GetType()
                    end)
                    if not okType or actualType ~= expectedType then
                        return string.format(
                            "%s[%d] expected GetType()=%q, got %q",
                            collectionName,
                            index,
                            expectedType,
                            tostring(actualType)
                        )
                    end

                    local okName, name = pcall(function()
                        return entry:GetName()
                    end)
                    if not okName or type(name) ~= "string" or name == "" then
                        return string.format(
                            "%s[%d] expected non-empty GetName(), got %q",
                            collectionName,
                            index,
                            tostring(name)
                        )
                    end
                end

                return nil
            end

            return validateCollection("systems", "system")
                or validateCollection("tables", "table")
                or validateCollection("functions", "function")
                or validateCollection("events", "event")
                or ""
            "#,
        )
        .expect("generated APIDocumentation mixin-shape probe must run cleanly");

    assert_eq!(
        "", failure,
        "every generated APIDocumentation entry must expose the expected mixin shape"
    );
}

#[cfg(feature = "optional-tests")]
#[test]
fn generated_api_documentation_registers_well_known_systems() {
    let env = load_generated_api_documentation();

    let failure: String = env
        .eval(
            r#"
            local expectedSystems = {
                "AbbreviateConfigAPI",
                "AccountInfo",
                "ChatInfo",
                "MapUI",
                "QuestLog",
            }

            for _, expectedName in ipairs(expectedSystems) do
                local system = APIDocumentation:FindSystemByName(expectedName)
                if system == nil then
                    return string.format("missing system %q", expectedName)
                end

                local actualName = system:GetName()
                if actualName ~= expectedName then
                    return string.format(
                        "system %q resolved to %q",
                        expectedName,
                        tostring(actualName)
                    )
                end
            end

            return ""
            "#,
        )
        .expect("generated APIDocumentation well-known system probe must run cleanly");

    assert_eq!(
        "", failure,
        "well-known generated APIDocumentation systems must be registered"
    );
}

#[cfg(feature = "optional-tests")]
#[test]
fn generated_api_documentation_registers_well_known_global_functions() {
    let env = load_generated_api_documentation();

    let failure: String = env
        .eval(
            r#"
            local expectedFunctions = {
                { name = "GetTime", hasArguments = false, hasReturns = true },
                { name = "UnitName", hasArguments = true, hasReturns = true },
                { name = "CreateFrame", hasArguments = false, hasReturns = false },
            }

            local function validateOptionalFieldArray(functionInfo, fieldName, shouldExist)
                local fieldValue = functionInfo[fieldName]
                if fieldValue == nil then
                    if shouldExist then
                        return string.format("%s missing %s", functionInfo:GetName(), fieldName)
                    end
                    return nil
                end

                if type(fieldValue) ~= "table" then
                    return string.format(
                        "%s expected %s table, got %s",
                        functionInfo:GetName(),
                        fieldName,
                        type(fieldValue)
                    )
                end

                for index, field in ipairs(fieldValue) do
                    if type(field.Name) ~= "string" or field.Name == "" then
                        return string.format(
                            "%s.%s[%d] missing field Name",
                            functionInfo:GetName(),
                            fieldName,
                            index
                        )
                    end

                    if type(field.Type) ~= "string" or field.Type == "" then
                        return string.format(
                            "%s.%s[%d] missing field Type",
                            functionInfo:GetName(),
                            fieldName,
                            index
                        )
                    end

                    if type(field.Nilable) ~= "boolean" then
                        return string.format(
                            "%s.%s[%d] missing boolean Nilable",
                            functionInfo:GetName(),
                            fieldName,
                            index
                        )
                    end
                end

                return nil
            end

            for _, expected in ipairs(expectedFunctions) do
                local functionInfo = APIDocumentation:FindAPIByName("function", expected.name)
                if functionInfo == nil then
                    return string.format("missing function %q", expected.name)
                end

                if functionInfo:GetType() ~= "function" then
                    return string.format(
                        "%s expected function type, got %q",
                        expected.name,
                        tostring(functionInfo:GetType())
                    )
                end

                local argumentFailure =
                    validateOptionalFieldArray(functionInfo, "Arguments", expected.hasArguments)
                if argumentFailure then
                    return argumentFailure
                end

                local returnFailure =
                    validateOptionalFieldArray(functionInfo, "Returns", expected.hasReturns)
                if returnFailure then
                    return returnFailure
                end
            end

            return ""
            "#,
        )
        .expect("generated APIDocumentation global function probe must run cleanly");

    assert_eq!(
        "", failure,
        "well-known generated APIDocumentation functions must be registered"
    );
}

#[cfg(feature = "optional-tests")]
#[test]
fn generated_api_documentation_registers_namespaced_map_function() {
    let env = load_generated_api_documentation();

    let failure: String = env
        .eval(
            r#"
            local functionInfo =
                APIDocumentation:FindAPIByName("function", "GetMapInfo", "MapUI")
            if functionInfo == nil then
                return "missing MapUI.GetMapInfo function"
            end

            if functionInfo.System == nil then
                return "MapUI.GetMapInfo missing System parent"
            end

            local namespace = functionInfo.System:GetNamespaceName()
            if namespace ~= "C_Map" then
                return string.format("expected C_Map namespace, got %q", tostring(namespace))
            end

            return ""
            "#,
        )
        .expect("generated APIDocumentation namespaced function probe must run cleanly");

    assert_eq!(
        "", failure,
        "generated C_Map.GetMapInfo documentation must be registered with its system parent"
    );
}

#[cfg(feature = "optional-tests")]
#[test]
fn generated_api_documentation_registers_well_known_events() {
    let env = load_generated_api_documentation();

    let failure: String = env
        .eval(
            r#"
            local expectedEvents = {
                "PlayerLogin",
                "ChatMsgSystem",
            }

            for _, expectedName in ipairs(expectedEvents) do
                local eventInfo = APIDocumentation:FindAPIByName("event", expectedName)
                if eventInfo == nil then
                    return string.format("missing event %q", expectedName)
                end

                local eventType = eventInfo:GetType()
                if eventType ~= "event" then
                    return string.format(
                        "%s expected event type, got %q",
                        expectedName,
                        tostring(eventType)
                    )
                end
            end

            return ""
            "#,
        )
        .expect("generated APIDocumentation event probe must run cleanly");

    assert_eq!(
        "", failure,
        "well-known generated APIDocumentation events must be registered"
    );
}

struct DocumentationCorpusCounts {
    systems: i64,
    tables: i64,
    functions: i64,
    events: i64,
}

impl From<(i64, i64, i64, i64)> for DocumentationCorpusCounts {
    fn from((systems, tables, functions, events): (i64, i64, i64, i64)) -> Self {
        Self {
            systems,
            tables,
            functions,
            events,
        }
    }
}

fn load_generated_api_documentation() -> wow_ui_sim::lua_api::WowLuaEnv {
    let ui_dir = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui_dir);
    load_panel_addons(&env);
    clear_recorded_lua_errors(&env);

    let loaded = load_blizzard_addon_closure_into_env(&env, &ui_dir, &[ROOT], &[]);
    assert!(
        loaded.iter().any(|addon| addon == ROOT),
        "{ROOT} must be included in the loaded addon closure; loaded={loaded:?}"
    );

    let errors = recorded_lua_errors(&env);
    assert!(
        errors.is_empty(),
        "{ROOT} must load without recorded Lua errors:\n  {}",
        errors.join("\n  ")
    );

    env
}
