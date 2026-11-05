//! Mixin-surface probes for `Blizzard_AnimaDiversionUI`.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::loader::BlizzardAddonOverride;

const ROOT: &str = "Blizzard_AnimaDiversionUI";
const IMPLICIT_DEPS: &[&str] = &["Blizzard_MapCanvas", "Blizzard_SharedMapDataProviders"];
const CLOSURE_OVERRIDES: &[BlizzardAddonOverride<'_>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: IMPLICIT_DEPS,
}];
const FRAME_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "HasAvailableNode",
    "UpdateTutorialTips",
    "SetExclusiveSelectionNode",
    "ClearExclusiveSelectionNode",
    "CanReinforceNode",
    "AddBolsterEffectToGem",
    "StopGemsFullSound",
    "SetupBolsterProgressBar",
    "SetupBolsterGem",
    "AddStandardDataProviders",
    "SetupTextureKits",
    "TryShow",
    "SetupCurrencyFrame",
];
const DATA_PROVIDER_MIXIN_METHODS: &[&str] = &[
    "OnShow",
    "OnHide",
    "OnEvent",
    "SetupConnectionOnPin",
    "ResetModelScene",
    "AddEffectOnPin",
    "ClearEffectOnPin",
    "ClearEffectOnAllPins",
    "RemoveAllData",
    "CanReinforceNode",
    "RefreshAllData",
    "AddNode",
    "AddOrigin",
    "AddModelScene",
];
const PIN_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "SetupOrigin",
    "IsConnected",
    "SetupNode",
    "SetVisualState",
    "SetReinforceState",
    "SetSelectedState",
    "OnMouseEnter",
    "HaveEnoughAnimaToActivate",
    "RefreshTooltip",
    "OnMouseLeave",
    "OnClick",
];
const CONNECTION_MIXIN_METHODS: &[&str] = &["Setup"];
const ENTER_LEAVE_METHODS: &[&str] = &["OnEnter", "OnLeave"];
const REINFORCE_INFO_MIXIN_METHODS: &[&str] = &[
    "OnHide",
    "Init",
    "CanReinforceAnything",
    "GetSelectedNode",
    "ClearSelectedNode",
    "SelectNodeToReinforce",
];
const REINFORCE_BUTTON_MIXIN_METHODS: &[&str] = &["OnClick", "OnEnter", "OnLeave"];
const FRAME_MIXIN_METHODS_PROBE: &str = r#"
local methods = {
    "OnLoad",
    "OnShow",
    "OnHide",
    "OnEvent",
    "HasAvailableNode",
    "UpdateTutorialTips",
    "SetExclusiveSelectionNode",
    "ClearExclusiveSelectionNode",
    "CanReinforceNode",
    "AddBolsterEffectToGem",
    "StopGemsFullSound",
    "SetupBolsterProgressBar",
    "SetupBolsterGem",
    "AddStandardDataProviders",
    "SetupTextureKits",
    "TryShow",
    "SetupCurrencyFrame",
}

local missing = {}
for _, method in ipairs(methods) do
    if type(AnimaDiversionFrameMixin[method]) ~= "function" then
        table.insert(missing, method .. ":" .. type(AnimaDiversionFrameMixin[method]))
    end
end

return type(AnimaDiversionFrameMixin), missing
"#;
const DATA_PROVIDER_MIXIN_METHODS_PROBE: &str = r#"
local methods = {
    "OnShow",
    "OnHide",
    "OnEvent",
    "SetupConnectionOnPin",
    "ResetModelScene",
    "AddEffectOnPin",
    "ClearEffectOnPin",
    "ClearEffectOnAllPins",
    "RemoveAllData",
    "CanReinforceNode",
    "RefreshAllData",
    "AddNode",
    "AddOrigin",
    "AddModelScene",
}

local missing = {}
for _, method in ipairs(methods) do
    if type(AnimaDiversionDataProviderMixin[method]) ~= "function" then
        table.insert(missing, method .. ":" .. type(AnimaDiversionDataProviderMixin[method]))
    end
end

return type(AnimaDiversionDataProviderMixin), missing
"#;
const PIN_MIXIN_METHODS_PROBE: &str = r#"
local methods = {
    "OnLoad",
    "SetupOrigin",
    "IsConnected",
    "SetupNode",
    "SetVisualState",
    "SetReinforceState",
    "SetSelectedState",
    "OnMouseEnter",
    "HaveEnoughAnimaToActivate",
    "RefreshTooltip",
    "OnMouseLeave",
    "OnClick",
}

local missing = {}
for _, method in ipairs(methods) do
    if type(AnimaDiversionPinMixin[method]) ~= "function" then
        table.insert(missing, method .. ":" .. type(AnimaDiversionPinMixin[method]))
    end
end

return type(AnimaDiversionPinMixin), missing
"#;
const CONNECTION_MIXIN_METHODS_PROBE: &str = r#"
local missing = {}
if type(AnimaDiversionConnectionMixin.Setup) ~= "function" then
    table.insert(missing, "Setup:" .. type(AnimaDiversionConnectionMixin.Setup))
