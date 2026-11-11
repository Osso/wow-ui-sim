# Quest Account Flag

`C_QuestLog.IsAccountQuest(questID)` reports the account-wide flag for a quest. The cached `Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua` consumer uses it to build account-quest text.

## What it must do

- [ ] Register the query for Game in Forever, Retail, and PTR (VERIFIED); return one boolean (INFERRED wrapper; the boolean leaf result is VERIFIED).
- [ ] Convert the required argument to a signed 32-bit integer. Accept numeric strings and truncate finite fractions toward zero.
- [ ] Raise errors for missing, nil, non-numeric, non-finite, or out-of-range values. Out-of-range errors report the signed 32-bit limits.
- [ ] Return false for IDs below 1, unknown quests, data that is not loaded, and known quests without an account flag. Return true only for a known quest with the account flag.
- [ ] Do not use combat or caller taint to select the result.
- [ ] Keep the simulator result backed by the supplied quest record; absent records and absent flags return false.

R27 native probe: registration in Forever Game and the false/true result decisions in Forever 1.60.1.70707, Retail 12.1.5.70688, and PTR 12.1.7.70668 are VERIFIED. Numeric-string parsing, signed 32-bit conversion, fraction truncation, and argument errors are VERIFIED in Forever; full parser parity for Retail/PTR is UNKNOWN. A deferred quest-data request for unknown or unloaded quests is INFERRED; no follow-up event is proven.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Quest reward item inputs](quest-log-reward-items.md)

## Implementation inventory

- `src/c_api/c_quest_log.rs`: registers the query and reads the supplied quest-account flag.
- `src/c_api/helpers.rs`: converts required untainted signed 32-bit arguments.
- `src/lua_api/state`: stores the supplied quest rows and account flags.

## Tests asserting this spec

- `tests/user_defaults_batch2.rs::user_defaults_batch2_quest_watch_membership`: checks numeric strings, fractional conversion, errors, and supplied false/true values.
- `tests/user_defaults_batch2.rs::user_defaults_batch2_world_map_quest_consumer`: checks the original `QuestUtil.GetAccountQuestText` consumer.

## Known gaps (current cycle)

- [ ] Native deferred loading for unknown or not-yet-loaded quest data and any follow-up event remain UNKNOWN.
- [ ] Targeted Forever tests remain pending slot 3 admission.

## Out of scope

- Quest-data transport and deferred-request lifecycle; the simulator uses supplied quest state.
