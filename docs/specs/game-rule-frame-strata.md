# Game-rule frame strata

`C_GameRules.GetGameRuleAsFrameStrata` projects a supplied rule value for the World Map and other consumers. Pinned declarations: Retail `GameRulesDocumentation.lua:143`, PTR `:188`, WowForever `:161`, Mists, Era and Anniversary `:137`; Wrath has no declaration.

## What it must do

- [ ] Validate the required numeric or numeric-string rule identifier against `Enum.GameRule`; reject invalid and overflowing identifiers.
- [ ] Return one nil for an absent rule. For present integer values 0–9, return WORLD, BACKGROUND, LOW, MEDIUM, HIGH, DIALOG, FULLSCREEN, FULLSCREEN_DIALOG, TOOLTIP and BLIZZARD respectively. Other integer values return UNKNOWN.
- [ ] The original World Map OnShow applies a supplied known strata and leaves the current strata unchanged for nil or UNKNOWN.

The return selection is based on Retail 70688 post-lookup emulation; other documented profiles use the same supplied-state behavior as a best guess.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/game_rules_strata.rs`: argument validation and supplied-state projection.
- `src/lua_api/globals/game_rules.rs`: rule ID lookup and profile-gated registration.

## Tests asserting this spec

- `tests/blizzard_world_map_loads.rs`: original consumer and supplied values.

## Known gaps (current cycle)

- [ ] Native float raw-bit interpretation and absent-tag payloads are not modeled.
- [ ] Runtime transitions and notifications are unknown.

## Out of scope

- Server rule installation and mode-specific values.
