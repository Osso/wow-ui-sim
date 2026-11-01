# Death recap events

Retail `C_DeathRecap.GetRecapEvents` publishes host-supplied recap events to the original Blizzard consumer and its deprecated alias.

## What it must do

- [ ] A missing or nil ID selects the newest host-marked eligible recap among the last 60. An explicit ID selects a matching recap, including an ineligible one.
- [ ] A missing recap returns zero values. A found recap returns one array with at most ten events.
- [ ] Each event exposes supplied `currentHP` and, when supplied, `amount`, `timestamp` and `sourceGUID`. No missing event field is invented.
- [ ] Ordinary nonnumeric values select ID zero; numeric inputs convert to signed integers and overflow reports an error.

## Best-guess basis

Retail build 70688 static evidence identifies lookup, eligibility and ten-event limit. The connection from lookup to Lua return is emulated rather than observed in a running client. `DeathRecapDocumentation.lua:11–22` declares an optional ID and event table; `Deprecated_CombatLog.lua:28` publishes the alias. `Blizzard_DeathRecap.lua:317–340` consumes the event array.

## How it works

- [Death recap current state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_death_recap.rs` registers and reads the Retail API.
- `src/lua_api/state_types/mythic_plus_scenario.rs` holds host-supplied recap records.

## Tests asserting this spec

- `tests/publication_deprecated_alias.rs` checks two supplied states through the loaded Blizzard alias.

## Known gaps (current cycle)

- [ ] Live return behavior, complete event schema, eligibility transitions, circular cursor and secret-value handling remain unverified.
- [ ] Other event fields and host input paths are not modeled by this contract.

## Out of scope

Runtime combat collection and recap persistence are not modeled here.
