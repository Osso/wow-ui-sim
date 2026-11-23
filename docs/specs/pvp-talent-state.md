# PvP talent state

`C_SpecializationInfo` exposes supplied selected PvP talent IDs and inspected selections on Retail and WoW Forever. Retail keeps its existing verified lock and slot-unlock behavior. **INFERRED (R88eq, native evidence pending):** WoW Forever accepts supplied slot unlock levels and refreshes UI consumers after state writes.

## What it must do

- [x] Return selected talent IDs in supplied slot order from `GetAllSelectedPvpTalentIDs`.
- [x] Return an inspected talent ID only for its supplied unit token and slot.
- [x] Preserve Retail `GetPvpTalentSlotUnlockLevel` absent-row behavior; expose host-supplied slot unlock levels on WoW Forever.
- [x] Preserve Retail lock behavior; expose lock state on WoW Forever through `IsPvpTalentLocked` and `SetPvpTalentLocked`.
- [x] Let `A_Admin` supply selected IDs and inspected selections on both profiles, and Forever slot unlock levels, without test writes to simulator state.
- [ ] Dispatch the inferred refresh events after WoW Forever mutations; native event parity is not verified.

## How it works

- [C API and Lua API state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_spec.rs` — PvP state queries, mutation, and host input registration.
- `src/c_api/helpers.rs` — shares Retail untainted truthiness conversion with WoW Forever.
- `src/lua_api/globals/admin.rs` — publishes the PvP state setters on Retail and WoW Forever.
- `src/lua_api/workarounds/temporary/pvp_talent_defaults.rs` — retains defaults only on profiles without the Rust model.

## Tests asserting this spec

- `tests/pvp_talent_state.rs::retail_supplied_pvp_talent_state_reaches_open_player_spells_panel`
- `tests/pvp_talent_state.rs::forever_supplied_pvp_talent_state_reaches_open_player_spells_panel`

## Known gaps (current cycle)

- [ ] Native default state, mutation effects and event semantics remain pending R88eq.

## Out of scope

- PvP talent catalog rows, eligibility rules, learning and persistence.
- Retail `GetPvpTalentUnlockLevel`, which uses the existing catalog model.

## Evidence

- **VERIFIED (Retail cache):** `Blizzard_PlayerSpells.toc` loads PvP talent slot and list templates; the slot consumer listens for `PLAYER_PVP_TALENT_UPDATE`, and `Blizzard_InspectUI/Mainline/InspectPVPFrame.lua` listens for `INSPECT_HONOR_UPDATE`.
- **VERIFIED (Retail and WoW Forever API documentation caches):** `SpecializationInfoDocumentation.lua` declares `GetAllSelectedPvpTalentIDs`, `GetInspectSelectedPvpTalent`, and `GetPvpTalentSlotUnlockLevel`.
- **INFERRED (R88eq, native evidence pending):** host-supplied mutation notifications use the events read by the corresponding Blizzard consumers.
