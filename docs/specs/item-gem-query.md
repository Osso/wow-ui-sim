# Item gem query

## What it must do

- Retail `C_Item.GetItemGem(item, index)` reads one of three gem IDs from an item link or a supplied owned item. A cached gem returns its name and a generated gem link; an empty socket, invalid socket number, or uncached gem returns zero values.
- The link uses the selected gem's ID, quality and socket bonus IDs. It does not inherit the parent item's context, main bonuses, modifiers or display label.
- Negative or out-of-range numeric indices raise an argument error. Numeric input in the item argument is searched as a name, not interpreted as an item ID.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua` declares the query. WM23 and WM28 native evidence for retail 12.1.5/70688 records the parsing and link fields; return publication is emulated, not observed on a live client. The player level and active specialization ID in the link are inferred from the builder evidence. The generated gem catalog is the locally available cache, not a model of asynchronous cache loading.

## Known gaps

- Derived item names, quality overrides, crafting-quality labels, item-name descriptions and their bonus-dependent transformations are not modeled.
- Owned-item search order and eligibility, non-gem ItemSparse cache coverage, asynchronous requests and completion events are not established. The supplied equipment and bag records are the simulator's available owned-item state.
- Complete parser behavior on malformed or exceptional numeric inputs and native locale-specific name comparison remain unverified.
