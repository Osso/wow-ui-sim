# Equipped item type

## What it must do

- Retail `C_Item.IsEquippedItemType(type)` searches supplied equipment slots 0–18 and returns one boolean. The required argument accepts strings and numbers; nil, missing, and other values raise an argument error.
- Compare the 35 inventory tokens in native order, first against the literal and then against its localized global. A positive matching token requires the item's inventory type to equal its index. An index-zero match or no token match instead compares the item class name, both subclass names and applicable subclass-mask names.
- Compare complete strings without case sensitivity. Item classification and English category names come from the generated Item, ItemClass, ItemSubClass and ItemSubClassMask catalog.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:1317` declares a required string and one boolean. WM23 and WM29 native evidence for retail 12.1.5/70688 establishes the matching order and local selection; boolean publication is emulated, not observed on a live client. The simulator's equipment records provide the candidate state. Rust lowercase comparison is an approximation of the native folded-text comparator.

## Known gaps

- Internal inventory slots 19–29 and recursive child-container searches are not modeled. Native item eligibility and the meanings of those slots are not fully established.
- The generated item classification catalog covers the simulator's selected item IDs, not every possible cached item. Category names are from the enUS DB2 export; non-English DB2 locale data is not modeled.
- Exact Unicode folding, native inventory transitions, item eligibility and whole-wrapper runtime behavior remain unverified.
