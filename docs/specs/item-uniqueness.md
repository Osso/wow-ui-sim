# Item uniqueness

## What it must do

- Retail `C_Item.GetItemUniqueness(itemInfo)` consults the item’s ItemSparse uniqueness flag and limit category.
- A present item with flag `524288` selects the inferred success result. Otherwise, a nonzero category and an existing ItemLimitCategory record are required. A missing item, zero category or missing category record returns zero values.
- The inferred success result is two numeric zero values. Required invalid arguments raise an error.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:1016` declares two numeric returns and `MayReturnNothing`. WM36 and its follow-up for retail 12.1.5/70688 establish the lookup decisions and caller initialization. The normal-success `(0, 0)` pair is inferred; fragment publication is emulated, not observed on a live client. ItemSparse and ItemLimitCategory data come from the 12.1.5.70391 CSV exports.

## Known gaps

- The success pair is inferred from bounded native control flow and emulated fragments, not full protected-client execution. Dynamic item modifiers, player context and load completion remain unverified.
