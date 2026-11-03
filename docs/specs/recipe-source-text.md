# Recipe source text

Retail and PTR expose `C_TradeSkillUI.GetRecipeSourceText(recipeID)` to the
Professions recipe form (`Blizzard_ProfessionsRecipeSchematicForm.lua`). Native
provenance of the returned text is **UNKNOWN**. No static source-text generator
was identified in this repository.

## What it must do

- [x] Read the current host-supplied text for the requested recipe ID.
- [x] Return nil when no text was supplied for that ID.
- [x] Keep separate recipe IDs independent and reflect host removal.

Argument error and secrecy behavior are not native-verified by this contract.

## Implementation inventory

- `src/c_api/c_trade_skill_source.rs` — query and profile registration.
- `src/lua_api/state/sim_state.rs` — supplied per-recipe text.

## Tests asserting this spec

- `tests/recipe_source_text.rs` — supplied records, missing records and removal.

## Out of scope

Native catalog provenance, localization and server/cache availability are unknown.
