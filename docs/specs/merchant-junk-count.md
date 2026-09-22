# Forever merchant junk count

`C_MerchantFrame.GetNumJunkItems` reads existing bag inventory and item metadata in `src/c_api/c_merchant_frame/junk.rs`. It supplies the numeric comparison required by cached `MerchantFrame_Update` during `BAG_UPDATE`; see [merchant investigation](../wiki/investigations/forever-addon-comparison.md).

## What it must do

- [ ] Return exactly one numeric count on Forever, including zero for empty or non-junk inventory; retain registration after bootstrap cleanup.
- [ ] Sum positive stack quantities only for known quality-0 items with positive `sell_price`, carried bags 0–4, and valid slots according to existing container capacities.
- [ ] Exclude banks, unmodeled bags, invalid slots, nonpositive quantities, and bags marked with existing junk-exclusion flag 64.
- [ ] Read current inventory without modifying it or sharing per-environment state; unknown metadata does not establish known-junk eligibility.
- [ ] Preserve other profiles and pass the existing cached Blizzard merchant `BAG_UPDATE` regression.

Eligibility and stack-unit counting are explicit simulator inferences, not native-verified rules. The required non-nil number is supported by cached `MerchantFrame.lua:210`; no broader sell-all behavior is implied.

## How it works

- [C API boundary and inventory investigation](../wiki/investigations/forever-addon-comparison.md)

## Implementation inventory

- `src/c_api/c_merchant_frame.rs` — Forever-only registration.
- `src/c_api/c_merchant_frame/junk.rs` — pure per-stack eligibility/count and inventory read.
- `src/c_api/item_spell/c_container.rs`, `mod.rs` — reuse existing capacity and exclusion flag helpers.

## Tests asserting this spec

- `tests/merchant_junk_count.rs` — concrete test-only ItemInfo fixtures for positive stacked junk, zero-price/nonpoor/unknown metadata, excluded bags, bank/slot boundaries; real Lua API against existing catalog, flags, state isolation and bootstrap.
- `tests/merchant_repair_capability.rs` — unchanged full cached UI startup and actual `BAG_UPDATE` dispatch.
- `tests/merchant_buyback_reads.rs` — existing adjacent merchant read regressions.

## Known gaps (current cycle)

- [ ] Positive junk through the live Lua/catalog path is unproven: generated item data and profession overrides currently contain no quality-0 entries. Positive behavior is covered by concrete metadata unit fixtures, not production catalog additions or metadata overrides.
- [ ] Native eligibility/count-unit conformance and incomplete-catalog classifications remain unverified.
- [ ] Targeted GREEN build and existing merchant event regression pending.

## Out of scope

- `IsSellAllJunkEnabled`, sale transactions, buyback mutation, repair costs/durability and guild repairs.
- Production item-data changes, general override infrastructure, addon/vendor changes, other-profile API changes, and full addon acceptance.
