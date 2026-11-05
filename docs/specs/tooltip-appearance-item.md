# Modified-appearance item tooltip

## Requirements

- Retail/PTR `C_TooltipInfo.GetItemByItemModifiedAppearanceID(id)` reads the
  existing supplied source model through `read_source_info`, parses its item
  link with the shared item-link parser and uses the unchanged item builder.
- Read current source state on every call. Preserve catalog lines and the
  [D56 publication contract](tooltip-data-serialization.md).
- Do not add a second source model, synthesize item names or substitute the
  appearance ID for an item ID.
- Missing sources, invalid source links and absent item metadata use the shared
  `unknown_getter_failure` seam: zero Lua return values are explicit
  **EMULATED simulator policy**, not a claim about native protected-failure arity.
- Native protected-failure publication remains **UNKNOWN**, as recorded by
  WM94 (`native-evidence-70688/WM94-tooltipinfo-getters.md`, protected-failure
  evidence limits). The seam is shared with bag-child misses; unrelated existing
  getter miss contracts are unchanged.

## Profile boundary

Retail and PTR pinned `TooltipInfoDocumentation.lua:458` declare the getter.
Forever declares it at the same line, but its build does not include the existing
rich `read_source_info` model. This change does not register the getter on
Forever or promote that state model. Forever support is a follow-up, not an
alternate lookup path.

## Evidence and proof

`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:526` installs the original
`SetItemByItemModifiedAppearanceID` accessor.
`Blizzard_SharedXMLGame/DressUpModelFrameMixin.lua:750–760` implements the original
hover/leave consumer. `tests/tooltip_appearance_item.rs` runs that hover for two
supplied source records backed by generated item metadata, checks every line's
text and displayed title, then changes the first source to prove live selection.
It also pins the shared failure seam's zero-return policy.

Source-to-item selection is modeled reuse, not independent proof of every native
appearance variant. No native output parity beyond the reused item builder is
claimed. Retail targeted RED/GREEN evidence belongs to the lane report; PTR and
broader regression execution belong to final integration.