end

return type(AnimaDiversionConnectionMixin), missing
"#;
const SCRIPT_HANDLER_MIXIN_METHODS_PROBE: &str = r#"
local function collect(mixin, methods)
    local missing = {}
    for _, method in ipairs(methods) do
        if type(mixin[method]) ~= "function" then
            table.insert(missing, method .. ":" .. type(mixin[method]))
        end
    end
    return missing
end

local enterLeaveMethods = {"OnEnter", "OnLeave"}
local reinforceInfoMethods = {
    "OnHide",
    "Init",
    "CanReinforceAnything",
    "GetSelectedNode",
    "ClearSelectedNode",
    "SelectNodeToReinforce",
}
local reinforceButtonMethods = {"OnClick", "OnEnter", "OnLeave"}

return type(AnimaDiversionCurrencyFrameMixin),
       collect(AnimaDiversionCurrencyFrameMixin, enterLeaveMethods),
       type(ReinforceProgressFrameMixin),
       collect(ReinforceProgressFrameMixin, enterLeaveMethods),
       type(ReinforceInfoFrameMixin),
       collect(ReinforceInfoFrameMixin, reinforceInfoMethods),
       type(AnimaNodeReinforceButtonMixin),
       collect(AnimaNodeReinforceButtonMixin, reinforceButtonMethods)
"#;

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_frame_mixin_exposes_expected_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: MixinMethodSurface = env
            .eval(FRAME_MIXIN_METHODS_PROBE)
            .expect("AnimaDiversionFrameMixin method probe must run cleanly");

        assert_mixin_methods("AnimaDiversionFrameMixin", FRAME_MIXIN_METHODS, surface);
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_data_provider_mixin_exposes_expected_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: MixinMethodSurface = env
            .eval(DATA_PROVIDER_MIXIN_METHODS_PROBE)
            .expect("AnimaDiversionDataProviderMixin method probe must run cleanly");

        assert_mixin_methods(
            "AnimaDiversionDataProviderMixin",
            DATA_PROVIDER_MIXIN_METHODS,
            surface,
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_pin_and_connection_mixins_expose_expected_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let pin_surface: MixinMethodSurface = env
            .eval(PIN_MIXIN_METHODS_PROBE)
            .expect("AnimaDiversionPinMixin method probe must run cleanly");
        let connection_surface: MixinMethodSurface = env
            .eval(CONNECTION_MIXIN_METHODS_PROBE)
            .expect("AnimaDiversionConnectionMixin method probe must run cleanly");

        assert_mixin_methods("AnimaDiversionPinMixin", PIN_MIXIN_METHODS, pin_surface);
        assert_mixin_methods(
            "AnimaDiversionConnectionMixin",
            CONNECTION_MIXIN_METHODS,
            connection_surface,
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn anima_diversion_xml_script_mixins_expose_expected_methods() {
    with_blizzard_addon_startup_shape(&[ROOT], CLOSURE_OVERRIDES, |env, _loaded| {
        let surface: ScriptHandlerMixinSurface = env
            .eval(SCRIPT_HANDLER_MIXIN_METHODS_PROBE)
            .expect("AnimaDiversion script-handler mixin probe must run cleanly");

        assert_script_handler_mixins(surface);
    });
}

type MixinMethodSurface = (String, Vec<String>);
type ScriptHandlerMixinSurface = (
    String,
    Vec<String>,
    String,
    Vec<String>,
    String,
    Vec<String>,
    String,
    Vec<String>,
);

fn assert_mixin_methods(mixin_name: &str, expected_methods: &[&str], surface: MixinMethodSurface) {
    let (mixin_type, missing_methods) = surface;

    assert_eq!(
        mixin_type, "table",
        "`{mixin_name}` must be exposed as a table"
    );
    assert!(
        missing_methods.is_empty(),
        "`{mixin_name}` must expose these methods as functions: {expected_methods:?}; \
         missing or wrong-type entries: {missing_methods:?}"
    );
}

fn assert_script_handler_mixins(surface: ScriptHandlerMixinSurface) {
    let (
        currency_type,
        currency_missing,
        progress_type,
        progress_missing,
        info_type,
        info_missing,
        button_type,
        button_missing,
    ) = surface;

    assert_mixin_methods(
        "AnimaDiversionCurrencyFrameMixin",
        ENTER_LEAVE_METHODS,
        (currency_type, currency_missing),
    );
    assert_mixin_methods(
        "ReinforceProgressFrameMixin",
        ENTER_LEAVE_METHODS,
        (progress_type, progress_missing),
    );
    assert_mixin_methods(
        "ReinforceInfoFrameMixin",
        REINFORCE_INFO_MIXIN_METHODS,
        (info_type, info_missing),
    );
    assert_mixin_methods(
        "AnimaNodeReinforceButtonMixin",
        REINFORCE_BUTTON_MIXIN_METHODS,
        (button_type, button_missing),
    );
}
