# ServerSnapshot carried bags

The project-owned ServerSnapshot addon captures carried inventory into account SavedVariables. The simulator imports it through the same character selection as action bars; no external game-server transport is involved. See [snapshot loading](../wiki/systems/server-snapshot-action-bars.md).

## What it must do

- [ ] Capture backpack, equipped carried bags and reagent bag (retail bag IDs 0–5), including capacities, family masks, optional bag identity, occupied item IDs, stack counts and exact available hyperlinks.
- [ ] Refresh on `BAG_UPDATE_DELAYED`, `BAG_CONTAINER_UPDATE` and world entry. Missing modern APIs, failed reads or an unready backpack omit the entire `bags` domain; never encode those as empty inventory or silently reuse a previous capture.
- [ ] Import through the selected/latest character's real SavedVariables path, including snapshots with no action-bar data. Missing `bags` preserves existing inventory; a present domain authoritatively replaces carried containers and clears stale/empty slots and unequipped bags.
- [ ] Use captured capacities for container size/free-slot queries and inventory placement, including reagent slots beyond the former fixed 16-slot limit. Preserve exact captured item links.
- [ ] Validate the complete bag domain before mutations. Report invalid capacities, IDs/counts, out-of-range slots, incomplete container ranges and conflicting equipment mappings with field context. Preserve non-carried storage and non-bag equipment; notify listeners only after the complete state is installed.

## Payload

`snapshot.bags = { maxBagID = n, containers = { [bagID] = row } }`.

Each ID from zero through `maxBagID` is present. Rows contain `numSlots`, `family`, optional `name`, optional `inventorySlot`/`itemID`/`hyperlink` for the equipped container, and `items = { [slot] = { itemID, stackCount, hyperlink? } }`. Omitted item slots are empty. `numSlots = 0` represents an unequipped container. The backpack has no equipped inventory slot. Retail's cached `BagIndexConstantsDocumentation.lua` identifies carried IDs 0–5; higher positive IDs and negative IDs are not part of this capture.

## How it works

- [Snapshot addon usage](../addons/ServerSnapshot/README.md)
- [Snapshot startup import](../wiki/systems/server-snapshot-action-bars.md)

## Implementation inventory

- `docs/addons/ServerSnapshot/ServerSnapshot.lua` — modern container API capture and event refresh.
- `src/server_snapshot_import.rs`, `src/server_snapshot_import/bags.rs` — selection, complete validation, atomic bag-domain replacement and notifications.
- `src/c_api/bag_info.rs` — container metadata and seeded no-snapshot defaults.
- `src/c_api/item_spell/c_container.rs` — capacity/family/name/link queries.
- Inventory placement and merchant checks use the same modeled capacities.

## Tests asserting this spec

- `tests/server_snapshot_capture_bags.rs` — actual producer, all carried IDs, sparse occupied slots, unavailable capture and event triggers.
- `tests/server_snapshot_import.rs` — actual producer → serialized SavedVariables → production loader/importer → observable container queries; replacement, absence, malformed-domain preservation and event ordering.
- `tests/inventory_counts.rs`, `tests/merchant_junk_count.rs`, `tests/test_crafting.rs` — modeled capacity and placement controls.

## Known gaps (current cycle)

- [ ] Current grouped verification and fresh native-client capture pending. Existing installed 0.2.0 snapshots contain no inventory; updating code cannot reconstruct that missing data.
- Item display metadata continues to use the simulator's existing item catalog; this capture adds inventory identity/count/link state, not a new item database.

## Out of scope

Bank/account-bank capture, general equipment capture, external server networking, and modifying user SavedVariables. Installing the updated project-owned addon is separately authorized; a native reload/logout is still required to persist a fresh capture.
