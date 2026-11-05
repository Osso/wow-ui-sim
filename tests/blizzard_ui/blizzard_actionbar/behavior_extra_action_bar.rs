//! Behavior pin: setting `state.extra_action_button.spell_id = Some(_)` and
//! firing `UPDATE_EXTRA_ACTIONBAR` flips `C_ActionBar.HasExtraActionBar()`
//! true, drives `ExtraActionBar_Update`'s `bar:Show()` branch, and
//! (with the matching `state.action_bars[ExtraActionButton1.action]`
//! seeded) populates `ExtraActionButton1.icon` with the spell's texture
//! after the standard `ACTIONBAR_SLOT_CHANGED` refresh hop.
//!
//! ## Source contract (`Interface/BlizzardUI/Blizzard_ActionBar/`,
//! `Interface/BlizzardUI/Blizzard_ActionBarController/`)
//!
//! 1. `ExtraActionBarFrame` is the
//!    `<Frame name="ExtraActionBarFrame" frameStrata="LOW"
//!     enableMouse="true" hidden="true">` declared at
//!    `Shared/ExtraActionBar.xml:93`. It is a plain `Frame` (not
//!    `EditModeActionBarTemplate`), so `:IsShown()` reads the literal
//!    Rust visibility flag — no override-mixin indirection like
//!    `PossessActionBar`. Its sole child button
//!    `ExtraActionButton1` (xml:116) inherits `ExtraActionButtonTemplate`
//!    (xml:3) which has `KeyValues` `isExtra=true` and
//!    `buttonType="EXTRAACTIONBUTTON"` (xml:84-85).
//!
//! 2. `ActionBarController_OnLoad`
//!    (`Blizzard_ActionBarController/ActionBarController.lua:8-52`)
//!    registers `UPDATE_EXTRA_ACTIONBAR` at lua:36.
//!    `ActionBarController_OnEvent` lua:91-93 routes that event to
//!    `ExtraActionBar_Update()`. Loading `Blizzard_ActionBarController` as
//!    the harness ROOT pulls in `Blizzard_ActionBar` (which owns
//!    `ExtraActionBar.lua`/`.xml`) and transitively
//!    `Blizzard_UIPanels_Game` (which owns `ExtraAbilityContainer`,
//!    referenced by `ExtraActionBar_Update` at lua:17, 28, 45, 59).
//!
//! 3. `ExtraActionBar_Update` (`Shared/ExtraActionBar.lua:10-30`):
//!    ```lua
//!    function ExtraActionBar_Update()
//!        local bar = ExtraActionBarFrame;
//!        if ( C_ActionBar.HasExtraActionBar() ) then
//!            bar:Show();
//!            local texture = C_ActionBar.GetOverrideBarSkin() or DefaultExtraActionStyle;
//!            bar.button.style:SetTexture(texture);
//!            bar.button:UpdateUsable();
//!            ExtraAbilityContainer:AddFrame(bar, ExtraActionButtonPriority);
//!            bar.outro:Stop();
//!            bar.intro:Play();
//!        elseif( bar:IsShown() ) then
//!            ...
//!        end
//!    end
//!    ```
//!    The `if` branch is selected by `C_ActionBar.HasExtraActionBar()`,
//!    which the simulator backs with
//!    `state.extra_action_button.spell_id.is_some()`
//!    (`src/lua_api/globals/action_bar_api.rs:378-385`). Note that
//!    `ExtraActionBar_Update` does NOT touch `bar.button.icon` — it only
//!    sets `bar.button.style:SetTexture(skin)` (the spell-push artwork
//!    layer, not the ability icon). The icon is populated by the standard
//!    action-button refresh path described in step 4.
//!
//! 4. `ExtraActionButton1`'s ability icon comes from the
//!    `ActionBarActionButtonMixin:Update` path
//!    (`Shared/ActionButton.lua:555-619`). `Update` reads
//!    `texture = C_ActionBar.GetActionTexture(self.action)` (lua:558) and
//!    at lua:617-619 calls `icon:SetTexture(texture)` when the lookup is
//!    non-nil. The button registers itself with
//!    `ActionBarButtonEventsFrame` at OnLoad (lua:454);
//!    `ActionBarButtonEventsFrameMixin:OnEvent` (lua:220-225) fans
//!    `ACTIONBAR_SLOT_CHANGED` to every registered button's
//!    `OnEvent` (lua:966-976), which gates on
//!    `arg1 == 0 or arg1 == tonumber(self.action)` and calls
//!    `:UpdateAction(true)` on match — `UpdateAction` invokes `:Update()`
//!    at lua:544 and refreshes the icon.
//!
//! 5. `:CalculateAction` for ExtraActionButton1: the simulator registers
//!    `CalculateAction` as a Rust method
//!    (`src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:283-303`)
//!    that returns `widget.user_id` directly — it does NOT replicate the
//!    Blizzard Lua chain
//!    (`SecureTemplates.lua:662-680` — `id + (page-1) * NUM_ACTIONBAR_BUTTONS`,
//!    where `page = C_ActionBar.GetExtraBarIndex() = 13` for `isExtra`
//!    buttons, yielding slot 145 in real WoW). Because XML
//!    `id="1"` (xml:116) seeds `user_id = 1`, `ExtraActionButton1.action`
//!    resolves to `1` in the simulator — the SAME slot as `ActionButton1`.
//!    The test therefore probes the live `.action` value rather than
//!    hardcoding `145`, so it pins simulator behavior rather than the
//!    real-WoW slot arithmetic. This is a known simulator simplification
//!    (separate from this task's contract); flagging it here so a future
//!    `CalculateAction` upgrade that splits extra-bar buttons onto their
//!    own slot does not silently fail this test.
//!
//! ## Why the test seeds `state.extra_action_button.spell_id`
//! rather than a Lua mutator
//!
//! Just like `IsPossessBarVisible` (cf. `behavior_possess_bar_show.rs`),
//! `HasExtraActionBar` is purely a server-pushed read — there is no
//! Lua-facing setter in the real client either. The grant comes from
//! quest/encounter scripts that the simulator's combat/quest model would
//! emit, but that model has no Lua mutator for "you just gained an
//! extra-action ability". Direct state mutation is the canonical write
//! seam, mirroring the pattern used by
//! `behavior_stance_select.rs:166-184` for `state.shapeshift_forms` and
//! `behavior_possess_bar_show.rs:229-232` for
//! `state.action_bar_state.possess_bar_visible`.
//!
//! ## Why the test seeds BOTH `extra_action_button.spell_id` AND
//! `state.action_bars[slot]`
//!
//! These pin two distinct contracts:
//!
//! - `extra_action_button.spell_id` drives `HasExtraActionBar()`, which
//!   is what `ExtraActionBar_Update` reads to decide whether to call
//!   `bar:Show()` (lua:13). This is the "should the bar be visible at
//!   all?" contract.
//! - `state.action_bars[slot]` drives `GetActionTexture(slot)`, which is
//!   what `ActionBarActionButtonMixin:Update` reads to populate the icon
//!   (lua:558, 617-619). This is the "what spell is bound to this slot?"
//!   contract.
//!
//! In real WoW the server pushes both pieces atomically (granting the
//! extra ability writes both to the slot's action and to the
//! HasExtraActionBar flag). The simulator splits them across two state
//! fields because action-slot persistence and per-quest-grant overlays
//! are independent concerns. Per `AGENTS.md` ("When a `C_*` function is
//! missing or wrong, default to implementing the backing system or state
//! model"), this test treats both fields as the canonical write seam
//! rather than coupling them via a hidden helper that auto-mirrors one
//! into the other.
//!
//! ## Why the test fires `UPDATE_EXTRA_ACTIONBAR` AND
//! `ACTIONBAR_SLOT_CHANGED` — not just one
//!
//! - `UPDATE_EXTRA_ACTIONBAR` only drives `ExtraActionBar_Update`, which
//!   shows the bar but does NOT touch the icon (lua:13-19 set the
//!   `style` artwork, never the `icon` texture).
//! - `ACTIONBAR_SLOT_CHANGED` only drives the button's icon refresh path
//!   via `OnEvent` lua:972-976 → `:UpdateAction(true)` → `:Update()`. It
//!   does NOT call `bar:Show()`.
//!
//! Both events are real-WoW co-emitted when the server grants an
//! extra-action ability. Firing them separately in the test pins each
//! event's contract independently; it would be a regression-detection
//! loss to fire only one and rely on side effects.
//!
//! ## Why the test fires events directly rather than calling
//! `ExtraActionBar_Update()` or `ExtraActionButton1:Update()`
//!
//! Direct method calls would prove the methods work in isolation but
//! would not catch a regression where `ActionBarController_OnLoad` stops
//! registering `UPDATE_EXTRA_ACTIONBAR` (lua:36) or
//! `ActionBarController_OnEvent` drops the routing arm at lua:91-93.
//! Firing the event proves the full registration → dispatch → Update
//! chain is wired. Same pattern as `behavior_possess_bar_show.rs:233-235`.
//!
//! ## Observations
//!
//! 1. **`ExtraActionBarFrame` and `ExtraActionButton1` exist after harness
//!    settle.** A nil reading means the XML didn't load (the controller
//!    didn't pull `Blizzard_ActionBar`, or the frame/button XML decl
//!    regressed).
//!
//! 2. **Cold-state `ExtraActionBarFrame:IsShown() == false` and
//!    `C_ActionBar.HasExtraActionBar() == false`.** Pinned by the
//!    `hidden="true"` xml:93 attribute (no EditMode override since this
//!    is a plain Frame) and by the default
//!    `state.extra_action_button.spell_id = None` (state.rs).
//!
//! 3. **After seeding `extra_action_button.spell_id = Some(spell)` AND
//!    `state.action_bars[slot] = spell`, then firing
//!    `UPDATE_EXTRA_ACTIONBAR`:**
//!    - `HasExtraActionBar() == true` (state-field read).
//!    - `ExtraActionBarFrame:IsShown() == true` (Update's success branch
//!      called `bar:Show()` at lua:13).
//!
//! 4. **After also firing `ACTIONBAR_SLOT_CHANGED arg=slot`,
//!    `ExtraActionButton1.icon:GetTexture()` is the spell icon's numeric
//!    fileDataID and `GetTextureFilePath()` preserves its manifest path.**
//!    Pins `ActionBarButtonEventsFrame` → `ExtraActionButton1:OnEvent` →
//!    `:UpdateAction(true)` → `:Update()` → `icon:SetTexture(texture)`,
//!    where `texture = C_ActionBar.GetActionTexture(slot)` resolved
//!    `state.action_bars[slot]` through `SPELL_DB` and
//!    `manifest_interface_data::get_texture_path`.
//!
//! ## Regression candidates the assertions catch
//!
//! - `ActionBarController_OnLoad` stops registering
//!   `UPDATE_EXTRA_ACTIONBAR` (lua:36): observation 3 fails (no listener
//!   picks up the fire), 1, 2, 4 still pass.
//! - `ActionBarController_OnEvent` drops the routing arm at lua:91-93:
//!   same as above.
//! - `ExtraActionBar_Update` regresses to always call the `else` branch
//!   regardless of `HasExtraActionBar()`: observation 3 fails.
//! - `HasExtraActionBar` regresses to a hardcoded boolean (the previous
//!   stub state): observation 2 fails (cold true) or 3 fails
//!   (post-seed false).
//! - `ActionBarButtonEventsFrame:RegisterFrame` not called for
//!   ExtraActionButton1 → observation 4 fails.
//! - The `arg1 == 0 or arg1 == tonumber(self.action)` gate inverted:
//!   observation 4 fails.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use rilua::Val;

