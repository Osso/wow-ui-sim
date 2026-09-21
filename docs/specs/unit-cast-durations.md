# Unit cast duration queries

Modeled player cast and channel duration queries live in `src/lua_api/channeling/durations.rs`. See [channel lifecycle](channel-empower-lifecycles.md) for simulator input contracts.

## What it must do

- [ ] Retail 12.1/PTR and Forever expose `UnitCastingDuration`, `UnitChannelDuration`, and `UnitEmpoweredChannelDuration` without enabling unrelated Retail APIs in Forever.
- [ ] Missing matching player state and unmodeled non-player units return zero results.
- [ ] Cast and channel objects snapshot stored start/end timestamps with rate one; repeated queries reflect timeline updates.
- [ ] Empowered duration defaults to including hold-at-max; explicit false excludes hold. Ordinary channel duration uses the base channel end (inferred from the native CastingBar's separate hold adjustment).
- [ ] Replacement, cancellation and natural completion change duration availability through existing simulator lifecycle inputs.
- [ ] Numeric cast-bar IDs in casting/channel tuples match update/stop event payloads so consumer identity guards work.

## How it works

- [Duration core](../wiki/systems/duration-core.md)
- [Channel lifecycle contract](channel-empower-lifecycles.md)

## Implementation inventory

- `src/lua_api/channeling/durations.rs`: duration snapshots over authoritative cast state.
- `src/lua_api/channeling.rs`: shared channel lifecycle and query registration.
- `Cargo.toml`: `player-cast-durations` capability shared by Retail 12.1 and Forever.
- `src/lua_api/{mod.rs,on_update.rs,spellcast_events.rs}`: lifecycle availability and event identity.
- `src/lua_api/globals/{admin.rs,combat_verbs.rs}`: channel input and replacement/cancel availability.
- `src/lua_api/globals/utility_system_spell/spell_api.rs`: query registration and identity tuples.

## Tests asserting this spec

- `tests/unit_cast_durations.rs`: idle/non-player absence, cast clocks and delays, channel updates/replacement/cancellation, empowered hold/completion, consumer-shaped identity matching through real cast-start events.
- `src/iced_app/casting/duration_tests.rs`: ordinary-cast removal and duration absence during callbacks at the GUI completion boundary. `WowLuaEnv::fire_on_update` alone does not perform this stage.

## Known gaps (current cycle)

- [ ] Integrated GREEN and real Ellesmere consumer acceptance pending.

## Out of scope

- Other-unit cast state: no backing model exists.
- Native execution or conformance claim: contracts come from pinned Forever `UnitDocumentation.lua`; `CastingBarFrame.lua` separately adds hold to `UnitChannelInfo` end time, but does not call the new duration APIs.
