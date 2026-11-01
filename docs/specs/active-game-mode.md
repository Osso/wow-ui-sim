# Active game mode

`C_GameRules.GetActiveGameMode` returns the published `Enum.GameMode` value. Retail `Blizzard_APIDocumentationGenerated/GameRulesDocumentation.lua:36-41` declares the return type; the retained live-client discovery at `docs/wow-client-diff/WowDiscovery.lua:14648-14652` records Standard=1, Plunderstorm=2, and WoWHack=3.

## What it must do

- [x] The initial simulator scenario returns `Enum.GameMode.Standard`.
- [x] A supplied Plunderstorm mode returns `Enum.GameMode.Plunderstorm` and makes `C_GameRules.IsPlunderstorm()` true.
- [x] A supplied Standard mode returns `Enum.GameMode.Standard` and makes `C_GameRules.IsPlunderstorm()` false.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/game_mode.rs`: shared published mode values.
- `src/lua_api/globals/enum_data/widget.rs`: Lua enum publication.
- `src/lua_api/sim_substates/mod.rs`: active mode state and initial scenario.
- `src/lua_api/globals/game_rules.rs`: game-rule queries and supplied state mutation.

## Tests asserting this spec

- `tests/game_rules.rs`: default and supplied mode queries.
- `tests/cmd_option_selected_unit.rs`: unchanged cached Blizzard slash-command consumer under the default mode.

## Known gaps (current cycle)

- [ ] Live mode-selection policy, transitions, and event timing are not established by the enum capture.

## Out of scope

- Plunderstorm addon selection, which is tracked separately in [Plunderstorm addon selection](plunderstorm-addon-selection.md).
