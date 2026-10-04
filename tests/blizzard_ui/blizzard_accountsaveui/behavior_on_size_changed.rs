//! Behavior pin: `OnSizeChanged` calls `Text:SetWidth` with
//! `ContentInsets:GetWidth()` so the SimpleHTML body reflows to the
//! current panel width.
//!
//! Source (`Interface/BlizzardUI/Blizzard_AccountSaveUI/
//! Blizzard_AccountSaveUI.lua`, lines 169-171):
//!
//! ```lua
//! function AccountSaveFrameMixin:OnSizeChanged()
//!     self.Text:SetWidth(self.ContentInsets:GetWidth());
//! end
//! ```
//!
//! `Blizzard_AccountSaveUI.xml:81` wires
//! `<OnSizeChanged method="OnSizeChanged"/>`, so any change to the
//! frame's effective size re-applies this invariant on the SimpleHTML
//! body.
//!
//! Why testing the invariant via "GetWidth() before vs after" doesn't
//! work: `Text` has both LEFT and RIGHT anchors to `ContentInsets`
//! (Blizzard_AccountSaveUI.xml:31-32), so its width is fully
//! determined by the anchor system regardless of explicit `SetWidth`
//! calls. A direct `Text:SetWidth(1)` to "break" the invariant simply
//! gets overridden by the next layout pass — the layout system
//! re-derives the width from the LEFT/RIGHT anchors and `Text:GetWidth()`
//! returns the anchor-derived value, not the explicit one. Verified
//! empirically: `Text:SetWidth(1)` followed by `Text:GetWidth()`
//! returns ~310 (the ContentInsets-derived width), not 1.
//!
//! Test strategy: wrap `Text.SetWidth` in a Lua-side capturing
//! closure (same `Mixin` per-instance __newindex shadowing pattern
//! used in `behavior_save_button_click.rs` and `behavior_update_sizing.rs`),
//! then call `OnSizeChanged` and read what argument was passed. This
//! pins the actual contract — what value `OnSizeChanged` requests —
//! without relying on the layout system to honor or reject it.
//!
//! Three regressions would surface as a captured-width mismatch:
//!   1. Body emptied or replaced with no-op — `captured_width` stays
//!      nil (returned as `-1.0` sentinel).
//!   2. Wrong dimension — `GetHeight()` instead of `GetWidth()`,
//!      `captured_width` would be ContentInsets:GetHeight().
//!   3. Wrong frame — `self:GetWidth()` instead of
//!      `self.ContentInsets:GetWidth()`, `captured_width` would be
//!      AccountSaveFrame's own 360 (vs. ContentInsets' ~310).
//!
//! `OnSizeChanged` is dispatched only by the per-tick layout pass, so the
//! test calls `OnSizeChanged` directly via the standard mixin dispatch. The
//! XML `<OnSizeChanged method="OnSizeChanged"/>` wiring is already
//! pinned by `surface_mixins.rs` (the method exists on the mixin) —
//! this fixture pins the method body's effect.

use crate::common::blizzard_addon_harness::with_blizzard_addon_glue_smoke_shape;

const ROOT: &str = "Blizzard_AccountSaveUI";
const MISSING_CALL_SENTINEL: f64 = -1.0;

#[test]
fn on_size_changed_calls_text_set_width_with_content_insets_width() {
    with_blizzard_addon_glue_smoke_shape(&[ROOT], &[], |env, _loaded| {
        {
            let mut state = env.state().borrow_mut();
            state.account_save_enabled = true;
            state.account_locked_post_save = false;
            state.account_save_in_progress = false;
        }

        let (captured_width, content_width, frame_width) = env
            .eval::<(f64, f64, f64)>(&format!(
                r#"
                assert(AccountSaveFrame, "AccountSaveFrame global must exist after Blizzard_AccountSaveUI load")

                AccountSaveFrame:UpdateAccountState()

                captured_set_width_arg = nil
                local original_set_width = AccountSaveFrame.Text.SetWidth
                AccountSaveFrame.Text.SetWidth = function(self, width)
                    captured_set_width_arg = width
                    return original_set_width(self, width)
                end

                AccountSaveFrame:OnSizeChanged()

                return captured_set_width_arg or {MISSING_CALL_SENTINEL},
                       AccountSaveFrame.ContentInsets:GetWidth(),
                       AccountSaveFrame:GetWidth()
                "#,
            ))
            .expect("OnSizeChanged Text:SetWidth-argument capture probe must run cleanly");

        assert!(
            content_width > 0.0,
            "ContentInsets:GetWidth() must report a positive width — otherwise the addon \
             never finished its layout pass and the assertions below are meaningless. Got \
             content_width = {content_width}."
        );

        assert!(
            (frame_width - content_width).abs() > 1.0,
            "AccountSaveFrame:GetWidth() and ContentInsets:GetWidth() must differ by more \
             than one pixel — otherwise the test can't distinguish a regression that uses \
             `self:GetWidth()` instead of `self.ContentInsets:GetWidth()`. \
             Blizzard_AccountSaveUI.xml:5 sets the frame to 360 wide; ContentInsets is inset \
             25 on each side (Blizzard_AccountSaveUI.xml:13-14), so they should differ by 50. \
             Got frame_width = {frame_width}, content_width = {content_width}."
        );

        assert!(
            captured_width != MISSING_CALL_SENTINEL,
            "OnSizeChanged must call Text:SetWidth at least once \
             (Blizzard_AccountSaveUI.lua:170 — `self.Text:SetWidth(...)`). The probe wraps \
             Text.SetWidth in a capturing closure that records the argument; a sentinel value \
             of {MISSING_CALL_SENTINEL} here means OnSizeChanged ran but never reached the \
             SetWidth call — the body was likely emptied or replaced with a no-op."
        );

        assert_eq!(
            captured_width, content_width,
            "OnSizeChanged must call Text:SetWidth with ContentInsets:GetWidth() \
             (Blizzard_AccountSaveUI.lua:170). The captured argument tells us what dimension \
             the addon is propagating to the SimpleHTML body. Three regressions would land here: \
             (a) wrong dimension — ContentInsets:GetHeight() instead of GetWidth() — \
             captured_width would match ContentInsets height; \
             (b) wrong frame — self:GetWidth() instead of self.ContentInsets:GetWidth() — \
             captured_width would equal frame_width ({frame_width}) instead of content_width \
             ({content_width}); \
             (c) hardcoded literal — captured_width would be a fixed value independent of the \
             panel layout. \
             Got captured_width = {captured_width}, expected {content_width}."
        );
    });
}
