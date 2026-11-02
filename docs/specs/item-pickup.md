# Item pickup by ID

## What it must do

- Retail `C_Item.PickupItem(itemInfo)` first clears the current cursor, then places an item-ID cursor (native mode 6) for a nonzero item. Item ID 0 only clears.
- Clearing a mode-6 cursor emits `ACTIONBAR_HIDEGRID`, then `CURSOR_CHANGED(true, 0, 6, oldItemID)`. Clearing an owned-item cursor (mode 1) emits `CURSOR_CHANGED(true, 0, 1, 0)`. An empty cursor emits nothing.
- A new pickup emits `ACTIONBAR_SHOWGRID`, then `CURSOR_CHANGED(false, 6, 0, 0)`; the fourth value is the stored ID after clearing, not the new ID.
- `GetCursorInfo()` returns `("item", itemID, link)` for a mode-6 cursor with item metadata, and zero values without metadata or with an empty cursor. `CursorHasItem()` is false for mode 6.
- A missing or unparseable item argument raises an error.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua` declares `PickupItem(itemInfo)`. WM44-A for retail 12.1.5/70688 verifies the clear, grid and event order, the event payloads, the mode-6 projection and the mode-1 `CursorHasItem` predicate. Original consumer: `Blizzard_RestrictedAddOnEnvironment/SecureHandlers.lua:143` through `SecureHandlerDragTemplate`. Item metadata means the generated item catalog that also builds item links.

## Known gaps

- Replacing any other cursor kind (spell, action, pet action, macro, talent, money, transmog outfit) clears it with the simulator's argument-less `CURSOR_CHANGED`; native mode numbers and stored IDs await WM46. `ClearCursor` and other cursor producers keep argument-less events and do not hide the grid.
- Owned-item replacement does not model lock-reason release or the old-item cleanup notification; the simulator drops the payload like `ClearCursor`. Placing a mode-6 cursor on an action bar is not modeled.
- PickupItem's own return count, listener delivery and server outcomes are unknown.
