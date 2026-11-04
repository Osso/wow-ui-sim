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
- **EMULATED/UNKNOWN:** Native order eligibility comes from the CraftingOrder metadata of a recipe, which the simulator does not export. All rows are treated as not eligible, so the result is currently false. This is a data gap, not a native claim about any recipe.
- The simulator reads favorites directly. A native client can return an earlier display projection immediately after a favorite change.

## Evidence

Handoff `native-evidence-70688/WM87-profspecs-tabstate-favorite-orders.md` (Retail build 70688). Tests: `tests/prof_specs_reminders.rs` (argument and state contract) and `tests/workarounds_professions.rs` (the original crafting-order favorites button).
