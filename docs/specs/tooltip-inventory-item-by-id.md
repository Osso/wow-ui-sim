# Owned-item tooltip by item ID

`C_TooltipInfo.GetInventoryItemByID` selects an owned copy of an item ID from the
player's inventory and publishes the ordinary item tooltip for that copy. The
original tooltip handler reaches it through `GameTooltip:SetInventoryItemByID`
(`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua`, accessor table).

Declared by Retail, PTR and Forever `TooltipInfoDocumentation.lua:407` with one
`itemID` number argument, `MayReturnNothing = true`.

Native evidence: Retail 12.1.5 (70688); Forever 1.60.1 (70707) shows the same
shape.

## What it must do

- [x] The argument uses the plain signed-integer reader. VERIFIED.
- [x] There is no semantic item-ID bound: zero and negative ids pass the reader
      and then fail the inventory search. VERIFIED.
- [x] Selection is one forward pass over the player's inventory that keeps the
      first acceptable candidate, with no preference for an equipped copy over
      a copy found earlier. VERIFIED.
- [x] The backpack range is searched before the equipment range. VERIFIED.
- [x] A candidate's child container is searched before the pass moves on.
      VERIFIED.
- [x] The request is built from the found item's ID and its own GUID twice.
      VERIFIED.
- [x] The published table carries the found instance's GUID as top-level `guid`.
      INFERRED: the shared item producer writes the resolved item's GUID and
      takes the GUID branch for a real instance.
- [x] A resolved instance publishes `type` = `Enum.TooltipDataType.Item` and the
      ordinary item lines from the shared item producer. `type` VERIFIED; the
      top-level `id` presence is UNKNOWN natively and the shared producer's value
      is kept.
- [x] A miss publishes no values, sets no identity and adds no lines. VERIFIED
      that the result count stays 0; INFERRED that the call returns nothing.
- [x] Ascending slot order inside one container. INFERRED from the contiguous
      native slot ranges.

## Simulator mapping

The simulator stores the backpack, the four carried bags and the equipment slots
separately and has no flat native inventory-slot array. The native category order
is reproduced over those stores in this order:

1. backpack (bag id 0), ascending slot;
2. the equipment range, ascending slot;
3. carried bags 1-4, ascending bag id then ascending slot.

VERIFIED: the backpack precedes the equipment range, and the equipment range
precedes the native container-slot range. INFERRED: that container-slot range is
the carried bags, so their contents are visited last, which is also where the
child-container recursion applies.

The native ranges listed between the backpack range and the equipment range are
unnamed (UNKNOWN), and the bank is not part of the modelled carried inventory, so
neither is searched.

## Known gaps (current cycle)

- [ ] The gem/socket projection is VERIFIED as an instance data copy before the
      item enters the builder, and the socket metadata row shape (`socketType`,
      `gemIcon`, `gemItemID`) is VERIFIED. Which items trigger which row and its
      line type are UNKNOWN, and the shared item producer emits no socket rows,
      so the projection has no observable output and is not modelled.
- [ ] `guid` is published only for a bag copy. An equipped copy has no simulator
      GUID string: `C_Item.GetItemGUID` models bag locations only.
- [ ] The two per-item state and usability tests of the native search are
      recorded but their gameplay meaning is UNKNOWN, so only a missing item or a
      missing catalog record rejects a candidate.
- [ ] Exact native argument-error text is not modelled; the shared signed reader
      reports its own message.

## Out of scope

- Bank, reagent-bank and other unnamed native inventory categories.
- `dataInstanceID` stability across calls and the native missing-result arity.

## Related

- [Tooltip and Lua API](../lua-api.md)
- [Tooltip data serialization](tooltip-data-serialization.md)
- [Owned-item identity](owned-item-identity.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_inventory_by_id.rs`: argument reader, ordered
  inventory scan, instance GUID and tooltip publication.
- `src/lua_api/globals/missing_surface/tooltip_info/`: shared item tooltip
  producer.

## Tests asserting this spec

- `tests/tooltip_inventory_item_by_id.rs`
