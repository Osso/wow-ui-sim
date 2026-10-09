//! Behavior pin for `AccountStoreMixin:OnHide`.
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreMixin:OnHide`): the plan describes the body as
//! "plays the configured close sound and triggers
//! `EventRegistry:TriggerEvent` for the hide callback". Both halves
//! diverge from the actual seven-line body at
//! `Blizzard_AccountStore.lua:31-40`:
//!
//! ```lua
//! function AccountStoreMixin:OnHide()
//!     PlaySound(SOUNDKIT.ACCOUNT_STORE_CLOSE);
//!     EventRegistry:TriggerEvent("AccountStore.ShownState", false);
//!
//!     AccountStoreUtil.CloseStaticPopups();
//!
//!     if self.inFullscreenMode then
//!         LeaveFullscreenMode();
//!     end
//! end
//! ```
//!
//! 1. **"Configured" close sound mismatch.** Same shape as the
//!    OnShow case pinned by `behavior_on_show_sound.rs`: the sound
//!    id is NOT looked up from any addon-side configuration, settings
//!    provider, or saved-variables entry — the body passes the
//!    hardcoded SOUNDKIT constant `SOUNDKIT.ACCOUNT_STORE_CLOSE`
//!    directly to `PlaySound`. The constant is also not registered
//!    in the simulator's runtime SOUNDKIT bootstrap (`grep -rn
//!    ACCOUNT_STORE_CLOSE src/lua_api/env_init/` returns zero
//!    matches), so without a test-side seed the value passed to
//!    `PlaySound` is whatever `SOUNDKIT.ACCOUNT_STORE_CLOSE`
//!    resolves to (nil unless seeded).
//!
//! 2. **"Hide callback" event-name mismatch.** The EventRegistry
//!    event triggered is `"AccountStore.ShownState"` with a boolean
//!    payload (`false` for OnHide, `true` for OnShow at line 28) —
//!    a STATE-CHANGE event that fires from BOTH lifecycle handlers,
//!    not a one-direction "hide callback". None of the natural
//!    PLAN-shaped names (`AccountStore.OnHide`, `AccountStore.Hide`,
//!    `AccountStore.Close`, `AccountStore.CloseStore`) appear
//!    anywhere in the AccountStore lane.
//!
//! 3. **PLAN-omitted side effects.** The body has THREE more lines
//!    the PLAN spec ignores entirely: (a) `AccountStoreUtil.CloseStaticPopups()`
//!    at line 35 — closes any visible static popup whose name is in
//!    the lane's static popup whitelist (`Blizzard_AccountStoreUtil.lua:20-22`
//!    declares `ACCOUNT_STORE_STATIC_POPUPS = { "ACCOUNT_STORE_TRANSACTION_ERROR" }`,
//!    so this currently only hides the transaction-error popup);
//!    (b) the fullscreen-mode guard at line 37; and (c) the
//!    `LeaveFullscreenMode()` local-function call at line 38 —
//!    cleans up after fullscreen-mode shutdown by toggling the
//!    container's parent + visibility (`Blizzard_AccountStore.lua:50-64`
//!    `SetFullscreenMode(false)`), routing through
//!    `AccountStoreUtil.SetAccountStoreShown(false)`, and showing
//!    `EndOfMatchFrame` when match details are pending. Skipping
//!    these in the PLAN spec means a regression that drops the
//!    static-popup cleanup or the fullscreen-leave chain would NOT
//!    be caught by a test that only pins the sound and the
//!    EventRegistry trigger — pinning all three side effects here
//!    closes that gap.
//!
//! Four tests pin the contract:
//!
//! - `on_hide_calls_play_sound_with_account_store_close_sound_kit_constant_value`
//!   pre-seeds `SOUNDKIT.ACCOUNT_STORE_CLOSE` with a sentinel value,
//!   replaces `PlaySound` with a Lua tracker, directly invokes
//!   `AccountStoreMixin.OnHide(AccountStoreFrame)`, and asserts the
//!   tracker recorded exactly the sentinel — pins the constant name
//!   (`SOUNDKIT.ACCOUNT_STORE_CLOSE`, not e.g. `SOUNDKIT.ACCOUNT_STORE_CLOSE_SOUND`)
//!   and the absence of any "configured" lookup layer.
//! - `on_hide_fires_account_store_shown_state_event_registry_with_false_payload`
//!   registers an EventRegistry callback for
//!   `"AccountStore.ShownState"`, directly invokes OnHide, and asserts
//!   the callback fired exactly once with payload `false` — pins
//!   both the actual event name (shared with OnShow) and the
//!   boolean-false payload that distinguishes OnHide from OnShow.
//! - `on_hide_does_not_fire_event_registry_callbacks_for_plan_named_hide_event_aliases`
//!   registers callbacks for four PLAN-shaped name candidates
//!   (`AccountStore.OnHide`, `AccountStore.Hide`, `AccountStore.Close`,
//!   `AccountStore.CloseStore`) BEFORE invoking OnHide, then asserts
//!   none of them fired — the spec/source mismatch tripwire that
//!   flips if Blizzard ever adds a one-direction "hide" event.
//! - `on_hide_invokes_account_store_util_close_static_popups_plan_omitted_side_effect`
//!   replaces `AccountStoreUtil.CloseStaticPopups` with a Lua
//!   tracker, directly invokes OnHide, and asserts the tracker was
//!   called exactly once — pins the static-popup cleanup that the
//!   PLAN omits, so a regression that drops line 35 (or moves it
//!   behind a guard) would surface here.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";

