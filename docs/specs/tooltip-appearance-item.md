# Modified-appearance item tooltip

## Requirements

- Retail/PTR `C_TooltipInfo.GetItemByItemModifiedAppearanceID(id)` reads the
  existing supplied source model through `read_source_info`, parses its item
  link with the shared item-link parser and uses the unchanged item builder.
- Read current source state on every call. Preserve catalog lines and the
  [D56 publication contract](tooltip-data-serialization.md).
- Retail/PTR do not add a second source model, synthesize item names or
  substitute the appearance ID for an item ID. Forever uses the empty-by-default
  supplied input described below and does not seed guessed rows.
- Missing sources, invalid source links and absent item metadata on Retail/PTR
  use the shared `unknown_getter_failure` seam: zero Lua return values are
  **EMULATED simulator policy** there, not a native protected-failure claim.
- **VERIFIED (Forever 1.60.1.70707, R88n.8):** a missing
  `ItemModifiedAppearance` record returns zero Lua values. Forever accepts a
  required signed-32 ID, including numeric strings and fractions truncated toward
  zero after range validation; its Usage text is
  `Usage: local data = C_TooltipInfo.GetItemByItemModifiedAppearanceID(itemModifiedAppearanceID)`.
- [ ] **INFERRED (Forever simulator input):** `SimState.forever_appearance_item_ids`
  is an empty-by-default host map from the getter's appearance ID to an item ID.
  A hit uses the shared item tooltip producer. R88n.8 does not prove this field
  mapping, a native writer or populated native-record parity.
- **UNKNOWN:** native failure results outside the verified missing-record path,
  as bounded by WM94 (`native-evidence-70688/WM94-tooltipinfo-getters.md`).

## Profile boundary

Retail, PTR and Forever pinned `TooltipInfoDocumentation.lua:458` declare the
getter. The simulator registers it on all three profiles. Retail/PTR retain the
existing `read_source_info` lookup and item builder. Forever reads only the
supplied `forever_appearance_item_ids` map; it does not use or seed a native
appearance catalogue.

## Evidence and proof

`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:526` installs the original
`SetItemByItemModifiedAppearanceID` accessor.
`Blizzard_SharedXMLGame/DressUpModelFrameMixin.lua:750–760` implements the original
hover/leave consumer. `tests/tooltip_appearance_item.rs` runs that hover for two
supplied source records backed by generated item metadata, checks every line's
text and displayed title, then changes the first source to prove live selection.
It also pins the shared failure seam's zero-return policy. The optional Forever
case in `tests/tooltip_forever_four_getters.rs` supplies two distinct
appearance-to-item rows, compares each result with `GetItemByID`, and checks the
zero-value miss. This proves the simulator's supplied-input route, not native
field mapping or native positive-record behavior.

Source-to-item selection is modeled reuse, not independent proof of every native
appearance variant. The Forever documentation declares a non-nil TooltipData
return and `MayReturnNothing`; the original shared accessor is at
`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:526`. The supplied map and
its ID relationship remain **INFERRED** simulator inputs. Tests do not establish
native source writers or populated-record parity.
