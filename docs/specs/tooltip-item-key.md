# Item-key tooltip

## Requirements

- `C_TooltipInfo.GetItemKey(itemID, itemLevel, itemSuffix, requiredLevel)`
  composes the shared catalog item with the item-key controls of
  `native-evidence-70688/WM94-tooltipinfo-getters.md` §2.
- A nonzero `itemLevel` publishes top-level integer `overrideItemLevel` with the
  supplied value and drives the displayed level. The working level used for line
  composition is bounded to 1..=1300; zero keeps the catalog level.
- A nonzero `requiredLevel` publishes the simple requirement line: the
  `ITEM_MIN_LEVEL` format, numeric line type 5 and integer `level`, combined with
  the item requirement by taking the greater value. Zero publishes no
  requirement line.
- `itemID` and `itemLevel` are unsigned, `itemSuffix` signed, and `requiredLevel`
  optional unsigned. All pass the WM94 secret-access check before conversion.
- A missing catalog item uses the shared `unknown_getter_failure` seam. Zero Lua
  return values are **EMULATED** simulator policy.

## Model limits

- **EMULATED:** the item requirement term is the catalog `RequiredLevel`. The
  native separate requirement calculation is not established by WM94.
- **UNKNOWN, not published:** the conditional
  `AUCTION_HOUSE_BUCKET_VARIATION_EQUIPMENT_TOOLTIP` line.
- **UNKNOWN, not published:** item-name suffix records. No suffix table is
  modeled, so a nonzero suffix leaves the base name in place; no top-level
  `itemSuffix` field is written.
- Requirement-line placement after the shared item lines is a bounded choice;
  WM94 does not fix its position.

## Profile boundary

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:484` declare the
getter; it is registered only on those profiles.

## Evidence and proof

`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:548` installs the original
`SetItemKey` accessor. The pinned auction-house callers pass an item key from
`C_AuctionHouse.GetItemKeyInfo` plus `C_AuctionHouse.GetItemKeyRequiredLevel`
(`Blizzard_AuctionHouseUI/Mainline/Blizzard_AuctionHouseUtil.lua:94`).
`tests/tooltip_item_key.rs` drives the accessor for two generated items and pins
the override, the working-level bound and the required-level combination.
