# Quest Faction Group

`GetQuestFactionGroup(questID)` reports the single-faction restriction of a supplied quest record. The record shares the cache used by [`C_QuestLog.IsAccountQuest`](quest-account-flag.md).

## What it must do

- [ ] Register as a Game global on Forever, Retail and PTR.
- [ ] Require one numeric quest ID and accept numeric strings. Check signed 32-bit range before truncating toward zero.
- [ ] Raise `Usage: GetQuestFactionGroup(questID` for a missing, nil or non-numeric argument. Raise `integer overflow attempting to store %f` for out-of-range, NaN or infinite values.
- [ ] Return zero values when the truncated ID is less than 1, the record is absent, or its faction mask does not select exactly one faction.
- [ ] Return one number only for an exact single-faction mask: `1` for Alliance and `2` for Horde. Never return `0` as a value.
- [ ] Read only the faction restriction from the supplied quest record. Do not use player faction. Missing faction data is empty by default.

## How it works

- [Quest account flag](quest-account-flag.md)
- [Quest-log state](../wiki/systems/quest-log.md)

## Implementation inventory

- `src/lua_api/globals/real/quest_faction_group.rs`: registers the global and reads the supplied record.
- `src/lua_api/sim_substates/quest_log.rs`: stores the optional faction mask on each quest record.
- `src/lua_api/globals/register.rs` and `src/lua_api/env_runtime.rs`: register the Game global for supported profiles and update it on screen changes.

## Tests asserting this spec

- `tests/user_defaults_batch2.rs::quest_faction_group_uses_native_argument_conversion`: checks argument conversion, results and absent-record behavior on Forever, Retail and PTR.
- `tests/user_defaults_batch2.rs::user_defaults_batch2_world_map_quest_consumer`: checks the original account-quest helper and WorldMap consumer.

## Known gaps (current cycle)

- [ ] Native queued quest-data requests and their completion event are not modeled; the simulator uses host-supplied quest records.
- [ ] No Glue installer was found in the inspected builds, so absence from Glue is inferred rather than verified.

## Out of scope

- Quest-data transport, pending-record lifecycle and network completion events.
