# Item specialization membership

## What it must do

- Retail `C_Item.DoesItemContainSpec(itemInfo, classID [, specID])` returns one boolean. Class zero returns true; a missing player returns false for other classes; a missing supplied derived record returns true.
- Either of the two supplied permissive record flags returns true. Otherwise, a nonzero specialization matches by ID; a zero specialization matches a record member's class. An unmatched query returns the inverse of the supplied rejection flag.
- A missing required or invalid argument raises an error. The pinned documentation declares zero as the default for an omitted specialization (`ItemDocumentation.lua:100`).

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:92` declares the input and boolean return. WM37 native evidence for retail 12.1.5/70688 establishes local decision order and separate Boolean fields. Their descriptive names and the identification of the player prerequisite are inferred. Boolean publication was emulated, not observed on a live client. The simulator accepts the derived specialization record as supplied state.

## Known gaps

- The recovered native initial value when `specID` is omitted remains unknown, despite the documented default. Automatic record construction from item stats, allowable classes, ItemSpec and ItemSpecOverride is not yet modeled.
- Complete numeric coercion, derived-record selector behavior, load timing and full protected-wrapper publication remain unverified.