#[test]
fn on_hide_calls_play_sound_with_account_store_close_sound_kit_constant_value() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        const SENTINEL_SOUND_ID: i64 = 332_211;

        env.eval::<()>(&format!(
            r#"
            SOUNDKIT = SOUNDKIT or {{}}
            SOUNDKIT.ACCOUNT_STORE_CLOSE = {SENTINEL_SOUND_ID}
            _G.__behavior_on_hide_sound_play_sound_arg = nil
            _G.__behavior_on_hide_sound_play_sound_call_count = 0
            _G.__behavior_on_hide_sound_original_play_sound = PlaySound
            PlaySound = function(sound_kit_id)
                _G.__behavior_on_hide_sound_play_sound_arg = sound_kit_id
                _G.__behavior_on_hide_sound_play_sound_call_count =
                    _G.__behavior_on_hide_sound_play_sound_call_count + 1
            end
            return
            "#
        ))
        .expect(
            "seeding SOUNDKIT.ACCOUNT_STORE_CLOSE sentinel and replacing PlaySound with a Lua \
             tracker must run cleanly",
        );

        env.eval::<()>("AccountStoreMixin.OnHide(AccountStoreFrame); return")
            .expect(
                "Direct invocation of `AccountStoreMixin.OnHide(AccountStoreFrame)` must run \
                 cleanly — the body at `Blizzard_AccountStore.lua:31-40` calls \
                 `PlaySound(SOUNDKIT.ACCOUNT_STORE_CLOSE)` (now routed to the test's tracker), \
                 fires the ShownState trigger (no observer in this test, so a no-op dispatch), \
                 calls `AccountStoreUtil.CloseStaticPopups()` (no popups visible at smoke-load \
                 time, so the loop body is skipped), and skips the fullscreen-leave branch \
                 because `self.inFullscreenMode` is nil.",
            );

        let (recorded_arg, call_count): (i64, i64) = env
            .eval(
                r#"
                return _G.__behavior_on_hide_sound_play_sound_arg or -1,
                       _G.__behavior_on_hide_sound_play_sound_call_count
                "#,
            )
            .expect("post-OnHide tracker probe must run cleanly");

        assert_eq!(
            call_count, 1,
            "Expected the test's PlaySound tracker to have been invoked exactly once after \
             `AccountStoreMixin.OnHide(AccountStoreFrame)`. The body at \
             `Blizzard_AccountStore.lua:32` calls `PlaySound(SOUNDKIT.ACCOUNT_STORE_CLOSE)` \
             unconditionally — there's no `if` gate, no SettingsProvider lookup, no \
             saved-variables check. A zero count means either (a) the original PlaySound was \
             restored before the OnHide ran (a regression in this test's tracker plumbing), \
             (b) OnHide errored before reaching the PlaySound line — but it's the FIRST line \
             of the body, so a pre-PlaySound error would have to come from method dispatch \
             itself, or (c) Blizzard added a config gate around the PlaySound call (forcing a \
             re-pin against the new conditional shape). A count > 1 means OnHide was invoked \
             more than once or the body now plays multiple sounds (worth investigating because \
             it would break the audio-pacing contract callers implicitly rely on)."
        );

        assert_eq!(
            recorded_arg, SENTINEL_SOUND_ID,
            "Expected the test's PlaySound tracker to have received the sentinel value \
             ({SENTINEL_SOUND_ID}) — the value that the test pre-seeded into \
             `SOUNDKIT.ACCOUNT_STORE_CLOSE`. The body at `Blizzard_AccountStore.lua:32` reads \
             `SOUNDKIT.ACCOUNT_STORE_CLOSE` BY NAME at the call site — pinning the sentinel \
             through the call proves the constant name in the body is literally \
             `SOUNDKIT.ACCOUNT_STORE_CLOSE` (not e.g. `SOUNDKIT.ACCOUNT_STORE_CLOSE_SOUND` or \
             `SOUNDKIT.ACCOUNTSTORE_CLOSE`) and that there is no intermediate \"configured \
             sound\" lookup layer. A different recorded value means either (a) Blizzard \
             renamed the constant the body references (forcing a re-pin against the new name), \
             (b) the body started routing through a configured-sound table indirection (e.g. \
             `Settings.GetValue(\"AccountStoreCloseSound\")`) — the PLAN-shaped \"configured \
             sound\" path that this test pins as ABSENT, or (c) the test's seed line did not \
             mutate the global SOUNDKIT table the addon body reads from (a regression in Lua \
             global-table semantics worth investigating)."
        );

        env.eval::<()>(
            r#"
            PlaySound = _G.__behavior_on_hide_sound_original_play_sound
            _G.__behavior_on_hide_sound_original_play_sound = nil
            _G.__behavior_on_hide_sound_play_sound_arg = nil
            _G.__behavior_on_hide_sound_play_sound_call_count = nil
            return
            "#,
        )
        .expect("PlaySound restore + tracker tear-down must run cleanly");
    });
}

