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
- [x] **VERIFIED (Retail 12.1.5.70688; R88aw; simulator test):** `GetItemByID`, item-link `GetHyperlink`, ordinary item `GetBagItem`, `GetLootRollItem`, and merchant item-offer `GetMerchantItem` pass mode 0. `GetBagItem` alone supplies an item-instance identity for status processing; merchant offers set preview, and item-link preview follows `hideVendorPrice`. The necessary test drives the original item, hyperlink, bag, loot-roll, and merchant accessors with distinct item IDs.
- [ ] **VERIFIED (Retail 12.1.5.70688; R88aw):** the keystone-link `GetHyperlink` route passes mode 0 and leaves preview clear. The simulator's keystone-link route remains unmodelled and untested.
- [ ] **INFERRED (simulator mapping):** native request, preview, and instance fields map to `RetailSellPriceRequest`; the tests prove the modeled outputs, not the native field layout.
- [x] Retail `SetBagItem` publishes the seven type-6 fields only from explicit, empty-default host acceptance and snapshot inputs; the native predicate and writer remain UNKNOWN.
- [x] **VERIFIED (Retail 12.1.5.70688; `pr46-49-decomp-answers.md`, R88p.p1; simulator test):** on the no-instance path, `maxPrice` uses the prepared maximum quantity only when it exceeds current quantity; the unit-price product wraps at 32 bits. Absent or non-increasing quantity inputs keep `maxPrice = -1`. The original `SellPrice` consumer renders distinct minimum and maximum rows for two supplied quantity pairs.

### Retail SellPrice behavior (R88p.p1, R88aw)

**VERIFIED (Retail 12.1.5.70688):** a zero initial unit-price result adds no type-11 row. A row requires request mode 0, no item-status suppression, clear sell-exclusion flags, and either a clear interaction-preview flag or the preview exception. Merchant-window state, quality threshold, bound state, and quest state do not gate this row. If the initial price is nonzero, later adjustments do not reject a zero result a second time.

**VERIFIED (Retail 12.1.5.70688):** only ordinary `GetBagItem` supplies the instance identity to status processing among the supported routes. A status record suppresses only when record mask 1 is clear and `recordedTime + 7200 - clientClock` is positive. Without a status record, the row is suppressed only when item-record mask 4096 in the first flag word is set and instance marker mask 2 is clear; this branch sets the marker. No instance skips status processing.

**VERIFIED (Retail 12.1.5.70688):** an emitted row has type 11, empty `leftText`, the resolved default `leftColor`, `price`, and `maxPrice`. `maxPrice` starts at -1 and is recomputed only without an item instance when prepared maximum quantity exceeds current quantity. The timed item-status path and one-shot instance marker control status suppression.

**UNKNOWN (Retail 12.1.5.70688):** R88p.p1 does not identify the resolved type-11 `leftColor` RGB(A) components. It verifies that the row carries the resolved default color, but it does not support an exact component assertion.

**INFERRED (Retail 12.1.5.70688):** the sell-exclusion masks map to ItemSparse `Flags_1` mask 4096 and `Flags_3` mask 524288; the preview exception maps to `Flags_3` mask 32768. The masks and branches are VERIFIED; these column names use another build's header.

**VERIFIED (simulator model):** generated `ItemInfo` carries optional `Flags_0`, `Flags_1`, `Flags_3`, and Item.csv class/subclass fields, in addition to `sell_price` and `stackable`. Status records, live instance markers, calculated prices, and prepared quantities are not catalog data. **INFERRED (simulator mapping):** generated flags drive the verified exclusion and preview masks; an explicit empty-default bag status snapshot supplies only the unavailable per-instance status inputs. Missing snapshots suppress the instance-path row instead of asserting that the native status test passed. Calculated unit prices use an explicit empty-default host input keyed by item ID and working level; missing values do not claim a native calculated price. Stored-price use is modeled only for the evidenced force-stored flag and inventory-type/classification branch.

**VERIFIED (simulator model):** `item_tooltip_sell_price_quantities` is a separate empty-default input keyed by item ID; its `(current, prepared maximum)` values are not derived from `Stackable`. The Retail producer multiplies its initial line `price` by the supplied prepared maximum with 32-bit wrapping only when the maximum exceeds current quantity and no item instance is present; otherwise missing or non-increasing values keep `maxPrice` at -1. The alternate `ITEM_UNSELLABLE` path and adjusted-to-zero path remain unmodeled.

