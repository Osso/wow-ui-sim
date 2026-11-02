# Item creation context

## What it must do

- Retail `C_Item.GetItemCreationContext(itemInfo)` returns the resolved item ID and a context name as two values.
- For a positive-ID item link, the twelfth `item:` field supplies the context byte. The 256-entry WM37 table maps that byte to its name; index zero and 192–255 map to an empty string. A missing required argument raises an error.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:448` declares the input and the two results. WM37 native evidence for retail 12.1.5/70688 recovers the link field and all context names. The named publications are verified locally, but full protected-wrapper return behavior is unknown. Numeric IDs and ordinary names use empty context as a simulator policy because their native initial context is unknown.

## Known gaps

- Malformed intermediate link fields, exceptional numeric conversions, ordinary-name and numeric-ID initialization, and full protected cleanup remain unverified.
