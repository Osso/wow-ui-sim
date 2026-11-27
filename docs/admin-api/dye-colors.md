# Dye colors

### `A_Admin.SetDyeColorsForItem(itemID, dyeColorIDs)`

Supplies the dye-color IDs returned for an item by the modeled `C_DyeColor` queries. This is host input for the Retail and WoW Forever item-dye model; it is not a claim that the simulator has the live item-dye catalog.

- `itemID` must be an unsigned integer.
- `dyeColorIDs` must be a dense array of positive integer IDs. An empty array removes the item's dye-color list from query results.
- Omitted items have an empty list.
- The original `Blizzard_Deprecated` item and item-location compatibility wrappers read these lists.

Example:

```lua
A_Admin.SetDyeColorsForItem(900001, {501, 502})
A_Admin.SetDyeColorsForItem(900002, {601})
```

INFERRED (R88g9, native evidence pending): the supplied ID order is preserved because the original compatibility wrapper selects the first ID.
