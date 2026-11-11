# Queued Character Upgrade Presence

`C_CharacterServices.HasQueuedUpgrade` reports whether the host has supplied a queued-upgrade GUID with non-zero type bits. The simulator stores this input in `CharacterServicesState`.

## What it must do

- [ ] Retail, PTR, and Forever expose the query on Glue screens and remove it on Game screens.
- [ ] The query takes no arguments and returns exactly one boolean.
- [ ] Return `true` only when the host-supplied queued-upgrade GUID presence input is set; return `false` by default.
- [ ] The original `CharacterSelectFrameMixin:OnUpdate` consumer refreshes the character-service UI when the query returns `true`.
- [ ] Do not infer queued-upgrade state, eligibility, or lifecycle events.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/c_character_services.rs`: registers the Glue-only query and reads the queued-GUID input.
- `src/lua_api/sim_substates/mod.rs`: stores the host-supplied queued-upgrade presence input.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`: expose and initialize the state.
- `src/lua_api/env_runtime.rs`: updates the API when the screen changes.
- `src/lua_api/workarounds/temporary/character_services_defaults.rs`: no longer supplies an assumed `false` function.

## Tests asserting this spec

- `tests/glue_character_select.rs::queued_upgrade_presence_drives_blizzard_character_select_consumer`

## Known gaps (current cycle)

- [ ] Queue mutation, GUID retrieval, and callback/event behavior are not modeled.

## Out of scope

- Character-upgrade eligibility, service assignments, and automatic upgrade flows.
