# Tooltip owner lifecycle

Commit `6da58da9f` moves `GameTooltip` owner release into the shared explicit visibility transition: `Hide()` and `SetShown(false)` release the owner of a shown `GameTooltip` without clearing its lines. `ClearLines()` retains ownership, and hiding an ordinary `Frame` does not affect an owned tooltip. This is simulator source/test evidence corroborated by current API documentation, not native Wrath execution.

## Evidence

`show_or_hide()` sets the requested visibility, then clears both `Frame.tooltip_owner_id` and `TooltipData.owner_id` only when hiding a `GameTooltip`. `FadeOut()` now delegates to that shared path. The guard is widget-type-specific, so the same shared hide operation on an ordinary `Frame` does not release `GameTooltip` ownership.

The retained `ClearLines()` path only empties tooltip lines; it does not clear either owner field.

The proof ledger records the normal `gui,client-wrath` grouped integration target at `6da58da9f`: 8/8 filtered matches pass for four unique owner cases (each included twice by the grouped target), and the existing `FadeOut` case passes 2/2. These prove simulator behavior at that revision. They do not establish native semantics, callback order, or inherited/effective visibility: no claim is made for a tooltip made effectively hidden by an ancestor.

## Coverage matrix

| Behavior | Evidence | Status |
| --- | --- | --- |
| Shown tooltip `Hide()` releases owner without clearing lines | Focused source test | Simulator-source GREEN |
| Shown tooltip `SetShown(false)` releases owner | Focused source test | Simulator-source GREEN |
| `ClearLines()` retains owner | Focused source test | Simulator-source GREEN |
| Ordinary `Frame:Hide()` leaves tooltip ownership alone | Focused source test | Simulator-source GREEN |
| Native Wrath behavior and callback order | No native execution | Unverified |
| Ancestor-driven effective hiding | Not tested or claimed | Unverified |
| Final independent verification | Pending | Pending |

## Sources

- [Tooltip owner lifecycle spec](../../specs/tooltip-owner-lifecycle.md) — contract and exclusions, including the current GameTooltip API reference.
- [Visibility transition](../../../src/lua_api/frame/methods/core_state/visibility.rs) — shared owner-release implementation at `6da58da9f`.
- [Tooltip owner methods](../../../src/lua_api/frame/methods/widgets/tooltip/owner.rs) — `FadeOut` delegation at `6da58da9f`.
- [Tooltip tests](../../../tests/tooltip_basic.rs) — owner and ordinary-frame controls at `6da58da9f`.
- `/tmp/wrath-tooltip-owner-proof.md` — exact build/test commands, RED boundary, and current focused results.
- [GameTooltip API reference](https://warcraft.wiki.gg/wiki/UIOBJECT_GameTooltip) — documented `Hide` owner clearing and `ClearLines` retention guidance; documented source, not native execution.

## See Also

- [[wrath-statusbar-value-callback]] — Stage 2 starts tooltip lifecycle work after StatusBar scope.
- [[tooltip-layout-timing]] — separate tooltip sizing/render timing issue.
- [[lua-api]] — simulator Lua API boundary.
