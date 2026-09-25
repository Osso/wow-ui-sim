# Modeled recipe schematic lookup

`C_TradeSkillUI.GetRecipeSchematic` exposes schematics from the simulator's static profession recipe data. This is a simulator absence policy, not a claim about native unknown-ID behavior: cached generated API documentation marks the result `Nilable=false`, but does not supply a recipe absent from the local model.

## What it must do

- [ ] A recipe absent from the static model returns nil rather than an incomplete, truthy schematic. A consumer checking `if not schematicInfo then return end` can skip reagent construction before iterating `reagentSlotSchematics`.
- [ ] A modeled recipe returns its ID, name, output item, output quantities, and populated reagent-slot schematics.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/profession_data.rs` — static modeled recipes.
- `src/lua_api/globals/missing_surface/professions.rs` — schematic query entry point.
- `src/lua_api/globals/missing_surface/professions/professions_tables.rs` — schematic value construction.

## Tests asserting this spec

- `tests/professions_api.rs` — unknown-ID consumer short-circuit and known-recipe fields.

## Known gaps (current cycle)

- [ ] Focused RED/GREEN execution is blocked by unrelated compilation failure in `src/lua_api/globals/real/modifier_keys.rs` until that concurrent change is corrected.

## Out of scope

- Inventing schematic fields or reagents for unmodeled recipes.
- Native-client unknown-ID return semantics; no native probe establishes them here.
- Other profession query fallback behavior.
