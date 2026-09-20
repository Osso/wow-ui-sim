# Forever clean sustained runtime

Forever `1.60.1.69913` now loads and ticks its matching Blizzard UI without collected Lua errors. Startup-only proof was insufficient: a sustained GUI run later exposed a repeated `Blizzard_WorldMap` `OnUpdate` failure.

## Root cause

`WorldMapMixin:OnShow()` calls the Camelot quest-map count refresh before `SetMapID()`. `QuestLogQuests_ShowQuestCount()` compares the current quest count with `Constants.QuestLogConsts.MAXIMUM_NUM_QUESTS_LOG_CAN_ACCEPT`.

The simulator omitted that source-published constant. The comparison aborted `OnShow`, so `SetMapID()` never initialized `WorldMapFrame.ScrollContainer.targetScale`. Later scroll-controller updates repeatedly failed in `IsZoomingOut()` while comparing the current scale with nil.

Commit `ed4c97a8a` publishes the Forever value `40`. No vendor comparison guard, target-scale fallback, or handler suppression was added.

## Evidence

The regression loads the full matching game UI, clears startup errors, calls `WorldMapFrame:Show()`, and performs 60 GUI-style update/timer ticks. It asserts a positive `targetScale`, quest limit `40`, and zero collected errors. Result: **1/1 passed**. Artifact: `/tmp/forever-world-map-runtime-test.{stdout,stderr}`.

A freshly built `gui,client-wowforever` binary then ran with `--no-addons --no-saved-vars` for 20 seconds. Startup completed at 6.892 seconds; the bounded run ended via timeout `124` with zero Lua error, `OnUpdate`, nil-comparison, warning, panic, or failure lines. Artifact: `/tmp/forever-gui-runtime-fixed.{stdout,stderr}`.

Formatting plus default and Forever checks passed without warnings. The earlier batch-fifteen `lua-errors` and scripted interaction proofs remain evidence for startup and those interactions, not substitutes for sustained runtime.

## Limits

This proves the reproduced WorldMap lifecycle and an idle sustained GUI session. It does not prove every optional panel, native Gamepad hardware input, native pet-storage offset, dynamic EditMode policy, or full secret-value semantics.

## Sources

- [Forever report](../../wowforever-1.60.1.md) — build identity, committed behavior, and complete proof scope
- [Forever finite constants](../../specs/forever-finite-constants.md) — source provenance and profile boundary
- [[forever-chat-overflow-slot-animations]] — separate late UI initialization root
- [[client-profiles]] — Forever profile identity

## See Also

- [[forever-chat-overflow-slot-animations]] — XML animation binding fix
- [[client-profiles]] — profile routing and cache selection
