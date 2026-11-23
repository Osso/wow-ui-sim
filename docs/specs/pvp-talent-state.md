# PvP talent state

`C_SpecializationInfo` exposes supplied selected PvP talent IDs, inspected selections, slot unlock levels and lock state on Retail and WoW Forever. **INFERRED (R88eq, native evidence pending):** state is empty until the host supplies it; writes notify the original UI consumers.

## What it must do

- [x] Return selected talent IDs in supplied slot order from `GetAllSelectedPvpTalentIDs`.
- [x] Return an inspected talent ID only for its supplied unit token and slot.
- [x] Return a supplied slot unlock level; return nil when no level was supplied.
- [x] Preserve lock state per talent ID through `IsPvpTalentLocked` and `SetPvpTalentLocked`.
- [x] Let `A_Admin` supply selected IDs, inspected selection and slot unlock level without test writes to simulator state.
- [ ] Dispatch the native refresh events after mutations; event parity is inferred, not verified.

## How it works

- [C API and Lua API state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_spec.rs` — PvP state queries, mutation, and host input registration.
- `src/lua_api/globals/admin.rs` — publishes the PvP state setters on Retail and WoW Forever.
- `src/lua_api/workarounds/temporary/pvp_talent_defaults.rs` — retains defaults only on profiles without the Rust model.

## Tests asserting this spec

- `tests/pvp_talent_state.rs::supplied_pvp_talent_state_reaches_the_open_player_spells_panel`

## Known gaps (current cycle)

- [ ] Native default state, mutation effects and event semantics remain pending R88eq.

## Out of scope

- PvP talent catalog rows, eligibility rules, learning and persistence.
- Retail `GetPvpTalentUnlockLevel`, which uses the existing catalog model.

## Evidence

- **VERIFIED (Retail cache):** `Blizzard_PlayerSpells.toc` loads PvP talent slot and list templates; the slot consumer listens for `PLAYER_PVP_TALENT_UPDATE`, and `Blizzard_InspectUI/Mainline/InspectPVPFrame.lua` listens for `INSPECT_HONOR_UPDATE`.
- **VERIFIED (Retail and WoW Forever API documentation caches):** `SpecializationInfoDocumentation.lua` declares `GetAllSelectedPvpTalentIDs`, `GetInspectSelectedPvpTalent`, and `GetPvpTalentSlotUnlockLevel`.
- **INFERRED (R88eq, native evidence pending):** host-supplied mutation notifications use the events read by the corresponding Blizzard consumers.
