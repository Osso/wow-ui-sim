# Challenge runs and forecast fixtures

Phase A supplies challenge state. Phase B adds an explicit manual environment clock. See the [scoped runtime evidence and blocked original criteria initialization](../wiki/systems/challenge-mode-run.md#evidence-status); no complete native-UI or clean-global-runtime claim is made.

These calls supply fixture state. They do not generate native dungeon catalogues, affixes, enemy credit, death penalties or time limits. Each environment has its own state. See the [contract](../specs/challenge-mode-run.md) for profile publication.

## Phase A: supplied state

```lua
A_Admin.SetChallengeModeMapInfo(mapChallengeModeID, mapInfo)
A_Admin.StartChallengeRun(run)
A_Admin.ClearChallengeRun()
A_Admin.CompleteChallengeRun(completionInfo)
A_Admin.ResetActiveKeystoneInfo(preserveKeystoneFields)
A_Admin.SetScenarioCriteria(stepInfo, orderedCriteria)
A_Admin.StartWorldElapsedTimer(timerID, name, timerType, elapsedSeconds)
A_Admin.SetWorldElapsedTime(timerID, elapsedSeconds)
A_Admin.AdvanceWorldElapsedTime(timerID, deltaSeconds)
```

`mapInfo` requires `name` (string), `id` (integer), `timeLimit` (finite nonnegative seconds), `backgroundTexture` (number), and `mapID` (integer). Only `texture` is optional (number or nil); omission returns nil in getter result 4. `backgroundTexture` must be supplied; the simulator does not invent a texture value. The pinned Retail/PTR `ChallengeModeInfoDocumentation.lua:178-179` declares `texture` as nilable and `backgroundTexture` as non-nilable. See the [map metadata contract](../specs/challenge-mode-run.md#map-metadata-validation).

`run` supplies `mapChallengeModeID`, `eventMapID`, `level`, `affixIDs`, `wasActiveKeystoneCharged`, `numDeaths`, and `timeLost`. A run must be clear before another start. Completion uses the documented `ChallengeCompletionInfo` fields, including milliseconds in `time`; it does not calculate a score or upgrade.

Retail, PTR and Forever also store the key level, up to four affix IDs and the charged byte as the `GetActiveKeystoneInfo` globals. A run start applies those three from `run`; `ResetActiveKeystoneInfo` zeroes them, or leaves them unchanged when `preserveKeystoneFields` is truthy. Clearing or completing a run does not change them, because the client has no writer for them on a named run event. Those profiles only; the call is absent on Mists, Era, Anniversary and Wrath. See the [keystone globals contract](../specs/challenge-mode-run.md#keystone-globals).

`stepInfo` supplies `stepID`, `title`, and `description`. Its ordered rows replace the previous rows for that step and select that step as current. Each criterion requires `criteriaID`, `description`, and `quantityString`; the other declared fields have the documented defaults: zero for numbers and false for booleans. Fields are `criteriaType`, `completed`, `quantity`, `totalQuantity`, `flags`, `assetID`, `duration`, `elapsed`, `failed`, `isWeightedProgress`, and `isFormatted`. Quantity text is supplied, not derived from quantity or locale.

Start publishes state before `CHALLENGE_MODE_START(eventMapID)`. Reset reuses the run's supplied event map ID. The native relationship between this event payload and catalogue IDs is not inferred. A timer requires an active run and is published before `WORLD_STATE_TIMER_START(timerID)`. Criteria are replaced before `SCENARIO_CRITERIA_UPDATE(criteriaID)` for each supplied row. Clear/completion removes timers before their stop events and the reset/completed event. Elapsed setters require an existing timer and finite nonnegative seconds. Advance requires a finite nonnegative delta. Neither operation changes `GetTime()`.

## Example: two progress samples

All numbers below are test fixtures, not a claim about a real dungeon. Use an InTimeBL profile's real map ID and supplied metadata for an addon forecast test.

```lua
A_Admin.SetChallengeModeMapInfo(9001, {
    name = "Forecast fixture", id = 9001, timeLimit = 1500,
    backgroundTexture = 456, mapID = 789,
})
A_Admin.StartChallengeRun({
    mapChallengeModeID = 9001, eventMapID = 789, level = 12, affixIDs = {},
    wasActiveKeystoneCharged = true, numDeaths = 0, timeLost = 0,
})
-- ChallengeMode=1 is declared in Enum.WorldElapsedTimerTypes.
A_Admin.StartWorldElapsedTimer(1, "Forecast fixture", 1, 0)

local step = {stepID = 1, title = "Fixture stage", description = "Fixture objectives"}
A_Admin.SetScenarioCriteria(step, {
    {criteriaID = 901, description = "Fixture boss", criteriaType = 165,
        quantity = 0, totalQuantity = 1, quantityString = "0"},
    {criteriaID = 902, description = "Enemy forces", isWeightedProgress = true,
        quantity = 25, totalQuantity = 100, quantityString = "25%"},
})
A_Admin.SetWorldElapsedTime(1, 300)
A_Admin.SetScenarioCriteria(step, {
    {criteriaID = 901, description = "Fixture boss", criteriaType = 165,
        quantity = 1, totalQuantity = 1, quantityString = "1", completed = true},
    {criteriaID = 902, description = "Enemy forces", isWeightedProgress = true,
        quantity = 70, totalQuantity = 100, quantityString = "70%"},
})
A_Admin.AdvanceWorldElapsedTime(1, 60)
assert(select(2, GetWorldElapsedTime(1)) == 360)
A_Admin.ClearChallengeRun()
```

## Phase B: explicit manual clock

```lua
A_Admin.SetManualClock(0)  -- explicit opt-in; do this before starting the run
A_Admin.AdvanceTime(300) -- GetTime, OnUpdate(delta), and due C_Timer callbacks
```

Real time remains the default. Opt in once, before supplying timed fixture state. Manual seconds and deltas must be finite and nonnegative. Pending C_Timer callbacks keep their remaining delay when the clock is enabled. `AdvanceTime` requires manual mode, advances the clock first, dispatches OnUpdate once with the delta, and drains due callbacks in deadline order. A positive-interval ticker can fire several times during one large advance. A zero-interval ticker runs at most once per advance. A second manual-clock opt-in fails. Callback-created timers use the already-advanced clock. Independently created duration-object manual clocks do not advance. World elapsed timers remain explicit supplied values; advance them separately when the fixture needs both clocks to agree. See the [clock contract](../specs/manual-simulation-clock.md).
