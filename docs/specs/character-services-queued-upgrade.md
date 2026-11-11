# Queued character upgrade

`C_CharacterServices.HasQueuedUpgrade()` reports whether the Glue character flow has a queued-upgrade GUID. Its selected consumer is the original `CharacterSelect.lua` update path.

## What it must do

- [x] On Retail, PTR, and WowForever Glue screens, expose `HasQueuedUpgrade()` and return exactly one boolean. It is true iff the queued GUID type bits are nonzero; a zero GUID returns false.
- [x] Do not expose `HasQueuedUpgrade` on the Game screen.
- [ ] Do not add eligibility, connection, or mode gates.

## How it works

- See the [C API boundary](../wiki/index.md) for namespace placement rules.

## Implementation inventory

- `src/c_api/c_character_services.rs` — host-supplied queued-GUID presence input and Glue registration.
- `src/lua_api/workarounds/temporary/character_services_defaults.rs` — unrelated temporary character-service defaults.
- `tests/c_character_services_probes.rs` — original CharacterSelect consumer coverage.

## Tests asserting this spec

- `tests/c_character_services_probes.rs::has_queued_upgrade_drives_the_original_character_select_update`

## Known gaps (current cycle)

- [ ] The native server-data writer for the queued GUID and any callback event name remain unknown.
- [ ] Startup processing after server data arrives is not modeled.

## Out of scope

- `GetQueuedUpgradeGUID`, `ClearQueuedUpgrade`, and their producer lifecycle are not implemented by this change.
