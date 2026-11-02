# Item specialization membership

## What it must do

- Retail `C_Item.DoesItemContainSpec(itemInfo, classID [, specID])` returns one boolean. Class zero returns true; a missing player returns false for other classes.
- The query reads a derived record cached per item and player-level band (below 41, 41–109, 110 and above). A hit is reused within its band without rebuilding. On a miss, missing ItemSparse/Item data or player level zero builds nothing and the query returns true; no negative entry is cached.
- Either permissive record flag returns true. Otherwise, a nonzero specialization matches by ID; a zero specialization matches a member's class. An unmatched query returns the inverse of the restriction flag, which can be true with an empty list.
- A missing required or invalid argument raises an error. An omitted specialization defaults to zero (`ItemDocumentation.lua:100`).

### Derived record

- ItemSpecOverride rows whose specialization passes ItemSparse.AllowableClass replace rule matching; their record has both permissive flags false and the restriction flag true. The class test uses bit `ChrSpecialization.ClassID - 1`; a missing specialization tests bit 31.
- Otherwise the item class selects a category and an optional structural trait: weapons are category 5 with a subclass trait; gems are category 7; armor uses the WM42 follow-up subclass/inventory table; other classes are category 0.
- Traits fill one list of capacity 10, in order: structural trait, gem property bits 6–16 (category 7 only, traits 29–39), then the ten stat slots. Capacity is checked before the duplicate scan. Structural traits count only toward the total.
- One ItemSpec traversal of the category uses wildcard 40, inclusive level bounds against the player level, and two last-value suppression markers initialised to 8 and 40. Marker writes follow the append attempt, including after class rejection. A matching, unsuppressed rule sets the restriction flag even when out of range or class-rejected.
- `noTraitsAndNoSpecializations` is true when the trait list and specialization list are empty; `noStatTraits` is true when no stat or gem trait was appended.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:92` declares the input and boolean return. WM37, WM42 and the WM42 structural-trait follow-up for retail 12.1.5/70688 verify the decision order, record construction, cache bands, trait tables, fill order and marker rules. The flags' descriptive names and the selector's meaning as player level are inferred. Data comes from the 12.1.5.70391 Item, ItemSparse, GemProperties, ItemSpec, ItemSpecOverride and ChrSpecialization CSVs.

## Known gaps

- Rule and override traversal uses CSV row order; native order is unknown, so this order is inferred. The 70391 rows may differ from build 70688.
- The selector uses the simulator's player level, not a separately adjusted effective level. The native cache reset trigger is unknown; the simulator never resets the cache.
- Category-5 wildcard-primary rows with a wildcard secondary do not occur in 70391 data; their treatment is a best guess.
- The recovered native initial value of an omitted `specID`, load timing and full protected-wrapper publication remain unverified.
