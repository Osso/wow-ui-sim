# C_Item.GetItemNumSockets

Status: implemented simulator policy, not native parity.
Authority: pinned declaration plus the client-handler decomp answer; the bonus-socket source is unmodelled.

## What it must do

- [ ] `GetItemNumSockets(itemInfo)` publishes one non-nil number for any argument.
- [ ] The argument resolves to an item like the other `itemInfo` readers: item ID, hyperlink, or item name.
- [ ] An item that does not resolve, or is not cached, publishes `0`. It never publishes nil and never returns nothing.
- [ ] Publication is Retail, PTR and WowForever only.

## Count sources

The client handler adds two sources (decomp answer, PTR 12.1.7: static count plus a resolving bonus key, no cap):

- [ ] Static sockets: the non-zero socket slots of the item's own record.
- [ ] Bonus and instance sockets: when the item info carries a bonus key whose lookup resolves, the matching entry's socket count is added to the static count.

The simulator models only the static source. Each item record carries three socket slots from `ItemSparse.SocketType`, and the simulator holds no bonus-list or instance-socket table, so the second addend is always absent. Giving an item link with bonus IDs therefore reports the record count, not a larger instance count.

## Declarations

`ItemDocumentation.lua`: Retail `854-867`, PTR `876-889`, WowForever `876-889` declare `itemInfo: ItemInfo` in and `socketCount: number` (`Nilable = false`) out. Mists, Era and Anniversary declare the same function (`586-598` in the Mists and Era caches) but are not published here.

## Implementation inventory

- `src/c_api/item_spell/c_item.rs` — `c_item_get_item_num_sockets`.

## Tests asserting this spec

`tests/c_item_probes.rs::get_item_num_sockets_counts_the_record_sockets`: two record sockets by item ID, one by hyperlink and by name, plus zero for an item without sockets and for an unknown ID.

## Known gaps

- [ ] The bonus/instance addend has no backing table, so a link that would add instance sockets reports the record count.
- [ ] Mists, Era and Anniversary declare the function and do not publish it.
