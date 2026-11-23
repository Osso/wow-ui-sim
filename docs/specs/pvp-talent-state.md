# PvP talent state

`C_SpecializationInfo` exposes PvP talent records, selected IDs, inspected selections, unlock levels, lock state and alert flags. **VERIFIED (R88eq eq1):** Retail returns the documented row values, list and Boolean outputs. **INFERRED (R88eq eq1):** WoW Forever uses host-supplied records with the same result shapes.

## What it must do

- [ ] Return selected talent IDs as a list and inspected selections as an ID or nil.
- [ ] Return integer unlock levels for matching talent/slot rows and nil for missing rows.
- [ ] Return supplied `PvpTalentInfo` and `PvpTalentSlotInfo` records through their original consumers.
- [ ] Return a Boolean from `IsPvpTalentLocked`; its setter normally returns no values.
- [ ] Return two Boolean alert values; their meanings remain unknown.
- [ ] Let `A_Admin` supply test rows and state without writes to `env.state()`.
- [ ] Dispatch an inferred Player PvP talent refresh event when simulator state changes.
- [ ] Dispatch an inferred Inspect Honor refresh event when inspect selection changes.

## How it works

- [C API and Lua API state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_spec.rs` — PvP state queries, mutations, row publication and host inputs.
- `src/c_api/c_tooltip_info_pvp_talent.rs` — Retail tooltip/slot source and Forever supplied slot records.
- `src/c_api/helpers.rs` — shared untainted-value conversions.
- `src/lua_api/globals/admin.rs` — publishes the PvP host setters.
- `src/lua_api/workarounds/temporary/pvp_talent_defaults.rs` — keeps defaults on profiles without the Rust model.

## Tests asserting this spec

- `tests/pvp_talent_state.rs::retail_supplied_pvp_talent_state_reaches_open_player_spells_panel`
- `tests/pvp_talent_state.rs::forever_supplied_pvp_talent_state_reaches_open_player_spells_panel`
- `tests/pvp_talent_native_consumers.rs` — existing Retail consumers and row behavior.

## Known gaps (current cycle)

- [ ] `GetPvpTalentInfo` missing-row public arity is UNKNOWN.
- [ ] Native no-host `GetPvpTalentSlotInfo` results remain UNKNOWN.
- [ ] Alert flag meanings and event/update semantics remain UNKNOWN.
- [ ] Forever result details are build-specific and UNKNOWN; the simulator mirrors Retail as INFERRED.

## Out of scope

- Talent learning, progression, persistence, and server-authoritative updates.

## Evidence

- **VERIFIED (R88eq eq1, Retail 12.1.5.70688):** matching unlock rows return integer levels and missing rows return nil; selected IDs are a list; inspect selection is nil or a cached ID; lock query returns a Boolean; the lock setter normally returns no values.
- **INFERRED (R88eq eq1, Retail 12.1.5.70688):** the lock setter truthy path inserts the talent ID and the false path erases it.
- **VERIFIED (R88eq eq1, Retail 12.1.5.70688):** populated `PvpTalentInfo` and `PvpTalentSlotInfo` paths build records. The pinned API documentation cache lists their fields.
- **VERIFIED (R88eq eq1, both builds):** `GetPvpTalentAlertStatus` publishes two Booleans. **UNKNOWN:** their meanings.
- **UNKNOWN (R88eq eq2, both builds):** native PvP talent event names, event order and update paths. **INFERRED:** simulator state changes dispatch `PLAYER_PVP_TALENT_UPDATE`; inspect selection changes dispatch `INSPECT_HONOR_UPDATE` to refresh the selected consumers.
- **VERIFIED (Retail and WoW Forever caches):** `Blizzard_PlayerSpells.toc` loads the PvP talent templates; the active profile TOCs select the Player Spells and Inspect consumers.
- **INFERRED (R88eq eq1, Forever 1.60.1.70707):** the simulator mirrors Retail result behavior for the same registered PvP talent members; their native state and tables are build-specific and UNKNOWN.
