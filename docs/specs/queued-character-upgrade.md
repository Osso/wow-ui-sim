# Queued Character Upgrade Presence

`C_CharacterServices.HasQueuedUpgrade` reports whether the host has supplied a queued-upgrade GUID with non-zero type bits. The simulator stores this input in `CharacterServicesState`.

## What it must do

- [x] Retail, PTR, and Forever expose the query on Glue screens and remove it on Game screens.
- [x] The query takes no arguments and returns exactly one boolean.
- [x] Return `true` only when the host-supplied queued-upgrade GUID type bits are nonzero; a zero GUID returns `false`, which is the default.
- [x] The original `CharacterSelectFrameMixin:OnUpdate` consumer refreshes the character-service UI when the query returns `true`.
- [x] Do not infer queued-upgrade state, eligibility, connection or mode gates, or lifecycle events.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- See the [C API boundary](../wiki/index.md) for namespace placement rules.

## Implementation inventory

- `src/c_api/c_character_services.rs`: defines `CharacterServicesState` (host-supplied queued-GUID presence), registers the Glue-only query, and reads the input.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`: expose and initialize the state.
- `src/lua_api/env_runtime.rs`: updates the registration when the screen changes.
- `src/lua_api/workarounds/temporary/character_services_defaults.rs`: no longer supplies an assumed `false` function.

## Tests asserting this spec

- `tests/c_character_services_probes.rs::has_queued_upgrade_drives_the_original_character_select_update`
- `tests/glue_character_select.rs::queued_upgrade_presence_drives_blizzard_character_select_consumer`

## Known gaps (current cycle)

- [ ] The native server-data writer for the queued GUID, GUID retrieval, queue mutation, and any callback event are not modeled.
- [ ] Startup processing after server data arrives is not modeled.

## Out of scope

- `GetQueuedUpgradeGUID`, `ClearQueuedUpgrade`, character-upgrade eligibility, service assignments, and automatic upgrade flows.
