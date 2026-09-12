# Tooltip money-line ownership

Commit `8097a844c` removed the simulator bootstrap definition of `GameTooltip_AddMoneyLine`.

## Root cause

The bootstrap function accepted `(tooltip, money, prefixText)`, prefixed plain `GetMoneyString` output, and called `AddLine` without a color. Cached `Blizzard_GameTooltip/Mainline/GameTooltip.lua` instead defines `(self, rawCopper, useRedLineColor)`, formats through `MoneyFormatterUtil.FormatMoney(rawCopper, GameTooltipMoneyFormat)`, and chooses `HIGHLIGHT_FONT_COLOR` or `RED_FONT_COLOR`.

The bootstrap behavior made `true` prefix the money string instead of choosing red. It also could not match the loaded helper's coin-atlas text or its configured single-space zero line.

## Ownership and removal

`Blizzard_GameTooltip` owns the global after its normal addon load. Its formatter and `GameTooltipMoneyFormat` come from cached `Blizzard_SharedXML/MoneyFormatter.lua` and `Blizzard_GameTooltip/Shared/GameTooltipConstants.lua`. Repository search found no pre-`Blizzard_GameTooltip` caller or independent bare-`WowLuaEnv` contract. The only prefix usage was the deleted bootstrap test.

The simulator must not replace this vendor behavior or duplicate MoneyFormatter.

## Development proof

`d99d0bc72` adds three grouped `retail-12-0-7` tests in `tests/tooltip_money_line.rs`. `/tmp/tooltip-money-development-ledger.json` records 3/3 focused cases passing: loaded helper zero output is a single space; false/default and true select highlight/red colors with concrete coin-atlas text; and real `InboxFrameItem_OnEnter` preserves label-before-money ordering for enclosed money and unaffordable COD.

The closure recorded 128 distinct dependency Lua-error headers, including unrelated FriendsFrame, EditMode, chat, and social API gaps. The assertions passed despite those errors, so this is focused consumer proof only—not clean whole-addon startup or broader addon compatibility proof. One earlier test invocation was aborted before its output was captured; its result is not used as evidence. Independent verification remains pending.

`dd701ae2e` credits `added:GameTooltip_AddMoneyLine` from the three loaded tests and replaces the retired startup bridge reference. Native historical-client behavior, locale/rendering variation, and complete addon-startup behavior remain unproven.

## See also

- [Tooltip money-line spec](../../specs/tooltip-money-line.md)
- [[patch-12-0-7-api-audit]]
