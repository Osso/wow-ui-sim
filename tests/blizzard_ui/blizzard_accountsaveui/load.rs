//! Load smoke for `Blizzard_AccountSaveUI`.
//!
//! TOC reference (`Interface/BlizzardUI/Blizzard_AccountSaveUI/
//! Blizzard_AccountSaveUI.toc`):
//!
//! ```text
//! ## Title: Blizzard_AccountSaveUI
//! ## LoadOnDemand: 1
//! ## AllowLoad: Glue
//! ## Dependencies: Blizzard_GlueXML
//! ```
//!
//! Two assertions are pinned here:
//!
//! 1. Loading the smoke-shape closure rooted at this addon completes
//!    cleanly with zero lane-specific Lua errors recorded — the addon's
//!    file-scope code (`Blizzard_AccountSaveUI.lua`,
//!    `Blizzard_AccountSaveUI.xml`) must not throw against the current
//!    SimState. The lane filter watches for the addon's own name plus
//!    every global it publishes (`AccountSaveFrameMixin`, the two
//!    `StaticPopupDialogs[ACCOUNT_SAVE_*]` keys, and the
//!    `ACCOUNT_SAVE_KICK_ERROR_CODE` constant).
//!
//! 2. The TOC's sole `## Dependencies: Blizzard_GlueXML` entry appears in
//!    the loaded-addon list after the smoke-shape harness runs. The closure
//!    walker pulls Blizzard_GlueXML's own deps transitively (the glue
//!    foundation set — Blizzard_StaticPopup_Glue, Blizzard_GlueMenuFrame,
//!    Blizzard_HelpPlate, …), so pinning Blizzard_GlueXML directly is the
//!    smallest assertion that survives any future closure-walker behavior
//!    change.
//!
//! The TOC's `## AllowLoad: Glue` line forces the harness onto the
//! glue/CharacterSelect screen — the standard `with_blizzard_addon_smoke_shape`
//! helper hardcodes `ScreenKind::Game`, which would silently filter this
//! addon out of the discovery pool (`TocFile::allows_screen(Game)` is
//! false for `## AllowLoad: Glue`). The dedicated
//! `with_blizzard_addon_glue_smoke_shape` helper pins
//! `ScreenKind::CharacterSelect`.
//!
//! The known `ACCOUNT_SAVE_RESULT` event-registration gap noted in PLAN.md
//! does not surface here: `RegisterEvent("ACCOUNT_SAVE_RESULT")` runs only
//! from `AccountSaveFrameMixin:OnLoad`, which fires on the
//! `<Frame mixin="AccountSaveFrameMixin">` instance during XML parsing.
//! That call is no-op routed through the simulator's event-name registry —
//! it does not require the event to be a known constant in the simulator's
//! event-dispatch table, so the load path is clean even though
//! `ACCOUNT_SAVE_RESULT` is not yet a recognised Lua surface event.

use crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape;

const ROOT: &str = "Blizzard_AccountSaveUI";
const REQUIRED_DEPS: &[&str] = &["Blizzard_GlueXML"];

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_ui_load_emits_no_lane_specific_lua_errors() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, loaded| {
        assert!(
            loaded.iter().any(|name| name == ROOT),
            "Glue smoke-shape harness MUST end up loading `{ROOT}` itself when it is the closure root. \
             If this regresses, either the closure walker dropped the root (e.g. ScreenKind \
             filtering on `## AllowLoad: Glue`) or `find_toc_file` failed to resolve the addon's \
             TOC variant. Loaded set: {loaded:?}"
        );

        let lane_lua_errors: Vec<String> = env
            .state()
            .borrow()
            .lua_errors
            .iter()
            .filter(|message| {
                message.contains(ROOT)
                    || message.contains("AccountSaveFrame")
                    || message.contains("ACCOUNT_SAVE_IN_PROGRESS")
                    || message.contains("ACCOUNT_SAVE_SUCCESS")
                    || message.contains("ACCOUNT_SAVE_KICK_ERROR_CODE")
            })
            .cloned()
            .collect();

        assert!(
            lane_lua_errors.is_empty(),
            "Blizzard_AccountSaveUI emitted lane-specific Lua errors during the glue \
             smoke-shape closure load. The addon defines AccountSaveFrameMixin plus the \
             ACCOUNT_SAVE_IN_PROGRESS / ACCOUNT_SAVE_SUCCESS StaticPopupDialogs at file \
             scope and registers `ACCOUNT_SAVE_RESULT` / `ACCOUNT_SAVE_ENABLED_UPDATE` / \
             `ACCOUNT_LOCKED_POST_SAVE_UPDATE` from `AccountSaveFrameMixin:OnLoad`; any \
             missing global, broken `GenerateClosure` chain, or strict event-registry \
             rejection would surface here. Got:\n  {}",
            lane_lua_errors.join("\n  ")
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_ui_dependency_closure_includes_glue_xml() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |_env, loaded| {
        for required in REQUIRED_DEPS {
            assert!(
                loaded.iter().any(|name| name == required),
                "Required dependency `{required}` MUST appear in the loaded set after the \
                 glue smoke-shape closure rooted at `{ROOT}` runs. The TOC's \
                 `## Dependencies: Blizzard_GlueXML` line is parsed by `split_metadata_list` \
                 and fed to the closure walker; if it stops appearing here either the \
                 closure walker stopped following Dependencies, or Blizzard_GlueXML's TOC \
                 lost its glue allowance and was filtered out. Loaded set: {loaded:?}"
            );
        }
    });
}
