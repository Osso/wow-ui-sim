# Recipe first craft

Retail 12.1.5 consumer: `Blizzard_ProfessionsRecipeSchematicForm.lua:652` (SchematicForm `Init`) calls `C_TradeSkillUI.IsRecipeFirstCraft(recipeID)` to show the first-craft bonus.

## Contract

- One required argument: a number or an unmarked numeric string. The bounds are -2147483648 through 2147483647, inclusive, and the value is truncated after the range check. Negative IDs are accepted, and extra arguments are ignored.
- Nil or missing arguments raise an error with `Usage: local result = C_TradeSkillUI.IsRecipeFirstCraft(recipeID)`. Nonnumeric, NaN, infinite and out-of-range values raise `outside of expected range -2147483648 to 2147483647 - <usage>`. A tainted caller that passes a secret value receives `<usage>. Secret values are only allowed during untainted execution for this argument.` This is the same converter as [GetStateForTab](profession-tab-state-favorite-orders.md), not the integer-overflow diagnostic of [GetRecipeCooldown](recipe-cooldown.md).
- The API returns exactly one Boolean. A recipe without an ability relation, including an unknown ID, returns false, not zero values.
- `GetRecipeInfo().firstCraft` publishes the same predicate. It was previously absent (nil).

## Seams

- **EMULATED:** The simulator does not export SkillLineAbility, the first SpellEffect (types 301, 288, 294, 296) or CraftingData. A catalog recipe stands for a resolved ability with first-craft metadata.
- **UNKNOWN:** Completion bookkeeping (the quest or character-flag source) is not modeled. Its unavailable state reads clear, as on the native missing-state paths, so catalog recipes return true. True does not prove that a character never crafted the recipe.
- **UNKNOWN:** The post-craft writer, its packet origin and any first-craft event are not modeled. Nothing in the simulator clears the first-craft state.
- The direct query and the `GetRecipeInfo` projection always agree in the simulator. A native client can show a stale projection until the next rebuild.

## Evidence

Handoff `native-evidence-70688/WM88-recipe-first-craft.md` (Retail build 70688). Test: `tests/workarounds_professions.rs::recipe_first_craft_resolves_ability_and_validates_arguments`.
