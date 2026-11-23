# Loot Journal item-set scenarios

`A_Admin.SetLootJournalItemSets` supplies the item-set catalog returned by the modeled `C_LootJournal` queries on Retail and WoWForever. These are simulator scenario inputs, not claims about the native catalog. Generated item-set data has names and item membership, but not the class/spec fields needed by the Loot Journal filter.

```lua
A_Admin.SetLootJournalItemSets({
    {
        setID = 501,
        name = "Example set",
        itemLevel = 620,
        classID = 2,
        specID = 70,
        items = {
            { itemID = 41001, icon = 91001, slot = 1, invType = 13 },
        },
    },
})
```

Each call replaces the full supplied catalog. Every set needs `setID`, `name`, `itemLevel`, and an `items` array. `classID` and `specID` are optional. Each item needs `itemID`, `icon`, `slot`, and `invType`.

`C_LootJournal.GetItemSets(classID, specID)` returns the supplied sets whose class and spec match the non-nil arguments. `C_LootJournal.GetItemSetItems(setID)` returns that set's items as `itemID`, `icon`, and `invType` records. An unknown set ID returns an empty array. The `slot` input is retained in scenario state but is not part of the native query result shape.
