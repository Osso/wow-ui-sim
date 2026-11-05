//! Event registration surface for `Blizzard_AccountSaveUI.xml`.
//!
//! `AccountSaveFrameMixin:OnLoad` runs during XML parse (the frame's
//! `mixin="AccountSaveFrameMixin"` attribute wires `OnLoad` automatically).
//! Source (`Interface/BlizzardUI/Blizzard_AccountSaveUI/
//! Blizzard_AccountSaveUI.lua`, lines 34-43):
//!
//! ```lua
//! function AccountSaveFrameMixin:OnLoad()
//!     self.LockEditBox:SetScript("OnTextChanged",   ...);
//!     self.LockEditBox:SetScript("OnEnterPressed",  ...);
//!     self.LockEditBox:SetScript("OnEscapePressed", ...);
//!     self.SaveButton:SetOnClickHandler(...);
//!
//!     self:RegisterEvent("ACCOUNT_SAVE_ENABLED_UPDATE");
//!     self:RegisterEvent("ACCOUNT_LOCKED_POST_SAVE_UPDATE");
//!     self:RegisterEvent("ACCOUNT_SAVE_RESULT");
//! end
//! ```
//!
//! These are the only events the frame listens to. Each one drives a
//! distinct dispatch path in `OnEvent`: `ACCOUNT_SAVE_ENABLED_UPDATE` and
//! `ACCOUNT_LOCKED_POST_SAVE_UPDATE` re-run `UpdateAccountState`,
//! while `ACCOUNT_SAVE_RESULT` shows a `StaticPopup` confirming the save
//! outcome. If any event drops out of the registration set, the frame
//! silently misses the corresponding state transition — which is why we
//! pin all three explicitly rather than just counting registrations.
//!
//! Note: `ACCOUNT_SAVE_RESULT` is not a recognised simulator event, but
//! the simulator routes unknown event names through a permissive
//! registry, so the registration call itself succeeds and
//! `IsEventRegistered` returns true. That's the desired behavior — the
//! simulator should not reject Blizzard event names it hasn't seen yet,
//! because the loader runs before the dispatch surface is fully wired.

use crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape;

const ROOT: &str = "Blizzard_AccountSaveUI";
const REGISTERED_EVENTS: &[&str] = &[
    "ACCOUNT_SAVE_ENABLED_UPDATE",
    "ACCOUNT_LOCKED_POST_SAVE_UPDATE",
    "ACCOUNT_SAVE_RESULT",
];

#[cfg(feature = "optional-tests")]
#[test]
fn account_save_frame_registers_on_load_events() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for event in REGISTERED_EVENTS {
            let probe = format!(
                r#"
                assert(AccountSaveFrame, "AccountSaveFrame must exist after Blizzard_AccountSaveUI load")
                return AccountSaveFrame:IsEventRegistered("{event}")
                "#
            );
            let registered = env.eval::<bool>(&probe).unwrap_or_else(|err| {
                panic!("AccountSaveFrame:IsEventRegistered(\"{event}\") raised: {err}")
            });
            assert!(
                registered,
                "AccountSaveFrame must register `{event}` during \
                 AccountSaveFrameMixin:OnLoad (Blizzard_AccountSaveUI.lua \
                 lines 40-42). All three events drive a distinct OnEvent \
                 dispatch path; if `{event}` drops out of the registration \
                 set, the frame silently misses the corresponding state \
                 transition. If this regresses, either the mixin's OnLoad \
                 stopped firing during XML parse (check that `mixin=` wires \
                 OnLoad before the frame is exposed as a global) or the \
                 RegisterEvent call rejected the event name (check the \
                 permissive event registry — unknown names should pass \
                 through, not raise)."
            );
        }
    });
}
