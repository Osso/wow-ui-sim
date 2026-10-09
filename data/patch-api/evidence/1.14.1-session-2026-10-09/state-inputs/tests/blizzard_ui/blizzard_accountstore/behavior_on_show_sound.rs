//! Behavior pin for `AccountStoreMixin:OnShow`.
//!
//! Spec/source mismatch finding (PLAN.md task for
//! `AccountStoreMixin:OnShow`): the plan describes the body as
//! "plays the configured open sound and triggers
//! `EventRegistry:TriggerEvent` for the show callback". Both halves
//! diverge from the actual two-line body at
//! `Blizzard_AccountStore.lua:26-29`:
//!
//! ```lua
//! function AccountStoreMixin:OnShow()
//!     PlaySound(SOUNDKIT.ACCOUNT_STORE_OPEN);
//!     EventRegistry:TriggerEvent("AccountStore.ShownState", true);
//! end
//! ```
//!
//! 1. **"Configured" sound mismatch.** The sound id is NOT looked up
//!    from any addon-side configuration table, settings provider, or
//!    saved-variables entry — the body passes the hardcoded SOUNDKIT
//!    constant `SOUNDKIT.ACCOUNT_STORE_OPEN` directly to `PlaySound`.
//!    "Configured" implies a runtime lookup that callers can override;
//!    the actual call shape is a compile-time constant reference. The
//!    constant is also not registered as a value in the simulator's
//!    runtime SOUNDKIT table (`grep -rn ACCOUNT_STORE_OPEN
//!    src/lua_api/env_init/` returns zero matches), so without a test-
//!    side seed the value passed to `PlaySound` is whatever
//!    `SOUNDKIT.ACCOUNT_STORE_OPEN` resolves to at the time of the
//!    call (nil unless seeded by a future Blizzard tier or by this
//!    test).
//!
//! 2. **"Show callback" event-name mismatch.** The EventRegistry
//!    event triggered is `"AccountStore.ShownState"` with a boolean
//!    payload (`true` for OnShow, `false` for OnHide at line 33). The
//!    PLAN's "show callback" framing implies a one-direction event
//!    (e.g., `AccountStore.OnShow`, `AccountStore.Show`,
//!    `AccountStore.Open`); the actual event is a STATE-CHANGE event
//!    that fires from BOTH lifecycle handlers. None of the natural
//!    PLAN-shaped names appear anywhere in the AccountStore lane —
//!    `grep -rn "AccountStore.OnShow\|AccountStore.Show\b\|AccountStore.Open"
//!    Interface/BlizzardUI/Blizzard_AccountStore/` returns zero
//!    matches. The closest behavioral analogue to a "show callback"
//!    in the EventRegistry namespace is the boolean-true branch of
//!    `AccountStore.ShownState`, but consumers must subscribe to the
//!    same event for both transitions and switch on the payload.
//!
//! Three tests pin both halves of the contract:
//!
//! - `on_show_calls_play_sound_with_account_store_open_sound_kit_constant_value`
//!   pre-seeds `SOUNDKIT.ACCOUNT_STORE_OPEN` with a sentinel value,
//!   replaces `PlaySound` with a Lua tracker, directly invokes
//!   `AccountStoreMixin.OnShow(AccountStoreFrame)`, and asserts the
//!   tracker recorded exactly the sentinel. This pins the constant
//!   name (not a different SOUNDKIT alias) and the lack of a
//!   "configured" lookup layer between OnShow and PlaySound.
//! - `on_show_fires_account_store_shown_state_event_registry_with_true_payload`
//!   registers an `EventRegistry` callback for
//!   `"AccountStore.ShownState"`, directly invokes
//!   `AccountStoreMixin.OnShow(AccountStoreFrame)`, and asserts the
//!   callback fired exactly once with payload `true`. This pins both
//!   the actual event name and the boolean-true payload that
//!   distinguishes OnShow from OnHide.
//! - `on_show_does_not_fire_event_registry_callbacks_for_plan_named_show_event_aliases`
//!   registers callbacks for four PLAN-shaped name candidates
//!   (`AccountStore.OnShow`, `AccountStore.Show`, `AccountStore.Open`,
//!   `AccountStore.OpenStore`) BEFORE invoking OnShow, then asserts
//!   none of them fired. This is the spec/source mismatch tripwire —
//!   if Blizzard ever adds a one-direction "show" event alongside
//!   `ShownState`, this flips and forces a re-pin against the new
//!   event surface.

use crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;

const ROOT: &str = "Blizzard_AccountStore";

#[test]
fn on_show_calls_play_sound_with_account_store_open_sound_kit_constant_value() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        const SENTINEL_SOUND_ID: i64 = 776_655;

        env.eval::<()>(&format!(
            r#"
            SOUNDKIT = SOUNDKIT or {{}}
            SOUNDKIT.ACCOUNT_STORE_OPEN = {SENTINEL_SOUND_ID}
            _G.__behavior_on_show_sound_play_sound_arg = nil
            _G.__behavior_on_show_sound_play_sound_call_count = 0
            _G.__behavior_on_show_sound_original_play_sound = PlaySound
            PlaySound = function(sound_kit_id)
                _G.__behavior_on_show_sound_play_sound_arg = sound_kit_id
                _G.__behavior_on_show_sound_play_sound_call_count =
                    _G.__behavior_on_show_sound_play_sound_call_count + 1
            end
            return
            "#
        ))
        .expect(
            "seeding SOUNDKIT.ACCOUNT_STORE_OPEN sentinel and replacing PlaySound with a Lua \
             tracker must run cleanly",
        );

        env.eval::<()>("AccountStoreMixin.OnShow(AccountStoreFrame); return")
            .expect(
                "Direct invocation of `AccountStoreMixin.OnShow(AccountStoreFrame)` must run \
                 cleanly — the body at `Blizzard_AccountStore.lua:26-29` only calls \
                 `PlaySound(SOUNDKIT.ACCOUNT_STORE_OPEN)` (now routed to the test's tracker) and \
                 `EventRegistry:TriggerEvent(\"AccountStore.ShownState\", true)` (no callbacks \
                 registered in this test, so dispatch is a no-op).",
            );

        let (recorded_arg, call_count): (i64, i64) = env
            .eval(
                r#"
                return _G.__behavior_on_show_sound_play_sound_arg or -1,
                       _G.__behavior_on_show_sound_play_sound_call_count
                "#,
            )
            .expect("post-OnShow tracker probe must run cleanly");

        assert_eq!(
            call_count, 1,
            "Expected the test's PlaySound tracker to have been invoked exactly once after \
             `AccountStoreMixin.OnShow(AccountStoreFrame)`. The body at \
             `Blizzard_AccountStore.lua:27` calls `PlaySound(SOUNDKIT.ACCOUNT_STORE_OPEN)` \
             unconditionally — there's no `if` gate, no SettingsProvider lookup, no \
             saved-variables check. A zero count means either (a) the original PlaySound was \
             restored before the OnShow ran (a regression in this test's tracker plumbing), \
             (b) OnShow errored before reaching the PlaySound line (likely a regression in the \
             mixin's body — but the only other line is the EventRegistry trigger, which fires \
             AFTER PlaySound), or (c) Blizzard added a config gate around the PlaySound call \
             (forcing a re-pin against the new conditional shape). A count > 1 means OnShow \
             was invoked more than once or the body now plays multiple sounds (worth \
             investigating because it would break the audio-pacing contract callers \
             implicitly rely on)."
        );

        assert_eq!(
            recorded_arg, SENTINEL_SOUND_ID,
            "Expected the test's PlaySound tracker to have received the sentinel value \
             ({SENTINEL_SOUND_ID}) — the value that the test pre-seeded into \
             `SOUNDKIT.ACCOUNT_STORE_OPEN`. The body at `Blizzard_AccountStore.lua:27` reads \
             `SOUNDKIT.ACCOUNT_STORE_OPEN` BY NAME at the call site — pinning the sentinel \
             through the call proves the constant name in the body is literally \
             `SOUNDKIT.ACCOUNT_STORE_OPEN` (not e.g. `SOUNDKIT.ACCOUNT_STORE_OPEN_SOUND` or \
             `SOUNDKIT.ACCOUNTSTORE_OPEN`) and that there is no intermediate \"configured \
             sound\" lookup layer. A different recorded value means either (a) Blizzard \
             renamed the constant the body references (forcing a re-pin against the new name), \
             (b) the body started routing through a configured-sound table indirection (e.g. \
             `Settings.GetValue(\"AccountStoreOpenSound\")`) — the PLAN-shaped \"configured \
             sound\" path that this test pins as ABSENT, or (c) the test's seed line \
             (`SOUNDKIT.ACCOUNT_STORE_OPEN = {SENTINEL_SOUND_ID}`) did not mutate the global \
             SOUNDKIT table the addon body reads from (a regression in Lua global-table \
             semantics worth investigating)."
        );

        env.eval::<()>(
            r#"
            PlaySound = _G.__behavior_on_show_sound_original_play_sound
            _G.__behavior_on_show_sound_original_play_sound = nil
            _G.__behavior_on_show_sound_play_sound_arg = nil
            _G.__behavior_on_show_sound_play_sound_call_count = nil
            return
            "#,
        )
        .expect("PlaySound restore + tracker tear-down must run cleanly");
    });
}

