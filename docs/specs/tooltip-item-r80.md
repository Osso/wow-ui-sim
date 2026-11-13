# Item tooltip composition: R80 evidence boundary

Item tooltip composition uses the shared item builder and host-supplied final stat amounts. See [tooltip data](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).

## What it must do

- [x] Omit ordinary stat rows when no final amounts are supplied. Never estimate amounts from item level, slot, class, or editor percentage.
- [x] Publish ordered host-supplied labels and signed final amounts through `C_TooltipInfo.GetItemByID`, `GameTooltip:SetItemByID`, and item `SetHyperlink`.
- [x] Publish supplied `guid`, `hyperlink`, nonzero `overrideItemLevel`, and optional `isCorruptedItem`, `isAzeriteEmpoweredItem`, and `isAzeriteItem` at the top level, not on lines.
- [x] Preserve getter selection, existing line positions, common type-0 color remapping (ERROR_COLOR to 41, DISABLED_FONT_COLOR to 42), and omission of unmarked wrap fields.

VERIFIED native basis: R80 Q1/Q2, Retail 12.1.5.70688, in `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md`. The final-amount seam is a simulator input contract, not a native DB2 mapping. Existing stat type/color/position are preservation, not native parity.

## Native evidence, not yet wired

VERIFIED (R80 Q2, Retail 70688): the admitted helper uses effective stat ID, editor percentage, inventory type, quality, subclass, and working level, not specialization. Stat ID >141 or quality <=1 returns zero; stats 22/23 return the editor integer. For ordinary inventory types, RandPropPoints supplies GoodF for quality 2, SuperiorF for 3/7, and EpicF for 4–6, with an inventory/subclass-selected budget slot. Other qualities return zero. The helper computes single-precision `percentage * budget * 0.0001` and subtracts supplied integer zero. Absolute budget <2^-22 skips the product/subtraction and uses zero. Stat IDs 12–21, 24, 25, 31, 32, 35–37, 40, 49, 61–64 use GameTable-3; stat 7 uses GameTable-4. Inventory types 29/30 use a separate unresolved budget path and skip both multipliers. Rounding is halfway away from zero followed by signed-int32 truncation, without a final clamp.

UNKNOWN (R80 Q2): this helper is not proved to calculate ordinary ItemSparse stat lines. Dynamic item-mod values use an unresolved helper chain. Therefore no unused calculator or inferred data mapping is added. Wiring this formula requires evidence connecting an original consumer to the admitted helper.

## How it works

- [Tooltip data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).

## Implementation inventory

- `src/c_api/item_tooltip.rs`: final-amount and optional metadata inputs.
- `src/lua_api/state/sim_state.rs`: inputs keyed by `(item_id, resolved_level)`; no alternate-key lookup.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs`: shared item publication.

## Tests asserting this spec

- `tests/tooltip_item_r80.rs`: original consumers, omission, supplied amounts, metadata.
- `tests/tooltip_item_context.rs`: supplied rows under resolved context levels.

## Known gaps (current cycle)

- [ ] Native ordinary stat-amount producer, hybrid stat labels, full line order, and exact serialized records remain UNKNOWN.
- [ ] Item ID and resolved level do not distinguish all owned-instance and descriptor variants. Hosts must not seed conflicting instances into this bounded seam.

## Out of scope

DB2 field inference, generated-data refresh, spell changes, and new conditional row predicates are excluded. C_Item.GetItemStats remains its independent exact-link supplied map; it does not use the removed estimate.
