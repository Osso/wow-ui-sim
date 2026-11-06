//! Behavior pin: `StanceBar:Select(2)` fans through `CastShapeshiftForm(2)`,
//! which marks form #2 active in sim state and fires `UPDATE_SHAPESHIFT_FORM`.
//! `Blizzard_ActionBarController` observes that event and calls
//! `StanceBar:Update()`, which routes through `UpdateState` and flips the
//! checked state of `StanceButton2`. The whole round trip is synchronous.
//!
//! Source contract (`Interface/BlizzardUI/Blizzard_ActionBar/`,
//! `Interface/BlizzardUI/Blizzard_ActionBarController/`):
//!
//! 1. `StanceBarMixin:Select(id)`
//!    (`Blizzard_ActionBar/Shared/StanceBar.lua:102-105`) writes
//!    `self.lastSelected = id` and calls `CastShapeshiftForm(id)`. There is
//!    no gate, no event-loop bounce — this is a straight function call. So
//!    after `StanceBar:Select(2)` returns, the simulator has had a chance to
//!    react to whatever `CastShapeshiftForm` does.
//!
//! 2. `CastShapeshiftForm(index)`
//!    (`src/lua_api/globals/shapeshift.rs:101-125`) is a Rust global. It
//!    clears `is_active` on every entry of `state.shapeshift_forms`, toggles
//!    the indexed form (so the second `CastShapeshiftForm(2)` after a first
//!    would deselect it), and fires `UPDATE_SHAPESHIFT_FORM` synchronously
//!    via `fire_named_event_state`. Out-of-range indexes silently no-op.
//!
//! 3. `ActionBarController_OnLoad` registers `UPDATE_SHAPESHIFT_FORM` along
//!    with `UPDATE_SHAPESHIFT_FORMS` and `UPDATE_SHAPESHIFT_USABLE`
//!    (`Blizzard_ActionBarController/ActionBarController.lua:27-29`).
//!    `ActionBarController_OnEvent` lua:78-82 dispatches all three to
//!    `StanceBar:Update()`. The controller frame is created in
//!    `ActionBarController.xml:3-8` parented to UIParent, so it picks up
//!    events as soon as the addon loads.
//!
//! 4. `StanceBarMixin:Update()`
//!    (`Blizzard_ActionBar/Shared/StanceBar.lua:27-44`) reads
//!    `GetNumShapeshiftForms()` into `self.numForms`, and when `numForms > 0`
//!    calls `self:UpdateBackgroundArt()` then `self:UpdateState()`. The
//!    `numForms > 0` gate is what makes the form-state pre-population step
//!    in this test load-bearing: with the cold-start empty
//!    `state.shapeshift_forms`, `Update` would early-out at lua:34 and never
//!    reach `UpdateState`.
//!
//! 5. `StanceBarMixin:UpdateState()` (lua:58-100) iterates
//!    `self.actionButtons` (the table populated by `ActionBar_OnLoad` —
//!    `ActionBar.lua:13-44` creates `numButtons = 10` `<KeyValue>` per
//!    `Mainline/StanceBar.xml:23` buttons named `StanceButton1`..`StanceButton10`
//!    via the explicit `if self == StanceBar then buttonName = "StanceButton"..i`
//!    branch at `ActionBar.lua:21-22`). For each `i <= numForms` it calls
//!    `GetShapeshiftFormInfo(i)` → `(texture, isActive, isCastable, spellID)`,
//!    and at lua:80-85 dispatches `button:SetChecked(isActive)`. Because
//!    `CastShapeshiftForm` cleared every other form before activating the
//!    target, exactly one button transitions to checked per Select call —
//!    and `StanceBar.lastSelected` already equals that index because the
//!    Select method wrote it before calling Cast.
//!
//! Why the test sets up two forms (Bear=spell 5487, Cat=spell 768) instead
//! of just one: the round-trip pin needs the *transition* to be observable,
//! so we need a button that goes from unchecked → checked in response to
//! Select(2). With one form, Select(1) would be ambiguous between "the
//! checked transition fired" and "the cold state is already checked because
//! form 1 was the only candidate". The two-form fixture forces the bar to
//! choose, and the test asserts both that #2 became checked and that #1
//! stayed unchecked — pinning the cross-button isolation.
//!
//! Why the test calls `StanceBar:Update()` after populating sim state but
//! before the Select call: the simulator seeds Paladin aura forms by default,
//! while this test needs a two-form Bear/Cat fixture for a precise transition.
//! Re-running Update after overwriting the state sets `self.numForms = 2` and
//! dispatches the first `UpdateState` call — at this point both buttons read
//! `isActive = false` so both are unchecked. That gives a clean cold state for
//! the Select(2) round trip.
//!
//! Why the test pre-populates state via the Rust `state().borrow_mut()`
//! seam rather than firing some Lua event: there is no sim-side Lua API for
//! "register a stance form" — `state.shapeshift_forms` is the canonical
//! source, written only by Rust (the simulator's class/spec/level system,
//! or test fixtures). `tests/c_shapeshift_globals.rs:14-32` uses the same
//! Bear+Cat fixture for the unit-level cast tests; this behavior test
//! reuses the data shape so the same form indexes match.
//!
//! The test pins the following observations:
//!   1. **`StanceBar` global exists and `StanceButton1`/`StanceButton2`
//!      globals exist after harness settle.** These are the
//!      `<Frame name="StanceBar">` element from `Mainline/StanceBar.xml:12`
//!      and the buttons created by `ActionBar_OnLoad` lua:31. A nil reading
//!      means the XML didn't load or the `if self == StanceBar` branch at
//!      `ActionBar.lua:21-22` regressed.
//!   2. **After populating two forms and calling `StanceBar:Update()`,
//!      `StanceBar.numForms == 2` and both buttons are unchecked.** This is
//!      the test fixture's clean cold state — proves the bar's Update read
//!      `GetNumShapeshiftForms()` (which reads
//!      `state.shapeshift_forms.len()`) and that `UpdateState` ran and
//!      called `SetChecked(false)` on each button (since both forms have
//!      `is_active = false` initially).
//!   3. **After `StanceBar:Select(2)`, `state.shapeshift_forms[1].is_active`
//!      is true and `state.shapeshift_forms[0].is_active` is false.** This
//!      pins the `CastShapeshiftForm` side effect: the indexed form
//!      activated, the other forms cleared. A different reading means
//!      either Select didn't call Cast, Cast had a bug in its toggle/clear
//!      math, or the form index off-by-one regressed.
//!   4. **After Select, `StanceBar.lastSelected == 2`.** Pinned directly by
//!      `Select` at lua:103 before the Cast call. This is the only piece
//!      `Select` writes directly — everything else is downstream of Cast.
//!   5. **After Select, `StanceButton2:GetChecked()` is true and
//!      `StanceButton1:GetChecked()` is false.** This is the round-trip
//!      payoff: `Select` → `Cast` → fires `UPDATE_SHAPESHIFT_FORM` →
//!      controller routes to `StanceBar:Update` → `UpdateState` iterates
//!      and flips the checked state. A false reading on `StanceButton2`
//!      means the controller event dispatch broke (the addon didn't load,
//!      or its OnLoad didn't register the event, or its OnEvent dispatch
//!      regressed). A true reading on `StanceButton1` means
//!      `CastShapeshiftForm` failed to clear non-target forms, or
//!      `UpdateState` skipped the `else` branch at lua:84 that calls
//!      `SetChecked(false)`.
//!
//! Regression candidates the assertions catch:
//!   - `Select` no longer calls `CastShapeshiftForm` (refactor reorders or
//!     drops the call): observation 3 fails (no form activates), 4 still
//!     passes (`lastSelected` is set unconditionally), 5 fails on #2.
//!   - `CastShapeshiftForm` toggle clobbers the indexed form when toggling
//!     off (the cold-state path picks the wrong branch): observation 3
//!     fails inverted (#2 stays inactive).
//!   - `Blizzard_ActionBarController` doesn't load (e.g. dependency
//!     regression): the chain breaks at step 3 (no UPDATE_SHAPESHIFT_FORM
//!     listener), so `StanceBar:Update` never runs after Cast. Observations
//!     3 and 4 still pass, observation 5 fails on #2.
//!   - `UpdateState` short-circuits before the `SetChecked` call (e.g. the
//!     `i <= numForms` gate at lua:66 inverted): observations 3 and 4
//!     still pass, observation 5 fails on #2.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

