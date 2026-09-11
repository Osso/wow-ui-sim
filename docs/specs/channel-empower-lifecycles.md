# Simulator channel and empower lifecycles

Explicit `A_Admin` inputs drive the channel slot and its six spellcast events. Payloads follow the pinned PTR source register; timing and transition choices below are simulator policies, not verified native gameplay. See [cast-bar returns](cast-bar-id-returns.md) and [spellcast payloads](spellcast-event-payloads.md).

## What it must do

- [ ] `StartChannel(spellID, name, icon, durationSeconds)` starts a channel; `UpdateChannel(durationSeconds)` changes total duration from its original start without changing identity.
- [ ] `StartEmpower(spellID, name, icon, stageSeconds, holdSeconds)` starts an empowered channel. `UpdateEmpower(stageSeconds, holdSeconds)` replaces timing without changing identity. Stages are a dense, nonempty array of positive finite seconds; hold is nonnegative.
- [ ] `StopChannel(complete=false)` stops either channel kind, returning whether state existed. Explicit true models successful early release; false models self-cancel. Repeated stops return false.
- [ ] Inputs validate before mutation, including millisecond overflow. An input for the wrong active kind returns false without mutation.
- [ ] `UnitChannelInfo` retains its eleven-field tuple. Its endpoint excludes empower hold; natural completion includes hold. Stage count derives from the stage vector rather than an independent mutable count.
- [ ] `GetUnitEmpowerStageDuration("player", zeroBasedIndex)` and `GetUnitEmpowerHoldAtMaxTime("player")` return **milliseconds**. Missing stages, missing empower state, and unsupported units return no values as simulator policy.
- [ ] Channel/empower starts and updates emit `(unit, castGUID, spellID, castBarID)`. Channel stop emits `(unit, castGUID, spellID, interruptedBy, castBarID)`; empower stop inserts `complete` before `interruptedBy`.
- [ ] Natural completion/early release uses nil `interruptedBy`; cancellation uses current player GUID. The nil completion convention follows Blizzard's consumer but conflicts with generated non-nil metadata; native conformance remains unclaimed.
- [ ] Channel deadlines are checked at the existing OnUpdate boundary against the same clock as GetTime. Tests advance that deadline clock without sleeping. No periodic gameplay ticks, damage, or healing are invented.
- [ ] Replacement leaves only one active cast/channel. Terminal state is extracted before callbacks. Reentrant replacement wins; old events retain immutable old IDs and cannot later complete.
- [ ] Existing `SetCasting`/`StopCasting` signatures remain unchanged. Ordinary cast producers clear replaced channel state; `SpellStopCasting` also self-cancels a channel when no ordinary cast exists.
- [ ] Real Blizzard channel/empower view, stage pips, update and stop handling observe the modeled state without vendor patches.

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

## Tests asserting this spec

- `tests/channel_lifecycle.rs`: public inputs, queries, callbacks, update/stop/completion and reentrancy.
- Real Blizzard consumer fixture to be recorded with development proof.

## Known gaps (current cycle)

- [ ] Complete focused RED/GREEN proof and real consumer tests.
- [ ] Native channel/empower stage values, automatic updates, stage achievements, target units, secret/restricted events and interruptedBy attribution are unverified.
- [ ] Public millisecond end-time excludes hold on empower start; consumer UPDATE behavior must be checked independently rather than altering vendor code or distorting query units.

## Out of scope

Native spell catalogs, channel damage ticks, automatic empower release input, enemy interruption modeling, artifact credit and final verification belong outside this implementation slice. Old client epochs without retail-12-1-0 keep their prior query surface.
