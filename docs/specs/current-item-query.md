# Current item query

## What it must do

- [ ] Retail `C_Item.IsCurrentItem(itemInfo)` returns one boolean. ID zero returns false; another ID matches when it is in the host-supplied resolved current-item list.
- [ ] Item identity uses [shared ItemInfo conversion](item-info-conversion.md) and VM secret-access restrictions.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:1241` declares the ItemInfo input and boolean output. The WM23 item-b build 70688 handoff identifies a zero-ID false decision and an ID equality search over resolved objects; publication is conditionally emulated, not observed in a live client. The simulator represents the resolved objects by host-supplied item IDs. An initially empty list is a simulator scenario, not a native default.

## Implementation inventory

- `src/c_api/item_spell/c_item.rs`: Retail query registration and membership decision.
- `SimState.current_item_ids`: supplied resolved item IDs.

## Tests asserting this spec

- `tests/current_item_query.rs`: API-level identity and membership changes. No selected Retail Blizzard consumer was found.

## Known gaps

- [ ] Native list population, object eligibility and lifetimes, full argument errors, and live return behavior remain unknown. No automatic current-item transitions are modeled.
