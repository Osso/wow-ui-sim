# Forever Chat Overflow Button-Slot Animations

Forever chat startup failed in `FCFDockOverflowButton_UpdatePulseState` because the DockManager overflow button's authored `HighlightTexture` animation group was not created. The simulator created the texture slot but omitted its XML-owned animations; this was not a missing chat-frame `minFrame`, tab glow, or child lifecycle-order bug.

## Content

## Symptoms

The pinned startup reported `ChatFrameUtil.StopFlash` with a nil `animGroup` for `ChatFrame1` and `CombatLogQuickButtonFrame_Custom`.

Initial hypotheses about `chatFrame.minFrame.glow.FlashAnim`, tab glow, and parent/child `OnLoad` ordering were falsified:

- post-load `ChatFrame1.ScrollToBottomButton.Flash.FlashAnim` and `ChatFrame1Tab.glow.FlashAnim` existed;
- normal XML finalization completes children before the parent lifecycle;
- replaying the real `OnLoad` handler captured the failing stack at `FloatingChatFrame.lua:2761`.

That stack enters `FCFDockOverflowButton_UpdatePulseState`, which calls `ChatFrameUtil.StopFlash(highlight, highlight.FlashAnim, true)`. The overflow button's `HighlightTexture` existed but lacked its authored `FlashAnim`.

## Root Cause

`loader/button.rs` created XML button texture slots without applying animation groups owned by those texture regions. The DockManager template defines `FlashAnim` on its `HighlightTexture`, so the consumer received the correct texture identity but an incomplete region object.

## Fix and Proof

`13ca52c1c` applies the existing animation-group generator to the existing XML button texture slot. It does not change Blizzard Lua or add a consumer-side nil guard.

The targeted RED command reproduced one actual DockManager overflow `HighlightTexture` without `FlashAnim`. The GREEN suite passed 4/4 at `016aec606`:

- exactly one animation group belongs to the existing highlight texture;
- `FlashAnim` identity and owner are correct;
- `StopFlash` works before the consumer unlocks the highlight;
- unchanged `FCFDockOverflowButton_UpdatePulseState` stops the animation and preserves the expected post-unlock state.

`32c666fd4` records the proof specification. Evidence is in `/tmp/forever-overflow-ledger.json`.

## Sources

- [button.rs](../../../src/loader/button.rs) — XML button texture-slot construction and animation application
- `~/.cache/wow-ui-sim/blizzard-ui/wowforever/AddOns/Blizzard_ChatFrameBase/Mainline/FloatingChatFrame.lua` — overflow pulse consumer
- `/tmp/forever-overflow-ledger.json` — RED/GREEN commands and results
- [Forever running report](../../wowforever-1.60.1.md) — pinned startup evidence

## See Also

- [[xml-template-system]] — XML templates and region ownership
- [[widget-system]] — texture regions and frame ownership
- [[chatframe-scrollbar-anchor-reapply]] — separate ChatFrame anchor defect