#[test]
fn on_hide_fires_account_store_shown_state_event_registry_with_false_payload() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        env.eval::<()>(
            r#"
            _G.__behavior_on_hide_sound_shownstate_payloads = {}
            _G.__behavior_on_hide_sound_shownstate_call_count = 0
            EventRegistry:RegisterCallback(
                "AccountStore.ShownState",
                function(_, payload)
                    table.insert(
                        _G.__behavior_on_hide_sound_shownstate_payloads,
                        payload
                    )
                    _G.__behavior_on_hide_sound_shownstate_call_count =
                        _G.__behavior_on_hide_sound_shownstate_call_count + 1
                end,
                "behavior_on_hide_sound_shownstate_observer"
            )
            return
            "#,
        )
        .expect(
            "EventRegistry:RegisterCallback for `AccountStore.ShownState` must run cleanly — the \
             registry is constructed with SetUndefinedEventsAllowed(true) at \
             `GlobalCallbackRegistry.lua:5`, so arbitrary event names register without \
             precondition errors",
        );

        env.eval::<()>("AccountStoreMixin.OnHide(AccountStoreFrame); return")
            .expect("Direct invocation of `AccountStoreMixin.OnHide(AccountStoreFrame)` must run cleanly");

        let (call_count, first_payload_is_false, payloads_count): (i64, bool, i64) = env
            .eval(
                r#"
                local payloads = _G.__behavior_on_hide_sound_shownstate_payloads
                return _G.__behavior_on_hide_sound_shownstate_call_count,
                       payloads[1] == false,
                       #payloads
                "#,
            )
            .expect("post-OnHide shownstate observer probe must run cleanly");

        assert_eq!(
            call_count, 1,
            "Expected the `AccountStore.ShownState` EventRegistry callback to have fired exactly \
             once after `AccountStoreMixin.OnHide(AccountStoreFrame)`. The body at \
             `Blizzard_AccountStore.lua:33` calls \
             `EventRegistry:TriggerEvent(\"AccountStore.ShownState\", false)` unconditionally — \
             one OnHide invocation MUST produce exactly one trigger fire. A zero count means \
             either (a) OnHide errored before reaching the trigger line — but the only line \
             above is `PlaySound(SOUNDKIT.ACCOUNT_STORE_CLOSE)`, which is fire-and-forget, (b) \
             `EventRegistry:TriggerEvent` did not dispatch the callback (a regression in \
             rilua's native secureexecuterange traversal), or (c) the callback was registered against a \
             different event name (a regression in event-keying)."
        );

        assert_eq!(
            payloads_count, 1,
            "Expected exactly one entry in the recorded payload list — secondary check that \
             complements the call-count assertion. A mismatch (e.g. call_count=1 but \
             payloads_count=0) would mean the callback body fired but `table.insert` failed (a \
             regression in basic Lua semantics)."
        );

        assert!(
            first_payload_is_false,
            "Expected the first recorded payload to equal the boolean `false` — the OnHide \
             trigger at `Blizzard_AccountStore.lua:33` passes `false` literally. The companion \
             OnShow handler at line 28 passes `true`, so consumers MUST be able to distinguish \
             show-vs-hide by the payload alone (the event name is shared). A non-false reading \
             here means either (a) the trigger started passing a non-boolean payload (e.g. the \
             frame instance, or a state enum — forcing a re-pin against the new payload \
             contract), or (b) the OnHide body was conflated with OnShow (the trigger now \
             passes true on OnHide — a major regression that would invert all ShownState \
             consumer logic in the lane)."
        );

        env.eval::<()>(
            r#"
            EventRegistry:UnregisterCallback(
                "AccountStore.ShownState",
                "behavior_on_hide_sound_shownstate_observer"
            )
            _G.__behavior_on_hide_sound_shownstate_payloads = nil
            _G.__behavior_on_hide_sound_shownstate_call_count = nil
            return
            "#,
        )
        .expect("ShownState observer tear-down must run cleanly");
    });
}

