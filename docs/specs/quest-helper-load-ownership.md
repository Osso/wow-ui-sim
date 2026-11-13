# B(a) QuestCache / QuestUtil load ownership

Source-only investigation on frozen review base
`1e51c302f54bf0bfc26d054ecd20a6861b544fbf`. No production edits or builds.
The objective-header slice remains frozen separately on `dot/stub-b-fixes`.
This preparation is on `dot/stub-b-quest-load-gap`, still task lt-2897.

## Static route found, not a missing dependency

Pinned Retail 12.1.5.70575 has the following ordinary dependency chain:

- ObjectiveTracker.toc:6 -> UIWidgets.
- UIWidgets.toc:3 -> Minimap.
- Minimap_Mainline.toc:4 -> ActionBar and FrameXMLUtil.
- ActionBar.toc:5 -> FrameXML.
- FrameXML.toc:2 -> ObjectAPI.
- ObjectAPI_Mainline.toc:4-5 allows Game/mainline; :10 selects
  Mainline/ObjectCache.lua and :16 selects Mainline/Quest.lua.
- FrameXMLUtil.toc:5 allows Game; :10 selects [Family]/QuestUtils.lua.
  Retail's family is Mainline.
- ObjectiveTracker.toc:35-36 selects the quest tracker Lua/XML.

Paths above are under their Blizzard_<addon> cache folders. The simulator's
loader/mod.rs:163-166 additionally supplies ObjectiveTracker -> POIButton;
it does not need to invent a direct ObjectAPI dependency when this existing
transitive chain is intact. Loader/mod.rs:729-754 traverses declared dependencies
and applicable optional dependencies. This static result contradicts the initial
suspicion that the ordinary tracker closure simply omits ObjectAPI/QuestUtils.
It does not prove the files successfully execute before every runtime consumer.

Original ObjectAPI/Mainline/Quest.lua:111 unconditionally publishes QuestCache
using ObjectCache_Create. ObjectCache.lua:5-17 retains object identity and calls
CheckRefresh on existing objects. Quest.lua:25-38 copies C_QuestLog.GetInfo fields.
QuestUtils.lua:358-364 owns QuestShowsItemByIndex; :367-379 owns the group-query
cache. The guarded bootstrap fallback in quest_objective_defaults.rs:37-100
should consequently be replaced by these originals before ordinary consumers.

## Next permitted runtime investigation, not executed

`tests/quest_helper_load_ownership.rs` is a Retail-only optional preservation
fixture. It loads the unmodified discovered tracker closure with no overrides,
requires ObjectAPI/FrameXMLUtil membership, records function provenance and runs
the real QuestObjectiveTracker:BuildQuestWatchInfos against existing watched
inputs. It requires nonempty consumption and original cache identity. It neither
removes a dependency/global nor fabricates a native producer to force RED.

Expected outcome is ownership/preservation, not a demonstrated bug. If it fails,
classify the first error: dependency discovery, original file execution, missing
native prerequisite, or actual fallback reachability. Record the exact selected
file/call chain and error. Do not replace functions or suppress errors to get a
pass. A genuine load gap needs a normal startup/runtime path reaching the
fallback, not a synthetic fixture that deliberately omits the original owner.

QuestUtil provenance checks are ownership only. They do not prove populated
CanCreateQuestGroup/QuestShowsItemByIndex native behavior or group-cache
invalidation. The real tracker right-edge consumers are
Blizzard_QuestObjectiveTracker.lua:273-276; exercising them requires actual
selected button/pool state and supported item/group inputs, not fabricated sinks.

## Exact native/model blocker for a populated flags test

The proposed differing quest flags are not existing supplied inputs:
`src/lua_api/globals/quest_surface/info_fields.rs:35-36,41` currently writes
isTask/isBounty/isAutoComplete as false for C_QuestLog.GetInfo, independently of
which QuestCache implementation is loaded. `QuestLogEntry` in
src/lua_api/sim_substates/quest_log.rs:17-61 has no supplied fields for these three.
A fake true flag would manufacture a producer. Original Quest.lua:25-38 copies
that result; tracker Lua:371-376 reads task/bounty and :63-69 reads autocomplete.
Replacing only QuestCache would not establish native flag behavior.

QUESTION, if not already owned/answered by the quest-log work: for Retail
12.1.5.70688 C_QuestLog.GetInfo(questLogIndex), identify the authoritative
isTask/isBounty/isAutoComplete sources and mappings for absent index/header,
ordinary quest, task, bounty and auto-complete quest, including transitions.
Give index validation (omitted/nil/type/out-of-range/extra), result count/nilability,
flag validity/lifetime, server/DB2/quest-record source and refresh events/order.
Generated QuestLogDocumentation.lua:190-204 declares nullable QuestInfo from a
nonnull luaIndex, and :1531-1538 declares these bool fields, but not their values.
Do not infer native values from the current false writes or duplicate an existing
owned native investigation. No new flag model is implemented here.

## Remaining B sequence

(a) First obtain honest normal ownership/load-gap evidence as above; do not claim
RED from provenance alone. (b) Determine whether any selected consumer reaches
the enum fallback before original publication; otherwise removable+preservation
test, or KEEP if real use is not settled. (c) QuestLog parent/event order remains
an exact native/lifecycle question, no guessed guard changes. (d) Encounter color
has no selected consumer: KEEP seam, no implementation fix. The overall task is
not closed by the already reviewed objective-header slice.
