# Item child equipment

## What it must do

- Retail `C_Item.GetItemChildInfo(itemInfo [, slotID])` resolves the item and returns one fresh numeric array. Each supplied child-equipment row contributes its two stored values in order.
- An omitted or nil slot selects all rows. A supplied one-based slot selects rows whose stored zero-based selector equals `slotID - 1`. An item with no rows returns an empty array.
- Subsequent calls read current supplied rows, and invalid required arguments raise an error.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:366` declares ItemInfo, optional luaIndex and a numeric table. WM23 item-a native evidence for retail 12.1.5/70688 establishes flat pairs and the slot adjustment; empty and two-number array publications are emulated, not observed on a live client. Supplied rows are simulator input, not an inferred default catalog.

## Known gaps

- Independent names and meanings of the two serialized fields and complete native iterator selection are not established. No ItemChildEquipment DB2 catalog is loaded automatically.
- Exact numeric conversion, exceptional errors, loading behavior and live-client publication remain unverified.
