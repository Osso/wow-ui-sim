# Tooltip data serialization

## Scope

Retail/PTR common populated tooltip builders follow WM82 and WM63 D56 (Retail
12.1.5 build 70688). Applying the shared rules to other getters using these
builders is **INFERRED**, not independent native getter proof. Other profiles and
existing miss shapes are unchanged.

Pinned API declarations on both profiles: `Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:439`
(`GetItemByID`), `:1045` (`GetSpellByID`), `:1181` (`GetUnit`). Existing getter
publication is not expanded by this change.

## Requirements

- [x] Populated publications have fresh tables/lines and an incrementing per-state `dataInstanceID`.
- [x] Common text is copied with markup intact, without synthetic segment arrays.
- [x] Supplied sides, including empty strings, publish text and RGB color; absent sides publish neither.
- [x] RGB uses packed bytes multiplied in single precision by `1/255`, without alpha; preserve usable color methods.
- [x] Both sides take that same RGB publication: a supplied right side publishes the packed color, not the raw factory color.
- [x] Populated identity omits zero `id`; later common identity assignments append ordered type-19 rows with `tooltipType`, nonzero-only `tooltipID`, empty left text and supplied left color.
- [x] Original `ProcessInfo`, GameTooltip line handlers and TooltipUtil retain their payloads; presentation parses inline markup independently.
- [x] The Retail `GameTooltip:SetItemByID` path emits a type-11 row for a nonzero backed catalog price with empty `leftText`, resolved default `leftColor`, `price`, and `maxPrice = -1` when no prepared quantity input exists. Mapping `ItemInfo.sell_price` to the native initial unit price is **INFERRED**.
- [ ] Retail `SetBagItem` publishes the seven type-6 fields only from explicit, empty-default host acceptance and snapshot inputs; the native predicate and writer remain UNKNOWN.

### Retail SellPrice behavior (R88p.p1)

**VERIFIED (Retail 12.1.5.70688):** a zero initial unit-price result adds no type-11 row. A row requires request mode 0, no item-status suppression, clear sell-exclusion flags, and either a clear interaction-preview flag or the preview exception. No quality, bound, quest, or merchant-open test gates this row. If the initial price is nonzero, later adjustments do not reject a zero result a second time.

**VERIFIED (Retail 12.1.5.70688):** an emitted row has type 11, empty `leftText`, the resolved default `leftColor`, `price`, and `maxPrice`. `maxPrice` starts at -1 and is recomputed only without an item instance when prepared maximum quantity exceeds current quantity. The timed item-status path and one-shot instance marker control status suppression.

**UNKNOWN (Retail 12.1.5.70688):** R88p.p1 does not identify the resolved type-11 `leftColor` RGB(A) components. It verifies that the row carries the resolved default color, but it does not support an exact component assertion.

**INFERRED (Retail 12.1.5.70688):** the sell-exclusion masks map to ItemSparse `Flags_1` mask 4096 and `Flags_3` mask 524288; the preview exception maps to `Flags_3` mask 32768. The masks and branches are VERIFIED; these column names use another build's header.

**VERIFIED (simulator model):** `ItemInfo` carries `sell_price` and `stackable`, but it does not carry `Flags_1`, `Flags_3`, item instances, status records, or prepared quantity state. **INFERRED (simulator policy):** `ItemInfo.sell_price` stands in for the native initial unit-price calculation. Therefore the simulator leaves status and flag gates unsuppressed and keeps `maxPrice` at -1; it does not add unbacked writers or exclusions. The alternate `ITEM_UNSELLABLE` path and adjusted-to-zero path remain unmodeled.

**UNKNOWN:** the relationship between prepared maximum quantity and `Stackable`, and the calculated-price inputs not represented by `ItemInfo`.

## Evidence limits

WM82 and D56 establish the publication boundary, not complete protected getter
connections or misses. Existing misses retain their prior shapes. No GUID is
fabricated. No getter is assigned new nested-row eligibility: the generic builder
supports later identity assignment, but no current producer requests one.

`scenario_color_byte` is the single **EMULATED** float-to-packed-byte input seam:
round to the nearest saturated byte. Native float-to-byte conversion is **UNKNOWN**.
The existing color factory is retained; full native `ColorMixin` contents remain
**UNKNOWN**. Default side colors use the supplied `NORMAL_FONT_COLOR` scenario
value, not an assertion about every native producer.

WM72 establishes that empty strings on both sides append no displayed row and
right-only rows can exist. It does not add TooltipData fields or imply geometry
changes here.

**VERIFIED (Retail 12.1.5.70688, R88n.4b):** `repairCost` and the seven
battle-pet fields are top-level TooltipData fields. `price` and `maxPrice`
belong on a line of type 11. **VERIFIED (simulator model):** item catalog rows
contain `sell_price`, and `C_Item.GetItemInfo` returns it. **INFERRED (simulator
policy):** the Retail item builder publishes a positive `sell_price` as
`price` on a type-11 line. It does not infer `maxPrice` from stack capacity or
LFG reward quantity. R88p.p1 (above) verifies the `maxPrice` source: it starts at
-1 and only the no-instance prepared-quantity branch recalculates it.

### R88n.4b placement audit (lt-bf18)

