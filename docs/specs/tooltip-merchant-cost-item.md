# C_TooltipInfo.GetMerchantCostItem

`C_TooltipInfo.GetMerchantCostItem(slot, costIndex)` returns the tooltip data for
one entry of a merchant offer's extended-cost record.

Evidence: decomp Round 6 §2 ("the loot and merchant-cost tooltip getters"),
Retail 12.1.5 build 70688, and R88n.8 for Forever 1.60.1.70707. **VERIFIED**
labels identify facts proved by a pinned client image; **INFERRED** labels identify
conclusions from those facts or shipped UI Lua; **UNKNOWN** identifies behavior
not established.

## Surface

- Declared in Retail and PTR `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:586`
  with `slot` and `costIndex` (`luaIndex`), `MayReturnNothing = true`,
  `SecretArguments = "AllowedWhenUntainted"` and a non-nilable `TooltipData`
  return (VERIFIED for the pinned Retail/PTR documentation).
- Forever's `TooltipInfoDocumentation.lua:586` also declares the getter with
  `MayReturnNothing`, `AllowedWhenUntainted`, and two required `luaIndex` inputs
  (VERIFIED). The simulator registers the same producer on Forever; PTR remains
  unchanged.
- The equivalent of `GameTooltip:SetMerchantCostItem` is this getter:
  `TooltipDataHandler.lua:581` maps the accessor to `GetMerchantCostItem`, and the
  tooltip reader has exactly one caller in the image, the getter callback
  (VERIFIED for the single caller; INFERRED for the UI mapping). There is no
  separate C-side line producer that could drift.

## Arguments

- **VERIFIED (Forever R88n.8):** each argument accepts ordinary numbers and
  convertible numeric strings in 0..=4294967295, checks range before subtracting
  one, then converts the result to signed 32-bit. This is not unsigned wrapping.
  Zero passes range validation but becomes a negative selection and selects no
  offer or cost.
- **UNKNOWN (Forever):** R88n.8 does not state the result when the decremented,
  truncated value is outside signed-32 range. Rust's saturating float-to-integer
  cast is only a neutral placeholder, not native-parity evidence.
- Retail/PTR keep the existing numeric-only `luaIndex` reader; this lane does not
  change their conversion.
- The index counter advances between the two reads, so the second read addresses
  argument 2 whether or not the first succeeded (VERIFIED).
- Usage text: `Usage: local data = C_TooltipInfo.GetMerchantCostItem(slot, costIndex)`
  (VERIFIED, identical in both images).
- **VERIFIED (Forever R88n.8):** missing/nil, unsupported types, and values
  outside the accepted range take the argument-error route. The exact Usage text
  is verified; the complete wrapped error string and secret-policy suffixes are
  **UNKNOWN**.
- There is no per-argument range check against the offer (VERIFIED).

## Selection

- **VERIFIED (Retail Round 6):** the record is scanned for up to five item-cost
  entries and then up to five currency-cost entries.
- **VERIFIED (Forever R88n.8):** the selected offer list is bounded by its stored
  count and applies availability and filter checks. Each offer supplies an
  `ItemExtendedCost` lookup; the item scan uses `ItemSparse` and `Item`.
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
- **VERIFIED (Retail):** currency cost uses identity type 5 (`Currency`) and the
  currency ID, with quantity zero and a zero owner GUID. Post-title rows are
  UNKNOWN.
- **VERIFIED (Forever R88n.8):** no selected offer, no extended-cost identifier,
  no matching `ItemExtendedCost` record, no selected eligible cost, or a false
  selected producer returns exactly zero Lua values on normal completion.
- Retail's pending-error raise branch and no-message native return arity remain
  **UNKNOWN**; the existing simulator miss seam returns no values.

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
- Item costs reuse the shared item producer. Currency costs use the shared
  currency producer with an explicit zero quantity on Retail/PTR only; Forever
  keeps its no-tooltip path for selected currency costs.
- Registration: `src/c_api/c_tooltip_info_merchant_cost.rs` shares item-cost
  production on Retail/PTR and Forever. Retail/PTR compile under `retail-12-1-5`;
  Forever uses `client_profile::ACTIVE`.

### Deliberately not implemented

- The pending-error raise arm: the simulator models no merchant-cost error text,
  so every miss returns no values. Its native text remains UNKNOWN; the verified
  Forever missing-source arms complete with zero Lua values.
- The item-producer and post-title currency line sequences: UNKNOWN, so this path
  emits the existing shared producers' lines unchanged.
- **VERIFIED (simulator):** Forever does not publish a Currency tooltip when a
  selected cost needs the Retail-only currency builder; it returns zero values
  instead of fabricating that result. **INFERRED (R88n.8):** the second native
  cost-entry set represents currency costs.
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

`tests/tooltip_merchant_cost_item.rs` (Retail/PTR) and the optional Forever
cases in `tests/tooltip_forever_four_getters.rs`:

- the original MoneyFrame consumer `SmallDenominationTemplate_OnEnter`
  (`MerchantFrame`/`MoneyFrame.lua:448`) forwards `self.index`/`self.item`
  through the `SetMerchantCostItem` accessor and shows the selected item cost;
- ordinals skip an empty identity and an unresolvable item identity, item costs
  come before currency costs, and the currency quantity is zero;
- Retail: misses (no offer, no record, a purely monetary record, a cost index
  past the selectable count, index 0) return no values and leave frame tooltip
  data unset.
- Forever: registration, string/fraction arguments, no offer, no extended-cost
  record and no selected cost each produce the verified zero-value miss result.

**UNKNOWN (Forever simulator):** the offer availability/filter inputs are not
represented separately. The current producer consumes only supplied merchant
items and extended-cost records; no missing eligibility state is fabricated.
