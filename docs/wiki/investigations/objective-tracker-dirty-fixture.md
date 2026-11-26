# ObjectiveTracker dirty state in the startup-shaped fixture

The ObjectiveTracker consumer test left its container dirty because its minimal addon fixture omitted the original `Blizzard_SplashFrame` provider. The original AutoQuest popup update then errored before quest-watch enumeration. The timer drain and frame-visibility paths were not the cause established by this reproduction.

## Evidence and fix

The pinned Retail and WoW Forever `Blizzard_AutoQuestPopUpTracker.lua` calls `SplashFrame:IsShown()` at line 14 before `Blizzard_QuestObjectiveTracker.lua` calls `BuildQuestWatchInfos()` from `LayoutContents()` at lines 354–366. `Blizzard_SplashFrame` provides the global through its game-enabled TOC. When the startup-shaped test loaded only `Blizzard_ObjectiveTracker`, the original consumer raised an attempt-to-index-nil error at line 14. `DirtiableMixin:SetDirtyMethod` clears `self.dirty` only after the update method returns, so the error left the container dirty and later quest events could not schedule a new callback.

The test now loads both original addons before settling startup. It supplies seeded quests 80000 and 80001 through `C_QuestLog.AddQuestWatch`, sends `QUEST_WATCH_LIST_CHANGED` through `A_Admin.FireEvent`, drains timers and OnUpdate ticks, then checks the original tracker blocks and titles. It does not change Blizzard source, timer scheduling, visibility, or product APIs.

**INFERRED (R88g8 (native evidence pending)):** native Forever title formatting may differ from this simulator fixture; the simulator currently emits `[80] ` before these two seeded titles. The test asserts the observed simulator strings only.

## Sources

- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ObjectiveTracker/Blizzard_AutoQuestPopUpTracker.lua`
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_ObjectiveTracker/Blizzard_QuestObjectiveTracker.lua`
- `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_SharedXML/MixinUtil.lua`
- [Regression test](../../../tests/objective_tracker_dirty.rs)
- [Event system](../systems/event-system.md)

## See Also

- [[addon-loading]] — addon discovery and startup-shaped fixture scope