**VERIFIED (simulator source, audited at `d54da90d9`):** no shared serializer
placement contradicts R88n.4b, so no producer changed.

- `repairCost`: `tooltip_for_bag_item` (`tooltip_info/sources.rs`) sets it on
  the top-level TooltipData of the Retail `GetBagItem` path, only for a positive
  quote from the host seam `bag_item_repair_costs`. That seam is empty by
  default; the quote calculation is UNKNOWN ([bag repair cost](bag-repair-cost.md)).
- `price` and `maxPrice`: only `push_retail_sell_price_line`
  (`tooltip_info/builders.rs`) writes them, on a type-11 line. No producer
  writes them at top level. `maxPrice` is the R88p.p1 initial value -1, not a
  guess; the prepared-quantity recalculation inputs are UNKNOWN and unmodeled.
- Battle-pet fields: the audit base had no type-6 producer or host seam.
  `battlePetSpeciesID` on Auction House `ItemKey` data is not a TooltipData
  producer. The Retail bag-item path now publishes the seven fields only from
  explicit host acceptance and snapshot inputs; those inputs are empty by
  default. The native acceptance predicate and upstream writer remain
  **UNKNOWN** and are not modeled.

## Retail battle-pet bag TooltipData

**VERIFIED (Retail 12.1.5.70688, R88n.4b):** the accepted alternate bag-item
branch produces TooltipData type 6 and can publish `battlePetSpeciesID`,
`battlePetLevel`, `battlePetBreedQuality`, `battlePetMaxHealth`,
`battlePetPower`, `battlePetSpeed`, and `battlePetName` at the top level.

**VERIFIED (simulator model):** the original `SetBagItem` path publishes type
6 only for an existing bag slot with explicit host acceptance and a complete
host snapshot. The snapshot's seven values stay at the top level and the
original `TooltipDataRules.BattlePet` consumer forwards them in order.
Acceptance and snapshot inputs have separate empty defaults; item identity does
not infer the native acceptance decision.

**UNKNOWN:** the native acceptance predicate, the upstream snapshot writer, and
whether every accepted native path supplies every listed field. The host inputs
do not claim those native details.

## Proof

- `tests/tooltip_data_identity.rs` drives the original accessors only:
  `GameTooltip:SetItemByID` (`TooltipDataHandler.lua:525` -> `C_TooltipInfo.GetItemByID`,
  producer `tooltip_for_item_context` -> `tooltip_for_item_info`) proves fresh data and
  line tables, `second.dataInstanceID == first.dataInstanceID + 1`, top-level
  `type`/`id`, present-left-only publication and packed RGB on every item row;
  `GameTooltip:SetEquipmentSet` (`TooltipDataHandler.lua:585` -> `C_TooltipInfo.GetEquipmentSet`)
  proves the double-line header publishes the same packed, alpha-free RGB on its right side.
- `tests/tooltip_d56_serialization.rs`: direct getters and original consumers.
- `src/c_api/tooltip_data_tests.rs`: nested identity order and present/absent sides.
- `tests/tooltip_serializer_fields.rs`: selected Retail consumers read top-level repair/pet fields and line-level `price`/`maxPrice`; the original `SetItemByID` path checks the inferred backed `price` projection.
- `tests/tooltip_serializer_consumer_output.rs::bag_item_accessor_publishes_accepted_battle_pet_fields_to_original_consumer`
  calls the original Retail `GameTooltip:SetBagItem` on two distinct explicit
  snapshots, an explicitly unaccepted bag item, and an absent slot. It asserts
  type 6, all seven top-level values, and the seven ordered arguments forwarded
  by the original `TooltipDataRules.BattlePet`; test snapshot values are host
  fixtures, not native pet defaults.
- `tests/tooltip_serializer_consumer_output.rs::bag_repair_rule_renders_only_top_level_positive_repair_cost`
  observes only original consumer output: the selected `SetBagItem` accessor and
  `FinalizeItemTooltip` render REPAIR_COST plus the money row for quotes 731 and
  2468, and nothing for a zero or absent quote. A line-level `repairCost` renders
  nothing, so the selector rejects that placement inside the consumer.
- `tests/tooltip_serializer_consumer_output.rs::bag_sell_price_rule_renders_only_type_11_line_prices`
  (**INFERRED** values: catalog `sell_price` as the initial unit price) renders
  one `SELL_PRICE: <money>` row through the original `SellPrice` rule for bag
  items priced 1 and 50, and none for a zero price. A `price` outside the
  type-11 line renders nothing. The single-row form preserves the VERIFIED
  item-instance `maxPrice` of -1; it is preservation, not a `maxPrice` RED
  selector, because no backed input varies `maxPrice`.
- Existing tooltip builder, GC and presentation tests remain regression coverage.

**Proved:** the shared publication helpers (`stamp_instance`, `publish_rgb`,
`publish_side`, `assign_identity`) and the item and equipment-set producers that
call them. **Unproved:** that each remaining getter's native producer uses these
same common builders, producer-specific miss shapes, producer-specific auxiliary
keys, and any getter that would request a second identity assignment.

## Related

- [Loot/buyback](tooltip-loot-buyback.md)
- [Lua API](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)
