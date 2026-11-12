# Profession tab state and favorite order recipes

Retail 12.1.5 consumers: `Blizzard_ProfessionsSpecializationsTemplates.lua:144` and `Blizzard_ProfessionsSpecializations.lua:110,300,584,668` call `C_ProfSpecs.GetStateForTab(tabTreeID, configID)`. `Blizzard_ProfessionsCrafterOrderPage.lua:434,523` calls `C_TradeSkillUI.HasFavoriteOrderRecipes()`. Both run when the Blacksmithing panel refreshes.

## GetStateForTab

- Both arguments are required. Each argument is a number or an unmarked numeric string. The bounds are -2147483648 through 2147483647, inclusive, and the value is truncated after the range check. Negative IDs are accepted.
- Nil or missing arguments raise an error with the usage string `Usage: local tabInfo = C_ProfSpecs.GetStateForTab(tabTreeID, configID)`.
- Nonnumeric, NaN, infinite and out-of-range values raise `outside of expected range -2147483648 to 2147483647 - <usage>`.
- A tainted caller that passes a secret value receives `<usage>. Secret values are only allowed during untainted execution for this argument.` Untainted callers can pass secret values.
- The API returns one number: `Enum.ProfessionsSpecTabState` 0 (Locked), 1 (Unlocked) or 2 (Unlockable). The state comes from the supplied `profession_spec_inputs.tab_states` of the [specialization reminders](profession-spec-reminders.md). A tree without a supplied state returns native 0.
- **UNKNOWN:** The simulator has no trait-configuration identity. The validated configID does not select a configuration, and a supplied tab state applies to every configID. The native decision (rank against the unlock entry, then purchase eligibility) is supplied, not computed.

## HasFavoriteOrderRecipes

- The API takes no arguments; extra arguments are ignored. It returns one Boolean.
- The result is true only when a loaded recipe row has both the [recipe favorite](recipe-favorites.md) flag and order eligibility. Loaded rows are the catalog rows from `GetAllRecipeIDs`.
- **VERIFIED (WM87, native build 70688):** Order eligibility is the existence of associated CraftingOrder metadata, not a customer-order count. The simulator projects the pinned **12.1.5.70391** `CraftingOrder.SkillLineAbilityID` through `SkillLineAbility.ID` to `SkillLineAbility.Spell`. No row means ineligible. `Field_10_0_2_46479_002` does not filter this projection. **UNKNOWN:** Record equality between the 70391 export and native build 70688.
- The simulator reads favorites directly. A native client can return an earlier display projection immediately after a favorite change.

## Evidence

Handoff `native-evidence-70688/WM87-profspecs-tabstate-favorite-orders.md` (Retail build 70688). Retail declaration: `Blizzard_APIDocumentationGenerated/TradeSkillUIDocumentation.lua:882`. Tests: `tests/prof_specs_reminders.rs` (argument and state contract) and `tests/workarounds_professions.rs` (the original crafting-order favorites button and API-level table cases).

## What it must do

- [ ] The original orders-page search button is enabled and its icon is not desaturated when a favorite loaded recipe has CraftingOrder metadata.
- [ ] A favorite without metadata keeps that button disabled and desaturated.
- [ ] The API returns exactly one Boolean for no favorites, only ineligible favorites, and at least one eligible favorite; extra arguments are ignored.
- [ ] Pinned 70391 recipe IDs 365729, 365730 and 438914 are eligible; recipe 2662 is not.

## How it works

- [Profession API system](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_trade_skill_favorites.rs`: favorite predicate and notifications.
- `src/c_api/crafting_order_recipe_data.rs`: sorted eligible recipe IDs and input SHA256 provenance.
- `tools/gen_crafting_order_recipes.py`: reproducible pinned CSV projection; missing ability relations are errors.

## Tests asserting this spec

- `tests/workarounds_professions.rs`: original UI consumer and API-level favorite cases.
- `src/c_api/c_trade_skill_favorites.rs`: exact 70391 exported recipe membership.
- `tests/prof_specs_reminders.rs`: specialization tab states and validation.

## Known gaps (current cycle)

- [ ] Native display-projection refresh timing and saved/initial favorites remain UNKNOWN.

## Out of scope

- Trait configuration identity and native trait purchase decisions remain supplied rather than computed, as described above.