const ROOT: &str = "Blizzard_ActionBarController";

/// Charge (warrior) — distinctive choice that is NOT in
/// `default_action_bars()` (Prot Paladin rotation, `game_data.rs:1111-1127`).
/// Picking a non-default spell guarantees the post-seed icon texture
/// differs from anything seeded by `SimState::default()`.
const SPELL_ID: u32 = 100;

/// Charge's icon identity. `GetTexture()` exposes the numeric fileDataID;
/// `GetTextureFilePath()` preserves the resolved manifest path.
const EXPECTED_ICON_FILE_DATA_ID: i64 = 132337;
const EXPECTED_ICON_SUFFIX: &str = "Ability_Warrior_Charge";

post_wrath_ui_fixture! {
#[test]
fn extra_action_button_show_and_icon_round_trip_through_update_events() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let cold_globals_exist: bool = env
            .eval(
                r#"
                return ExtraActionBarFrame ~= nil
                    and ExtraActionButton1 ~= nil
                    and ExtraActionButton1.icon ~= nil
                "#,
            )
            .expect("extra-action global existence probe must run cleanly");
        assert!(
            cold_globals_exist,
            "After the startup-shape harness loads `{ROOT}` (which \
             transitively pulls Blizzard_ActionBar), `ExtraActionBarFrame`, \
             `ExtraActionButton1`, and `ExtraActionButton1.icon` must \
             exist as globals/keys. `ExtraActionBarFrame` is the \
             `<Frame name=\"ExtraActionBarFrame\">` declared at \
             Shared/ExtraActionBar.xml:93. `ExtraActionButton1` is the \
             `<CheckButton name=\"ExtraActionButton1\">` at xml:116. \
             `ExtraActionButton1.icon` is the `$parentIcon` Texture \
             declared at xml:7 inside `ExtraActionButtonTemplate` (xml:3). \
             A nil reading means the XML didn't load, the controller \
             didn't pull Blizzard_ActionBar via dependency, or the \
             template's icon parentKey regressed."
        );

        let cold_is_shown: bool = env
            .eval("return ExtraActionBarFrame:IsShown()")
            .expect("cold IsShown probe must run cleanly");
        assert!(
            !cold_is_shown,
            "Cold-state `ExtraActionBarFrame:IsShown()` must be false. \
             The frame is declared `hidden=\"true\"` at \
             Shared/ExtraActionBar.xml:93 and is NOT \
             `EditModeActionBarTemplate`, so there is no override mixin \
             indirection — `:IsShown()` reads the literal Rust visibility \
             flag. A truthy cold reading means the harness fired \
             UPDATE_EXTRA_ACTIONBAR (or another Show path) before the \
             test got a chance to seed state."
        );

        let cold_has_extra: bool = env
            .eval("return C_ActionBar.HasExtraActionBar()")
            .expect("cold HasExtraActionBar probe must run cleanly");
        assert!(
            !cold_has_extra,
            "Cold-state `C_ActionBar.HasExtraActionBar()` must be false. \
             It reads `state.extra_action_button.spell_id.is_some()` \
             (action_bar_api.rs:378-385), which defaults to `None` per \
             `ExtraActionButtonState::default` (state.rs). A truthy cold \
             reading means the default flipped or another test left state \
             dirty."
        );

        let slot: u32 = env
            .eval::<f64>("return ExtraActionButton1.action")
            .expect("ExtraActionButton1.action probe must run cleanly")
            as u32;
        assert!(
            slot > 0,
            "`ExtraActionButton1.action` must resolve to a positive slot \
             after `UpdateAction` runs. The simulator's Rust \
             `:CalculateAction` returns `widget.user_id` (button_anchor_\
             hierarchy/buttons.rs:283-303); `ExtraActionButton1` is \
             declared with XML `id=\"1\"` (xml:116) which seeds \
             `user_id = 1`. A zero or negative reading means the user_id \
             plumbing or the AttributeChanged → UpdateAction kick \
             regressed."
        );

        {
            let mut state = env.state().borrow_mut();
            state.extra_action_button.spell_id = Some(SPELL_ID);
            state.action_bars.insert(slot, SPELL_ID);
        }

        env.fire_event("UPDATE_EXTRA_ACTIONBAR")
            .expect("UPDATE_EXTRA_ACTIONBAR fire must dispatch cleanly");

        let post_show_has_extra: bool = env
            .eval("return C_ActionBar.HasExtraActionBar()")
            .expect("post-show HasExtraActionBar probe must run cleanly");
        assert!(
            post_show_has_extra,
            "After seeding `state.extra_action_button.spell_id = Some(_)`, \
             `C_ActionBar.HasExtraActionBar()` must return true. Pinned by \
             the field read at action_bar_api.rs:378-385. A false reading \
             means the state field wasn't wired through to the C_ActionBar \
             namespace registration."
        );

        let post_show_is_shown: bool = env
            .eval("return ExtraActionBarFrame:IsShown()")
            .expect("post-show IsShown probe must run cleanly");
        assert!(
            post_show_is_shown,
            "After UPDATE_EXTRA_ACTIONBAR fires with `HasExtraActionBar()` \
             returning true, `ExtraActionBarFrame:IsShown()` must be \
             true. Pinned by `ExtraActionBar_Update`'s `if` branch at \
             Shared/ExtraActionBar.lua:13 (`bar:Show()`). Reaching that \
             branch requires every link in the chain: \
             `ActionBarController_OnLoad` registered \
             `UPDATE_EXTRA_ACTIONBAR` (ActionBarController.lua:36), \
             `ActionBarController_OnEvent` routed to \
             `ExtraActionBar_Update()` (lua:91-93), and \
             `HasExtraActionBar()` returned true. A false reading means \
             one of those links broke."
        );

        env.fire_event_with_args("ACTIONBAR_SLOT_CHANGED", &[Val::Num(slot as f64)])
            .expect("ACTIONBAR_SLOT_CHANGED fire must dispatch cleanly");

        let (icon_file_data_id, icon_path): (i64, String) = env
            .eval(
                "return ExtraActionButton1.icon:GetTexture(), \
                 ExtraActionButton1.icon:GetTextureFilePath()",
            )
            .expect("extra-action icon identity probe must run cleanly");
        assert_eq!(
            icon_file_data_id, EXPECTED_ICON_FILE_DATA_ID,
            "After seeding `state.action_bars[{slot}] = {SPELL_ID}`, \
             `ExtraActionButton1.icon:GetTexture()` must return Charge's \
             fileDataID. A mismatch means the event refresh did not set \
             the expected icon or texture identity stopped preserving \
             known fileDataIDs. Got path: {icon_path:?}."
        );
        assert!(
            icon_path
                .to_ascii_lowercase()
                .contains(&EXPECTED_ICON_SUFFIX.to_ascii_lowercase()),
            "`ExtraActionButton1.icon:GetTextureFilePath()` must preserve \
             Charge's resolved manifest path. Got: {icon_path:?}."
        );
    });
    }
}
}