use wow_ui_sim::lua_api::state::ShapeshiftForm;

const ROOT: &str = "Blizzard_ActionBarController";
const TARGET_INDEX: i32 = 2;

// `with_blizzard_addon_startup_shape` loads `panel_fixtures::PANEL_ADDONS`,
// which requires `Blizzard_TimerunningUtil` (tests/common/panel_fixtures.rs:119-133).
// Its pinned TOC declares `## AllowLoadGameType: mainline`
// (Blizzard_TimerunningUtil/Blizzard_TimerunningUtil.toc:5), so the shared panel
// harness cannot build on Mists/Era/Anniversary and the seeded-aura assertions are
// never reached there. Wrath keeps its own `wrath_action_case!` path.
timerunning_util_panel_or_wrath_fixture! {
#[test]
fn seeded_paladin_aura_forms_populate_stance_bar_buttons() {
    wrath_action_case!(stance_icons);

    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        env.exec("StanceBar:Update()")
            .expect("seeded Paladin StanceBar update must run cleanly");

        let (num_forms, shown, first_spell, second_spell, third_spell, fourth_visible): (
            i32,
            bool,
            i32,
            i32,
            i32,
            bool,
        ) = env
            .eval(
                r#"
                return StanceBar.numForms,
                    StanceBar:IsShown() and true or false,
                    StanceButton1.spellID or 0,
                    StanceButton2.spellID or 0,
                    StanceButton3.spellID or 0,
                    StanceButton4:IsShown() and true or false
                "#,
            )
            .expect("seeded Paladin StanceBar button probe must run cleanly");

        assert_eq!(num_forms, 3);
        assert!(shown, "StanceBar should show when the player has Paladin aura forms");
        assert_eq!(first_spell, 465);
        assert_eq!(second_spell, 32223);
        assert_eq!(third_spell, 183435);
        assert!(
            !fourth_visible,
            "Only the three seeded Paladin aura buttons should be visible"
        );
    });
    }
}
}

