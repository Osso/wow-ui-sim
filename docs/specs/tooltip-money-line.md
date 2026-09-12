# Tooltip money lines

## What it must do

- [ ] Use the loaded, unmodified Blizzard `GameTooltip_AddMoneyLine(self, rawCopper, useRedLineColor)` helper, not a simulator prefix-text substitute.
- [ ] Preserve loaded money formatting, including coin atlas markup and the single-space zero-money line.
- [ ] Select highlight color by default or for false, and red for true.
- [ ] Preserve mail tooltip label-before-money ordering for enclosed money and unaffordable COD.

## Ownership

`Blizzard_GameTooltip` owns this Lua helper; its money formatting depends on `Blizzard_SharedXML/MoneyFormatter.lua` and `GameTooltipConstants.lua`. A bare `WowLuaEnv` does not promise this addon-defined global. Normal addon loading must provide the helper before consumers invoke it.

The removed simulator bootstrap shim treated argument three as prefix text and used plain `GetMoneyString` output. No pre-load consumer or independent bare-environment contract was found. Do not restore this alternate implementation or duplicate MoneyFormatter.

## Verification boundary

Focused grouped integration tests must exercise loaded Blizzard tooltip/mail code and observable text, colors, and ordering. Cached source tests establish simulator consumer behavior, not native historical-client conformance or whole-addon compatibility. Verification is pending.

## Out of scope

MoneyFormatter redesign, vendor edits, native locale/rendering guarantees, mail-system redesign, and earlier-profile helper availability.
