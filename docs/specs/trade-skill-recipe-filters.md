# Trade Skill Recipe Filters

`C_TradeSkillUI` recipe-list filters used by the mainline Professions filter
helpers in `Blizzard_ProfessionsTemplates/Blizzard_Professions.lua`
(`Professions.SetDefaultFilters`, `IsUsingDefaultFilters`,
`GetCurrentFilterSet`, `ApplyfilterSet`, `InitFilterMenu`).

## Profiles

Retail, PTR and Forever. Other profiles keep only the name search
(`SetRecipeItemNameFilter`, `GetRecipeItemNameFilter`) and
`GetFilteredRecipeIDs`.

## Behaviour

- `GetFilteredRecipeIDs` returns catalogue recipe IDs, in catalogue order, that
  pass every active filter:
  - name search (case-insensitive substring);
  - `GetShowLearned` / `GetShowUnlearned` by the recipe's learned state;
  - `GetOnlyShowMakeableRecipes` keeps craftable recipes only;
  - `SetRecipeCategoryFilter(categoryID, filtered)` hides that category;
  - `SetRecipeItemLevelFilter(min, max)` keeps recipes whose item level is in
    the inclusive range; `(0, 0)` clears the range.
- Every setter and clear call fires `TRADE_SKILL_LIST_UPDATE`.
- `SetRecipeSourceTypeFilter(index, filtered)` and
  `IsRecipeSourceTypeFiltered(index)` track filtered source indices.
  `GetSourceTypeFilter` / `SetSourceTypeFilter` expose them as a bitmask where
  bit `index - 1` means filtered.
- `SetInventorySlotFilter(index, shown, exclusive)` records the slot as
  filtered when `shown` is false; `exclusive` clears other slots first.
  `GetAllFilterableInventorySlotsCount` returns 0, and
  `GetFilterableInventorySlotName` raises an out-of-range error.
- `AreAnyInventorySlotsFiltered` and `AnyRecipeCategoriesFiltered` report
  whether any slot or category filter is set.

## Defaults

Before the UI applies `Professions.SetDefaultFilters`: learned shown,
unlearned hidden, all other filters off. This keeps the pre-existing
learned-only list; native startup defaults are UNKNOWN (INFERRED).

## Known gaps

- Skill-up and first-craft flags are stored only: the catalogue has no
  relative difficulty or first-craft state.
- Source-type and inventory-slot filters never exclude a recipe: the catalogue
  has no recipe source or output-slot data.
