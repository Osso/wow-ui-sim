# Item family

## What it must do

- Retail `C_Item.GetItemFamily(itemInfo)` resolves the shared item identifier and reads the family value of a supplied cached item record.
- A present zero or nonzero family returns one number. An absent record returns one nil. Missing or invalid required arguments raise an error.
- Changes to supplied cached records are visible to later calls.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:464` declares the ItemInfo argument and nullable numeric result. WM23 item-a native evidence for retail 12.1.5/70688 identifies a cache read and distinguishes zero from a missing record. Its nil/number publication was emulated, not observed on a live client. The simulator accepts family values as host-supplied cached-record state; an absent input represents an unavailable record, not a native initial-cache claim.

## Known gaps

- The exact DB2 field identity is awaiting native evidence. The model does not populate this state automatically from generated ItemSparse data.
- Cache loading, remote requests, completion events and whole-wrapper live-client behavior are not modeled.
