# Tooltip owner lifecycle

Commit `1e9674bcd` completes the Stage 2 direct tooltip content/owner lifecycle source slice: `SetOwner` internally hides and clears content while retaining its new owner; populated `SetText` shows; line appends remain hidden; and explicit `Hide`, `SetShown(false)`, and `FadeOut` release ownership even when the tooltip is already hidden. `ClearLines()` retains ownership, and hiding an ordinary `Frame` does not affect an owned tooltip. This is simulator source/test evidence under normal `gui,client-wrath`, not native Wrath execution.

## Evidence

`SetOwner` writes the new owner and anchor into both owner representations, clears tooltip content, and performs its hide internally. That internal hide retains the new owner. `SetText` replaces the tooltip text and shows an owned tooltip; `AddLine` and `AddDoubleLine` add lines without showing it. Payload population such as `SetSpellByID` still shows after `SetOwner`.

The shared explicit visibility path clears both `Frame.tooltip_owner_id` and `TooltipData.owner_id` for a `GameTooltip` before testing whether its local visibility changes. Thus direct `Hide()`, `SetShown(false)`, and `FadeOut()` release ownership even after `SetOwner` has already made the tooltip locally hidden. The widget-type guard leaves ordinary-frame hiding unrelated to tooltip ownership. `ClearLines()` remains content-only and retains both owner fields.

The proof ledger records normal `gui,client-wrath` grouped integration at `1e9674bcd`: `tooltip_basic::` is GREEN 62/62. Independent format/check pass; check has no warnings. Wider filters initially passed 75 and failed seven instances. Five failures came from three fixtures assuming `SetOwner` showed the tooltip; `12734d0ae` explicitly shows populated tooltips before sizing/cursor tracking, and independent focused reruns pass 5/5. The other two instances are one full-UI case requiring absent `BuffFrame.UpdateAuras`; it remains failing, without preexisting attribution or a full-green claim. This establishes the simulator contract at that revision. The current GameTooltip API page is an independent documentation source for owner, replacement-text, and append visibility behavior, but it is not native execution. No native Wrath result exists. No claim is made about callback order or an effectively hidden tooltip whose ancestor is hidden.

## Coverage matrix

| Behavior | Evidence | Status |
| --- | --- | --- |
| `SetOwner` hides and clears content while retaining its new owner/anchor | `tooltip_basic::` in normal `gui,client-wrath` | Simulator-source GREEN |
| Owned populated `SetText` shows; `AddLine`/`AddDoubleLine` append without showing | Same 62/62 batch | Simulator-source GREEN |
| Payload population still shows after `SetOwner`; `ClearLines()` retains owner | Same 62/62 batch | Simulator-source GREEN |
| Direct `Hide`, `SetShown(false)`, and `FadeOut` release owner even when already hidden | Same 62/62 batch | Simulator-source GREEN |
| Ordinary `Frame:Hide()` leaves tooltip ownership alone | Same 62/62 batch | Simulator-source GREEN |
| Native Wrath behavior | No native execution | Unverified |
| Callback order and ancestor-driven effective hiding | Not tested or claimed; internal `SetOwner`/`SetText` visibility changes bypass visibility script dispatch | Unverified lifecycle edges, not native parity |
| Format/check and corrected visible fixtures | Independent normal-profile checks pass; five corrected sizing/cursor fixture instances pass | Scoped PASS; unrelated test-file length finding retained |
| Full-UI aura/tooltip case | Two duplicated instances fail at `tests/tooltip_full_env_helpers.rs:208`, missing `BuffFrame.UpdateAuras` | Current integration blocker; not attributed to these commits without baseline proof |

## Sources

- [Tooltip owner lifecycle spec](../../specs/tooltip-owner-lifecycle.md) — contract and exclusions, including the current GameTooltip API reference.
- [Visibility transition](../../../src/lua_api/frame/methods/core_state/visibility.rs) — direct explicit-hide owner release at `1e9674bcd`.
- [Tooltip owner methods](../../../src/lua_api/frame/methods/widgets/tooltip/owner.rs) — owner retention across internal `SetOwner` hiding and `FadeOut` delegation at `1e9674bcd`.
- [Text methods](../../../src/lua_api/frame/methods/text_attribute_event/text.rs) and [tooltip content](../../../src/lua_api/frame/methods/widgets/tooltip/content.rs) — replacement-text and payload visibility at `1e9674bcd`.
- [Tooltip line data](../../../src/lua_api/frame/methods/widgets/tooltip/line_data.rs) — append-without-show behavior at `1e9674bcd`.
- [Tooltip tests](../../../tests/tooltip_basic.rs) — grouped `tooltip_basic::` source proof at `1e9674bcd`.
- `/tmp/wrath-tooltip-content-lifecycle-proof.md` and its full referenced logs — exact RED/GREEN commands and 62/62 result.
- [GameTooltip API reference](https://warcraft.wiki.gg/wiki/UIOBJECT_GameTooltip) — independent API documentation, not native execution.

## See Also

- [[wrath-statusbar-value-callback]] — Stage 2 starts tooltip lifecycle work after StatusBar scope.
- [[tooltip-layout-timing]] — separate tooltip sizing/render timing issue.
- [[lua-api]] — simulator Lua API boundary.
