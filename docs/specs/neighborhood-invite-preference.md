# Neighborhood invite preference

Forever exposes independent neighborhood invite auto-decline state through two real globals in `src/lua_api/globals/real/neighborhood_invites.rs`. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

- [x] Forever getter returns its per-environment boolean; setter stores true/false and returns no values.
- [x] Neighborhood state remains independent of guild invites and recent-allies location visibility, other environments, and bootstrap restoration.
- [x] Omitted/nil setter argument defaults to false, following the project's optional-argument convention. Other nonboolean public values fail without changing state.
- [x] Documented `AllowedWhenUntainted` secret arguments use existing VM caller validation before default/type conversion; tainted callers may still supply public booleans/default arguments.
- [x] Existing `hooksecurefunc` consumers can observe updated state in the AccountUI save/change/load pattern.

Cached Forever `PlayerScriptDocumentation.lua` documents getter return bool and setter `allow` bool, `Nilable=false`, `Default=false`, `SecretArguments=AllowedWhenUntainted`. Its argument default does **not** establish stored initial state. Initial false is an explicit simulator guess. No update event is documented; none is introduced.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/neighborhood_invites.rs`: getter, defaulted setter and VM validation.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs`: Forever-only registration.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: distinct state field and guessed initial false.

## Tests asserting this spec

- `tests/neighborhood_invite_preference.rs`: grouped integration tests for round trips, defaults, independence/bootstrap, security and addon-shaped hook/save/load.

## Known gaps (current cycle)

Targeted development proof: 5/5 tests pass at `6eccec458`; two identical Lua bodies fail against the prior frozen binary. Commands, source hashes, build features and frozen executable SHA are recorded in `/tmp/forever-addon-audit/neighborhood-preference-development-ledger.json`.

- [ ] Independent verification remains parent-owned.
- [ ] Parent-owned unchanged AccountUI full-save/load replay remains required; this producer alone does not prove addon compatibility.

## Out of scope

Native initial-state conformance, cross-process persistence, actual invitation acceptance/decline delivery, other profiles, guild/location changes and undocumented events.
