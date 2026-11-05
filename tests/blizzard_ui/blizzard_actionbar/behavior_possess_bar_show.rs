//! Behavior pin: seeding `state.action_bar_state.possess_bar_visible = true`
//! and firing `UPDATE_POSSESS_BAR` causes `PossessActionBar:Update()` to call
//! `:Show()`, flipping its `isShownExternal` to true; clearing the field and
//! firing again drives the matching `:Hide()` back to false.
//!
//! ## PLAN/source-code mismatch
//!
//! The PLAN entry describes this as
//! *"entering a vehicle (`UnitHasVehicleUI(\"player\") = true`) makes
//! `PossessActionBar` visible; exiting hides it"*. That description is the
//! **opposite** of what the source code does. `PossessActionBarMixin:Update`
//! (`Blizzard_ActionBar/Shared/PossessActionBar.lua:10-22`) reads:
//!
//! ```lua
//! function PossessActionBarMixin:Update()
//!     if ( not MainActionBar.busy and not UnitHasVehicleUI("player") ) then
//!         if ( C_ActionBar.IsPossessBarVisible() ) then
//!             if ( not self:IsShown() ) then self:Show(); end
//!             self:UpdateState();
//!         elseif ( self:IsShown() ) then
//!             self:Hide();
//!         end
//!     end
//! end
//! ```
//!
//! `UnitHasVehicleUI("player") = true` actually **early-outs** Update — the
//! whole show/hide body is gated behind `not UnitHasVehicleUI("player")`. The
//! real driver of the show/hide transition is
//! `C_ActionBar.IsPossessBarVisible()`, which on the server side flips true
//! when a possession (mind-control / pet possess / non-vehicle override)
//! grants the player the 2-slot possess bar. The vehicle-UI flag is what
//! prevents Update from clobbering the bar mid-vehicle-mount animation.
//!
//! The test pins the **actual source contract** rather than the inverted
//! PLAN description. Per `AGENTS.md`, "When a `C_*` function is missing or
//! wrong, default to implementing the backing system or state model. Do not
//! reach for shims just to satisfy a failing test." Before this test landed,
//! `C_ActionBar.IsPossessBarVisible` was a no-op stub
//! (`src/lua_api/globals/action_bar_api.rs:382-384`) returning hardcoded
//! `false`. The test wires it to a new
//! `state.action_bar_state.possess_bar_visible` flag and proves the
//! event-driven Show/Hide round trip works through that backing field.
//!
//! ## Source contract (`Interface/BlizzardUI/Blizzard_ActionBar/`,
//! `Interface/BlizzardUI/Blizzard_ActionBarController/`)
//!
//! 1. `PossessActionBar` is the
//!    `<Frame name="PossessActionBar" parent="UIParent" inherits="EditModeActionBarTemplate" mixin="PossessActionBarMixin" hidden="true">`
//!    declared at `Mainline/PossessActionBar.xml:13`. `numButtons = 2`
//!    (xml:22) creates `PossessButton1` and `PossessButton2` via
//!    `ActionBar_OnLoad` (`Shared/ActionBar.lua:3-44`).
//!
//! 2. The bar inherits `EditModeActionBarTemplate`, so
//!    `EditModeActionBarMixin:SetupVisibilityFunctionOverrides`
//!    (`ActionBar.lua:266-277`) replaces `Show`/`Hide`/`IsShown`/`SetShown`
//!    with their `*Override` variants. `ShowOverride` (lua:316-321) writes
//!    `self.isShownExternal = true` then calls `self:UpdateVisibility()`;
//!    `HideOverride` (lua:323-332) writes `false`. So after a `Update`-driven
//!    `:Show()` call, `self.isShownExternal == true` is the most direct
//!    observable that proves `Update`'s show branch ran.
//!
//! 3. `PossessActionBarMixin:Update` (`Shared/PossessActionBar.lua:10-22`):
//!    - Outer gate at lua:11: `not MainActionBar.busy and not
//!      UnitHasVehicleUI("player")`. The cold simulator state has both gates
//!      open (`state.action_bar_state.busy = false` and
//!      `state.player.has_vehicle_ui = false`), so Update's body runs.
//!    - Inner branch at lua:12: `C_ActionBar.IsPossessBarVisible()` selects
//!      between Show (when true and not currently shown) and Hide (when
//!      false and currently shown).
//!    - The `not self:IsShown()` / `self:IsShown()` guards mean Show is only
//!      called when transitioning from hidden → visible, and Hide only when
//!      transitioning from visible → hidden.
//!
//! 4. `ActionBarController_OnLoad`
//!    (`Blizzard_ActionBarController/ActionBarController.lua:33`) registers
//!    `UPDATE_POSSESS_BAR`. `ActionBarController_OnEvent` lua:85-88 routes
//!    that event to `PossessActionBar:Update()` (and `StanceBar:Update()`).
//!    The controller frame is created in `ActionBarController.xml:3-8`
//!    parented to UIParent. So firing `UPDATE_POSSESS_BAR` directly is
//!    equivalent to the canonical event path the server uses to notify
//!    clients of possess-bar-state changes.
//!
//! ## Why the test seeds `state.action_bar_state.possess_bar_visible`
//! rather than calling some Lua mutator
//!
//! The simulator has no Lua-facing mutator for "the player just entered a
//! mind-control / possess effect". `IsPossessBarVisible` is purely a server
//! read (the real client never writes it from Lua either). The canonical
//! write seam is the simulator's possession/buff model — which the test
//! fixture stands in for via direct state mutation. This is the same pattern
//! `behavior_stance_select.rs:166-184` uses for `state.shapeshift_forms`.
//!
//! ## Why the test reads `isShownExternal` rather than `:IsShown()`
//!
//! `EditModeActionBarMixin:IsShownOverride` (`ActionBar.lua:296-306`)
//! returns either `self.isShownExternal` or `self:IsShownBase()` depending
//! on EditMode `self.visibility` resolution
//! (`UpdateSystemSettingVisibleSetting` at
//! `EditModeSystemTemplates.lua:1128-1139`). That makes `:IsShown()` a
//! signal that mixes "did Update call Show?" with "what does EditMode
//! visibility resolution say?" — two separate contracts. `isShownExternal`
//! is written *only* by `ShowOverride`/`HideOverride`, so it cleanly pins
//! the Show/Hide call without coupling the test to EditMode's visibility
//! resolution. The same approach is used by `behavior_pet_bar_update.rs:55-64`.
//!
//! ## Why the test fires `UPDATE_POSSESS_BAR` directly rather than calling
//! `PossessActionBar:Update()`
//!
//! Calling Update directly would prove `Update` works in isolation but
//! would not catch a regression where `ActionBarController_OnLoad` stops
//! registering `UPDATE_POSSESS_BAR` (lua:33) or
//! `ActionBarController_OnEvent` drops the routing arm at lua:85-88. Firing
//! the event proves the full registration → dispatch → Update chain is wired.
//!
//! ## Observations
//!
//! 1. **`PossessActionBar`, `PossessButton1`, `PossessButton2` exist after
//!    harness settle.** A nil reading means the XML didn't load (TOC walk
//!    regressed) or `ActionBar_OnLoad`'s implicit `numButtons` button-
//!    creation loop regressed.
//!
//! 2. **Cold-state `PossessActionBar.isShownExternal == false` and
//!    `C_ActionBar.IsPossessBarVisible() == false`.** The bar is declared
//!    `hidden="true"` (xml:13), so `EditModeActionBar_OnLoad`'s
//!    `self.isShownExternal = self:IsShown()` (lua:260) seeds it false.
//!    `IsPossessBarVisible` reads the new
//!    `state.action_bar_state.possess_bar_visible` field, which defaults
//!    to false. A non-clean cold reading means the harness is leaking
//!    possess state from somewhere the test can't observe.
//!
//! 3. **After seeding `possess_bar_visible = true` and firing
//!    `UPDATE_POSSESS_BAR`, `PossessActionBar.isShownExternal == true` and
//!    `IsPossessBarVisible() == true`.** This pins the full event chain:
//!    `ActionBarController` heard the event (lua:85-88 routing arm), called
//!    `PossessActionBar:Update()`, the outer gate passed
//!    (`!MainActionBar.busy && !UnitHasVehicleUI("player")` both true), the
//!    inner `IsPossessBarVisible()` branch ran (PossessActionBar.lua:12),
//!    and `self:Show()` resolved to `ShowOverride` which wrote
//!    `isShownExternal = true` (ActionBar.lua:316-321).
//!
//! 4. **After clearing `possess_bar_visible = false` and firing
//!    `UPDATE_POSSESS_BAR` again, `PossessActionBar.isShownExternal == false`
//!    and `IsPossessBarVisible() == false`.** This pins the Hide branch
//!    at PossessActionBar.lua:18-20: the bar is currently shown
//!    (precondition from observation 3), `IsPossessBarVisible()` is now
//!    false, so the `elseif ( self:IsShown() ) then self:Hide()` arm runs
//!    and `HideOverride` writes `isShownExternal = false`.
//!
//! ## Regression candidates the assertions catch
//!
//! - `ActionBarController_OnLoad` stops registering `UPDATE_POSSESS_BAR`
//!   (lua:33): observations 3 and 4 both fail (no listener picks up the
//!   fire), 1 and 2 still pass.
//! - `ActionBarController_OnEvent` drops the `UPDATE_POSSESS_BAR` routing
//!   arm at lua:85-88: same as above.
//! - `PossessActionBarMixin:Update` regresses to always call Show or always
//!   call Hide regardless of `IsPossessBarVisible()`: observation 3 fails
//!   (always-hide) or observation 4 fails (always-show).
//! - The outer `not UnitHasVehicleUI("player")` gate at lua:11 inverts:
//!   observations 3 and 4 both fail because the simulator's cold-state
//!   `has_vehicle_ui = false` would now skip Update entirely.
//! - `IsPossessBarVisible` regresses to a hardcoded boolean (the previous
//!   stub state): observation 3 fails (stuck at false) or 4 fails (stuck
//!   at true).

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ActionBarController";

