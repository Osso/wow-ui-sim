# Forever stable read state

The Forever stable reads use simulator-supplied state. Native basis: Forever client build 70707, F3 stable API evidence. The pinned `Blizzard_APIDocumentationGenerated/StableInfoDocumentation.lua` declares `GetNextStableSlotCost` at line 37, `GetNumStablePets` at 55, `GetNumStableSlots` at 64, and `GetStablePetInfo` at 88.

## Contract

- Each getter returns exactly one value. Slot count is the supplied raw byte. Pet count includes only records with a nonzero native record ID and flags bit 2 set.
- Slot cost reads the supplied table at slot count plus one; a missing row yields zero. The supplied cost table and its empty default are **best-guess, native UNKNOWN**: no pinned cost data has been established. No cost rows are invented.
- A missing or out-of-range pet returns a fresh empty table. A bad index raises `Usage: local petInfo = C_StableInfo.GetStablePetInfo(index)`. Supplied pet records produce snapshots of documented fields.
- The empty scenario starts with two slots as a **best-guess, native UNKNOWN** simulator default. It does not assert the native initial server state.
- Other profiles retain their existing behavior. `IsAtPetStable` remains independent.

## Consumer and proof

The Forever StableUI TOC permits Camelot and selects `[Game]/Blizzard_StableUI.lua`; `[Game]` resolves to Camelot. `Camelot/Blizzard_StableUI.lua:383-385` reads slots and cost in the purchase button update. `tests/wowforever_stable_reads.rs` covers state and this original consumer. The empty-table RED is a public API preservation boundary, not a consumer RED: its assertion runs before the consumer.

## Known gaps

- Native empty-result table fields and zero values are not fully mapped; the simulator currently returns an empty table.
- Native index coercion beyond the existing integer converter is unknown.
- The slot-cost DB2 identity, layout, and pinned data are not established. Cost rows are supplied state, not a generated native catalog.
- `PET_STABLE_UPDATE` is a real dispatched event in build 70707, but its relationship to the slot and record writers is unknown. No refresh-event behavior is inferred.
- Full stable-window behavior with empty-result tables is not established. Purchase, swapping, persistence, and server updates remain out of scope.

## See also

- [Stable pet slots](../wiki/systems/stable-pet-info.md)
- [Forever spell data pattern](forever-spell-data.md)
