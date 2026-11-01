# Supplied challenge run and scenario criteria

## Contract

The environment owns at most one active challenge run. Admin input supplies its challenge-map ID, event map ID, key level, affix IDs, charged flag, deaths and time lost. The event map ID is explicit input; its identity is not inferred from the challenge-map ID or UI-map ID. No catalogue or difficulty policy is inferred. Starting stores the snapshot before `CHALLENGE_MODE_START(eventMapID)`. Clearing removes the active run and its timers before `WORLD_STATE_TIMER_STOP(timerID)` and `CHALLENGE_MODE_RESET(eventMapID)`. Completion first stores the supplied completion snapshot, removes the run/timers, then emits `CHALLENGE_MODE_COMPLETED`.

Map metadata is an environment-local supplied catalogue. Existing metadata is not expanded with invented entries. A missing supplied map has no result on the modeled profiles. Ordered criteria belong to a supplied scenario step. Updates replace that step's ordered rows before one `SCENARIO_CRITERIA_UPDATE(criteriaID)` per changed supplied row. Returned records and affix arrays are detached copies. The modern namespace is used; retired `C_Scenario.GetCriteriaInfo*` members remain retired.

World elapsed timer records belong to the active run and contain the supplied timer ID, display name, type and elapsed seconds. They are exposed before `WORLD_STATE_TIMER_START(timerID)`. Set/advance are explicit, nonnegative finite inputs; no wall clock is used. Missing timers have no result. Run reset/completion removes its timers.

## Pinned declarations and publication

Paths below are relative to each profile's pinned `AddOns/Blizzard_APIDocumentationGenerated/` cache; the committed manifests select the inputs.

| Profiles | File and lines | Members |
|---|---|---|
| Retail, PTR, Forever | `ChallengeModeInfoDocumentation.lua:34,43,81,154,163,273` | `GetActiveChallengeMapID`, `GetActiveKeystoneInfo`, `GetDeathCount`, `GetMapTable`, `GetMapUIInfo`, `IsChallengeModeActive` |
| Mists, Era, Anniversary | `ChallengeModeInfoDocumentation.lua:11,129,138,202` | `GetActiveChallengeMapID`, `GetMapTable`, `GetMapUIInfo`, `IsChallengeModeActive`; no key-info or death-count additions |
| Retail, PTR, Forever | `ScenarioInfoDocumentation.lua:11-41,263-284` | modern criteria readers and 14-field record |
| Mists, Era, Anniversary | `ScenarioInfoDocumentation.lua:11-39,256-277` | same modern criteria readers and record |
| Wrath | No corresponding pinned files | No new namespace members |

The already-present elapsed-timer globals keep their publication scope. Wrath is excluded from routing and retains its pre-existing ungrounded `(id, 0, 0)` placeholder; native Wrath elapsed-timer behavior requires a separate lane. `WorldStateInfoDocumentation.lua:64-83` declares start/stop payloads; `WorldElapsedTimerConstantsDocumentation.lua:14` declares ChallengeMode=1 on modern profiles. Admin input supplies timer types; it does not infer them.

## Map metadata validation

`SetChallengeModeMapInfo` accepts a supplied table. `name`, `id`, `timeLimit`, `backgroundTexture`, and `mapID` are required. Only `texture` can be omitted or nil; the getter returns nil in its fourth result without substituting an asset. The fifth result is the supplied numeric `backgroundTexture`. Retail 70448, PTR 70493 and Forever declare these two numeric results at `ChallengeModeInfoDocumentation.lua:178-179`; Mists, Era and Anniversary declare the same nilability at lines 152-153.

Invalid metadata errors identify `mapInfo.<field>`, including missing required fields, wrong types, integer conversion failures and invalid time limits. They do not expose the temporary bridge stack index `argument -1`. A rejected call does not replace previously supplied metadata. The original ChallengesUI dungeon-icon consumer must observe both supplied texture values and nil after texture omission. No native dungeon-to-texture relationship is inferred.

## Selected consumers

Retail selects `Blizzard_ObjectiveTracker.toc` and `Blizzard_ChallengesUI_Mainline.toc`. PTR selects unified `Blizzard_ObjectiveTracker.toc` and `Blizzard_ChallengesUI.toc` (Standard family). Forever selects the objective tracker but its `camelot` game type excludes the ChallengesUI TOC (`standard, mists` only). Scenario criteria and timer proof use the unmodified selected `Blizzard_ScenarioObjectiveTracker.lua`, original XML frames and method identity. Mists has a Mists ChallengesUI and legacy WorldStateFrame timer consumer; Era/Anniversary exclude that mists-only TOC. Wrath is excluded.

## Inactive keystone boundary

The inactive `GetActiveKeystoneInfo` return is unimplemented pending native evidence. Its declaration has three non-nil outputs, and selected callers use them without an active-run guard. The simulator reports this gap explicitly instead of inventing a tuple. `GetDeathCount` returns nothing when inactive; the active-map query returns nil.

## InTimeBL boundary

Core reads map ID and active flag on addon load/start/resync, key level and deaths on start, `(name, elapsed)` from world timer 1, and map time limit from result 3. Tracker reads the current scenario step count and each indexed criteria record; it distinguishes weighted forces from boss criteria (165, or 0 with total=1), uses `quantityString`, `totalQuantity`, `assetID`, completion and description. Recorder reads the last completion snapshot. Forecast refresh uses a one-second `C_Timer.NewTicker` and `GetTime()-startTime`, so a controllable common clock is required for deterministic minutes-long acceptance.

## Criteria rendering boundary

Original modern ScenarioObjectiveTracker initialization calls `C_Scenario.ShouldShowCriteria` unguarded at line 74. Neither pinned Retail nor PTR generated documentation declares this member or its namespace, and neither pinned addon tree defines a Lua alias. Original criteria rendering remains blocked before its query producer. Native clarification and any missing producer belong to a separate lane. Do not swallow the initialization error, replace vendor behavior or report direct query tests as rendering proof.

## Proof requirements

Each original-consumer selector must contradict constant/no-op producers independently. Preserve byte-identical tests on a producer-subtracted RED sibling. Add external InTimeBL forecast acceptance without changing its sources. Final gates include all seven zero-warning checks, default-mode Retail/PTR stock with bound binary/revision/argv/receipt, individual added-key tracing, formatting and changed-code readability.

## Scenario criteria visibility

- [x] `C_Scenario.ShouldShowCriteria()` (Retail/PTR; undocumented native read by `ScenarioObjectiveTracker:InitModule`) returns `true` unless `A_Admin.SetScenarioCriteriaShown(false)` supplied otherwise.
- [x] `A_Admin.SetScenarioCriteriaShown(shown)` stores the flag and fires `SCENARIO_CRITERIA_SHOW_STATE_UPDATE(shown)`.
- Best-guess basis: best-guess, native UNKNOWN. Standard scenarios (Delves, Mythic+) show objectives; no hidden-criteria scenario data is modeled.
