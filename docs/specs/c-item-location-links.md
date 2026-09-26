# Shared C_Item location queries and links

`C_Item.GetItemID`, `DoesItemExist`, and `GetItemLink` query modeled bag and equipment locations. `GetItemLink` preserves a captured bag hyperlink where present; otherwise it uses catalog metadata when available.

## What it must do

- [x] Resolve an occupied `{ equipmentSlotIndex }` to its modeled item ID and presence, and an empty equipment location to nil ID and false presence.
- [x] Return the canonical catalog link for occupied equipment when metadata exists, or a captured bag-equipment hyperlink from modeled `bag_info`; return nil for empty equipment.
- [x] Resolve equipped bag IDs/presence from modeled `bag_info` inventory slots even without an `EquippedItem`, retaining their captured links.
- [x] Preserve a captured `BagItem.hyperlink` byte-for-byte, matching `C_Container.GetContainerItemLink` for that bag slot.
- [x] Resolve occupied bag IDs/presence independently of metadata, and return nil for empty locations and uncataloged items without captured links.
- [x] Keep item-info accepting C_Item queries available for numeric item IDs and item links; do not infer inventory quantity from item-info input.

## How it works

- [C API architecture](../lua-api.md).

## Implementation inventory

- `src/c_api/item_spell/c_item.rs` — location item resolution, C_Item presence, ID, and link queries.
- `src/c_api/item_spell/c_container.rs` — container link query honoring captured bag hyperlinks.
- `src/lua_api/state_types/collections.rs` — bag/equipment item state.
- `src/c_api/bag_info.rs` — equipped bag inventory slot and captured link metadata.

## Tests asserting this spec

- `tests/c_item_api/c_item.rs` — grouped pure-model equipment, bag, and empty location cases; existing item-info cases.

## Known gaps (current cycle)

- [x] Focused RED→GREEN: four location/link cases and 44 C_Item cases pass under the default grouped integration target; logs `/tmp/cross-version-item-location-proof.md`.
- [ ] Native client verification remains unavailable for this slice.

## Out of scope

- GUID synthesis: bag and equipped items have no stable item GUID field; `GetItemGUID` remains separate.
- Metadata-cache redesign: `IsItemDataCached` currently treats modeled location presence as cached data and positive item IDs as synthetically cached, not as a real item-cache lifecycle.
- Numeric aggregate item counts and native-client semantics.
