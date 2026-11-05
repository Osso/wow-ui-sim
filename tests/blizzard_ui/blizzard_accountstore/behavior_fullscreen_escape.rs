//! Behavior pin for `FullscreenAccountStoreContainerMixin:OnKeyDown` ESCAPE
//! dispatch.
//!
//! Spec/source mismatch finding (PLAN.md task:
//! `FullscreenAccountStoreContainerMixin:OnKeyDown("ESCAPE")` invokes
//! `LeaveButton:Click` to close the storefront when the container is in
//! fullscreen mode): the plan makes three claims that diverge from the actual
//! source at `Blizzard_AccountStore.lua:7-14, 95-100` and
//! `Blizzard_AccountStore.xml:167`.
//!
//! 1. **The ESCAPE handler does NOT call `LeaveButton:Click`.** The body at
//!    lines 95-100 reads:
//!
//!    ```lua
//!    function FullscreenAccountStoreContainerMixin:OnKeyDown(key)
//!        -- Since the parent is capturing input, we need to manually implement an Escape key
//!        if key == "ESCAPE" then
//!            LeaveFullscreenMode();
//!        end
//!    end
//!    ```
//!
//!    `LeaveFullscreenMode` is a FILE-LOCAL function (line 7) that calls
//!    `AccountStoreFrame:SetFullscreenMode(false)`,
//!    `AccountStoreUtil.SetAccountStoreShown(false)`, and conditionally
//!    `EndOfMatchFrame:Show()`. The handler does NOT route through any
//!    button's `:Click()` method. The leave button's `OnClick` (lines 110-112)
//!    independently calls the SAME `LeaveFullscreenMode` function — both
//!    paths share the effect, but neither chains into the other.
//!
//! 2. **The button's parentKey is `LeaveStoreButton`, NOT `LeaveButton`.**
//!    The XML at line 167 declares
//!    `<Button parentKey="LeaveStoreButton" ... mixin="FullscreenLeaveAccountStoreButtonMixin">`.
//!    There is no `LeaveButton` parentKey anywhere in the addon. Even if the
//!    PLAN-named call had been correct, the field name is wrong.
//!
//! 3. **OnKeyDown is NOT gated on `inFullscreenMode`.** The body has no
//!    `if self.inFullscreenMode then` check — the ESCAPE dispatch fires
//!    whenever the container's OnKeyDown handler runs at all. The implicit
//!    "in fullscreen mode" gate the PLAN names is supplied by the framing
//!    fact that the container is hidden (and therefore not receiving key
//!    events) outside fullscreen mode, but the body itself is unconditional.
//!    A regression that called OnKeyDown directly (bypassing the show-state
//!    gate) would still fire LeaveFullscreenMode unconditionally, which is
//!    what the unconditional-on-self test below pins.
//!
//! Five tests pin the contract:
//!
//! - `on_key_down_method_exists_on_fullscreen_account_store_container_mixin`
//!   — surface check that `FullscreenAccountStoreContainerMixin.OnKeyDown`
//!   is a function. A non-function reading would prove the handler moved
//!   off the mixin (e.g. onto a different namespace or onto the leave
//!   button itself), forcing a re-pin against the new dispatch shape.
//!
//! - `fullscreen_container_exposes_leave_store_button_not_plan_named_leave_button`
//!   — asserts `FullscreenAccountStoreContainer.LeaveStoreButton` is a
//!   non-nil userdata AND `FullscreenAccountStoreContainer.LeaveButton` is
//!   nil. Pins the parentKey-name mismatch at PLAN's expected name. A
//!   non-nil `.LeaveButton` reading would mean Blizzard renamed the
//!   parentKey toward the PLAN-shaped name (and the XML's
//!   `parentKey="LeaveStoreButton"` would have to be re-pinned at the new
//!   name).
//!
//! - `on_key_down_with_escape_calls_account_store_util_set_account_store_shown_false`
//!   — replaces `AccountStoreUtil.SetAccountStoreShown` with a tracker;
//!   directly invokes
//!   `FullscreenAccountStoreContainerMixin.OnKeyDown(stub_self, "ESCAPE")`;
//!   asserts the tracker recorded exactly one call with `shown=false`. Pins
//!   the actual chain `OnKeyDown -> LeaveFullscreenMode ->
//!   AccountStoreUtil.SetAccountStoreShown(false)`. The stub_self is a bare
//!   table because the OnKeyDown body does not reference `self` at all
//!   (only `key`) — so the test verifies the unconditional dispatch shape.
//!
//! - `on_key_down_with_non_escape_keys_is_no_op_on_account_store_util_set_account_store_shown`
//!   — replaces SetAccountStoreShown with a tracker; invokes OnKeyDown with
//!   "ENTER", "SPACE", "TAB", "ESC" (the lowercase variant — Blizzard uses
//!   the all-caps `ESCAPE` token, so `ESC` should NOT match), and the empty
//!   string; asserts the tracker received zero calls across all five
//!   invocations. Pins the `key == "ESCAPE"` filter — a regression that
//!   loosened it (e.g. case-insensitive match, or matched on a key prefix)
//!   would surface here.
//!
//! - `on_key_down_with_escape_does_not_invoke_leave_store_button_click`
//!   — replaces `LeaveStoreButton.Click` with a tracker (via the
//!   `debug.getfenv(frame)[1]` per-instance override path documented in
//!   AGENTS.md), AND replaces SetAccountStoreShown with a tracker that
//!   confirms the ESCAPE dispatch fired; asserts the LeaveStoreButton.Click
//!   tracker received ZERO calls while the SetAccountStoreShown tracker
//!   received exactly one. PLAN tripwire that flips if a future change
//!   wires the OnKeyDown handler to call `self.LeaveStoreButton:Click()`
//!   (matching the PLAN's claim) instead of calling LeaveFullscreenMode
//!   directly.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_AccountStore";