#[test]
fn possess_bar_show_hide_round_trips_through_update_possess_bar_event() {
    wrath_action_case!(possess_visibility);

    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let cold_globals_exist: bool = env
            .eval(
                r#"
                return PossessActionBar ~= nil
                    and PossessButton1 ~= nil
                    and PossessButton2 ~= nil
                "#,
            )
            .expect("possess bar global existence probe must run cleanly");
        assert!(
            cold_globals_exist,
            "After the startup-shape harness loads `{ROOT}` (which transitively \
             pulls Blizzard_ActionBar), `PossessActionBar`, `PossessButton1`, \
             and `PossessButton2` must exist as globals. `PossessActionBar` is \
             the `<Frame name=\"PossessActionBar\">` declared at \
             Mainline/PossessActionBar.xml:13. The buttons are created by \
             `ActionBar_OnLoad` (Shared/ActionBar.lua:3-44) reading the \
             `numButtons = 2` KeyValue at xml:22. A nil reading means the XML \
             didn't load, the controller didn't pull Blizzard_ActionBar via \
             dependency, or the button-creation loop regressed."
        );

        let cold_shown_external: bool = env
            .eval("return PossessActionBar.isShownExternal == true")
            .expect("cold-state isShownExternal probe must run cleanly");
        assert!(
            !cold_shown_external,
            "Cold-state `PossessActionBar.isShownExternal` must be false. The \
             frame is declared `hidden=\"true\"` at \
             Mainline/PossessActionBar.xml:13, and \
             `EditModeActionBarMixin:EditModeActionBar_OnLoad` at \
             ActionBar.lua:260 seeds `self.isShownExternal = self:IsShown()` \
             before `SetupVisibilityFunctionOverrides` swaps out the \
             methods. So after harness settle the flag is the literal cold \
             frame visibility (false). A truthy cold reading means the \
             harness fired UPDATE_POSSESS_BAR (or another Show path) before \
             the test got a chance to seed state."
        );

        let cold_is_possess_bar_visible: bool = env
            .eval("return C_ActionBar.IsPossessBarVisible() == true")
            .expect("cold IsPossessBarVisible probe must run cleanly");
        assert!(
            !cold_is_possess_bar_visible,
            "Cold-state `C_ActionBar.IsPossessBarVisible()` must be false. \
             It reads `state.action_bar_state.possess_bar_visible` \
             (action_bar_api.rs:382-385), which defaults to false per \
             `ActionBarStateInfo::default` (state.rs). A truthy cold \
             reading means the default flipped or another test left state \
             dirty."
        );

        env.state()
            .borrow_mut()
            .action_bar_state
            .possess_bar_visible = true;

        env.fire_event("UPDATE_POSSESS_BAR")
            .expect("UPDATE_POSSESS_BAR fire must dispatch cleanly to ActionBarController_OnEvent");

        let post_show_is_visible: bool = env
            .eval("return C_ActionBar.IsPossessBarVisible() == true")
            .expect("post-show IsPossessBarVisible probe must run cleanly");
        assert!(
            post_show_is_visible,
            "After seeding `state.action_bar_state.possess_bar_visible = true`, \
             `C_ActionBar.IsPossessBarVisible()` must return true. Pinned by \
             the field read at action_bar_api.rs:382-385. A false reading \
             means the state field wasn't wired through to the C_ActionBar \
             namespace registration."
        );

        let post_show_external: bool = env
            .eval("return PossessActionBar.isShownExternal == true")
            .expect("post-show isShownExternal probe must run cleanly");
        assert!(
            post_show_external,
            "After UPDATE_POSSESS_BAR fires with `IsPossessBarVisible()` \
             returning true, `PossessActionBar.isShownExternal` must be \
             true. Pinned by `EditModeActionBarMixin:ShowOverride` at \
             ActionBar.lua:316-321 (`self.isShownExternal = true`), which \
             is what `:Show()` resolves to after \
             `SetupVisibilityFunctionOverrides`. Reaching `Show` requires \
             every link in the chain: `ActionBarController_OnLoad` \
             registered `UPDATE_POSSESS_BAR` (ActionBarController.lua:33), \
             `ActionBarController_OnEvent` routed to \
             `PossessActionBar:Update()` (lua:85-88), \
             `PossessActionBarMixin:Update` outer gate passed \
             (`!MainActionBar.busy && !UnitHasVehicleUI(\"player\")` — \
             both seeded false), the inner `IsPossessBarVisible()` branch \
             at PossessActionBar.lua:12 read true, and the \
             `not self:IsShown()` guard at lua:13 read true (cold state). \
             A false reading means one of those links broke."
        );

        env.state()
            .borrow_mut()
            .action_bar_state
            .possess_bar_visible = false;

        env.fire_event("UPDATE_POSSESS_BAR")
            .expect("second UPDATE_POSSESS_BAR fire must dispatch cleanly");

        let post_hide_is_visible: bool = env
            .eval("return C_ActionBar.IsPossessBarVisible() == true")
            .expect("post-hide IsPossessBarVisible probe must run cleanly");
        assert!(
            !post_hide_is_visible,
            "After clearing `state.action_bar_state.possess_bar_visible`, \
             `C_ActionBar.IsPossessBarVisible()` must return false again. \
             Mirror of the post-show check; same wiring."
        );

        let post_hide_external: bool = env
            .eval("return PossessActionBar.isShownExternal == true")
            .expect("post-hide isShownExternal probe must run cleanly");
        assert!(
            !post_hide_external,
            "After UPDATE_POSSESS_BAR fires with `IsPossessBarVisible()` \
             back to false, `PossessActionBar.isShownExternal` must be \
             false. Pinned by `EditModeActionBarMixin:HideOverride` at \
             ActionBar.lua:323-332 (`self.isShownExternal = false`), which \
             is what `:Hide()` resolves to. The Hide branch at \
             PossessActionBar.lua:18-20 only runs when the bar is currently \
             shown — observation 3 establishes that precondition. A truthy \
             reading means the Hide arm regressed (e.g., the elseif at \
             lua:18 inverted, or `IsPossessBarVisible()` stayed pinned to \
             true)."
        );
    });
    }
}
