# Owned-inventory item tooltip

## Requirements

- `C_TooltipInfo.GetInventoryItemByID(itemID)` selects an owned instance of
  `itemID` and composes the shared catalog item lines for it.
- Search reaches the bag contents and the equipment range. Ordering is not
  observable here because per-instance identity is not modeled (below).
- The argument is an unsigned number and passes the WM94 secret-access check
  before conversion (`native-evidence-70688/WM94-tooltipinfo-getters.md` §1).
- A missing owned copy, or a selected catalog item without generated metadata,
  uses the shared `unknown_getter_failure` seam. Zero Lua return values are
  explicit **EMULATED simulator policy**, not a claim about native protected
  failure arity.

## Model limits

- WM94 establishes that the request is instance-backed, carries the instance's
  gem information and publishes `guid` on the common identity branch. The
  simulator has no per-instance identity or gem population, so the producer
  publishes the catalog item lines only.
- WM94 also describes recursive child-container search. Nested containers are
  not modeled; only the flat bag map and equipment slots are searched.
- Bag-versus-equipment precedence therefore has no observable effect and is not
  emulated.

## Profile boundary

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:407` declare the
getter; it is registered only on those profiles.

## Evidence and proof

`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:547` installs the
original `SetInventoryItemByID` accessor. `tests/tooltip_inventory_item_by_id.rs`
drives that accessor for a bag copy and the equipment range, then pins the
missing route. No other pinned Blizzard addon calls the accessor directly.
