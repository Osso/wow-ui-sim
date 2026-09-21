# Unit cast duration queries

Modeled player cast and channel duration queries live in `src/lua_api/channeling/durations.rs`. See [channel lifecycle](channel-empower-lifecycles.md) for simulator input contracts.

## What it must do

- [ ] Retail 12.1/PTR and Forever expose `UnitCastingDuration`, `UnitChannelDuration`, and `UnitEmpoweredChannelDuration` without enabling unrelated Retail APIs in Forever.
- [ ] Missing matching player state and unmodeled non-player units return zero results.
- [ ] Cast and channel objects snapshot stored start/end timestamps with rate one; repeated queries reflect timeline updates.
- [ ] Empowered duration defaults to including hold-at-max; explicit false excludes hold. Ordinary channel duration uses the base channel end (inferred from the native CastingBar's separate hold adjustment).
- [ ] `UnitEmpoweredStagePercentages(unit, includeHoldAtMaxTime=true)` returns a dense vector of individual stage fractions, not cumulative offsets. Default/true appends the hold fraction and includes hold in the denominator; false excludes both. For stages `{1,2}` and hold `3`, results are `{1/6,2/6,3/6}` or `{1/3,2/3}`. A zero hold still contributes a final zero element when included. Missing empowered player state/non-player units return zero results; updates, replacement and removal follow authoritative state.
- [ ] Replacement, cancellation and natural completion change duration availability through existing simulator lifecycle inputs.
- [ ] Numeric cast-bar IDs in casting/channel tuples match update/stop event payloads so consumer identity guards work.
- [ ] `UnitNameFromGUID` resolves the modeled player interrupt actor to current name and simulator realm; unknown GUIDs return no values. `UnitClassFromGUID` availability is unchanged.

## How it works

- [Duration core](../wiki/systems/duration-core.md)
- [Channel lifecycle contract](channel-empower-lifecycles.md)

## Implementation inventory

- `src/lua_api/channeling/durations.rs`: duration snapshots and empowered stage fractions over authoritative cast state.
- `src/lua_api/channeling.rs`: shared channel lifecycle and query registration.
- `Cargo.toml`: `player-cast-durations` capability shared by Retail 12.1 and Forever.
- `src/lua_api/{mod.rs,on_update.rs,spellcast_events.rs}`: lifecycle availability and event identity.
- `src/lua_api/globals/{admin.rs,combat_verbs.rs}`: channel input and replacement/cancel availability.
- `src/lua_api/globals/utility_system_spell/spell_api.rs`: query registration and identity tuples.
- `src/lua_api/globals/real/player_identity.rs`: shared interrupt actor name query.

## Tests asserting this spec

- `tests/unit_cast_durations.rs`: idle/non-player absence, cast clocks and delays, channel updates/replacement/cancellation, empowered hold/completion, consumer-shaped identity matching through real cast-start events.
- `tests/unit_empowered_stage_percentages.rs` and `tests/fixtures/unit_empowered_stage_percentages.lua`: unequal stage fractions, optional/zero hold, absence, mutation isolation, updates, replacement and completion. The fixture also provides `CheckEllesmereEmpowerPips()` for external real-addon replay: actual `UpdatePips` must place a 600-wide bar's pips at `{100,300,600}`, then `{200,600}` without hold and hide the third pip.
- `src/iced_app/casting/duration_tests.rs`: ordinary-cast removal and duration absence during callbacks at the GUI completion boundary. `WowLuaEnv::fire_on_update` alone does not perform this stage.

## Known gaps (current cycle)

- [ ] Integrated GREEN and real Ellesmere consumer acceptance pending.

## Out of scope

- Other-unit cast state: no backing model exists.
- Native execution or conformance claim: contracts come from pinned Forever `UnitDocumentation.lua`; `CastingBarFrame.lua` separately adds hold to `UnitChannelInfo` end time, but does not call the new duration APIs.
