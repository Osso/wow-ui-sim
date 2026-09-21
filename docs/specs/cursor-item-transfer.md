# Cursor item transfer

`src/c_api/container_inventory.rs` owns the shared bag/cursor/equipment transfer operations exposed by `C_Container.PickupContainerItem` and the legacy pickup/equip globals. This bounded correction follows the [cached addon comparison](../forever-addon-comparison.md), not a full inventory redesign.

## What it must do

- [x] Execute EasyFishing's exact sequence: `C_Container.PickupContainerItem(0, 5)`, `PickupInventoryItem(16)`, then `C_Container.PickupContainerItem(0, 5)` only when `CursorHasItem()` is true. A fishing pole (6256) moves to main hand; the displaced weapon (19019) returns to the original bag slot; cursor clears; item IDs/counts are conserved.
- [x] Equip the held pole into an empty equipment slot without creating a displaced item.
- [x] Both namespaced and legacy container pickup entry points pick up occupied slots, exchange held items with occupied bag slots, and drop held stacks into empty slots without losing stack counts.
- [x] Empty-slot pickup with an empty cursor remains a no-op; existing equipment pickup and `EquipCursorItem` swap behavior remain available.

## How it works

- [Addon comparison investigation](../wiki/investigations/forever-addon-comparison.md).
- The existing equipment swap operation serves both `EquipCursorItem` and a cursor-holding `PickupInventoryItem`; the addon sequence does not acquire an extra API call.

## Implementation inventory

- `src/c_api/container_inventory.rs` — shared transfer operations and namespaced registration.
- `src/lua_api/globals/inventory_verbs.rs` — legacy registrations route to the same operations.
- `src/lua_api/workarounds/temporary/container_default_shapes.rs` — superseded pickup no-op removed; unrelated defaults unchanged.
- `src/c_api/mod.rs` — module declaration.

## Tests asserting this spec

`tests/inventory_verbs.rs`, in the existing grouped `integration` target:

- `fishing_cursor_transfer_preserves_both_items` — exact three-call sequence through both entry points.
- `fishing_cursor_transfer_equips_empty_destination` — no displaced item.
- `fishing_cursor_transfer_swaps_and_drops_bag_stacks` — occupied-slot exchange and empty-slot drop, seven-item stack conserved.
- `fishing_cursor_transfer_namespace_preserves_pickup_and_empty_slot_behavior` — namespace pickup and empty-slot behavior.
- Existing pickup and `equip_cursor_item_*` regressions preserve prior supported operations.

## Evidence

Cached EasyFishing `EasyFishing.lua`, `EquipBagPole`, from CurseForge project 1686925:

| Package | File ID | SHA-256 |
| --- | --- | --- |
| EasyFishing-1.6.0.zip | 8835476 | `2780b0ff58d551ecae721f0c8ccb32103ee48a82601c83f939abcbab733cacb3` |
| EasyFishing-1.7.6.zip | 8935539 | `1fa9c16cead0ca611d3e7348a13a409719def1aaec3def7733e29be51308ebcd` |

Local Forever 1.60.1.69913 authored evidence:

- `Blizzard_APIDocumentationGenerated/ContainerDocumentation.lua:419–429` declares namespaced pickup with bag and slot arguments.
- `Blizzard_UIPanels_Game/Camelot/PaperDollFrame.lua:2044–2067` passes cursor-held items to `PickupInventoryItem` for auto-equip.
- `Blizzard_UIPanels_Game/Camelot/BankFrame.lua:382` and Mainline `ContainerFrame.lua` use namespaced bag pickup.

These sources and focused tests establish the bounded transfer contract, not complete native inventory conformance or full EasyFishing compatibility.

Development proof: exact-transfer RED 0/4 at `0f236b638` plus tests; focused inventory-module GREEN 25/25 at `215a4080a`, including four new regressions and existing pickup/equip tests. No full-addon runtime proof is claimed.

## Known gaps (current cycle)

- [ ] Parent-owned final verification remains separate from targeted development RED/GREEN.

## Out of scope

- Existing `CursorInfo::Item` stores ID/count/origin, not item hyperlinks, enchants, or gems. Transfers do not preserve those unsupported metadata fields; this change does not expand `BagItem`, `EquippedItem`, or `CursorInfo`.
- Equipment retains the existing one-item, unvalidated slot model. Stack splitting/merging, item eligibility, bag capacity, bank state, security/combat restrictions, and event-lifecycle fidelity are not introduced.
- No `EquipItemByName`, addon fallback, vendor edit, automatic return on `ClearCursor`, or new inventory subsystem.
