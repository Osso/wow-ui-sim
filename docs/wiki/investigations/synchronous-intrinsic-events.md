# Synchronous events skipped intrinsic scripts

Native Forever AuraContainer could paint an aura during construction but retain it after `A_Admin.RemoveBuff` removed the producer state. The synchronous event route called only the normal OnEvent binding. The correction shares the existing unit-filtered, all-binding frame dispatcher; compiled GREEN remains pending.

## Evidence and root cause

The trusted GUI observer recorded spell `19750` assigned with icon `135907`, stack text `3`, and a 30-second cooldown. Removal made the producer query return nil, yet the native assignment remained for ten seconds. Runtime inspection showed the container registered for `UNIT_AURA` on `player`, with no normal OnEvent script. This is compatible with an intrinsic-only listener, not proof that the loader lost scripts.

`dispatch_event_now` previously called `get_script`, which selects the normal binding. The working named-event route instead used `get_scripts_for_dispatch` and the frame's unit filter. XML template loading already installs intrinsic bindings; no loader correction is part of this slice.

## Correction and boundaries

The shared per-frame dispatcher now owns both unit filtering and ordered precall/normal/postcall execution. Synchronous dispatch calls it after the existing per-frame callback delivery. Callback filtering remains separate: a callback for `target` can fire even when that same frame's OnEvent scripts are registered for `player`. Global callbacks remain before frame delivery, and protected script errors retain the existing reporting path.

Synthetic regressions load real XML with `intrinsicOrder` and inspect bindings 0/1/2 before firing. `SetScript` only replaces the normal binding, so an extra numeric argument would not create a valid intrinsic test. The native regression settles a cold container before adding an aura, then adds/removes it twice through admin producers and ordinary update ticks; no private handler is invoked manually.

## Verification status

Runtime RED is preserved in the two observation ledgers below. New regressions and the correction are not compiled in this slice: the integrating caller owns Cargo and final acceptance. No full Ellesmere compatibility claim follows from this change alone.

## Sources

- [Synchronous dispatch contract](../../specs/synchronous-event-dispatch.md)
- [Event architecture](../../event-system.md)
- `src/lua_api/globals/state_backed_queries.rs`
- `src/lua_api/script_helpers/event_dispatch.rs`
- `/tmp/ellesmere-forever/gui-host-observed-acceptance-ledger.json`
- `/tmp/ellesmere-forever/aura-event-state-trace-ledger.json`

## See Also

- [[ellesmereui-forever]] — broader compatibility work and independent remaining boundaries.
