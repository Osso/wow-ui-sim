//! Public globals exposed by `Blizzard_AccountSaveUI.lua` at file scope.
//!
//! Source (`Interface/BlizzardUI/Blizzard_AccountSaveUI/
//! Blizzard_AccountSaveUI.lua`):
//!
//! ```lua
//! ACCOUNT_SAVE_KICK_ERROR_CODE = 241;
//! ...
//! AccountSaveFrameMixin = {};
//! ```
//!
//! Both globals are written unconditionally during file-scope execution,
//! so any regression that prevents the addon from running its top-level
//! statements (taint mismatch, parser desync, missing `StaticPopupDialogs`
//! sandbox table, etc.) trips one of the assertions below long before the
//! richer surface tests get a chance to fail.
//!
//! `AccountSaveFrameMixin` is checked as a Lua `"table"` rather than for
//! membership of any specific method. The mixin's method table is
//! populated later in the same file (`function AccountSaveFrameMixin:OnLoad()`,
//! …) — those are pinned by the dedicated behavior fixtures. This file
//! only guards the file-scope assignment itself.

use crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape;

const ROOT: &str = "Blizzard_AccountSaveUI";

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_mixin_is_a_table() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let mixin_type = env
            .eval::<String>("return type(AccountSaveFrameMixin)")
            .expect("type(AccountSaveFrameMixin) must evaluate cleanly");
        assert_eq!(
            mixin_type, "table",
            "AccountSaveFrameMixin must be a table after Blizzard_AccountSaveUI.lua \
             runs at file scope. The source assigns `AccountSaveFrameMixin = {{}};` \
             unconditionally; if this regresses, either the addon never executed its \
             top-level statements (taint/loader regression) or something later in the \
             closure clobbered the global. Got: type = `{mixin_type}`."
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_kick_error_code_is_241() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let code = env
            .eval::<i64>("return ACCOUNT_SAVE_KICK_ERROR_CODE")
            .expect("ACCOUNT_SAVE_KICK_ERROR_CODE must be readable as an integer");
        assert_eq!(
            code, 241,
            "ACCOUNT_SAVE_KICK_ERROR_CODE must be exactly 241 — the constant is the \
             contract that AccountSaveFrame uses to recognise the boot-after-save \
             ACCOUNT_SAVE_RESULT payload. Source line 1 of \
             Blizzard_AccountSaveUI.lua: `ACCOUNT_SAVE_KICK_ERROR_CODE = 241;`. \
             Got: {code}."
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_static_popup_dialogs_explicit_acknowledge() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let (in_progress_ack, success_ack) = env
            .eval::<(bool, bool)>(
                r#"
                local in_progress = StaticPopupDialogs["ACCOUNT_SAVE_IN_PROGRESS"]
                local success     = StaticPopupDialogs["ACCOUNT_SAVE_SUCCESS"]
                assert(type(in_progress) == "table",
                    "StaticPopupDialogs[ACCOUNT_SAVE_IN_PROGRESS] must be a table")
                assert(type(success) == "table",
                    "StaticPopupDialogs[ACCOUNT_SAVE_SUCCESS] must be a table")
                return in_progress.explicitAcknowledge == true,
                       success.explicitAcknowledge == true
                "#,
            )
            .expect(
                "StaticPopupDialogs[ACCOUNT_SAVE_IN_PROGRESS] and \
                 StaticPopupDialogs[ACCOUNT_SAVE_SUCCESS] must both be registered \
                 tables after Blizzard_AccountSaveUI.lua runs",
            );
        assert!(
            in_progress_ack && success_ack,
            "Both ACCOUNT_SAVE_IN_PROGRESS and ACCOUNT_SAVE_SUCCESS dialogs must \
             carry `explicitAcknowledge = true` (Blizzard_AccountSaveUI.lua lines \
             3-23). The flag forces the player to dismiss the dialog manually \
             instead of auto-clearing on focus loss — required because the kick \
             flow needs an explicit user action before disconnect. Got: \
             ACCOUNT_SAVE_IN_PROGRESS.explicitAcknowledge={in_progress_ack}, \
             ACCOUNT_SAVE_SUCCESS.explicitAcknowledge={success_ack}."
        );
    });
}

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_mixin_visual_state_enum() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let (disabled, enabled_locked, enabled_unlocked) = env
            .eval::<(i64, i64, i64)>(
                "return AccountSaveFrameMixin.VisualState.Disabled,
                        AccountSaveFrameMixin.VisualState.EnabledLocked,
                        AccountSaveFrameMixin.VisualState.EnabledUnlocked",
            )
            .expect(
                "AccountSaveFrameMixin.VisualState must expose Disabled / EnabledLocked / \
                 EnabledUnlocked as integer keys",
            );
        assert_eq!(
            (disabled, enabled_locked, enabled_unlocked),
            (1, 2, 3),
            "AccountSaveFrameMixin.VisualState pins three enum values referenced by \
             AccountSaveFrameMixin:UpdateVisualState — Disabled=1, EnabledLocked=2, \
             EnabledUnlocked=3 (Blizzard_AccountSaveUI.lua lines 27-31). If this \
             regresses, either the file-scope assignment never ran or a downstream \
             addon overwrote the table. Got: \
             (Disabled={disabled}, EnabledLocked={enabled_locked}, \
             EnabledUnlocked={enabled_unlocked})."
        );
    });
}
