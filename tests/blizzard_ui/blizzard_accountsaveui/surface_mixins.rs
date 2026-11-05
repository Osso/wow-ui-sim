//! Mixin method surface for `Blizzard_AccountSaveUI`.
//!
//! `AccountSaveFrameMixin` is the table that drives every dynamic
//! behavior of `AccountSaveFrame`. Source
//! (`Interface/BlizzardUI/Blizzard_AccountSaveUI/
//! Blizzard_AccountSaveUI.lua`):
//!
//! ```lua
//! AccountSaveFrameMixin = {};
//! AccountSaveFrameMixin.VisualState = { Disabled = 1, EnabledLocked = 2, EnabledUnlocked = 3 };
//!
//! function AccountSaveFrameMixin:OnLoad()                       (line 34)
//! function AccountSaveFrameMixin:OnShow()                       (line 45)
//! function AccountSaveFrameMixin:UpdateAccountState()           (line 49)
//! function AccountSaveFrameMixin:OnLockEditBoxTextChanged()     (line 91)
//! function AccountSaveFrameMixin:OnLockEditBoxEnterPressed()    (line 95)
//! function AccountSaveFrameMixin:OnLockEditBoxEscapePressed()   (line 101)
//! function AccountSaveFrameMixin:OnSaveButtonClicked()          (line 105)
//! function AccountSaveFrameMixin:DoesLockStringMatch()          (line 111)
//! function AccountSaveFrameMixin:SaveAccountData()              (line 115)
//! function AccountSaveFrameMixin:OnEvent(event, ...)            (line 125)
//! function AccountSaveFrameMixin:ProcessAccountSaveError(code)  (line 144)
//! function AccountSaveFrameMixin:OnSizeChanged()                (line 169)
//! function AccountSaveFrameMixin:UpdateSizing()                 (line 184)
//! ```
//!
//! Behavior of these methods is pinned by the dedicated `behavior_*`
//! fixtures. This file pins only the *shape* of the mixin — that every
//! method is loaded as a Lua function on the mixin table. If a method
//! disappears (e.g. a Blizzard refactor moves a handler onto a child
//! widget's mixin), the corresponding behavior fixture would still
//! catch the change but with a confusing error like "attempt to call a
//! nil value". This shape probe surfaces the deletion directly.

use crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape;

const ROOT: &str = "Blizzard_AccountSaveUI";
const MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnShow",
    "UpdateAccountState",
    "OnLockEditBoxTextChanged",
    "OnLockEditBoxEnterPressed",
    "OnLockEditBoxEscapePressed",
    "OnSaveButtonClicked",
    "DoesLockStringMatch",
    "SaveAccountData",
    "OnEvent",
    "ProcessAccountSaveError",
    "OnSizeChanged",
    "UpdateSizing",
];

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_mixin_exposes_all_methods() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for method in MIXIN_METHODS {
            let probe = format!(
                r#"
                assert(AccountSaveFrameMixin, "AccountSaveFrameMixin global must exist after Blizzard_AccountSaveUI load")
                return type(AccountSaveFrameMixin.{method})
                "#
            );
            let kind = env
                .eval::<String>(&probe)
                .unwrap_or_else(|err| panic!("AccountSaveFrameMixin.{method} probe raised: {err}"));
            assert_eq!(
                kind, "function",
                "AccountSaveFrameMixin.{method} must be a Lua function loaded from \
                 Blizzard_AccountSaveUI.lua. If this regresses, either the method was \
                 deleted upstream (Blizzard refactor moved it onto a child widget's \
                 mixin) or the addon's Lua file failed to execute past the method's \
                 declaration line. Got type = `{kind}` for `{method}`."
            );
        }
    });
}
