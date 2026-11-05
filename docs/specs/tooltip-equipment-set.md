# Equipment-set tooltip

## Requirements

- `C_TooltipInfo.GetEquipmentSet(setID)` summarizes a saved equipment set from
  `C_EquipmentSet` state as `native-evidence-70688/WM94-tooltipinfo-getters.md`
  §6 describes.
- Line order: a header double line with the set name and the formatted
  `ITEMS_VARIABLE_QUANTITY` count, then only the nonzero `ITEMS_EQUIPPED`
  (normal color), `ITEMS_IN_INVENTORY` (highlight color) and `ITEM_SLOTS_IGNORED`
  (disabled color) counts, then one `ITEM_MISSING` line per unfilled,
  non-ignored inventory slot in reverse slot order.
- Inventory slots are the 19 equipment slots, using the pinned slot labels
  (`HEADSLOT` … `TABARDSLOT`, with distinct finger and trinket label keys).
- The identity is `Enum.TooltipDataType.EquipmentSet` with the set ID.
- An unknown set ID uses the shared `unknown_getter_failure` seam. Zero Lua
  return values are **EMULATED** simulator policy.
- The argument is a required signed number and passes the WM94 secret-access
  check before conversion.

## Model limits

- **UNKNOWN:** the native line types for this payload and the header's own color.
  They are published as plain lines; the header color is left to the shared
  default.
- **EMULATED:** counting. "Equipped" counts saved slots whose current equipment
  holds the same item; "in inventory" counts saved items present in the bag map;
  "ignored" counts the set's ignored slot entries.
- The shared named-color lookup reads the raw global table, while the simulator
  synthesizes color constants through `_G.__index`. Miss lines therefore publish
  the shared default color instead of the pinned error color; that is a
  pre-existing simulator limitation shared with every tooltip color lookup.

## Profile boundary

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:271` declare the
getter; it is registered only on those profiles.

## Evidence and proof

`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:575` installs the original
`SetEquipmentSet` accessor and
`Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua:2173` hovers a set row with
`GameTooltip:SetEquipmentSet(self.setID)`.
`tests/tooltip_equipment_set.rs` builds three saved sets with distinct
equipped/inventory/ignored counts and asserts the header, the count selection and
the reverse-order miss list, plus the unknown-set route.
