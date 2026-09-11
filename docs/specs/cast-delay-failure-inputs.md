# Timed-cast delay and failure inputs

Simulator-owned `A_Admin.DelayCasting` and `A_Admin.FailCasting` drive the existing timed player cast. They are not WoW globals or native failure detection. Pinned event shapes live in the [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json); identity follows [spellcast payloads](spellcast-event-payloads.md).

## What it must do

- [x] `DelayCasting(seconds)` requires a finite nonnegative number and returns one boolean. No current cast returns false. An active cast extends its deadline and accumulates `UnitCastingInfo` result 11 in milliseconds, preserving start and identity, then emits `UNIT_SPELLCAST_DELAYED` with `(unit, castGUID, spellID, castBarID)`.
- [x] Zero delay succeeds on an active cast and emits one DELAYED notification. Invalid or overflowing timing is rejected before mutation or notification.
- [x] `FailCasting(quiet=false)` returns false without events when inactive; otherwise takes the old state before callbacks and emits FAILED or FAILED_QUIET, followed by STOP, each with the old four-field identity. Invalid nonboolean quiet values fail atomically.
- [x] Failed casts never later complete, apply spell effects, or apply their deferred specialization. A failed specialization clears its pending action before callbacks, without clearing a reentrant replacement.
- [x] Failure and STOP callbacks may start a replacement cast; it retains its state and completes normally. Repeated failure while inactive returns false.
- [x] Existing SetCasting, StopCasting, SpellStopCasting, and cast/channel query arities remain unchanged. New cast state starts with zero accumulated delay. On current retail/PTR, casting slot 7 returns the shared synthetic event GUID as the pinned `WOWGUID` contract requires; numeric castBarID stays in slot 10. Older epochs retain their numeric slot 7.
- [x] The real Blizzard cast bar updates its duration on DELAYED and ends casting on failure/paired STOP. Quiet failure has no added failure-flash contract.

## Simulator policies

The input accepts any active timed-cast state, including an expired deadline not yet consumed by the existing completion tick. Zero delay is a notification, not a no-op. Explicit nil for `quiet` uses false. Failure-before-STOP ordering and the synthetic `Cast-Sim-<id>` identifier are model policies, not native timing or GUID evidence. GCDs/cooldowns are not reset. Specialization ownership cleanup is limited to failure of the existing specialization-activation spell; this slice does not redesign unrelated deferred actions or self-cancel behavior.

## How it works

- [Event system](../event-system.md)
- [Admin combat inputs](../admin-api/combat.md)
- [Cast-bar returns](cast-bar-id-returns.md)

## Implementation inventory

- `src/lua_api/globals/admin/cast_inputs.rs`: input validation, atomic delay/failure state transition, notifications.
- `src/lua_api/spellcast_events.rs`: shared four-field identity dispatch.
- `src/lua_api/game_data.rs`: cumulative delay seconds on casting state.
- `src/lua_api/globals/utility_system_spell/spell_api.rs`: milliseconds query output.
- `src/c_api/c_spec.rs`: specialization activation identity used for failed-action cleanup.

## Tests asserting this spec

- `src/iced_app/casting/input_tests.rs`: actual spell producers, delay/failure callbacks, completion/effect boundary, reentrancy and specialization leak regression.
- `tests/spell_casting.rs`: real Blizzard cast bar consumer.

### Development proof

Five lifecycle tests and one real Blizzard consumer test passed per profile at `5de64a2b3`; 59 existing cast/query/self-cancel/specialization regressions per profile passed at `0508553b4`. The subsequent source change only moved the admin helper module; the added overflow test also passes. No final acceptance gates were run.

Commands use the existing library and grouped integration targets with `--offline --no-default-features --features sound,gui,client-<profile>`. Current input filter: `spellcast_input_`; regression filters: `spellcast_interrupted_`, `spellcast_payload_`, `cast_bar_id::`, `c_vehicle_possession_globals::unit_`, `c_spell_flyout_probes::`, `spell_casting::`, and `admin_spec_talent_api::c_spec_set_specialization`. The repeated full consumer is skipped in the supplemental PTR regression command.

Logs: `/tmp/cast-input-5de64a2b3-{ptr,retail}.log`, `/tmp/cast-input-0508553b4-{ptr,retail}.log`, `/tmp/cast-input-existing-0508553b4-ptr.log`. Exact proof scope and command ledger: `/tmp/cast-input-proof-ledger.json`.

## Known gaps (current cycle)

- [ ] Native delay generation, failure reasons, secret/restricted payloads, cast GUID encoding and event timing remain unverified.
- [ ] Pre-12.1 API epoch branches were preserved but not executed by this bounded profile proof.

## Out of scope

Channel/empower producers, native failure detection, replacement-start APIs, gameplay interruption sources, security enforcement, broader action ownership redesign, audit artifact credit, and deployment.