#[test]
fn stance_bar_select_fans_cast_shapeshift_form_and_flips_target_button_to_checked() {
    wrath_action_case!(stance_selection);

    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
        let stance_bar_exists: bool = env
            .eval(
                r#"
                return StanceBar ~= nil
                    and StanceButton1 ~= nil
                    and StanceButton2 ~= nil
                "#,
            )
            .expect("stance bar global existence probe must run cleanly");
        assert!(
            stance_bar_exists,
            "After the startup-shape harness loads `{ROOT}` (which depends \
             on `Blizzard_ActionBar`), `StanceBar`, `StanceButton1`, and \
             `StanceButton2` must exist as globals. `StanceBar` is the \
             `<Frame name=\"StanceBar\">` declared in \
             Mainline/StanceBar.xml:12, parented to UIParent. The buttons \
             are created by `ActionBar_OnLoad` (Shared/ActionBar.lua:13-44) \
             via the explicit `if self == StanceBar then buttonName = \
             \"StanceButton\"..i` naming branch at lua:21-22. A nil reading \
             on any of them means the XML didn't load (TOC walk regressed) \
             or the StanceBar naming branch regressed."
        );

        {
            let mut state = env.state().borrow_mut();
            state.shapeshift_forms = vec![
                ShapeshiftForm {
                    name: "Bear Form".to_string(),
                    texture: "Interface/Icons/Ability_Racial_BearForm".to_string(),
                    spell_id: 5487,
                    is_active: false,
                    is_castable: true,
                },
                ShapeshiftForm {
                    name: "Cat Form".to_string(),
                    texture: "Interface/Icons/Ability_Druid_CatForm".to_string(),
                    spell_id: 768,
                    is_active: false,
                    is_castable: true,
                },
            ];
        }

        env.exec("StanceBar:Update()")
            .expect("StanceBar:Update fixture refresh must run cleanly");

        let num_forms_after_update: i32 = env
            .eval("return StanceBar.numForms or -1")
            .expect("StanceBar.numForms post-Update probe must run cleanly");
        assert_eq!(
            num_forms_after_update, 2,
            "After populating `state.shapeshift_forms` with two entries and \
             calling `StanceBar:Update()`, `StanceBar.numForms` must equal \
             2. `StanceBarMixin:Update` (Shared/StanceBar.lua:28) reads \
             `GetNumShapeshiftForms()` (registered in \
             src/lua_api/globals/social_probes.rs) which returns \
             `state.shapeshift_forms.len()`. A different value means either \
             the state-population path regressed, the GetNumShapeshiftForms \
             reader regressed, or Update isn't writing the field. Got: \
             {num_forms_after_update}."
        );

        let pre_select_button2_checked: bool = env
            .eval("return StanceButton2:GetChecked() and true or false")
            .expect("StanceButton2 pre-Select GetChecked probe must run cleanly");
        let pre_select_button1_checked: bool = env
            .eval("return StanceButton1:GetChecked() and true or false")
            .expect("StanceButton1 pre-Select GetChecked probe must run cleanly");
        assert!(
            !pre_select_button1_checked && !pre_select_button2_checked,
            "After the fixture Update, both StanceButton1 and StanceButton2 \
             must be unchecked. Both forms have `is_active = false` in the \
             fixture, so `UpdateState` (Shared/StanceBar.lua:80-85) takes \
             the `else` branch and calls `SetChecked(false)`. A true \
             reading here means the cold state isn't clean — the test \
             can't observe a transition. Got: button1={pre_select_button1_checked}, \
             button2={pre_select_button2_checked}."
        );

        env.exec(&format!("StanceBar:Select({TARGET_INDEX})"))
            .expect("StanceBar:Select must be callable as a method on the StanceBar global");

        let last_selected: i32 = env
            .eval("return StanceBar.lastSelected or -1")
            .expect("StanceBar.lastSelected post-Select probe must run cleanly");
        assert_eq!(
            last_selected, TARGET_INDEX,
            "After `StanceBar:Select({TARGET_INDEX})`, `StanceBar.lastSelected` \
             must equal {TARGET_INDEX}. `StanceBarMixin:Select` \
             (Shared/StanceBar.lua:102-105) writes `self.lastSelected = id` \
             at lua:103 unconditionally before calling \
             `CastShapeshiftForm(id)`. A different value means Select was \
             refactored to skip this write or write a different argument. \
             Got: {last_selected}."
        );

        let (target_active, other_active) = {
            let state = env.state().borrow();
            let target = state.shapeshift_forms[1].is_active;
            let other = state.shapeshift_forms[0].is_active;
            (target, other)
        };
        assert!(
            target_active && !other_active,
            "After `StanceBar:Select({TARGET_INDEX})`, sim state must have \
             `shapeshift_forms[1].is_active = true` and \
             `shapeshift_forms[0].is_active = false`. \
             `StanceBarMixin:Select` (Shared/StanceBar.lua:104) calls \
             `CastShapeshiftForm({TARGET_INDEX})`, which clears every form's \
             `is_active` and toggles the target \
             (src/lua_api/globals/shapeshift.rs:101-125). A false reading on \
             the target means Select didn't call Cast, Cast's index math \
             regressed, or Cast's toggle picked the wrong branch. A true \
             reading on the other form means Cast skipped the clear loop. \
             Got: target_active={target_active}, other_active={other_active}."
        );

        let post_select_button2_checked: bool = env
            .eval("return StanceButton2:GetChecked() and true or false")
            .expect("StanceButton2 post-Select GetChecked probe must run cleanly");
        let post_select_button1_checked: bool = env
            .eval("return StanceButton1:GetChecked() and true or false")
            .expect("StanceButton1 post-Select GetChecked probe must run cleanly");
        assert!(
            post_select_button2_checked && !post_select_button1_checked,
            "After `StanceBar:Select({TARGET_INDEX})`, `StanceButton2` must \
             be checked and `StanceButton1` must be unchecked. The chain: \
             Select calls `CastShapeshiftForm({TARGET_INDEX})` \
             (StanceBar.lua:104), which fires `UPDATE_SHAPESHIFT_FORM` \
             synchronously (shapeshift.rs:122). \
             `Blizzard_ActionBarController` registers that event \
             (ActionBarController.lua:27) and dispatches \
             `StanceBar:Update()` at lua:78-82. `Update` calls `UpdateState` \
             which iterates `self.actionButtons`, reads \
             `GetShapeshiftFormInfo(i)` per index, and dispatches \
             `SetChecked(isActive)` at lua:80-85. A false reading on \
             button2 means either the controller addon didn't load \
             (dependency regression), `Blizzard_ActionBarController` \
             stopped registering UPDATE_SHAPESHIFT_FORM, the OnEvent \
             dispatch at lua:78-82 regressed, or `UpdateState`'s SetChecked \
             call at lua:82 doesn't fire. A true reading on button1 means \
             Cast failed to clear the previously-active form, or \
             UpdateState skipped the else branch. Got: \
             button1={post_select_button1_checked}, \
             button2={post_select_button2_checked}."
        );
    });
    }
}
