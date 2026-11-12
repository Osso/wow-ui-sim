# Loot method style

`C_PartyInfo.GetLootMethodStyle()` reports the active loot-style rule from `C_GameRules`. The simulator reads the supplied `GameRulesState`; see the [Lua API system](../wiki/systems/lua-api.md).

## What it must do

- [ ] Return exactly one numeric value. Ignore extra arguments.
- [ ] Read `Enum.GameRule.LootMethodStyle` (rule 157), with 0 (`Mainline`) when the rule is absent.
- [ ] Return the supplied rule value independently of group membership and the selected loot method; `SetLootMethod` does not change it.
- [ ] When the rule collection changes, callers can re-read the getter on `GAME_RULES_CHANGED`; there is no loot-style-specific event.
- [ ] Use the same rule lookup on Retail and PTR 12.1.7.

## How it works

- [Lua API system](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_party_info/loot_method.rs`: registers `C_PartyInfo.GetLootMethodStyle` and reads its value.
- `src/lua_api/sim_substates/mod.rs`: defines the supplied game-rule map.

## Tests asserting this spec

- `tests/party_loot_method.rs::loot_method_style_reads_rule_157_and_ignores_loot_method`
- `tests/party_loot_method.rs::ptr_loot_method_style_rereads_rule_157_on_game_rules_changed`

## Known gaps (current cycle)

- [ ] Server-provided rule values in live sessions are not modeled; the simulator uses only supplied state and the absent-rule default.

## Out of scope

- Producing `GAME_RULES_CHANGED` when external code replaces the game-rule collection.
- Native behavior for server rule installation and group/session transitions.
