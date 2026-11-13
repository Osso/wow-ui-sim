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

## Executed negative finding and remaining B plan (2026-11-13)

At `9c1fc20e4de06cf65c73c64a0f559c953a37f9ef`, based on merged
`ed4eef3c761b4a853b9aeaf17683a798049dbeda`, Retail lib+integration no-run
compiled successfully with zero compiler warnings. The one normal-closure
ownership test passed (1/1). Source/document hashes and the executed integration
binary hash were reverified against receipts. No normal QuestCache/QuestUtil load
gap was reproduced. This is preservation, not a production RED; it does not prove
populated native quest flags. No QuestCache/QuestUtil replacement is removed.
Untracked receipts: `wow-build-tools/stub-b-receipts/deferred-9c1fc20e4de06cf65c73c64a0f559c953a37f9ef/ownership`.

### B(b): enum fallback, source-only removal candidate

`env_init/mod.rs:61` calls `init_enum_globals` before temporary bootstrap at82.
`env_init/enums.rs:257` executes `missing_enums.lua`; its lines11549-11563
supply QuestClassification, including Campaign=2 and Calling=3. Temporary
`quest_objective_defaults.rs:38-41` therefore cannot install its -1/-2 defaults
in the normal initialized environment. The actual original quest helper closure
already passes; it never requires manufacturing an absent Enum table.

Plan: add a normal-environment preservation assertion that Campaign=2 and
Calling=3 before addon load and after actual original tracker closure, without
nil assignments, fake globals or artificial missing-provider inputs. Then remove
only the redundant four Enum setup lines38-41, keeping the QuestCache adapter
and other profiles' behavior. The known provider runs without profile gating;
verify the normal bootstrap assertion across profiles before accepting a shared
removal. This is unreachable-fallback removal plus preservation, not native RED.
No removal/test edit or additional build has yet occurred at this checkpoint.

### B(c): lifecycle evidence question, no guessed fix

No dedicated QuestLog parent/event ordering contract was found in docs/specs.
Pinned Retail original `Blizzard_UIPanels_Game/Mainline/QuestMapFrame.lua:278-284`
requires a parent in GetCurrentMapID; :858-874 does so in UpdateAll. Original
`Blizzard_WorldMap/Blizzard_WorldMap.lua:152` invokes AttachQuestLog; :541-547
sets QuestMapFrame's parent to the map. Temporary `quest_log_mixin.rs:11-42`
permits missing-parent refresh and bypasses original callback work.

QUESTION for Retail12.1.5.70688: In the real startup and load-on-demand lifecycle,
what ordering exists between QuestMapFrame creation/OnLoad, WorldMap OnLoad and
AttachQuestLog/SetParent, and first QUEST_LOG_UPDATE/UNIT_QUEST_LOG_CHANGED and
refresh dispatch? Can either original GetCurrentMapID or UpdateAll legitimately
run before parenting, and if so what guarantees a nonnil parent? For no-parent,
default hidden parent and shown populated parent, establish native CreateFrame,
GetParent/SetParent visibility callback and event-queue ordering, including
reentrant dispatch. Identify which facts are native frame/event behavior versus
original Lua order and which quest server state triggers events. Do not invent a
native QuestLogMixin wrapper; return exact verified primitives/lifecycle facts
and mark unavailable observations. Existing C_Map result semantics are separate.

### B(d): retained seam

Keep encounter color bookkeeping unchanged: no selected original consumer was
established for the inventoried color setter. Absence of that consumer does not
certify the public setter's validation or native behavior. No opportunistic fix.

## Native evidence disposition (coordinator #713)

The three current false GetInfo flags are KEEP neutral seams because supplied
quest rows have no corresponding fields, not certified native false results.
Coordinator #713 reports PTR12.1.7 C_QuestLog.GetInfo.md VERIFIED absent/nil
out-of-range, isHeader mask0x20 and serializer27fields including these flags;
other values, omission and validation are UNKNOWN. IsQuestTask/IsQuestBounty
predicate record meanings remain UNKNOWN. Retail70688 mapping, predicate
agreement and refresh are assigned pending R88; do not duplicate the question
or implement flags from PTR partial evidence. B(a) remains preservation only.

B(b) tests now assert Campaign2/Calling3 in an ordinary newly initialized
environment on every profile and after the actual original Retail dependency
closure. No Enum deletion or fake input. Baseline and post-removal tests are
preservation; no production RED is expected for an unreachable fallback.
