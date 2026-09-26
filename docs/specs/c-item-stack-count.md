# C_Item.GetStackCount location counts

`C_Item.GetStackCount(itemLocation)` reads modeled quantity at one bag or equipment location. It does not query item metadata, maximum stack size, or aggregate inventory quantity; `C_Item.GetItemCount` handles aggregate counts.

## What it must do

- [x] Accept an ItemLocation table with `bagID` and `slotIndex`, returning that modeled bag stack count or zero for an empty slot.
- [x] Accept an ItemLocation table with `equipmentSlotIndex`, returning one for occupied equipment and zero for an empty slot, regardless of item metadata.
- [x] Reject non-location number, item-link string, and nil arguments with an explicit contextual error, without claiming native error wording.

## How it works

- [C API architecture](../lua-api.md).

## Implementation inventory

- `src/c_api/item_spell/c_item.rs` — location-only stack-count query against existing bag and equipped-item state.

## Tests asserting this spec

- `tests/c_item_api/c_item.rs` — modeled bag/equipment, empty locations, and non-location rejection in the grouped `integration` target.

## Known gaps (current cycle)

- [ ] Native-client semantics and broader profile acceptance remain unverified.

## Out of scope

- Item-ID/link count fallback, aggregate quantity, maximum stack size, new inventory model, and invented ItemLocation field coercion.