post_wrath_ui_fixture! {
#[test]
fn on_key_down_method_exists_on_fullscreen_account_store_container_mixin() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let on_key_down_type: String = env
            .eval("return type(FullscreenAccountStoreContainerMixin.OnKeyDown)")
            .expect("FullscreenAccountStoreContainerMixin.OnKeyDown probe must run cleanly");

        assert_eq!(
            on_key_down_type, "function",
            "Expected `type(FullscreenAccountStoreContainerMixin.OnKeyDown) == \"function\"` \
             (`Blizzard_AccountStore.lua:95-100`), got `{on_key_down_type}`. A non-function \
             reading would prove the handler moved off the mixin (e.g. onto the leave button \
             itself, or onto a separate KeyDownDispatcher namespace), forcing a re-pin against \
             the new dispatch shape — and likely breaking the XML
             `<OnKeyDown method=\"OnKeyDown\"/>` script binding at line 184."
        );
    });
}
}

post_wrath_ui_fixture! {
#[test]
fn fullscreen_container_exposes_leave_store_button_not_plan_named_leave_button() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        let leave_store_button_type: String = env
            .eval("return type(FullscreenAccountStoreContainer.LeaveStoreButton)")
            .expect("FullscreenAccountStoreContainer.LeaveStoreButton probe must run cleanly");

        assert_ne!(
            leave_store_button_type, "nil",
            "Expected `type(FullscreenAccountStoreContainer.LeaveStoreButton) ~= \"nil\"` — \
             the XML at `Blizzard_AccountStore.xml:167` declares the leave button with \
             `parentKey=\"LeaveStoreButton\"`, so the parentKey lookup MUST land a non-nil \
             handle on the container. Got `{leave_store_button_type}`. A nil reading would \
             prove either the parentKey was renamed (forcing a re-pin against the new name) \
             or the child Button was removed entirely (a Blizzard change that would also \
             break the OnClick path — `FullscreenLeaveAccountStoreButtonMixin:OnClick` at \
             `Blizzard_AccountStore.lua:110-112` parallels the OnKeyDown ESCAPE path)."
        );

        let leave_button_type: String = env
            .eval("return type(FullscreenAccountStoreContainer.LeaveButton)")
            .expect("FullscreenAccountStoreContainer.LeaveButton probe must run cleanly");

        assert_eq!(
            leave_button_type, "nil",
            "Expected `type(FullscreenAccountStoreContainer.LeaveButton) == \"nil\"` — the \
             PLAN-named parentKey `LeaveButton` does NOT exist on the container; the actual \
             parentKey is `LeaveStoreButton`. Got `{leave_button_type}`. A non-nil reading \
             would prove either Blizzard renamed the parentKey toward the PLAN-shaped name \
             (forcing a re-pin against the new XML — and likely a re-pin of any sibling \
             addon's parentKey reference), or some sibling addon (or template inheritance) \
             added a `LeaveButton` parentKey alias on the container."
        );
    });
}
}

