# AnimationGroup Stop Callback

`AnimationGroup:Stop()` now dispatches group `OnStop` only for a playing group after resetting modeled playback state. This is a source-backed simulator contract, not a native-client timing claim.

## Content

Commit `af1b54f2b` changes `animation_group_stop` from state reset only to: capture whether the group was playing, reset playback/animation state, refresh flipbook and action-bar state, release the simulator borrow, then dispatch resolved group `OnStop` handlers with the group as `self`.

The exact committed RED/GREEN case has these phases:

1. Create a playing group with an Alpha child and a visible sibling frame.
2. `OnStop` observes `IsPlaying() == false`, `IsPaused() == false`, `IsDone() == true`, and `GetElapsed() == 0`; it hides the sibling and calls `Play()` reentrantly.
3. Explicit `Stop()` must call `OnStop` once, preserve the hidden sibling, retain reentrant playback, and not call `OnFinished`.

At parent `b8596d6dc`, the targeted Wrath integration binary was RED 0/1 with `Stop should dispatch OnStop once`. At `af1b54f2b`, the same exact invocation was GREEN 1/1. The proof covers only explicit Stop while playing.

Unmodified retail Blizzard sources use group `OnStop` for minimap glow hiding and Objective Tracker active-animation cleanup. The committed model supports those consumers, but neither source inspection nor the targeted simulator test establishes real-client callback timing.

## Limits

- Paused or inactive `Stop()`, hidden-subtree stops, child-animation `OnStop`, natural finish, `Pause`, and other callbacks are outside this proof.
- The later callback-error-path test is uncompiled and has no GREEN result. The direct dispatch path's error behavior therefore remains pending.
- No full suite, combined verification, native-client probe, vendor change, or new Cargo target is claimed.

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
