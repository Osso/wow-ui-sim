# Synchronous `FireEvent` skipped intrinsic scripts

`FireEvent` and `A_Admin.FireEvent` previously selected only the normal OnEvent binding. `c6d970cf3` corrects that API route by sharing the existing unit-filtered, all-binding frame dispatcher. A later route-attribution correction established that `A_Admin.AddBuff`/`RemoveBuff` never used this dispatcher, so this commit is not the cause or fix of the observed AuraContainer removal.

## Evidence and root cause

The trusted GUI observer recorded spell `19750` assigned with icon `135907`, stack text `3`, and a 30-second cooldown. Removal made the producer query return nil, yet the native assignment remained. The earlier event trace showed a registered `UNIT_AURA(player)` listener with no normal OnEvent script, which is compatible with an intrinsic-only listener rather than proof that the loader lost scripts.

`dispatch_event_now` previously called `get_script`, which selects the normal binding. The working named-event route instead used `get_scripts_for_dispatch` and the frame's unit filter. XML template loading already installs intrinsic bindings; no loader correction is part of this slice. However, `A_Admin.AddBuff`/`RemoveBuff` call `fire_named_event_state`, not `dispatch_event_now`, and therefore already used the all-binding named-event route.

## Correction and boundaries

The shared per-frame dispatcher now owns both unit filtering and ordered precall/normal/postcall execution. Synchronous dispatch calls it after the existing per-frame callback delivery. Callback filtering remains separate: a callback for `target` can fire even when that same frame's OnEvent scripts are registered for `player`. Global callbacks remain before frame delivery, and protected script errors retain the existing reporting path.

Synthetic regressions load real XML with `intrinsicOrder` and inspect bindings 0/1/2 before firing. `SetScript` only replaces the normal binding, so an extra numeric argument would not create a valid intrinsic test. The native cold-container regression adds/removes an aura twice through admin producers and ordinary update ticks; it validates the pre-existing named-event lifecycle, not `c6d970cf3` causality.

## Diagnosis correction and verification status

The prior claim that normal-only `dispatch_event_now` caused the observed aura removal was incorrect. `/tmp/ellesmere-forever/aura-post-removal-state-ledger.json` records the relevant post-`ReloadFrames` state: visible `pball` frame 10, but `UNIT_AURA` unregistered and `dynamicEvents` nil. `src/lua_api/frame/methods/core_state/visibility.rs` recursively invokes only the normal `OnShow` binding, skipping AuraContainer's intrinsic `OnShow` re-registration path. That visibility route is the current implementation/proof target.

`c6d970cf3` remains a valid `FireEvent`/`A_Admin.FireEvent` correction with API-level all-binding regressions. The passing native cold-event test is not RED-to-GREEN proof for the GUI removal, and no GREEN claim is made for the visibility diagnosis or full Ellesmere acceptance.

## Sources

- [Synchronous dispatch contract](../../specs/synchronous-event-dispatch.md)
- [Event architecture](../../event-system.md)
- `src/lua_api/globals/state_backed_queries.rs`
- `src/lua_api/script_helpers/event_dispatch.rs`
- `/tmp/ellesmere-forever/gui-host-observed-acceptance-ledger.json`
- `/tmp/ellesmere-forever/aura-event-state-trace-ledger.json`
- `/tmp/ellesmere-forever/aura-post-removal-state-ledger.json`

## See Also

- [[ellesmereui-forever]] — broader compatibility work and independent remaining boundaries.