post_wrath_ui_fixture! {
#[test]
fn on_key_down_with_escape_calls_account_store_util_set_account_store_shown_false() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        seed_set_account_store_shown_tracker(env);

        env.eval::<()>(
            r#"
            FullscreenAccountStoreContainerMixin.OnKeyDown({}, "ESCAPE")
            return
            "#,
        )
        .expect("OnKeyDown(stub, \"ESCAPE\") must run cleanly");

        let (call_count, captured_shown_type, captured_shown_value): (i64, String, bool) = env
            .eval(
                r#"
                local s = _G.__behavior_fullscreen_escape_set_shown_calls or 0
                local v = _G.__behavior_fullscreen_escape_set_shown_last_arg
                return s, type(v), v == false
                "#,
            )
            .expect("set-shown tracker readout must run cleanly");

        assert_eq!(
            call_count, 1,
            "Expected exactly ONE `AccountStoreUtil.SetAccountStoreShown` call after \
             OnKeyDown(stub, \"ESCAPE\") — `LeaveFullscreenMode` at \
             `Blizzard_AccountStore.lua:7-14` calls it on line 9. Got {call_count}. A zero \
             reading would prove the OnKeyDown ESCAPE branch stopped routing through \
             LeaveFullscreenMode (e.g. an early return was added or the `key == \"ESCAPE\"` \
             comparison was inverted); a value > 1 would prove fan-out dispatch (e.g. \
             SetAccountStoreShown was wired into LeaveFullscreenMode at multiple call sites)."
        );

        assert_eq!(
            captured_shown_type, "boolean",
            "Expected `type(captured_shown_arg) == \"boolean\"` — line 9 of LeaveFullscreenMode \
             reads `AccountStoreUtil.SetAccountStoreShown(false);` with a literal `false`. Got \
             `{captured_shown_type}`. A non-boolean reading would prove the call shape changed \
             (e.g. an id was added before the boolean) — a Blizzard change toward a richer \
             SetAccountStoreShown signature."
        );

        assert!(
            captured_shown_value,
            "Expected the captured shown argument to equal the literal `false` — line 9 of \
             LeaveFullscreenMode passes `false` UNCONDITIONALLY to close the store. A `true` \
             reading would prove the boolean was inverted (a regression that would make \
             ESCAPE *open* the store instead of closing it — silently masked in normal play \
             by the show-state gate, but caught here)."
        );

        teardown_set_account_store_shown_tracker(env);
    });
}
}

post_wrath_ui_fixture! {
#[test]
fn on_key_down_with_non_escape_keys_is_no_op_on_account_store_util_set_account_store_shown() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        seed_set_account_store_shown_tracker(env);

        for non_escape_key in ["ENTER", "SPACE", "TAB", "ESC", ""] {
            env.eval::<()>(&format!(
                r#"
                FullscreenAccountStoreContainerMixin.OnKeyDown({{}}, {non_escape_key:?})
                return
                "#
            ))
            .unwrap_or_else(|error| {
                panic!("OnKeyDown(stub, {non_escape_key:?}) must run cleanly: {error}")
            });
        }

        let call_count: i64 = env
            .eval("return _G.__behavior_fullscreen_escape_set_shown_calls or 0")
            .expect("set-shown call-count probe must run cleanly");

        assert_eq!(
            call_count, 0,
            "Expected ZERO `AccountStoreUtil.SetAccountStoreShown` calls after OnKeyDown was \
             invoked with five non-ESCAPE keys ([\"ENTER\", \"SPACE\", \"TAB\", \"ESC\", \"\"]). \
             The `if key == \"ESCAPE\" then` filter at `Blizzard_AccountStore.lua:97` is an \
             EXACT-string match on the all-caps token; the lowercase `ESC` token MUST NOT \
             match (Blizzard's keybinding system normalizes to all-caps). Got {call_count}. \
             A non-zero reading would prove the filter was loosened — case-insensitive match, \
             a prefix match, or a fall-through path that fired LeaveFullscreenMode for any \
             keypress."
        );

        teardown_set_account_store_shown_tracker(env);
    });
}
}

