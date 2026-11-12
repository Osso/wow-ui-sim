# Recipe cooldown

Retail 12.1.5 `C_TradeSkillUI.GetRecipeCooldown(recipeID)` returns no values for a recipe absent from the simulator recipe catalog. For a present recipe it returns four values: remaining seconds or nil, day-cooldown flag, available charges, and maximum charges. Supplied spell cooldown and charge inputs are keyed by recipe ID. A recovering charge replaces the ordinary cooldown result; full charges leave the ordinary result intact. The simulator does not advance charge state or emit cooldown events automatically.

Inputs accept numbers and unmarked numeric strings, truncate fractions toward zero, and reject nonnumeric or inaccessible secret values with the usage error. Out-of-range numbers raise integer overflow. Secret numeric access follows the VM's context policy.

The day flag comes from pinned Retail 12.1.5.70391 data: the recipe spell's base-difficulty `SpellCategories.Category` selects `SpellCategory.Flags`, and mask 8 indicates a day cooldown. Duration does not determine the flag. A missing category is false; equivalence of 70391 records to build 70688 is unknown. Server-side day reset behavior remains unknown. Wrapped 32-bit remaining milliseconds are converted as unsigned to single-precision seconds; only positive ordered results are published. Ordinary expired timer candidates are rejected before conversion. A recovering charge's expired nonzero end is adjusted to the selected clock; a zero end is not. Final conversion reads the clock again. Live stale-state behavior and the global charge override remain unknown. Native evidence: handoff `native-evidence-70688/WM85-recipe-cooldown.md` and `WM85b-signedness.md`.

The static lookup is regenerated from the pinned CSV inputs with `python3 tools/gen_recipe_day_cooldowns.py --spell-category <70391>/SpellCategory.csv --spell-categories <70391>/SpellCategories.csv --output src/c_api/recipe_day_cooldown_data.rs`, followed by `cargo fmt --all`. Input SHA-256 values are recorded in the generated file. These are existing pinned 70391 CSVs, not a fresh export from the current install.

## PTR registration

VERIFIED: PTR 70493 `Blizzard_ProfessionsTemplates/Blizzard_ProfessionsRecipeSchematicForm.lua:563` consumes this API. The registered contract is the same as Retail WM85, **INFERRED for PTR** pending native R85 evidence. Retail native evidence does not establish PTR parity.
