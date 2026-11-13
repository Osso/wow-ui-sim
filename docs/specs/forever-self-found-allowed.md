# Forever Self-Found Allowance

`C_GameRules.IsSelfFoundAllowed` reports the Forever self-found capability from numeric game rule 22.

## What it must do

- [x] Forever registers `C_GameRules.IsSelfFoundAllowed` and returns exactly one Boolean value.
- [x] A missing rule-22 entry returns `false`.
- [x] **INFERRED:** a nonzero numeric rule-22 value returns `true`.

## How it works

- [C API and Lua API architecture](../lua-api.md)
- [Game-rule data flow](../wiki/systems/game-rules.md)

## Implementation inventory

- `src/c_api/c_game_rules.rs`: reads numeric Forever rule values and registers the query.
- `src/lua_api/globals/game_rules.rs`: installs the Forever query on `C_GameRules`.
- `src/lua_api/sim_substates/mod.rs`: stores numeric rules by ID.

## Tests asserting this spec

- `tests/game_rules.rs::forever_self_found_allowed_uses_rule_22_and_defaults_false`

## Known gaps (current cycle)

- [ ] The normal-realm rule-22 value depends on server-populated rule state and is unknown.

## Out of scope

- The PTR `game_rules.self_found_allowed` Boolean remains separate from `rules_by_id`; this change does not synchronize or unify those stores.
