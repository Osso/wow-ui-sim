# Container item search

`C_Container.SetItemSearch(searchString)` drives bag-search dimming in Blizzard container, bank and auction-house item buttons.

## Requirements

- [x] `SetItemSearch` stores the search text per environment and fires `INVENTORY_SEARCH_UPDATE` with no arguments.
- [x] `C_Container.GetContainerItemInfo(...).isFiltered` is `false` while the search text is empty.
- [x] With a search text, `isFiltered` is `false` only when the item name contains the text, compared case-insensitively.
- [x] Items without a known name are filtered by any non-empty search.
- [x] A non-string argument raises a Lua error.

## Best-guess basis

best-guess, native UNKNOWN for match rules. In game, bag search dims items whose name does not match the typed text. Native matching also considers tooltip text (item type, stats); this simulator matches names only.

## Known gaps

- Tooltip-text, item-type and stat matching are not modeled.
- Search text does not persist across UI reloads.

## Tests

- `bags::test_item_search_marks_container_buttons_by_item_name` (original `ContainerFrame` consumer).
