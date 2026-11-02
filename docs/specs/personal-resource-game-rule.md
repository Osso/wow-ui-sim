# Personal resource display game rule

## What it must do

- [ ] Retail `C_GameRules.IsPersonalResourceDisplayEnabled()` returns one boolean: false when `PersonalResourceDisplayDisabled` rule 129 is true; otherwise the effective `nameplateShowSelf` setting controls the result.
- [ ] A missing disabling rule is false. Ordered nonzero numeric rule values are true; zero and NaN are false. An absent setting uses the recovered cached default false.
- [ ] The original personal-resource frame reflects the decision in `UpdateShownState`.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/GameRulesDocumentation.lua:231` declares the boolean and identifies the CVar. `Blizzard_PersonalResourceDisplay/Blizzard_PersonalResourceDisplay.lua:176` consumes it. The WM23-rules build 70688 handoff provides the local game-rule/CVar decision and conditional emulation of boolean publication; no live-client result is claimed.

## Implementation inventory

- `src/c_api/game_rules_personal_resource.rs`: decision over the existing game-rule and CVar state.
- `src/lua_api/globals/game_rules.rs`: Retail registration.

## Tests asserting this spec

- `tests/personal_resource_game_rule.rs`: original frame visibility through CVar and rule transitions.

## Known gaps

- [ ] Live CVar object and cached-setting update timing, native treatment of unmodeled rule types, and extra arguments remain unverified. The simulator reads the effective CVar string as an integer.
