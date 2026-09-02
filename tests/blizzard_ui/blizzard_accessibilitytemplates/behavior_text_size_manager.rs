//! Behavior pin for `TextSizeManager:Init()` (called inline at the bottom of
//! `TextSizeManagerGame.lua`, line 7).
//!
//! `Init()` runs unconditionally at file scope, so by the time the smoke-shape
//! closure completes, every documented Init side effect must already be visible
//! on `_G.TextSizeManager`:
//!
//! - `self.registeredObjects = {}` (initial empty registry)
//! - `self.defaultScaleWeight = 0.5`
//! - `self:SetCVarNames("userFontScale", "userFontScaleGlue")` configures both screen CVars
//! - `self:SetMinimumScale(0.8)` ⇒ `GetMinimumScale() == math.max(0.8, 0.5) == 0.8`
//! - `self:BuildFonts()` ⇒ `self.fonts` is a (possibly empty) table; the
//!   simulator's `GetFonts()` returns an empty list so the loop body in
//!   `BuildFonts` never executes, but the table assignment itself must still
//!   happen before any consumer of `GetFonts()` runs
//! - `CVarCallbackRegistry:RegisterCallback("userFontScale", CVarChangedCB)`
//!   ⇒ `CVarCallbackRegistry:HasRegistrantsForEvent("userFontScale")` is true
//!
//! The `EventUtil.ContinueAfterAllEvents(..., GetInitialUpdateEvents())` line
//! at the end of `Init()` is intentionally NOT asserted here — the deferred
//! `UpdateFonts()` call is queued against `VARIABLES_LOADED`, an event that
//! hasn't fired during a smoke load, so the eventual `SetTextScale` side
//! effects are not yet observable.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccessibilityTemplates";

#[test]
fn init_seeds_text_size_manager_state() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let registered_objects_type: String = env
            .eval("return type(TextSizeManager.registeredObjects)")
            .expect("failed to probe TextSizeManager.registeredObjects type");
        assert_eq!(
            registered_objects_type, "table",
            "Init() must initialise `registeredObjects = {{}}` before any \
             RegisterObject call lands. Got `{registered_objects_type}`."
        );

        let default_scale_weight: f64 = env
            .eval("return TextSizeManager.defaultScaleWeight")
            .expect("failed to probe TextSizeManager.defaultScaleWeight");
        assert_eq!(
            default_scale_weight, 0.5,
            "Init() pins `defaultScaleWeight = 0.5` (used by GetWeightedScale \
             when registrationInfo.scaleWeight is absent)."
        );

        let cvar_names: Vec<String> = env
            .eval("return TextSizeManager:GetCVarNames()")
            .expect("failed to probe TextSizeManager:GetCVarNames()");
        assert_eq!(
            cvar_names,
            vec!["userFontScale".to_string(), "userFontScaleGlue".to_string()],
            "Init() must configure both game and glue font-scale CVars"
        );

        let read_cvar_name: String = env
            .eval("return TextSizeManager:GetReadCVarName()")
            .expect("failed to probe TextSizeManager:GetReadCVarName()");
        assert_eq!(
            read_cvar_name, "userFontScale",
            "Game TextSizeManager must read userFontScale first"
        );

        let minimum_scale: f64 = env
            .eval("return TextSizeManager:GetMinimumScale()")
            .expect("failed to probe TextSizeManager:GetMinimumScale()");
        assert_eq!(
            minimum_scale, 0.8,
            "Init() calls `SetMinimumScale(0.8)`; the setter clamps via \
             math.max(scale, 0.5), so the visible value is 0.8."
        );

        let fonts_type: String = env
            .eval("return type(TextSizeManager.fonts)")
            .expect("failed to probe TextSizeManager.fonts type");
        assert_eq!(
            fonts_type, "table",
            "BuildFonts must assign `self.fonts = {{}}` even when GetFonts() \
             returns an empty list — downstream SetTextScale iterates this \
             table with `pairs`, which would error on a nil. Got `{fonts_type}`."
        );
    });
}

#[test]
fn init_registers_user_font_scale_cvar_callback() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let has_registrants: bool = env
            .eval("return CVarCallbackRegistry:HasRegistrantsForEvent(\"userFontScale\")")
            .expect("failed to probe CVarCallbackRegistry:HasRegistrantsForEvent");
        assert!(
            has_registrants,
            "Init() must register the local `CVarChangedCB` against \
             CVarCallbackRegistry for the \"userFontScale\" CVar — without \
             this, slider changes never propagate to UpdateFonts/SetTextScale \
             and registered UserScaledElement consumers stop rescaling."
        );
    });
}