post_wrath_ui_fixture! {
#[test]
fn on_key_down_with_escape_does_not_invoke_leave_store_button_click() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        seed_set_account_store_shown_tracker(env);
        seed_leave_store_button_click_tracker(env);

        env.eval::<()>(
            r#"
            FullscreenAccountStoreContainerMixin.OnKeyDown({}, "ESCAPE")
            return
            "#,
        )
        .expect("OnKeyDown(stub, \"ESCAPE\") must run cleanly");

        let (set_shown_calls, click_calls): (i64, i64) = env
            .eval(
                r#"
                return _G.__behavior_fullscreen_escape_set_shown_calls or 0,
                       _G.__behavior_fullscreen_escape_leave_click_calls or 0
                "#,
            )
            .expect("tracker dual-readout must run cleanly");

        assert_eq!(
            set_shown_calls, 1,
            "Expected exactly ONE `AccountStoreUtil.SetAccountStoreShown` call after \
             OnKeyDown(stub, \"ESCAPE\") — confirms the ESCAPE dispatch actually fired and \
             reached LeaveFullscreenMode (otherwise the click tracker assertion below would \
             pass vacuously). Got {set_shown_calls}. A zero reading would prove the ESCAPE \
             dispatch never ran (e.g. the SetAccountStoreShown override patched the wrong \
             namespace or was shadowed by a global re-export)."
        );

        assert_eq!(
            click_calls, 0,
            "Expected ZERO `LeaveStoreButton:Click` calls after OnKeyDown(stub, \"ESCAPE\") — \
             the OnKeyDown body at `Blizzard_AccountStore.lua:95-100` calls the file-local \
             `LeaveFullscreenMode` function DIRECTLY; it does NOT route through any button's \
             `:Click()` method. The PLAN-named `LeaveButton:Click` chain is a phantom — the \
             actual leave button's `OnClick` (lines 110-112) parallels the OnKeyDown path \
             (both call the same LeaveFullscreenMode function), but neither chains into the \
             other. Got {click_calls}. A non-zero reading would prove a future change wired \
             the OnKeyDown handler to invoke `self.LeaveStoreButton:Click()` (matching the \
             PLAN's claim), which would fire the LeaveFullscreenMode effect TWICE per ESCAPE \
             — once from the click handler, once from any cross-handler chain — and break \
             the simple-dispatch contract pinned by the SetAccountStoreShown call-count=1 \
             assertion above."
        );

        teardown_leave_store_button_click_tracker(env);
        teardown_set_account_store_shown_tracker(env);
    });
}
}

fn seed_set_account_store_shown_tracker(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        _G.__behavior_fullscreen_escape_set_shown_calls = 0
        _G.__behavior_fullscreen_escape_set_shown_last_arg = nil
        _G.__behavior_fullscreen_escape_original_set_shown =
            AccountStoreUtil.SetAccountStoreShown
        AccountStoreUtil.SetAccountStoreShown = function(shown)
            _G.__behavior_fullscreen_escape_set_shown_calls =
                _G.__behavior_fullscreen_escape_set_shown_calls + 1
            _G.__behavior_fullscreen_escape_set_shown_last_arg = shown
        end
        return
        "#,
    )
    .expect("seeding SetAccountStoreShown tracker must run cleanly");
}

fn teardown_set_account_store_shown_tracker(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        AccountStoreUtil.SetAccountStoreShown =
            _G.__behavior_fullscreen_escape_original_set_shown
        _G.__behavior_fullscreen_escape_original_set_shown = nil
        _G.__behavior_fullscreen_escape_set_shown_calls = nil
        _G.__behavior_fullscreen_escape_set_shown_last_arg = nil
        return
        "#,
    )
    .expect("SetAccountStoreShown tracker tear-down must run cleanly");
}

fn seed_leave_store_button_click_tracker(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        _G.__behavior_fullscreen_escape_leave_click_calls = 0
        local button = FullscreenAccountStoreContainer.LeaveStoreButton
        local env_table = debug.getfenv(button)
        local instance_table = env_table and env_table[1]
        if instance_table then
            instance_table.Click = function(_self)
                _G.__behavior_fullscreen_escape_leave_click_calls =
                    _G.__behavior_fullscreen_escape_leave_click_calls + 1
            end
        end
        return
        "#,
    )
    .expect("seeding LeaveStoreButton.Click tracker must run cleanly");
}

fn teardown_leave_store_button_click_tracker(env: &WowLuaEnv) {
    env.eval::<()>(
        r#"
        local button = FullscreenAccountStoreContainer.LeaveStoreButton
        local env_table = debug.getfenv(button)
        local instance_table = env_table and env_table[1]
        if instance_table then
            instance_table.Click = nil
        end
        _G.__behavior_fullscreen_escape_leave_click_calls = nil
        return
        "#,
    )
    .expect("LeaveStoreButton.Click tracker tear-down must run cleanly");
}
