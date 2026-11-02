# Applied item transmog

`C_Item.GetAppliedItemTransmogInfo` exposes supplied appearance modifiers for an individual equipped or bag item. Retail declaration: `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:240`; PTR declaration: the same file at line 262. The Blizzard consumer is `Blizzard_TransmogShared/Blizzard_TransmogShared.lua:239`.

## What it must do

- [ ] For a present modeled item, publish appearance, secondary appearance and illusion IDs with `ItemTransmogInfoMixin` methods. Absent supplied modifiers default to zero; this is inferred from the local 70688 evidence.
- [ ] Preserve supplied modifiers across equipment, cursor, and bag moves, including swaps and stack splits.
- [ ] Reject invalid required item-location arguments.

## How it works

- Inventory movement: `src/c_api/container_inventory.rs`.

## Implementation inventory

- `src/c_api/applied_item_transmog.rs`: query and result publication.
- `src/lua_api/state_types/collections.rs`: per-item supplied state.
- `src/lua_api/state_types/runtime.rs`: cursor payload.

## Tests asserting this spec

- `tests/startup_collection_regressions.rs`: Blizzard consumer, distinct supplied items and cursor swap.

## Known gaps (current cycle)

- [ ] Empty-slot publication and the native record-kind rejection need end-to-end native evidence. The 70688 and corrected 12.1.7 evidence do not establish an unconditional empty-slot result.

## Out of scope

- The unknown native flagged-modifier selection and event lifecycle.
