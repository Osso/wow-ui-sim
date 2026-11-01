# Action bar visibility rule

`C_GameRules.IsMultiActionBarVisibilityForced` is declared by pinned `Blizzard_APIDocumentationGenerated/GameRulesDocumentation.lua` for Retail:222, PTR:285, WowForever:249, Mists:221, Era:221 and Anniversary:221. Wrath has no pinned declaration.

## What it must do

- [ ] Return one boolean: true if either supplied integer `ForcedMultiActionBarSetting` or `HideAllMultiActionBars` is nonzero, including negative values; otherwise false.
- [ ] Do not treat string or boolean rules as integer values merely because another query can coerce them.
- [ ] The original action-bar settings consumer creates seven checkboxes when neither integer rule is active and none when either forces visibility.

## Best-guess basis

Retail 12.1.5 build 70688 evidence verifies the two local lookups and decision. Connection to Lua publication is emulated, not verified end-to-end. Other documented profiles use the same supplied-state interpretation as a best guess. No per-game-mode rule values are inferred.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/game_rules_visibility.rs`: supplied rule predicate.
- `src/lua_api/sim_substates/mod.rs`: original integer-value representation.
- `src/lua_api/globals/game_rules.rs`: supplied rule mutation and documented-profile publication.

## Tests asserting this spec

- `tests/blizzard_settings_definitions_frame_loads.rs`: original Settings consumer and supplied integer and non-integer rules.

## Known gaps (current cycle)

- [ ] Native game-mode-to-rule mappings, runtime transitions and notification timing are unknown.

## Out of scope

- Server rule installation and action bar layout rendering.
