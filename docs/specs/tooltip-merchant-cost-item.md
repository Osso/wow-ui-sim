# C_TooltipInfo.GetMerchantCostItem

`C_TooltipInfo.GetMerchantCostItem(slot, costIndex)` returns the tooltip data for
one entry of a merchant offer's extended-cost record.

Evidence: decomp Round 6 §2 ("the loot and merchant-cost tooltip getters"),
Retail 12.1.5 build 70688. Labels follow that answer: **VERIFIED** is proved from
the client image, **INFERRED** follows from verified facts or from the shipped UI
Lua whose build is not verified, **UNKNOWN** is not established.

## Surface

- Declared in Retail and PTR `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:586`
  with `slot` and `costIndex` (`luaIndex`), `MayReturnNothing = true`,
  `SecretArguments = "AllowedWhenUntainted"` and a non-nilable `TooltipData`
  return (VERIFIED for the pinned Retail/PTR documentation).
- Forever's documentation declares the member too, but Forever's extended-cost
  selection block is undecoded, so this getter is **not** registered on Forever.
- The equivalent of `GameTooltip:SetMerchantCostItem` is this getter:
  `TooltipDataHandler.lua:581` maps the accessor to `GetMerchantCostItem`, and the
  tooltip reader has exactly one caller in the image, the getter callback
  (VERIFIED for the single caller; INFERRED for the UI mapping). There is no
  separate C-side line producer that could drift.

## Arguments

- Both arguments are read by the shared `luaIndex` reader: each must be a number
  in the closed range 0 to 4294967295 and is converted to a zero-based value by
  subtracting one, then truncating toward zero (VERIFIED for the Forever reader;
  INFERRED for 12.1.5 from the same reader family and reason strings).
- The index counter advances between the two reads, so the second read addresses
  argument 2 whether or not the first succeeded (VERIFIED).
- Usage text: `Usage: local data = C_TooltipInfo.GetMerchantCostItem(slot, costIndex)`
  (VERIFIED, identical in both images).
- Wrong type, an out-of-domain number, or a missing argument is an argument
  error built from the reason `outside of expected range 0 to 4294967295`, a
  value description, ` - ` and the usage string, with the three secret-policy
  suffixes for tainted values (VERIFIED for the reason strings and their
  composition; the appended value description and the final Lua wrapping are
  UNKNOWN).
- There is no per-argument range check against the offer (VERIFIED).

## Selection

- The record is scanned for up to five item-cost entries and then up to five
  currency-cost entries (VERIFIED, complete decode of the selection block).
- An entry leaves the selectable sequence when its identity is zero, when an
  item's sparse-item lookup fails, when an item row is missing, or when an item
  row's numeric class field equals 10 — the money cost class (VERIFIED for the
  skips; INFERRED that class 10 is the money cost).
- The surviving entries form the ordinals: all surviving item costs first, in
  record order, then all currency costs, in record order. `costIndex` is matched
  against that ordinal sequence, not against the record position (VERIFIED).
- Consequence: a purely monetary vendor cost has no ordinal at all and cannot be
  requested through this getter (VERIFIED as a consequence of the class-10 skip).

## Results

- Success returns exactly one table from the shared publisher; top level always
  contains `lines` (a possibly empty array) and a fresh `dataInstanceID`
  (VERIFIED).
- Item cost: the shared item producer sets identity type 0 (`Item`) and the item
  ID (VERIFIED); the ordered line sequence of that producer is UNKNOWN.
- Currency cost: identity type 5 (`Currency`) and the currency ID, and the
  quantity passed to the currency builder is explicitly zero rather than the
  extended-cost amount (VERIFIED). The currency line is emitted with a zero owner
  GUID (VERIFIED); the post-title currency rows are UNKNOWN.
- Absence has three verified causes that reach one exit: the slot does not
  resolve to a merchant offer, the offer has no extended-cost record, or
  `costIndex` exceeds the selectable count (VERIFIED). The native exit raises
  when a pending error message exists and otherwise continues to a site that was
  not decoded, so the no-message return arity is INFERRED from the documented
  `MayReturnNothing` (UNKNOWN byte level).

## Simulator implementation

- Model: `SimState.merchant_offer_costs: HashMap<u32, MerchantCostRecord>` keyed
  by zero-based merchant slot. `MerchantCostRecord` holds the record's five item
  and five currency identities; the two lists keep the native scan order. An
  absent key is an absent record, never an empty one.
- Offer presence is `SimState.merchant_items`, the same store `GetMerchantItem`
  reads; an entry of zero is not an offer.
- The item class comes from the resident item base record
  (`item_spec_derive::item_class`), the same record the item-specification
  builder reads; the sparse lookup is `crate::items::get_item`.
- Item costs reuse the shared item producer; currency costs reuse the shared
  currency producer with an explicit zero quantity.
- Registration: `src/c_api/c_tooltip_info_merchant_cost.rs`, Retail and PTR only,
  behind the `retail-12-1-5` epoch feature.

### Deliberately not implemented

- The pending-error raise arm: the simulator models no merchant-cost error text,
  so every miss returns no values. The native text is UNKNOWN.
- The item-producer and post-title currency line sequences: UNKNOWN, so this path
  emits the existing shared producers' lines unchanged.
- `dataInstanceID` stability across calls: the shared builder stamps every call.

### Reachability gaps in the resident data

- No resident item has both an `ItemSparse` row and an item class of 10, and every
  resident `ItemSparse` row has an item base record. The money-class and
  missing-item-row skips are therefore implemented but cannot fire on the current
  catalog; they stay because the native rule is verified and the catalog is
  regenerable.
- A currency identity with no resident currency row keeps the verified type-5
  identity and publishes no lines (INFERRED shape; the native rows are UNKNOWN).

## Test coverage

`tests/tooltip_merchant_cost_item.rs` (Retail and PTR):

- the original MoneyFrame consumer `SmallDenominationTemplate_OnEnter`
  (`MerchantFrame`/`MoneyFrame.lua:448`) forwards `self.index`/`self.item`
  through the `SetMerchantCostItem` accessor and shows the selected item cost;
- ordinals skip an empty identity and an unresolvable item identity, item costs
  come before currency costs, and the currency quantity is zero;
- a miss (no offer, no record, a purely monetary record, a cost index past the
  selectable count, index 0) returns no values and leaves the frame's tooltip
  data unset.
