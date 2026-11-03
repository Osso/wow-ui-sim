# CursorCanGoInSlot (Retail)

`C_PaperDollInfo.CanCursorCanGoInSlot(slotIndex)` is published on Retail only (PaperDollInfoDocumentation.lua:20). It returns one Boolean. The result is a local compatibility decision, not equip eligibility: item locks, combat and input permission do not affect it. VERIFIED (WM65, WM67).

## Argument

The argument must be a number from 0 through 4294967295. Any other value raises a Lua error. The native slot is the argument minus one, then truncated. Public 1–19 select native 0–18; public 0 wraps to the unsigned all-ones slot. Numeric-string acceptance is UNKNOWN; the simulator rejects strings. VERIFIED (WM65).

## Cursor

Only an owned-item cursor (native mode 1) can pass. An empty cursor, every other mode, a missing player and an item without resident Item metadata return false. VERIFIED (WM65).

## Gates

All of these must hold. VERIFIED (WM67) unless labeled otherwise.

- Native slots 17, 86–87, 90–91, 93–95 and 97–99 are rejected.
- An ItemChildEquipment row of the item whose slot field equals the native slot rejects that slot. The child item need not exist.
- Native slots 35 and above pass the inventory-type check. Below 35, the InventoryType mask for types 0–30 must contain the native slot. Native 34 accepts only a type-18, subclass-11 bag, and that bag accepts only native 34.
- InventoryType 17 at native 16 also requires Item class 2, a subclass whose low five bits are 1, 5, 6, 8 or 10, and the off-hand enable marker. The marker's upstream meaning is UNKNOWN; its scenario default is clear. It is not a main-hand occupancy check.
- An equipped bag that holds items is rejected outside native slots 30–34, 63–68 and 100–104. Contents are modeled only for equipped bags.
- Item class 19 also requires the native slot to be in its profession list. Subclasses 0–13 select Profession rows 1–14, whose field 2 is the key; other subclasses use key 15. Key 5 allows native 25–26 and key 10 allows 27–29. For other keys, the Profession row with that key gives a skill line (field 1). If that skill line is the first primary profession, native 19–21 are allowed; if it is the second, native 22–24 are allowed. Otherwise no slot is allowed. INFERRED: field names, and the player fields being the learned primaries.

InventoryType values above 30 have no recovered mask. Their best-guess behavior is no slot below native 35 (UNKNOWN).