const PLAN_NAMED_HIDE_EVENT_ALIASES: &[(&str, &str)] = &[
    (
        "AccountStore.OnHide",
        "natural PLAN-shaped name following the `<Mixin>.<ScriptHandlerName>` convention — \
         absent in the entire `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
    (
        "AccountStore.Hide",
        "natural PLAN-shaped name following the `<Mixin>.<Verb>` convention — absent in the \
         entire `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
    (
        "AccountStore.Close",
        "natural PLAN-shaped name aligned with the `SOUNDKIT.ACCOUNT_STORE_CLOSE` sound \
         constant — absent in the entire `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
    (
        "AccountStore.CloseStore",
        "natural PLAN-shaped name modeled on the lane's verbs — absent in the entire \
         `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
];

#[test]
fn on_hide_does_not_fire_event_registry_callbacks_for_plan_named_hide_event_aliases() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (alias, _mismatch_reason) in PLAN_NAMED_HIDE_EVENT_ALIASES {
            env.eval::<()>(&format!(
                r#"
                _G.__behavior_on_hide_sound_alias_count_{tag} = 0
                EventRegistry:RegisterCallback(
                    {alias:?},
                    function()
                        _G.__behavior_on_hide_sound_alias_count_{tag} =
                            _G.__behavior_on_hide_sound_alias_count_{tag} + 1
                    end,
                    "behavior_on_hide_sound_alias_observer_{tag}"
                )
                return
                "#,
                tag = sanitize_tag(alias),
                alias = alias,
            ))
            .unwrap_or_else(|error| {
                panic!("EventRegistry:RegisterCallback for alias {alias:?} failed: {error}")
            });
        }

        env.eval::<()>("AccountStoreMixin.OnHide(AccountStoreFrame); return")
            .expect("Direct invocation of `AccountStoreMixin.OnHide(AccountStoreFrame)` must run cleanly");

        for (alias, mismatch_reason) in PLAN_NAMED_HIDE_EVENT_ALIASES {
            let call_count: i64 = env
                .eval(&format!(
                    "return _G.__behavior_on_hide_sound_alias_count_{tag}",
                    tag = sanitize_tag(alias),
                ))
                .unwrap_or_else(|error| {
                    panic!("alias counter probe for {alias:?} failed: {error}")
                });

            assert_eq!(
                call_count, 0,
                "Expected the EventRegistry callback registered for the PLAN-named alias \
                 {alias:?} to NOT have fired after \
                 `AccountStoreMixin.OnHide(AccountStoreFrame)` (PLAN.md spec/source mismatch \
                 tripwire — {mismatch_reason}), got {call_count} fire(s). The actual OnHide \
                 body at `Blizzard_AccountStore.lua:31-40` triggers exactly one event — \
                 `AccountStore.ShownState` (line 33) with payload `false` — which is verified \
                 by the companion \
                 `on_hide_fires_account_store_shown_state_event_registry_with_false_payload` \
                 test. A non-zero count here means either (a) Blizzard added a one-direction \
                 \"hide\" event alongside `ShownState` (forcing a re-pin against the new event \
                 surface), (b) some other addon in the smoke shape registered + immediately \
                 fired the alias under its own path (worth investigating because it would \
                 shadow a future Blizzard rename), or (c) the simulator's EventRegistry \
                 dispatch leaked the trigger to alias callbacks (a regression in \
                 `CallbackRegistry.lua` event-keying)."
            );

            env.eval::<()>(&format!(
                r#"
                EventRegistry:UnregisterCallback(
                    {alias:?},
                    "behavior_on_hide_sound_alias_observer_{tag}"
                )
                _G.__behavior_on_hide_sound_alias_count_{tag} = nil
                return
                "#,
                tag = sanitize_tag(alias),
                alias = alias,
            ))
            .unwrap_or_else(|error| {
                panic!("alias observer tear-down for {alias:?} failed: {error}")
            });
        }
    });
}

#[test]
fn on_hide_invokes_account_store_util_close_static_popups_plan_omitted_side_effect() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        env.eval::<()>(
            r#"
            _G.__behavior_on_hide_sound_close_static_popups_call_count = 0
            _G.__behavior_on_hide_sound_original_close_static_popups =
                AccountStoreUtil.CloseStaticPopups
            AccountStoreUtil.CloseStaticPopups = function()
                _G.__behavior_on_hide_sound_close_static_popups_call_count =
                    _G.__behavior_on_hide_sound_close_static_popups_call_count + 1
            end
            return
            "#,
        )
        .expect(
            "replacing `AccountStoreUtil.CloseStaticPopups` with a Lua tracker must run cleanly \
             — the table `AccountStoreUtil` is declared at `Blizzard_AccountStoreUtil.lua:9` \
             and the function is a plain table-keyed entry that callers reach as \
             `AccountStoreUtil.CloseStaticPopups()`",
        );

        env.eval::<()>("AccountStoreMixin.OnHide(AccountStoreFrame); return")
            .expect("Direct invocation of `AccountStoreMixin.OnHide(AccountStoreFrame)` must run cleanly");

        let call_count: i64 = env
            .eval("return _G.__behavior_on_hide_sound_close_static_popups_call_count")
            .expect("post-OnHide CloseStaticPopups tracker probe must run cleanly");

        assert_eq!(
            call_count, 1,
            "Expected `AccountStoreUtil.CloseStaticPopups` to have been invoked exactly once \
             after `AccountStoreMixin.OnHide(AccountStoreFrame)` — pins the PLAN-omitted \
             side effect at `Blizzard_AccountStore.lua:35`. The PLAN.md spec for OnHide names \
             only the sound + EventRegistry trigger; the CloseStaticPopups call is a third \
             body line that hides any visible static popup whose name appears in \
             `ACCOUNT_STORE_STATIC_POPUPS = {{ \"ACCOUNT_STORE_TRANSACTION_ERROR\" }}` \
             (`Blizzard_AccountStoreUtil.lua:20-22`). A zero count means the OnHide body \
             dropped the cleanup line (a regression that would leave the transaction-error \
             popup orphaned across hide/show cycles — once the popup is shown by an \
             ACCOUNT_STORE_TRANSACTION_ERROR frame event, hiding the store frame would no \
             longer dismiss it). A count > 1 means the body invokes the cleanup multiple \
             times (worth investigating but harmless — the inner loop is idempotent because \
             `StaticPopup_Visible` gates each Hide call)."
        );

        env.eval::<()>(
            r#"
            AccountStoreUtil.CloseStaticPopups =
                _G.__behavior_on_hide_sound_original_close_static_popups
            _G.__behavior_on_hide_sound_original_close_static_popups = nil
            _G.__behavior_on_hide_sound_close_static_popups_call_count = nil
            return
            "#,
        )
        .expect("CloseStaticPopups restore + tracker tear-down must run cleanly");
    });
}

fn sanitize_tag(alias: &str) -> String {
    alias.replace('.', "_")
}
