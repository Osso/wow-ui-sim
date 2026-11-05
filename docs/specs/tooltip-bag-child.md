# Bag-child item tooltips

## Scope

Retail, PTR and Forever publish `C_TooltipInfo.GetBagItemChild(bagIndex,
slotIndex, equipSlotIndex)`. Their pinned
`Blizzard_APIDocumentationGenerated/TooltipInfoDocumentation.lua:154` declares
this getter with three required indices, `AllowedWhenUntainted` arguments and
`MayReturnNothing`.

## Requirements

- Read the current bag item and its current `ItemChildEquipment` rows. Select the
  row whose stored zero-based slot equals `equipSlotIndex - 1`.
- Use the child ID, not the parent ID, with the existing item tooltip builder.
  Publish only generated item metadata; do not synthesize missing names or lines.
- Missing bag items, unmatched slots and children absent from the item catalog
  produce no tooltip. Zero return values are an **EMULATED** miss policy, not
  measured native arity.
- Authenticate all arguments before state lookup. Numeric conversion follows the
  existing simulator integer bridge; native exceptional conversion is unproven.
- Preserve the shared [D56 serialization](tooltip-data-serialization.md) contract.

## Data and evidence

The item generator selects `ChildItemID` values directly from the pinned
`ItemChildEquipment.csv`, in addition to its other item selection sources. The
70391 `ItemSparse.csv` lacks children 238721 and 238724; these remain catalog
misses. Refreshing source CSVs is separate from fabricating missing metadata.

The lookup composes the existing modeled bag and child-equipment state. The
one-based slot adjustment matches the related
[GetItemChildInfo contract](item-child-equipment.md); applying it to this getter
is **INFERRED**, not independent native selection proof. Full item-variant and
native exceptional-argument behavior are not claimed.

The original `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:544` maps
`GameTooltip:SetBagItemChild` to the getter. Its shared TOC includes that handler
with `ExcludeLoadGameType vanilla`; Retail/PTR use Mainline. The integration test
loads the original tooltip addons without test-local replacements and checks two
generated parent/child pairs, displayed text, packed RGB, wrong-slot rejection,
missing bags and a child absent from ItemSparse.

## Proof

`tests/tooltip_bag_child.rs` covers the original accessor, not merely namespace
callability. Retail targeted RED/GREEN is recorded in the lane handoff; other
profile execution and broad regression proof belong to final integration.
