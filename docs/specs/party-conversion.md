# Party conversion to raid

`C_PartyInfo.ConvertToRaid()` changes the represented active group type. Retail and Forever declare the member in `PartyInfoDocumentation.lua`; Mists has a Blizzard cache consumer but no generated declaration. Wrath retains the legacy `ConvertToRaid()` global.

## What it must do

- [ ] Publish `C_PartyInfo.ConvertToRaid()` on Retail, Forever and Mists. The documented Retail/Forever signature has no arguments or returns; Mists publication is inferred from the selected Blizzard consumer.
- [ ] On an active group, convert the stored group type to raid without changing its roster. `IsInRaid()` and `UnitInRaid()` must read that group type.
- [ ] Return no Lua values and emit no modeled event.
- **INFERRED (simulator mapping; native state/event behavior UNKNOWN, R88bk pending):** the ordinary conversion updates the active group immediately. PartySync confirmation and event emission remain UNKNOWN and are not modeled.

## How it works

- [Lua API and simulator state](../lua-api.md).
- [Group state](party-connection.md).

## Implementation inventory

- `src/c_api/c_party_info.rs` — conversion registration and state update.
- `src/lua_api/state/sim_state.rs` — explicit group-kind input; `None` retains the roster-size inference.
- `src/lua_api/globals/unit_api.rs` — shared raid-state query for `IsInRaid`, `UnitInRaid`, and related readers.

## Tests asserting this spec

- `tests/party_convert_to_raid.rs` — selected Retail, Forever and Mists `Blizzard_UnitPopup` consumers, exact source path, return count and observable group-reader changes.

## Known gaps (current cycle)

- [ ] Native PartySync confirmation flow and event emission remain UNKNOWN pending R88bk evidence.

## Out of scope

Wrath's legacy global and PartySync confirmation. Do not add events without native evidence.
