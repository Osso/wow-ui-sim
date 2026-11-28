# Garrison mission lists

**INFERRED (R88dg pending):** The Retail and WoW Forever `C_Garrison` mission-list queries use host-supplied records stored by the Rust mission model. They serve original Garrison and Covenant mission consumers. Native query behavior is pending. See [Lua API architecture](../lua-api.md).

## What it must do

- [ ] **INFERRED (R88dg pending):** `A_Admin.SetGarrisonMissions` replaces the supplied mission records for the current simulator session.
- [ ] **INFERRED (R88dg pending):** Each record carries the host-supplied `missionID`, `name`, `followerTypeID`, `level`, `durationSeconds`, `duration`, `timeLeft`, `xp`, `canBeCompleted`, `rewards`, `inProgress`, and `completed` fields, plus optional `encounterIconInfo`.
- [ ] **INFERRED (R88dg pending):** `C_Garrison.GetAvailableMissions` returns records for the requested follower type that are neither in progress nor completed.
- [ ] **INFERRED (R88dg pending):** `C_Garrison.GetInProgressMissions` returns records for the requested follower type whose `inProgress` value is true.
- [ ] **INFERRED (R88dg pending):** `C_Garrison.GetCompleteMissions` returns records for the requested follower type whose `completed` value is true.
- [ ] **INFERRED (R88dg pending):** The available and in-progress queries fill and return a supplied output table, or return a new table when called with a follower type or `nil` plus a follower type.
- [ ] **INFERRED (R88dg pending):** The original Retail and Forever Covenant mission-list consumer displays all supplied available, completed, and in-progress mission records; it must not pass with an empty producer.
- [ ] `C_Garrison.GetMissionEncounterIconInfo(missionID)` returns the documented `MissionEncounterIconInfo` shape, populated from an optional supplied mission field. An unsupplied mission returns that shape with zero numeric fields and false boolean fields. **INFERRED (R88dg pending).**
- [ ] `A_Admin.SetGarrisonFollowerQualityTable(followerTypeID, table)` supplies the quality-to-XP entries returned by `C_Garrison.GetFollowerQualityTable(followerTypeID)`. No generated DB2 quality table exists; an unset follower type returns an empty table. **INFERRED (R88dg pending).**
- [ ] The Rust model and setters are registered on Retail and WoW Forever only. **VERIFIED:** PTR keeps the existing Lua shim unchanged.

## How it works

- [Lua API architecture](../lua-api.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

WoW Forever publishes `C_Garrison.IsAtGarrisonMissionNPC` (its `GarrisonInfoDocumentation.lua:437-446` block is byte-identical to Retail's) and `A_Admin.SetAtGarrisonMissionNPC(bool)`. Forever excludes Blizzard_GarrisonUI (`## ExcludeLoadGameType: camelot`); `tests/garrison_mission_npc_forever.rs` calls the API with both states.

## Implementation inventory

- `src/c_api/c_garrison_mission_lists.rs` — mission records, query functions, and `A_Admin.SetGarrisonMissions`.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — per-environment mission state.
- `src/lua_api/globals/admin.rs` — profile-scoped setter registration.
- `src/lua_api/workarounds/mod.rs` — retain the legacy shim outside Retail and Forever, then register the Rust queries on those profiles.
- `tests/blizzard_garrison_ui_loads.rs` — selected original Covenant consumer test.
- `tests/fixtures/garrison_mission_lists.lua` — host-supplied mission and follower-quality fixture.

## Tests asserting this spec

- `tests/blizzard_garrison_ui_loads.rs::supplied_missions_update_the_real_covenant_mission_list` (Retail and Forever original consumer)
- `tests/garrison_combat_ally.rs::combat_ally_mission_tracks_two_supplied_missions_and_clears` (direct model behavior)

## Known gaps (current cycle)

- [ ] Native query signatures, table-clearing behavior, reward record shape, and default results are pending evidence from R88dg.
- [ ] The Rust combat-ally mission model and direct input test exist, but native query signatures, result shape, and defaults remain pending evidence.

## Out of scope

- PTR continues to use the legacy Lua shim.
- Automatic mission discovery, progression, timing, and completion events are not modeled by this host-supplied snapshot.
