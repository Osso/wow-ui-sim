//! Global surface probes for `Blizzard_ArtifactUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_ArtifactUI";
const MIXIN_GLOBALS: &[&str] = &[
    "ArtifactUIMixin",
    "ArtifactFrameUnderlayMixin",
    "ArtifactPerksMixin",
    "ArtifactTitleTemplateMixin",
    "ArtifactPowerButtonMixin",
    "ArtifactAppearancesMixin",
    "ArtifactAppearanceSlotMixin",
];
const ARTIFACT_UI_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "OnTraitsRefunded",
    "OnAppearanceChanging",
    "EvaulateForgeState",
    "SetTab",
    "SetupPerArtifactData",
    "RefreshKnowledgeRanks",
    "OnKnowledgeEnter",
    "OnKnowledgeLeave",
    "OnInventoryItemMouseEnter",
    "OnInventoryItemMouseLeave",
];
const ARTIFACT_UI_HELPERS: &[&str] = &[
    "ArtifactUI_CanViewArtifact",
    "ArtifactUI_HasPurchasedAnything",
];
const ARTIFACT_UI_CONSTANTS: &[(&str, f64)] = &[
    ("ARTIFACT_ITEM_SPEED_FACTOR", 0.15),
    ("ARTIFACT_ITEM_BASE_Y_ROTATION", 0.0),
    ("ARTIFACT_ITEM_DRAG_FACTOR", 0.0065),
];

// ArtifactUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_exports_expected_mixin_globals_after_load_addon() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(&surface_probe())
            .expect("ArtifactUI global surface probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must expose expected mixin globals and ArtifactUIMixin methods; \
             mismatches: {mismatches:?}"
        );
    });
}
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[test]
fn artifact_ui_exports_expected_module_helpers_and_constants_after_load_addon() {
    with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
        load_artifact_ui(env);

        let mismatches: Vec<String> = env
            .eval(&module_global_probe())
            .expect("ArtifactUI module-global surface probe should run cleanly");

        assert!(
            mismatches.is_empty(),
            "`{ROOT}` must expose expected module helpers and constants; mismatches: {mismatches:?}"
        );
    });
}
}

fn load_artifact_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, error): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_ArtifactUI")"#)
        .expect("C_AddOns.LoadAddOn probe should run cleanly");
    assert!(
        loaded,
        "`{ROOT}` must load before surface probes; error={error:?}"
    );
}

fn surface_probe() -> String {
    let mixin_list = lua_array_literal(MIXIN_GLOBALS);
    let method_list = lua_array_literal(ARTIFACT_UI_MIXIN_METHODS);

    format!(
        r#"
        local mixinGlobals = {{{mixin_list}}}
        local methodNames = {{{method_list}}}
        local mismatches = {{}}

        for _, mixinName in ipairs(mixinGlobals) do
            if type(_G[mixinName]) ~= "table" then
                table.insert(mismatches, mixinName .. ":" .. type(_G[mixinName]))
            end
        end

        for _, methodName in ipairs(methodNames) do
            local methodType = type(ArtifactUIMixin) == "table"
                and type(ArtifactUIMixin[methodName])
                or "nil"
            if methodType ~= "function" then
                table.insert(mismatches, "ArtifactUIMixin." .. methodName .. ":" .. methodType)
            end
        end

        return mismatches
        "#
    )
}

fn module_global_probe() -> String {
    let helper_list = lua_array_literal(ARTIFACT_UI_HELPERS);
    let constant_list = lua_numeric_pair_array_literal(ARTIFACT_UI_CONSTANTS);

    format!(
        r#"
        local helperNames = {{{helper_list}}}
        local constants = {{{constant_list}}}
        local mismatches = {{}}

        for _, helperName in ipairs(helperNames) do
            if type(_G[helperName]) ~= "function" then
                table.insert(mismatches, helperName .. ":" .. type(_G[helperName]))
            end
        end

        for _, constant in ipairs(constants) do
            local constantName = constant[1]
            local expected = constant[2]
            local actual = _G[constantName]
            if type(actual) ~= "number" or math.abs(actual - expected) > 0.000001 then
                table.insert(mismatches, constantName .. ":" .. tostring(actual))
            end
        end

        return mismatches
        "#
    )
}

fn lua_array_literal(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn lua_numeric_pair_array_literal(values: &[(&str, f64)]) -> String {
    values
        .iter()
        .map(|(name, value)| format!("{{{name:?}, {value}}}"))
        .collect::<Vec<_>>()
        .join(", ")
}
