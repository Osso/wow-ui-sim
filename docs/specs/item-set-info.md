# Item set info

## What it must do

- Retail `C_Item.GetItemSetInfo(setID)` returns the ItemSet row's name for a present set and exactly one nil for an absent set. A present empty name is an empty string.
- The set ID must be convertible to a signed 32-bit integer; numeric strings are accepted. A missing or unconvertible argument raises an error.
- Retail `C_Item.GetItemInfo` publishes the item's ItemSparse.ItemSet as `setID`, and nil when the item has no set or is unknown. Other profiles publish nil because no matching set data exists.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:917` declares the set-name query; `ItemDocumentation.lua:630` declares `setID` nilable. WM23 item-a and WM34-A for retail 12.1.5/70688 verify the direct ItemSet lookup, int32 numeric-string parsing and the absent-record nil (emulated). Original consumer: `Blizzard_FrameXML/BossBannerToast.lua:95`. Data comes from the 12.1.5.70391 ItemSparse and ItemSet CSVs.

## Known gaps

- Publishing nil for items without a set is inferred from the nilable declaration, not from native evidence. Locale variants and runtime data loading are not modeled.