**UNKNOWN:** native item-specific status snapshots, the relationship between prepared maximum quantity and `Stackable`, and calculated-price inputs not represented by `ItemInfo`.

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
  writes them at top level. Prepared-quantity values remain UNKNOWN host inputs;
  the producer uses them only when the maximum exceeds current quantity, else it
  preserves the R88p.p1 initial value -1.
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
  producer `tooltip_for_item_context_with_sell_price` -> `tooltip_for_item_info_with_request`) proves fresh data and
  line tables, `second.dataInstanceID == first.dataInstanceID + 1`, top-level
  `type`/`id`, present-left-only publication and packed RGB on every item row;
  `GameTooltip:SetEquipmentSet` (`TooltipDataHandler.lua:585` -> `C_TooltipInfo.GetEquipmentSet`)
  proves the double-line header publishes the same packed, alpha-free RGB on its right side.
- `tests/tooltip_d56_serialization.rs`: direct getters and original consumers.
- `src/c_api/tooltip_data_tests.rs`: nested identity order and present/absent sides.
- `tests/tooltip_verified_lane.rs`: necessary original-accessor coverage for request mode, preview, bag-instance status input, prepared maximum quantity, 32-bit wrapping, and visible SellPrice output.
- `tests/tooltip_serializer_fields.rs`: selected Retail consumers read line-level `price`/`maxPrice`; fixture checks cover generated item flags/classification, preview routes, calculated-price host inputs, timed bag status, and instance versus no-instance `maxPrice`.
- `tests/tooltip_serializer_consumer_output.rs::bag_item_accessor_publishes_accepted_battle_pet_fields_to_original_consumer`
  calls the original Retail `GameTooltip:SetBagItem` on two distinct explicit
  snapshots, an explicitly unaccepted bag item, and an absent slot. It asserts
  type 6, all seven top-level values, and the seven ordered arguments forwarded
  by the original `TooltipDataRules.BattlePet`; test snapshot values are host
  fixtures, not native pet defaults.
- `tests/tooltip_serializer_fields.rs`: **VERIFIED (Retail 12.1.5.70783 cache):** the selected fixture checks `Blizzard_SharedXMLGame`, `Blizzard_GameTooltip` and `Blizzard_UIPanels_Game`, and checks original source provenance for `SetItemByID`, `TooltipDataRules.SellPrice` and `GameTooltip_OnTooltipAddMoney`. The no-instance selector follows `TooltipDataHandler.lua:525` → `TooltipDataRules.lua:143-149` → `Blizzard_GameTooltip/Mainline/GameTooltip.lua:325-333`; it checks line `maxPrice` and callback arguments. Bag-instance tests assert status suppression and keep `maxPrice = -1`.
- `tests/tooltip_serializer_consumer_output.rs::bag_repair_rule_renders_only_top_level_positive_repair_cost`
  observes only original consumer output: the selected `SetBagItem` accessor and
  `FinalizeItemTooltip` render REPAIR_COST plus the money row for quotes 731 and
  2468, and nothing for a zero or absent quote. A line-level `repairCost` renders
  nothing, so the selector rejects that placement inside the consumer.
- `tests/tooltip_serializer_consumer_output.rs::bag_sell_price_rule_renders_only_type_11_line_prices`
  (**INFERRED simulator mapping; VERIFIED base records per R88aw**) renders one
  `SELL_PRICE: <money>` row through the original `SellPrice` rule for bag items
  priced 1 and 50 when explicit no-status-record snapshots are supplied, and
  none for a zero price. The type-11 line owns `price`; separate field tests
  cover prepared `maxPrice`, timed status suppression, and the item-instance
  `maxPrice = -1` rule.
- Existing tooltip builder, GC and presentation tests remain regression coverage.

**Proved:** the shared publication helpers (`stamp_instance`, `publish_rgb`,
`publish_side`, `assign_identity`) and the item and equipment-set producers that
call them. **Unproved:** that each remaining getter's native producer uses these
same common builders, producer-specific miss shapes, producer-specific auxiliary
keys, and any getter that would request a second identity assignment.

## Related

- [Loot/buyback](tooltip-loot-buyback.md)
- [Lua API](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation)
