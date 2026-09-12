# Tooltip money lines

Loaded Blizzard tooltip code owns money-line rendering. See [[tooltip-money-line]] for load ownership and the removed bootstrap shim.

## What it must do

- [x] Loaded `GameTooltip_AddMoneyLine(self, rawCopper, useRedLineColor)` formats money with the Blizzard money formatter.
- [x] A zero copper value produces the loaded tooltip formatter's single-space line.
- [x] `false` or omitted uses highlight color; `true` uses red.
- [x] Mail tooltip consumers add their label before enclosed-money or unaffordable-COD money lines.

## How it works

- [[tooltip-money-line]]
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs` — no longer defines a pre-load `GameTooltip_AddMoneyLine` substitute.
- Cached `Blizzard_GameTooltip` and `Blizzard_SharedXML` runtime files — own the loaded helper and formatter.

## Tests asserting this spec

- `tests/tooltip_money_line.rs` — loaded helper zero/boolean color behavior and real mail enclosed-money/COD ordering.

## Known gaps (current cycle)

- [ ] Native locale/rendering, historical-client, and whole-addon-clean-startup behavior remain unproven. The focused dependency closure emitted 128 distinct Lua-error headers and 18 suppression notices; `EditModeSystemTemplates.lua:27` lacks `GetSystemSettingDisplayInfoMap`.

## Out of scope

MoneyFormatter redesign, vendor edits, native locale/rendering guarantees, mail-system redesign, and pre-addon helper availability.
