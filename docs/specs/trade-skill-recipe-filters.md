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
- Filter setters and clears request a shared `TRADE_SKILL_LIST_UPDATE` only
  when their stored state changes. Requests coalesce until a later update tick
  drains one event; setters do not dispatch it synchronously. Changes made by
  listeners during a drain remain pending for a later tick. The tick boundary
  is a simulator choice; native timing beyond deferred delivery is unknown.
  Retail 12.1.5 build 70688 static evidence establishes this behavior for the
  traced source, category, inventory-slot, learned and skill/makeable filters.
  Applying it to the other modeled filters and profiles is a simulator default;
  their native notification behavior is not established.
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
