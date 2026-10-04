# Retail 12.0.7 removed native surface

Simulator registrations must stop publishing the page-listed retired natives at cumulative `retail-12-0-7`. Cached Blizzard deprecation Lua owns optional legacy forwarding. See [client profiles](../wiki/systems/client-profiles.md).

## What it must do

- [x] Before cached deprecation load, omit the 17 page-listed globals/namespace entries, including six previously registered aliases.
- [x] Real cached deprecation files forward exact inputs, coercions and result tuples only when their CVar permits loading.
- [x] Six retired Minimap texture setters remain absent from lookup, calls and metatable before and after cached deprecation and Game UI load; zoom remains usable.
- [ ] Preserve existing older-epoch positive assertions while retaining current live APIs, including Maw links, global modifiers and KickUnit.

## How it works

- [Client profiles and cumulative epochs](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_spell_maw_powers.rs`: atlas lifetime gate; live link query retained.
- `src/lua_api/globals/group_verbs.rs`: UninviteUnit publication gate; KickUnit retained.
- `src/lua_api/frame/methods/map_frames.rs`: six setter publication/body gates.
- `src/lua_api/workarounds/temporary/auto_complete_defaults.rs`: old alias publication lifetime.
- `src/lua_api/workarounds/temporary/click_bindings_defaults.rs`: old namespace alias publication lifetime.
- `src/lua_api/workarounds/temporary/spell_static_defaults.rs`: prevent retired atlas republishing.

## Tests asserting this spec

- `tests/patch_12_0_7_removed_native_surface.rs`: native absence and unchanged cached forwarding.
- `tests/spell_maw_powers.rs`, `tests/group_verbs.rs`, `tests/global_frames.rs`, `tests/click_targeting.rs`: epoch splits and live behavior.
- `tests/c_auto_complete_probes.rs`, `tests/startup_api_stubs.rs`: live namespace and retired globals.
- `src/loader/tests/wow_api_globals/startup_globals.rs`, `src/loader/tests/minimap_specialized.rs`: startup and widget epoch controls.
- Bootstrap module tests: namespace and live-global controls.

## Known gaps (current cycle)

- [ ] Older epoch branches remain unexecuted in this round.

## Out of scope

Historical native availability, service/backend parity, alternate-feature builds and vendor modifications. Later cached sources do not establish historical chronology.
