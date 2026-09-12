# Tooltip money-line ownership

Commit `8097a844c` removed the simulator bootstrap definition of `GameTooltip_AddMoneyLine`.

## Root cause

The bootstrap function accepted `(tooltip, money, prefixText)`, prefixed plain `GetMoneyString` output, and called `AddLine` without a color. Cached `Blizzard_GameTooltip/Mainline/GameTooltip.lua` instead defines `(self, rawCopper, useRedLineColor)`, formats through `MoneyFormatterUtil.FormatMoney(rawCopper, GameTooltipMoneyFormat)`, and chooses `HIGHLIGHT_FONT_COLOR` or `RED_FONT_COLOR`.

The bootstrap behavior made `true` prefix the money string instead of choosing red. It also could not match the loaded helper's coin-atlas text or its configured single-space zero line.

## Ownership and removal

`Blizzard_GameTooltip` owns the global after its normal addon load. Its formatter and `GameTooltipMoneyFormat` come from cached `Blizzard_SharedXML/MoneyFormatter.lua` and `Blizzard_GameTooltip/Shared/GameTooltipConstants.lua`. Repository search found no pre-`Blizzard_GameTooltip` caller or independent bare-`WowLuaEnv` contract. The only prefix usage was the deleted bootstrap test.

The simulator must not replace this vendor behavior or duplicate MoneyFormatter.

## Verification

`d99d0bc72` adds three grouped `retail-12-0-7` tests in `tests/tooltip_money_line.rs`. `/tmp/verify-tooltip-money-ledger.json` reuses their hash-matched 3/3 focused consumer proof and records fresh 2/2 library checks for the removed bootstrap surface. The consumer cases assert loaded helper single-space zero output, false/default highlight versus true red with concrete coin-atlas text, and real `InboxFrameItem_OnEnter` label-before-money ordering for enclosed money and unaffordable COD.

At `e393e2f8e`, format, default check, both default binary builds, zero-error default startup, three manifest validators, and all-manifest hash scanning pass. The first 12.0.7 validator failed only because `load_addon` metadata was invalid; `e393e2f8e` corrects it and the one validator retry passes. One earlier focused test invocation was aborted before output capture; it is not evidence.

The dependency closure emitted 128 distinct Lua-error headers, including nested duplicates, plus 18 suppression notices. `Blizzard_EditMode/Shared/EditModeSystemTemplates.lua:27` lacks `GetSystemSettingDisplayInfoMap`; FriendsFrame, chat, and social gaps also occur. The assertions therefore prove only the focused consumer boundary, not clean whole-addon, container, or layout compatibility. Native historical-client behavior, locale/rendering variation, and complete addon startup remain unproven.

## See also

- [Tooltip money-line spec](../../specs/tooltip-money-line.md)
- [[patch-12-0-7-api-audit]]
