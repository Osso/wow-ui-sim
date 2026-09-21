# Simulator channel and empower lifecycles

Explicit `A_Admin` inputs drive the channel slot and its six spellcast events. Payloads follow the pinned PTR source register; timing and transition choices below are simulator policies, not verified native gameplay. See [cast-bar returns](cast-bar-id-returns.md) and [spellcast payloads](spellcast-event-payloads.md).

## What it must do

- [x] `StartChannel(spellID, name, icon, durationSeconds)` starts a channel; `UpdateChannel(durationSeconds)` changes total duration from its original start without changing identity.
- [x] `StartEmpower(spellID, name, icon, stageSeconds, holdSeconds)` starts an empowered channel. `UpdateEmpower(stageSeconds, holdSeconds)` replaces timing without changing identity. Stages are a dense, nonempty array of positive finite seconds; hold is nonnegative.
- [x] `StopChannel(complete=false)` stops either channel kind, returning whether state existed. Explicit true models successful early release; false models self-cancel. Repeated stops return false.
- [x] Inputs validate before mutation, including millisecond overflow. An input for the wrong active kind returns false without mutation.
- [x] `UnitChannelInfo` retains its eleven-field tuple. Its endpoint excludes empower hold; natural completion includes hold. Stage count derives from the stage vector rather than an independent mutable count.
- [x] `GetUnitEmpowerStageDuration("player", zeroBasedIndex)` and `GetUnitEmpowerHoldAtMaxTime("player")` return **milliseconds**. Missing stages, missing empower state, and unsupported units return no values as simulator policy.
- [x] Channel/empower starts and updates emit `(unit, castGUID, spellID, castBarID)`. Channel stop emits `(unit, castGUID, spellID, interruptedBy, castBarID)`; empower stop inserts `complete` before `interruptedBy`.
- [x] Natural completion/early release uses nil `interruptedBy`; cancellation uses current player GUID. The nil completion convention follows Blizzard's consumer but conflicts with generated non-nil metadata; native conformance remains unclaimed.
- [x] `UnitChannelInfo` ends at stage duration and excludes empower hold. The deadline includes hold; stage and hold query values are milliseconds.
- [x] Channel deadlines are checked at the existing OnUpdate boundary against the same clock as GetTime. Tests advance that deadline clock without sleeping. No periodic gameplay ticks, damage, or healing are invented.
- [x] Replacement leaves only one active cast/channel. Terminal state is extracted before callbacks. Reentrant replacement wins; old events retain immutable old IDs and cannot later complete.
- [x] Existing `SetCasting`/`StopCasting` signatures remain unchanged. Ordinary cast producers clear replaced channel state; `SpellStopCasting` also self-cancels a channel when no ordinary cast exists.
- [x] Real Blizzard channel/empower view, stage pips, update and stop handling observe the modeled state without vendor patches.

## How it works

- [Spellcast event model](spellcast-event-payloads.md)
- [Timed-cast inputs](cast-delay-failure-inputs.md)

## Implementation inventory

- `src/lua_api/channeling.rs`: ownership transitions, terminal events, deadline ticking.
- `src/lua_api/channeling/inputs.rs`: simulator inputs and atomic timing validation.
- `src/lua_api/channeling/queries.rs`: consumer-required millisecond queries.
- `src/lua_api/game_data.rs`: channel/empower timing state.
- `src/lua_api/on_update.rs`: shared update boundary.
- `src/lua_api/spellcast_events.rs`: stable existing cast identity and cross-mode START handling.
- `src/lua_api/globals/admin.rs`: input registration and existing casting integration.
- `src/lua_api/globals/real/player_identity.rs`: local-player `UnitNameFromGUID` supports Blizzard cancellation labels under the shared cast-duration capability; `UnitClassFromGUID` remains Retail-only. Other GUIDs remain unmodeled.

## Tests asserting this spec

- `tests/channel_lifecycle.rs`: public inputs, queries, callbacks, update/stop/completion and reentrancy.
- `tests/channel_reentrancy.rs`: callback replacement wins over stale producers and cancels pending specialization ownership.
- `tests/channel_blizzard.rs`: unmodified Blizzard channel/empower handlers, three stage-pip offsets, natural/early/cancel stops, and the observed UPDATE boundary.
- `tests/cast_bar_id.rs`: complete eleven-field tuple assertions through the real inputs.

Development proof at `7a34d3891`: 11 library + 63 integration cases passed per profile, with one old simultaneous-mode fixture failing. `1de909e5d` replaced internal fixture mutation with public input/tuple checks; its three affected tests passed on each profile. `a844e36f8` strengthened stage-count update/hold validation and passed on both profiles. `89c03462c` records the source-backed lifecycle/consumer boundary. Together these cover 75 unique focused tests per profile (10 new, 65 existing); this is combined development proof, not one final acceptance run. Logs and exact commands: `/tmp/channel-development-ledger.json`, `/tmp/channel-tuple-final-ledger.json`, and `/tmp/channel-stage-count-final-{ptr,retail}.log`.

## Known gaps (current cycle)

- [x] Focused RED/GREEN and real consumer tests cover all six event producers. Initial RED: four missing-input failures; further RED exposed stale START cancellation and a replaced specialization action leak, both fixed.
- [ ] Native channel/empower stage values, automatic updates, stage achievements, target units, secret/restricted events and interruptedBy attribution are unverified.
- [ ] The unmodified Blizzard empower UPDATE handler omits hold when recomputing `maxValue` and does not rebuild stage pips. After updating stages/hold, public queries and natural deadline remain coherent, but that handler retains old pips and a charging-only display maximum. `channel_blizzard_empower_update_exposes_vendor_hold_boundary` records this observed ambiguity. No vendor patch, fake global, or unit distortion is applied; native update semantics remain unresolved.
- [ ] Earlier epochs before `retail-12-1-0` were not executed; feature gating preserves their previous query/input surface by construction.
- [ ] Local-player `UnitNameFromGUID` supports self-cancel labels on Retail 12.1+ and Forever. `UnitClassFromGUID`, other GUID identity resolution and secret-value behavior remain unmodeled on Forever.

## Out of scope

Native spell catalogs, channel damage ticks, automatic empower release input, enemy interruption modeling, artifact credit and final verification belong outside this implementation slice. Old client epochs without retail-12-1-0 keep their prior query surface.