#[test]
fn on_show_fires_account_store_shown_state_event_registry_with_true_payload() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        env.eval::<()>(
            r#"
            _G.__behavior_on_show_sound_shownstate_payloads = {}
            _G.__behavior_on_show_sound_shownstate_call_count = 0
            EventRegistry:RegisterCallback(
                "AccountStore.ShownState",
                function(_, payload)
                    table.insert(
                        _G.__behavior_on_show_sound_shownstate_payloads,
                        payload
                    )
                    _G.__behavior_on_show_sound_shownstate_call_count =
                        _G.__behavior_on_show_sound_shownstate_call_count + 1
                end,
                "behavior_on_show_sound_shownstate_observer"
            )
            return
            "#,
        )
        .expect(
            "EventRegistry:RegisterCallback for `AccountStore.ShownState` must run cleanly — the \
             registry is constructed via CreateFromMixins(CallbackRegistryMixin) at \
             `GlobalCallbackRegistry.lua:1` and SetUndefinedEventsAllowed(true) at line 5, so \
             arbitrary event names register without precondition errors",
        );

        env.eval::<()>("AccountStoreMixin.OnShow(AccountStoreFrame); return")
            .expect("Direct invocation of `AccountStoreMixin.OnShow(AccountStoreFrame)` must run cleanly");

        let (call_count, first_payload, payloads_count): (i64, bool, i64) = env
            .eval(
                r#"
                local payloads = _G.__behavior_on_show_sound_shownstate_payloads
                return _G.__behavior_on_show_sound_shownstate_call_count,
                       payloads[1] == true,
                       #payloads
                "#,
            )
            .expect("post-OnShow shownstate observer probe must run cleanly");

        assert_eq!(
            call_count, 1,
            "Expected the `AccountStore.ShownState` EventRegistry callback to have fired exactly \
             once after `AccountStoreMixin.OnShow(AccountStoreFrame)`. The body at \
             `Blizzard_AccountStore.lua:28` calls \
             `EventRegistry:TriggerEvent(\"AccountStore.ShownState\", true)` unconditionally — \
             one OnShow invocation MUST produce exactly one trigger fire, and the trigger MUST \
             dispatch to all registered callbacks via secureexecuterange \
             (`CallbackRegistry.lua:204` / `:213`). A zero count means either (a) the OnShow \
             body errored before reaching the trigger line — but the only line above is \
             `PlaySound(SOUNDKIT.ACCOUNT_STORE_OPEN)`, which is a PCALL-style fire-and-forget, \
             (b) `EventRegistry:TriggerEvent` did not dispatch the callback (a regression in \
             rilua's native secureexecuterange traversal), or (c) the callback was registered against a different \
             event name (a regression in `EventRegistry:RegisterCallback`'s string-keying). A \
             count > 1 means the trigger fired multiple times (worth investigating because it \
             would break ShownState consumers that toggle on every transition)."
        );

        assert_eq!(
            payloads_count, 1,
            "Expected exactly one entry in the recorded payload list — secondary check that \
             complements the call-count assertion. A mismatch between call_count and \
             payloads_count (e.g. call_count=1 but payloads_count=0) would mean the callback \
             body fired but `table.insert` failed (a regression in basic Lua semantics)."
        );

        assert!(
            first_payload,
            "Expected the first recorded payload to equal the boolean `true` — the OnShow \
             trigger at `Blizzard_AccountStore.lua:28` passes `true` literally. The companion \
             OnHide handler at line 33 passes `false`, so consumers MUST be able to distinguish \
             show-vs-hide by the payload alone (the event name is shared). A non-true reading \
             here means either (a) the trigger started passing a non-boolean payload (e.g. the \
             frame instance, or a state enum — forcing a re-pin against the new payload \
             contract), or (b) the OnShow body was conflated with OnHide (the trigger now \
             passes false on OnShow — a major regression that would invert all ShownState \
             consumer logic in the lane)."
        );

        env.eval::<()>(
            r#"
            EventRegistry:UnregisterCallback(
                "AccountStore.ShownState",
                "behavior_on_show_sound_shownstate_observer"
            )
            _G.__behavior_on_show_sound_shownstate_payloads = nil
            _G.__behavior_on_show_sound_shownstate_call_count = nil
            return
            "#,
        )
        .expect("ShownState observer tear-down must run cleanly");
    });
}

