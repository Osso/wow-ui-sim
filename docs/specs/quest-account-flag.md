# Quest Account Flag

`C_QuestLog.IsAccountQuest(questID)` reports the account-wide flag for a quest. The cached `Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua` consumer uses it to build account-quest text and calls [`GetQuestFactionGroup`](quest-faction-group.md) for faction-specific labels.

## What it must do

- [ ] Forever registers the query in Game (VERIFIED); return one boolean (INFERRED wrapper; the boolean leaf result is VERIFIED).
- [ ] In Forever, convert the required argument to a signed 32-bit integer. Accept numeric strings and truncate finite fractions toward zero.
- [ ] In Forever, raise errors for missing, nil, non-numeric, non-finite, or out-of-range values. Out-of-range errors report the signed 32-bit limits.
- [ ] Retail and PTR use the same false/true result decisions. Their parser parity is UNKNOWN; keep the existing simulator parser there without claiming native behavior.
- [ ] Return false for IDs below 1, unknown quests, data that is not loaded, and known quests without an account flag. Return true only for a known quest with the account flag.
- [ ] Do not use combat or caller taint to select the result.
- [ ] Keep the simulator result backed by the supplied quest record; absent records and absent flags return false.

R27 native probe: Forever Game registration and the false/true result decisions in Forever 1.60.1.70707, Retail 12.1.5.70688, and PTR 12.1.7.70668 are VERIFIED. Numeric-string parsing, signed 32-bit conversion, fraction truncation, and argument errors are VERIFIED in Forever; Retail/PTR parser parity is UNKNOWN. A deferred quest-data request for unknown or unloaded quests is INFERRED; no follow-up event is proven.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Quest reward item inputs](quest-log-reward-items.md)
- [Quest faction group](quest-faction-group.md)

## Implementation inventory

- `src/c_api/c_quest_log.rs`: registers the query and reads the supplied quest-account flag.
- `src/c_api/helpers.rs`: converts required untainted signed 32-bit arguments.
- `src/lua_api/state`: stores the supplied quest rows and account flags.

## Tests asserting this spec

- `tests/user_defaults_batch2.rs::user_defaults_batch2_quest_watch_membership`: checks supplied false/true values.
- `tests/user_defaults_batch2.rs::forever_account_quest_argument_conversion_matches_native`: checks Forever numeric strings, fractional conversion, errors, and result arity.
- `tests/user_defaults_batch2.rs::user_defaults_batch2_world_map_quest_consumer`: checks the original `QuestUtil.GetAccountQuestText` consumer.

## Known gaps (current cycle)

- [ ] Native deferred loading for unknown or not-yet-loaded quest data and any follow-up event remain UNKNOWN.
- [ ] Retail/PTR parser parity remains UNKNOWN; the simulator retains the earlier parser path on those profiles.
- [ ] Targeted Forever tests remain pending slot 3 admission.

## Out of scope

- Quest-data transport and deferred-request lifecycle; the simulator uses supplied quest state.
