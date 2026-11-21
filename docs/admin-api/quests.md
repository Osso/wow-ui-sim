# Quests

`A_Admin` functions are simulator-only, not native WoW APIs.

## SetMinimapQuestBlobInside

Retail, PTR and Forever: `A_Admin.SetMinimapQuestBlobInside(questID, inside)` supplies a boolean containment observation for the quest. It does not emit an event or calculate geometry. An unsupplied quest uses the simulator's false scenario default; the native default is unknown. See the [contract](../specs/minimap-quest-blob-observation.md).

## SetQuestObjectives

Retail/Forever only: `A_Admin.SetQuestObjectives(questID, rows, percentage?)` replaces
an existing active rich quest's ordered objective snapshot. Rows supply `text`,
`type`, boolean `finished`, nonnegative integer `numFulfilled`/`numRequired` and
optional positive integer `criteriaID`; percentage is optional finite 0–100.
Returns changed-state boolean; invalid typed data errors before mutation. It does
not infer progress, grant rewards or create membership. Identical input is inert.
[Contract](../specs/quest-objective-progress.md) owns projections, declared bootstrap
cardinality and committed-before-notify behavior. Both-profile behavioral RED is
recorded; producer GREEN remains pending.

## SetQuestProgressRequirements

Retail/Forever only: `A_Admin.SetQuestProgressRequirements(questID, requirements)` supplies progress text, required money, ordered item and currency rows, required-item visibility, and turn-in portrait inputs for an existing active quest. All top-level fields are required: `money`, `hideRequiredItems`, `text`, `portraitDisplayID`, `portraitName`, `portraitText`, `items`, and `currencies`. Money and portrait display ID are nonnegative integers; the visibility fields are booleans; text and portrait names are strings. Each item row has string `name` and `texture`, positive integer `requiredAmount`, and boolean `hidden`. Each currency row has positive integer `currencyID` and `texture`, string `name`, nonnegative integer `quality`, and positive integer `requiredAmount`. Returns `true` only when valid input changes the stored snapshot; invalid data errors before mutation, and identical data returns `false`. It does not discover requirements or grant items, currency, money, or rewards. [Contract](../specs/quest-progress-requirements-empty.md) covers the original Blizzard progress-panel consumer and proof boundary.

## SetQuestReadiness

Retail/Forever only: `A_Admin.SetQuestReadiness(questID, ready, autoComplete)`
supplies active completion for non-objective conditions and configures opt-in
completion notification. Unknown/failed rows are rejected; permanent history is
independent. Nonempty later objective snapshots replace supplied completion.
A first ready opted-in state notifies once per active lifetime; it never turns in
or grants rewards. [QL-2 contract](../specs/quest-readiness-dialog.md) owns dialog
eligibility, committed-before-notify projections and proof boundaries.

## TurnInSelectedQuest

```lua
local changed = A_Admin.TurnInSelectedQuest()
```

Turn in the selected existing rich quest row only when complete and not failed. Returns `true` on commitment; `false` for absent/invalid selection, missing row, incomplete or failed quest. Select through existing `SelectActiveQuest`, `C_GossipInfo.SelectActiveQuest` or quest-log selection paths. Completion must already be supplied in the simulator model; this endpoint does not progress objectives.

Commit removes active rich/light membership and modeled Forever ordinary-watch metadata, records existing completion history, clears the consumed selection/matching pending offer and current matching gossip rows. Unrelated offers/membership/history remain. Original synchronous listeners then receive `QUEST_TURNED_IN(questID, 0, 0)`; zero XP/money is explicit simulator policy, not a native reward amount. No inventory/currency/reward grants occur. Repeating the call or reselecting the removed quest returns `false` without an event.

Task active/map/time projections suppress completion-history IDs; catalog metadata is not erased. [Owning contract and proof status](../specs/quest-turn-in-model.md) distinguish supplied-model fixtures from native behavior and current integration evidence.

Proof status: producer committed as `b5dd5947a`; GREEN has not been run. Tests were authored in `96ba1ba4d`; receipt snapshot `b7e380f2` is not the test-authoring commit or executed runtime source. Accepted shared Retail RED is 0 PASS/3 FAIL, exit 101, against retained pre-producer source `bbeac6213`; later assertions remain unreached. The fourth, Forever-only watch/event-order/reentry/repeat case was not executed. See the owning contract for receipt and accepted audit paths.
