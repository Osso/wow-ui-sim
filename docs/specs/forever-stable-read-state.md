# Forever stable read state

The Forever stable reads use simulator-supplied state. Native basis: Forever client build 70707, F3 stable API evidence. The pinned `Blizzard_APIDocumentationGenerated/StableInfoDocumentation.lua` declares `GetNextStableSlotCost` at line 37, `GetNumStablePets` at 55, `GetNumStableSlots` at 64, and `GetStablePetInfo` at 88.

## Contract

- Each getter returns exactly one value. Slot count is the supplied raw byte. Pet count includes only records with a nonzero native record ID and flags bit 2 set.
- Slot cost reads the supplied table at slot count plus one; a missing row yields zero. The supplied cost table and its empty default are **best-guess, native UNKNOWN**: no pinned cost data has been established. No cost rows are invented.
- A missing or out-of-range pet returns nil. A bad index raises `Usage: local petInfo = C_StableInfo.GetStablePetInfo(index)`. Supplied pet records produce snapshots of documented fields.
- The empty scenario starts with two slots as a **best-guess, native UNKNOWN** simulator default. It does not assert the native initial server state.
- Other profiles retain their existing behavior. `IsAtPetStable` remains independent.

## Consumer and proof

The Forever StableUI TOC permits Camelot and selects `[Game]/Blizzard_StableUI.lua`; `[Game]` resolves to Camelot. `Camelot/Blizzard_StableUI.lua:383-385` reads slots and cost in the purchase button update; lines 351-364 select populated or empty slot presentation based on nil. `tests/wowforever_stable_reads.rs` covers state and these original consumers.

## Known gaps

- F3 says the empty path returns a table, but this conflicts with the documented `Nilable = true` and the shipped consumer. Documentation plus consumer govern here; the native surface for the table claim is unresolved.
- Native index coercion beyond the existing integer converter is unknown.
- The slot-cost DB2 identity, layout, and pinned data are not established. Cost rows are supplied state, not a generated native catalog.
- `PET_STABLE_UPDATE` is a real dispatched event in build 70707, but its relationship to the slot and record writers is unknown. No refresh-event behavior is inferred.
- Purchase, swapping, persistence, and server updates remain out of scope.

## See also

- [Stable pet slots](../wiki/systems/stable-pet-info.md)
- [Forever spell data pattern](forever-spell-data.md)
