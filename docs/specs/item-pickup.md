# Item pickup by ID

## What it must do

- Retail `C_Item.PickupItem(itemInfo)` first runs the shared cursor clear, then places the item ID on an ActionBar cursor (native mode 6; WM46 names it ActionBar, not Item, which is mode 1) for a nonzero item. Item ID 0 only clears.
- Clear, grid and `CURSOR_CHANGED` rules are the shared [cursor model](cursor-model.md). For this pickup: `ACTIONBAR_SHOWGRID`, then `CURSOR_CHANGED(false, 6, 0, S)`, where S is the stored ID after clearing, not the new ID.
- `GetCursorInfo()` returns `("item", itemID, link)` for a mode-6 cursor with item metadata, and zero values without metadata or with an empty cursor. `CursorHasItem()` is false for mode 6.
- A missing or unparseable item argument raises an error.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua` declares `PickupItem(itemInfo)`. WM44-A for retail 12.1.5/70688 verifies the clear, grid and event order, the event payloads, the mode-6 projection and the mode-1 `CursorHasItem` predicate. Original consumer: `Blizzard_RestrictedAddOnEnvironment/SecureHandlers.lua:143` through `SecureHandlerDragTemplate`. Item metadata means the generated item catalog that also builds item links.

## Known gaps

- Owned-item replacement does not model lock-reason release or the old-item cleanup notification. Placing a mode-6 cursor on an action bar is not modeled.
- PickupItem's own return count, listener delivery and server outcomes are unknown.
