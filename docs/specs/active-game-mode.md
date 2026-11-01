# Active game mode

`C_GameRules.GetActiveGameMode` returns the published `Enum.GameMode` value. Retail `Blizzard_APIDocumentationGenerated/GameRulesDocumentation.lua:36-41` declares the return type; the retained live-client discovery at `docs/wow-client-diff/WowDiscovery.lua:14648-14652` records Standard=1, Plunderstorm=2, and WoWHack=3.

## What it must do

- [x] The initial simulator scenario returns `Enum.GameMode.Standard`.
- [x] A supplied Plunderstorm mode returns `Enum.GameMode.Plunderstorm` and makes `C_GameRules.IsPlunderstorm()` true.
- [x] A supplied Standard mode returns `Enum.GameMode.Standard` and makes `C_GameRules.IsPlunderstorm()` false.
- [ ] On the six documented profiles, `C_GameRules.IsStandard()` returns one boolean that is true exactly when the supplied mode is Standard.

## Best-guess basis

Retail 12.1.5 build 70688 evidence verifies comparison of the cached mode to Standard; its connection to Lua boolean publication is emulated. The same supplied-state behavior on the other documented profiles is a best guess. `IsStandard` is declared in each profile's `Blizzard_APIDocumentationGenerated/GameRulesDocumentation.lua`: Retail:250, PTR:340, WowForever:295, Mists:258, Era:258, Anniversary:258. Wrath has no pinned declaration.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/game_mode.rs`: shared published mode values.
- `src/lua_api/globals/enum_data/widget.rs`: Lua enum publication.
- `src/lua_api/sim_substates/mod.rs`: active mode state and initial scenario.
- `src/lua_api/globals/game_rules.rs`: game-rule queries and supplied state mutation.

## Tests asserting this spec

- `tests/game_rules.rs`: default and supplied mode queries and `IsStandard`.
- `tests/blizzard_game_menu_loads.rs`: original GameMenu consumer.
- `tests/cmd_option_selected_unit.rs`: unchanged cached Blizzard slash-command consumer under the default mode.

## Known gaps (current cycle)

- [ ] Live mode-selection policy, transitions, and event timing are not established by the enum capture. The native rule installer's full integration with supplied state is not modeled.

## Out of scope

- Plunderstorm addon selection, which is tracked separately in [Plunderstorm addon selection](plunderstorm-addon-selection.md).
