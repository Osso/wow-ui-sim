# Tooltip money-line ownership

Commit `8097a844c` removed the simulator bootstrap definition of `GameTooltip_AddMoneyLine`.

## Root cause

The bootstrap function accepted `(tooltip, money, prefixText)`, prefixed plain `GetMoneyString` output, and called `AddLine` without a color. Cached `Blizzard_GameTooltip/Mainline/GameTooltip.lua` instead defines `(self, rawCopper, useRedLineColor)`, formats through `MoneyFormatterUtil.FormatMoney(rawCopper, GameTooltipMoneyFormat)`, and chooses `HIGHLIGHT_FONT_COLOR` or `RED_FONT_COLOR`.

The bootstrap behavior made `true` prefix the money string instead of choosing red. It also could not match the loaded helper's coin-atlas text or its configured single-space zero line.

## Ownership and removal

`Blizzard_GameTooltip` owns the global after its normal addon load. Its formatter and `GameTooltipMoneyFormat` come from cached `Blizzard_SharedXML/MoneyFormatter.lua` and `Blizzard_GameTooltip/Shared/GameTooltipConstants.lua`. Repository search found no pre-`Blizzard_GameTooltip` caller or independent bare-`WowLuaEnv` contract. The only prefix usage was the deleted bootstrap test.

The simulator must not replace this vendor behavior or duplicate MoneyFormatter. Loaded consumer proof remains pending; the audit row `added:GameTooltip_AddMoneyLine` must not claim the deleted startup bridge as evidence.

## See also

- [Tooltip money-line spec](../../specs/tooltip-money-line.md)
- [[patch-12-0-7-api-audit]]
