# Party map-position input

Frozen [8.0.1 source](../../data/patch-api/sources/8.0.1-api-changes.txt) and current Retail `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:436–451` explicitly say `C_Map.GetPlayerMapPosition` works for the player and party members; the result is a nilable `vector2`. Simulator producer now includes the explicit party-input path; runtime GREEN is pending. This slice supplies the missing party-positive path from explicit simulator input, not native coordinate acquisition. [Map API architecture](../lua-api.md) describes the surrounding surface.

## What it must do

- [ ] A populated active party slot with supplied normalized coordinates for the requested known UI map returns one position vector; `GetXY()` yields that member's supplied coordinates, not the player's.
- [ ] Updating the member's input changes subsequent reads. Different member records retain independent positions.
- [ ] Preserve existing player-position behavior and the existing vector factory; do not redesign player projection or `GetBestMapForUnit`.
- [ ] **Simulator input policy, not native parity:** no supplied matching position, inactive group, missing slot or unknown map yields the existing nil result. Do not invent geographic conversion or substitute player coordinates.
- [ ] Position input belongs to the member record; ordinary new/replaced members start without a supplied position. No vendor behavior, new Lua setter or map catalog acquisition is required.

## How it works

- [Lua/C API boundary](../lua-api.md)
- [Historical 8.0.1 audit](../wiki/investigations/patch-8-0-1-api-audit.md)

## Implementation inventory

- `src/c_api/c_map.rs` — `UnitMapPosition { ui_map_id, position: (f64, f64) }` input record; party producer present, using existing `unit_api::parse_party_index` and vector factory. State borrow ends before Lua vector allocation. **GREEN PENDING**.
- `src/lua_api/game_data.rs` — `PartyMember.map_position: Option<UnitMapPosition>`; ordinary member construction initializes it to `None`.
- `tests/c_map_probes.rs` — unchanged player/target controls and original party-positive RED case; one separate party-input/roster-reset control case added, unexecuted.

## Tests asserting this spec

`tests/c_map_probes.rs::get_player_map_position_uses_independent_active_party_member_input` activates two actual party slots, supplies distinct player/member coordinates and checks the real query's vector/`GetXY` and live independent updates. Main captured genuine RED at `/home/osso/.local/state/wow-ui-sim/verification/party-map-red-current/20261010T175747Z`: compile exit 0, source equality confirmed, exact case exit 101 with `active party1 position must be present`. Original RED case bytes remain unchanged. Existing player-position, unknown-map and target-token probes remain unchanged.

`tests/c_map_probes.rs::get_player_map_position_party_input_controls_and_roster_reset` covers absent input, known mismatched map 84, unknown map 999999, inactive group with populated input, and actual `A_Admin.SetPartySize(0)` / `SetPartySize(1)` removal/re-growth clearing old slot input. These nil policies are inferred simulator input policy, not native observations. Producer and new controls are **GREEN PENDING**; main must commit, run the whole bounded C_Map controls and complete the independent gate.

## Known gaps (current cycle)

- [ ] Runtime acceptance pending: genuine party-positive RED captured; producer present, **GREEN PENDING**.
- [ ] Main to verify nonzero bounded C_Map GREEN and unchanged player/target plus new roster controls through the independent gate.
- [ ] Native map hierarchy/projection, invalid-token behavior, raid distinctions, secrecy and historical parity remain unknown; positive supplied-input proof does not settle them.

## Out of scope

World-coordinate conversion, geographic/FDID catalogs, new native-policy guesses, unrelated player-map behavior, group lifecycle redesign and `GetBestMapForUnit` expansion. Current-profile native coordinates require actual client evidence; internal explicit input does not claim a native backing mechanism.
