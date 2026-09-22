# Merchant buyback reads

Forever exposes three legacy buyback queries from `src/lua_api/globals/real/merchant_buyback.rs`, backed by a separate per-environment collection. See [Lua API architecture](../wiki/systems/lua-api.md).

## What it must do

- [x] `GetNumBuybackItems()` returns exactly one number equal to the buyback collection length, independently of merchant stock, open state, and repair capability. The initial empty collection is an explicit simulator scenario, not a native account-state default.
- [x] `GetBuybackItemInfo(index)` returns name, icon, total copper price, stack quantity, availability, usability, and binding for a populated one-based slot. Name/icon use existing modeled item metadata; remaining fields come from the configured stack snapshot.
- [x] `GetBuybackItemLink(index)` returns the existing item-link formatter's link for the same slot. No invented item name, icon, or link fallback is permitted; a configured unknown item raises a descriptive error.
- [x] Zero, negative, and out-of-range slots return no values from either indexed read (thus nil in a single-value expression). Required numeric indices use existing integer argument conversion. Missing/malformed arguments fail without changing state.
- [x] Preserve configured snapshots across bootstrap restoration and merchant closing; environments and merchant stock remain independent. Reads do not buy, sell, reorder, or remove stacks.
- [x] The existing actual cached startup plus `BAG_UPDATE` regression passes with no Lua errors and hidden repair buttons for the closed merchant scenario.

### Evidence and inferred boundaries

Local wowless `data/products/wow/apis.yaml` and `wow_classic_era/apis.yaml` declare a numeric index, the first six info results, a string link result, and numeric count; they supply no implementation or invalid-index semantics. Cached Forever `Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua:416–423` unconditionally calls info at the count (including zero), reads a seventh binding result, and requests a link only when name exists. These establish the consumer shape, not native probe conformance.

One-based collection order, zero-result absence, persistence across merchant closing, total-stack copper price, and the supplied availability/usability/binding policy are explicitly simulator inferences. No price calculation, stock-availability rule, player-equipment usability calculation, or binding derivation is claimed. The concrete tests configure Copper Bar (2840) and Copper Chain Pants (2852) with their existing modeled names/icons and explicit transaction fields.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Merchant repair capability](merchant-repair-capability.md)

## Implementation inventory

- `src/lua_api/globals/real/merchant_buyback.rs`: buyback snapshot fields and three modeled legacy reads.
- `src/lua_api/globals/real/mod.rs`, `src/lua_api/globals/register.rs`: Forever-only publication.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: independent collection and empty scenario initialization.
- `tests/merchant_buyback_reads.rs`: tests within the existing grouped integration target.

## Tests asserting this spec

- `tests/merchant_buyback_reads.rs`: empty/bootstrap, concrete seven-field tuples and links, slot ordering/removal, independent stock/capability/environments, malformed input and unknown metadata.
- `tests/merchant_repair_capability.rs::merchant_repair_cached_bag_event`: unchanged cached Blizzard event regression; previously failed at missing buyback count after passing repair-capability dispatch.

## Known gaps (current cycle)

- [x] Four focused buyback tests, the adjacent repair/event test, and the junk-count test total 11/11 under hash-matched reuse. Fresh formatting/default offline checking and changed-Rust readability also pass: `/tmp/forever-addon-audit/verify-merchant-read-family-ledger.json`.
- [x] The exact generic-TOC BagMeter archive replays its bounded count workflow with empty Lua-error JSON after the adjacent junk read is published.
- [ ] The `_Forever`/Classic BagMeter variant, buyback transactions, and native buyback semantics remain unproven.

## Out of scope

Buyback transactions, sale mutation, repair actions/cost/durability, guild repair, new `C_*` APIs, other-profile publication, native client conformance, and full merchant/addon acceptance.
