# C_Item.GetItemNumSockets

Status: implemented, matching the native count contract for the builds we hold.
Authority: pinned declarations plus the Round 2 handler decomp
(`/home/osso/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`, Q2),
proved in Retail 12.1.5.**70688**, PTR 12.1.7.**70668** and Forever 1.60.1.**70707**.

## What it must do

- [x] `GetItemNumSockets(itemInfo)` publishes one number for any argument.
- [x] The argument resolves to an item like the other `itemInfo` readers: item ID, hyperlink, or item name.
- [x] An item that does not resolve, or is not cached, publishes the integer `0`. It never publishes nil and never returns nothing (Q2: the carried value stays 0 and is published as an integer).
- [x] Publication is every profile with a pinned declaration: Retail, PTR, Forever, Mists, Era and Anniversary. Wrath pins no generated documentation.

## Count sources

The handler sums two sources in one 32-bit add, with **no clamp** (Q2.e):

1. **Fixed sockets — modelled.** The number of non-zero entries in the item record's three
   fixed socket slots (`SocketType_0..2`, maximum 3). This is the whole effective total in the
   builds we hold.
2. **Bonus sockets — code-live, data-dead.** The first of three `SpellItemEnchantment`
   selector words equal to **8** contributes its positionally paired effect-points dword
   (Q2.a, Q2.b, Q2.c). The key that selects those entries is a dword inside `itemInfo` that
   **nothing writes** in 70688, 70668 or 70707 — the link parser, the GUID/instance converter
   and the default-constructed per-call `itemInfo` all leave it zero (Q2.d). The guard
   therefore always takes the skip edge, so the addend is 0.

Consequences the simulator implements:

- [x] A record reports at most 3 sockets, because only the first three fixed slots count.
- [x] The record type holds exactly those three slots (`public socket_types: [u8; 3]`,
      `crates/sim-data/data/items.rs:22`, generated in `src/bin/wow_cli/gen_items.rs:1451`), so a
      wider record is unrepresentable rather than counted and then clamped.
- [x] A link that carries bonus-id fields still reports the record count: the bonus half adds 0.
- [x] No bonus path is implemented. It retires when a producer for the `itemInfo` key exists, and
      then the addend must be added without a clamp (Q2.e).

Not proved in the evidence: the selector's intended producer (weakest link is the key writer,
inferred from the queried table), and the behaviour for builds 70493 and 70575, which are not held.

## Declarations

`ItemDocumentation.lua`: Retail `854-867`, PTR `876-889`, WowForever `876-889`, Mists/Era/Anniversary
`586-598`, all declaring `itemInfo: ItemInfo` in and `socketCount: number`
(`Nilable = false`) out. Retail, PTR and WowForever also declare
`SecretArguments = "AllowedWhenUntainted"`; Mists, Era and Anniversary do not.

## Implementation inventory

- `src/c_api/item_spell/c_item.rs` — `c_item_get_item_num_sockets`, with
  `RECORD_SOCKET_SLOTS` bounding the count to the three fixed slots.

## Tests asserting this spec

`tests/c_item_probes.rs` (module of the `integration` target):

- `get_item_num_sockets_counts_the_record_sockets` — two record sockets by item ID, one by
  hyperlink and by name, plus zero for an item without sockets and for an unknown ID.
- `get_item_num_sockets_caps_at_the_three_fixed_slots` — the value is a number and stays inside
  the three record slots.
- `get_item_num_sockets_ignores_the_unwritten_bonus_fields` — a link with bonus-id fields reports
  the same record count as the plain link.
- `get_item_num_sockets_publishes_a_number_for_an_unknown_item` — `type(count) == "number"` and
  `count == 0` for an uncached ID.

## Known gaps

- [ ] The `SecretArguments = "AllowedWhenUntainted"` argument policy is not enforced on Retail, PTR
      and WowForever; the reader takes the argument value directly, like the other readers in this
      file did before their own secret unwrapping landed. Not in this lane's scope.
