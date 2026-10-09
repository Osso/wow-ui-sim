//! Mixin-method surface pinned by `Blizzard_AccessibilityTemplates`.
//!
//! Three free-form Lua tables grow methods via `function Mixin:Method() ... end`
//! syntax (just sugar for assigning a function to `Mixin.Method`):
//!
//! - `UIThemeContainerMixin`, defined in
//!   `Mainline/AccessibilityTemplates.lua`, mixed into the
//!   `UIThemeContainerFrame` intrinsic via `mixin="UIThemeContainerMixin"`
//!   in `AccessibilityIntrinsics.xml`. Drives the parchment / stone theme
//!   swap that QuestText / GossipFrame / ItemText containers depend on.
//!
//! - `TextSizeManagerBase`, defined in `TextSizeManager.lua`. Used by
//!   `TextSizeManager` (a `CreateFromMixins(TextSizeManagerBase)` instance
//!   in `TextSizeManagerGame.lua`) and by any cross-flavor consumer that
//!   maintains its own `CreateFromMixins(TextSizeManagerBase)` font-scale
//!   subclass.
//!
//! - `UserScaledElementMixin`, defined in `UserScaledElementTemplates.lua`.
//!   Mixed into `UserScaledFrameTemplate` / `UserScaledFontStringTemplate` /
//!   `UserScaledSliderTemplate` via `mixin="UserScaledElementMixin"`. Each
//!   instance registers itself with `TextSizeManager` in
//!   `OnLoad_UserScaledElement` and gets its width/height rescaled in
//!   `OnTextScaleUpdated` whenever the user font-scale CVar changes.
//!
//! If any pinned method disappears — or, more subtly, gets shadowed by a
//! non-function value during the file's load — every consumer that mixes
//! the table in starts nil-calling at the first event tick. Pinning the
//! function-shaped surface is the minimum guard against that class of
//! regression.
//!
//! The four `..._OnPre*`/`..._OnPost*` intrinsic-script entry points on
//! `UIThemeContainerMixin` are intentionally NOT pinned here — they're
//! exercised end-to-end by the load smoke (which would catch any nil-call
//! from the script chain), so duplicating that coverage at the
//! function-table level would just be redundant.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_AccessibilityTemplates";

const UI_THEME_CONTAINER_METHODS: &[&str] = &[
    "UpdateTheme",
    "CheckUpdateTheme",
    "GetCVarValue",
    "IsDarkMode",
    "UpdateFontStrings",
    "UpdateFrames",
    "UpdateBackground",
    "RegisterObject",
    "RegisterObjects",
    "RegisterFontString",
    "RegisterFontStrings",
    "RegisterFrame",
    "RegisterFrames",
    "RegisterBackgroundTexture",
];

const TEXT_SIZE_MANAGER_BASE_METHODS: &[&str] = &[
    "Init",
    "BuildFonts",
    "GetFonts",
    "GetFontBaseHeight",
    "GetResizedFontHeight",
    "SetTextScale",
    "UpdateFonts",
    "GetMinimumScale",
    "GetScale",
    "GetDefaultScaleWeight",
    "SetMinimumScale",
    "SetCVarNames",
    "GetCVarNames",
    "GetReadCVarName",
    "GetSettingValue",
    "GetSettingDefaultValue",
    "SetSettingValue",
    "GetWeightedScale",
    "GetScaledValue",
    "GetScaledValueWeighted",
    "UpdateRegisteredObjects",
    "UpdateRegisteredSystems",
    "RegisterObject",
    "UpdateObject",
];

const USER_SCALED_ELEMENT_METHODS: &[&str] = &[
    "OnLoad_UserScaledElement",
    "UpdateWidth",
    "GetWeightedScale",
    "GetScaledDesiredDimension",
    "SetDesiredWidth",
    "GetDesiredWidth",
    "GetScaledDesiredWidth",
    "GetDesiredHeight",
    "GetScaledDesiredHeight",
    "OnTextScaleUpdated",
];

fn assert_mixin_methods(env: &WowLuaEnv, mixin_name: &str, methods: &[&str], defining_file: &str) {
    for method in methods {
        let probe = format!("return type({mixin_name}[{method:?}])");
        let actual_type: String = env.eval(&probe).unwrap_or_else(|error| {
            panic!("failed to probe `{mixin_name}.{method}` type: {error}")
        });
        assert_eq!(
            actual_type, "function",
            "Expected `{mixin_name}.{method}` to be a function after `{ROOT}` loads, got \
             `{actual_type}`. Defined in `{defining_file}` via `function {mixin_name}:{method}(...) end`. \
             A non-function (or nil) here means every consumer that mixes the table in will \
             nil-call at the first event tick."
        );
    }
}

#[test]
fn ui_theme_container_mixin_exposes_expected_methods() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        assert_mixin_methods(
            env,
            "UIThemeContainerMixin",
            UI_THEME_CONTAINER_METHODS,
            "Mainline/AccessibilityTemplates.lua",
        );
    });
}

#[test]
fn text_size_manager_base_exposes_expected_methods() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        assert_mixin_methods(
            env,
            "TextSizeManagerBase",
            TEXT_SIZE_MANAGER_BASE_METHODS,
            "TextSizeManager.lua",
        );
    });
}

#[test]
fn user_scaled_element_mixin_exposes_expected_methods() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        assert_mixin_methods(
            env,
            "UserScaledElementMixin",
            USER_SCALED_ELEMENT_METHODS,
            "UserScaledElementTemplates.lua",
        );
    });
}
