//! Load smoke for `Blizzard_AccessibilityTemplates`.
//!
//! TOC reference (`Interface/BlizzardUI/Blizzard_AccessibilityTemplates/
//! Blizzard_AccessibilityTemplates.toc`):
//!
//! ```text
//! ## Title: Blizzard_AccessibilityTemplates
//! ## RequiredDep: Blizzard_SharedXML, Blizzard_Colors
//! ## AllowLoad: both
//! ```
//!
//! Two assertions are pinned here:
//!
//! 1. Loading the smoke-shape closure rooted at this addon completes
//!    cleanly with zero lane-specific Lua errors recorded — the addon's
//!    file-scope code (`AccessibilityTemplates.lua`,
//!    `AccessibilityIntrinsics.xml`, `QuestTextContrast.lua`,
//!    `TextSizeManager.lua`, `TextSizeManagerGame.lua`,
//!    `UserScaledElementTemplates.lua/.xml`, `UserScaledSliderTemplates.xml`)
//!    must not throw against the current SimState.
//!
//! 2. Every TOC `RequiredDep` (Blizzard_SharedXML, Blizzard_Colors) appears
//!    in the loaded-addon list after the smoke-shape harness runs. The
//!    closure walker pulls SharedXML's own deps transitively (SharedXMLBase /
//!    Fonts_Shared / PrintHandler / Menu / Colors / HelpPlate), so the lane's
//!    declared deps form the smallest assertion that survives any future
//!    closure-walker behavior change.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccessibilityTemplates";
const REQUIRED_DEPS: &[&str] = &["Blizzard_SharedXML", "Blizzard_Colors"];

#[cfg(feature = "optional-tests")]
#[test]
fn accessibility_templates_load_emits_no_lane_specific_lua_errors() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "Smoke-shape harness MUST end up loading `{ROOT}` itself when it is the closure root. \
             If this regresses, either the closure walker dropped the root or `find_toc_file` \
             failed to resolve the addon's TOC variant. Loaded set: {loaded:?}"
        );

        let lane_lua_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|message| {
                message.contains(ROOT)
                    || message.contains("UIThemeContainer")
                    || message.contains("QuestTextContrast")
                    || message.contains("TextSizeManager")
                    || message.contains("UserScaledElement")
                    || message.contains("UserScaledSlider")
            })
            .cloned()
            .collect();

        assert!(
            lane_lua_errors.is_empty(),
            "Blizzard_AccessibilityTemplates emitted lane-specific Lua errors during the \
             smoke-shape closure load. The addon defines UIThemeContainerMixin / \
             QuestTextContrast / TextSizeManagerBase / TextSizeManager / \
             UserScaledElementMixin at file scope plus the AccessibilityIntrinsics XML \
             registration — any nil-call or missing global would surface here. Got:\n  {}",
            lane_lua_errors.join("\n  ")
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn accessibility_templates_dependency_closure_includes_required_deps() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |_env, loaded| {
        for required in REQUIRED_DEPS {
            assert!(
                loaded.iter().any(|name| name == required),
                "RequiredDep `{required}` MUST appear in the loaded set after the smoke-shape \
                 closure rooted at `{ROOT}` runs. The TOC's `## RequiredDep: Blizzard_SharedXML, \
                 Blizzard_Colors` line is parsed by `split_metadata_list` and fed to the \
                 closure walker; if either name is missing, downstream addons that inherit \
                 UserScaledFrameTemplate / UserScaledFontStringTemplate would fail to resolve \
                 the parent template chain. Loaded set: {loaded:?}"
            );
        }
    });
}