const PLAN_NAMED_SHOW_EVENT_ALIASES: &[(&str, &str)] = &[
    (
        "AccountStore.OnShow",
        "natural PLAN-shaped name following the `<Mixin>.<ScriptHandlerName>` convention — \
         absent in the entire `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
    (
        "AccountStore.Show",
        "natural PLAN-shaped name following the `<Mixin>.<Verb>` convention — absent in the \
         entire `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
    (
        "AccountStore.Open",
        "natural PLAN-shaped name aligned with the `SOUNDKIT.ACCOUNT_STORE_OPEN` sound constant \
         — absent in the entire `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
    (
        "AccountStore.OpenStore",
        "natural PLAN-shaped name modeled on the lane's verbs — absent in the entire \
         `Interface/BlizzardUI/Blizzard_AccountStore/` lane",
    ),
];

#[test]
fn on_show_does_not_fire_event_registry_callbacks_for_plan_named_show_event_aliases() {
    with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, _loaded| {
        for (alias, _mismatch_reason) in PLAN_NAMED_SHOW_EVENT_ALIASES {
            env.eval::<()>(&format!(
                r#"
                _G.__behavior_on_show_sound_alias_count_{tag} = 0
                EventRegistry:RegisterCallback(
                    {alias:?},
                    function()
                        _G.__behavior_on_show_sound_alias_count_{tag} =
                            _G.__behavior_on_show_sound_alias_count_{tag} + 1
                    end,
                    "behavior_on_show_sound_alias_observer_{tag}"
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

        env.eval::<()>("AccountStoreMixin.OnShow(AccountStoreFrame); return")
            .expect("Direct invocation of `AccountStoreMixin.OnShow(AccountStoreFrame)` must run cleanly");

        for (alias, mismatch_reason) in PLAN_NAMED_SHOW_EVENT_ALIASES {
            let call_count: i64 = env
                .eval(&format!(
                    "return _G.__behavior_on_show_sound_alias_count_{tag}",
                    tag = sanitize_tag(alias),
                ))
                .unwrap_or_else(|error| {
                    panic!("alias counter probe for {alias:?} failed: {error}")
                });

            assert_eq!(
                call_count, 0,
                "Expected the EventRegistry callback registered for the PLAN-named alias \
                 {alias:?} to NOT have fired after \
                 `AccountStoreMixin.OnShow(AccountStoreFrame)` (PLAN.md spec/source mismatch \
                 tripwire — {mismatch_reason}), got {call_count} fire(s). The actual OnShow \
                 body at `Blizzard_AccountStore.lua:26-29` triggers exactly one event — \
                 `AccountStore.ShownState` (line 28) with payload `true` — which is verified \
                 by the companion \
                 `on_show_fires_account_store_shown_state_event_registry_with_true_payload` \
                 test. A non-zero count here means either (a) Blizzard added a one-direction \
                 \"show\" event alongside `ShownState` (forcing a re-pin against the new event \
                 surface — and likely retiring or pairing `ShownState`), (b) some other addon \
                 in the smoke shape registered + immediately fired the alias under its own \
                 path (worth investigating because it would shadow a future Blizzard rename), \
                 or (c) the simulator's EventRegistry dispatch leaked the trigger to alias \
                 callbacks (a regression in `CallbackRegistry.lua` event-keying)."
            );

            env.eval::<()>(&format!(
                r#"
                EventRegistry:UnregisterCallback(
                    {alias:?},
                    "behavior_on_show_sound_alias_observer_{tag}"
                )
                _G.__behavior_on_show_sound_alias_count_{tag} = nil
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

fn sanitize_tag(alias: &str) -> String {
    alias.replace('.', "_")
}
