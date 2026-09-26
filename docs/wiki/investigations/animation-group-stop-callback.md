# AnimationGroup Stop Callback

`AnimationGroup:Stop()` dispatches group `OnStop` only for a playing group after resetting modeled playback state. At `43025255d`, handler failures are routed through the simulator error handler rather than escaping `Stop()`. This is source-backed simulator behavior, not a native-client timing claim.

## Content

Commit `af1b54f2b` changes `animation_group_stop` from state reset only to: capture whether the group was playing, reset playback/animation state, refresh flipbook and action-bar state, release the simulator borrow, then dispatch resolved group `OnStop` handlers with the group as `self`. Commit `43025255d` keeps that ordering and dispatches each ordered handler through `protected_lua_pcall_state`; an error is sent to `call_error_handler_state` and later handlers/`Stop()` continue.

The exact committed RED/GREEN case has these phases:

1. Create a playing group with an Alpha child and a visible sibling frame.
2. `OnStop` observes `IsPlaying() == false`, `IsPaused() == false`, `IsDone() == true`, and `GetElapsed() == 0`; it hides the sibling and calls `Play()` reentrantly.
3. Explicit `Stop()` must call `OnStop` once, preserve the hidden sibling, retain reentrant playback, and not call `OnFinished`.

At parent `b8596d6dc`, targeted `gui,client-wrath` integration was RED 0/1 with `Stop should dispatch OnStop once`. At `af1b54f2b`, the same exact invocation was GREEN 1/1. The proof covers only explicit Stop while playing.

The later error-path case was valid RED 0/1 before `43025255d` with `OnStop errors must not abort the Stop caller`. Its post-correction GREEN has not run. The source implementation is documented above; final independent verification remains pending.

Unmodified retail Blizzard sources use group `OnStop` for minimap glow hiding and Objective Tracker active-animation cleanup. The committed model supports those consumers, but neither source inspection nor the targeted simulator test establishes real-client callback timing.

## Limits

- Paused or inactive `Stop()`, hidden-subtree stops, child-animation `OnStop`, natural finish, `Pause`, and other callbacks are outside this proof.
- The `43025255d` callback-error implementation has valid pre-correction RED only; post-correction GREEN and final independent verification remain pending.
- No full suite, full-profile, combined verification, native-client probe, vendor change, or new Cargo target is claimed.

## Sources

- [animation control](../../../src/lua_api/frame/methods/button_anchor_hierarchy/animations.rs) — modeled explicit Stop transition and dispatch
- [animation group tests](../../../tests/animation_group.rs) — exact lifecycle case and pending error-path case
- [animation query lifecycle](../../specs/animation-query-lifecycle.md) — bounded API inventory
- `/tmp/cross-version-animation-stop-proof.md` — source boundary and RED/GREEN ledger
- `/tmp/cross-version-animation-stop-{red,green}.log` — actual targeted test outputs
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Minimap/Mainline/Minimap.xml` — source consumer using `OnStop`
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ObjectiveTracker/Blizzard_ObjectiveTrackerAnimTemplates.lua` — source consumer cleanup
- [AnimScriptProbe](../../addons/AnimScriptProbe/README.md) — live registration support only

## See Also

- [[event-system]] — script dispatch lifecycle
- [[widget-system]] — frame visibility state
- [[achievement-panel-hide]] — child animation finish callback boundary
