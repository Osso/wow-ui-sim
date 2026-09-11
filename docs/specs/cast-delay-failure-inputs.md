# Timed-cast delay and failure inputs

Simulator-owned `A_Admin.DelayCasting` and `A_Admin.FailCasting` drive the existing timed player cast. They are not WoW globals or native failure detection. Pinned event shapes live in the [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json); identity follows [spellcast payloads](spellcast-event-payloads.md).

## What it must do

- [ ] `DelayCasting(seconds)` requires a finite nonnegative number and returns one boolean. No current cast returns false. An active cast extends its deadline and accumulates `UnitCastingInfo` result 11 in milliseconds, preserving start and identity, then emits `UNIT_SPELLCAST_DELAYED` with `(unit, castGUID, spellID, castBarID)`.
- [ ] Zero delay succeeds on an active cast and emits one DELAYED notification. Invalid or overflowing timing is rejected before mutation or notification.
- [ ] `FailCasting(quiet=false)` returns false without events when inactive; otherwise takes the old state before callbacks and emits FAILED or FAILED_QUIET, followed by STOP, each with the old four-field identity. Invalid nonboolean quiet values fail atomically.
- [ ] Failed casts never later complete, apply spell effects, or apply their deferred specialization. A failed specialization clears its pending action before callbacks, without clearing a reentrant replacement.
- [ ] Failure and STOP callbacks may start a replacement cast; it retains its state and completes normally. Repeated failure while inactive returns false.
- [ ] Existing SetCasting, StopCasting, SpellStopCasting, and cast/channel query arities remain unchanged. New cast state starts with zero accumulated delay.
- [ ] The real Blizzard cast bar updates its duration on DELAYED and ends casting on failure/paired STOP. Quiet failure has no added failure-flash contract.

## Simulator policies

The input accepts any active timed-cast state, including an expired deadline not yet consumed by the existing completion tick. Zero delay is a notification, not a no-op. Explicit nil for `quiet` uses false. Failure-before-STOP ordering and the synthetic `Cast-Sim-<id>` identifier are model policies, not native timing or GUID evidence. GCDs/cooldowns are not reset. Specialization ownership cleanup is limited to failure of the existing specialization-activation spell; this slice does not redesign unrelated deferred actions or self-cancel behavior.

## How it works

- [Event system](../event-system.md)
- [Admin combat inputs](../admin-api/combat.md)
- [Cast-bar returns](cast-bar-id-returns.md)

## Implementation inventory

- `src/lua_api/globals/admin_cast_inputs.rs`: input validation, atomic delay/failure state transition, notifications.
- `src/lua_api/spellcast_events.rs`: shared four-field identity dispatch.
- `src/lua_api/game_data.rs`: cumulative delay seconds on casting state.
- `src/lua_api/globals/utility_system_spell/spell_api.rs`: milliseconds query output.
- `src/c_api/c_spec.rs`: specialization activation identity used for failed-action cleanup.

## Tests asserting this spec

- `src/iced_app/casting/input_tests.rs`: actual spell producers, delay/failure callbacks, completion/effect boundary, reentrancy and specialization leak regression.
- `tests/spell_casting.rs`: real Blizzard cast bar consumer.

## Known gaps (current cycle)

- [ ] Native delay generation, failure reasons, secret/restricted payloads, cast GUID encoding and event timing remain unverified.
- [ ] Focused development proof pending; no final acceptance gates run by this implementation.

## Out of scope

Channel/empower producers, native failure detection, replacement-start APIs, gameplay interruption sources, security enforcement, broader action ownership redesign, audit artifact credit, and deployment.
