# Party map-position input

Frozen [8.0.1 source](../../data/patch-api/sources/8.0.1-api-changes.txt) and current Retail `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:436–451` explicitly say `C_Map.GetPlayerMapPosition` works for the player and party members; the result is a nilable `vector2`. Current simulator supports only the player. This slice supplies the missing party-positive path from explicit simulator input, not native coordinate acquisition. [Map API architecture](../lua-api.md) describes the surrounding surface.

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

- `src/c_api/c_map.rs` — `UnitMapPosition { ui_map_id, position: (f64, f64) }` input record; existing player-only query/vector construction, party producer pending.
- `src/lua_api/game_data.rs` — `PartyMember.map_position: Option<UnitMapPosition>`; ordinary member construction initializes it to `None`.
- `tests/c_map_probes.rs` — existing player controls and planned party-positive behavioral case.

## Tests asserting this spec

`tests/c_map_probes.rs::get_player_map_position_uses_independent_active_party_member_input` activates two actual party slots, supplies distinct player/member coordinates and checks the real query's vector/`GetXY` and live independent updates. Preparation only: it has not executed. Existing player-position, unknown-map and target-token probes remain unchanged. A compiled test that reaches the missing party result is required before calling the initial result behavioral RED.

## Known gaps (current cycle)

- [ ] Prepare explicit input and reach the real party-positive RED boundary.
- [ ] Implement the shared query, then verify nonzero targeted GREEN and player/roster controls with formatting/type checks.
- [ ] Native map hierarchy/projection, invalid-token behavior, raid distinctions, secrecy and historical parity remain unknown; positive supplied-input proof does not settle them.

## Out of scope

World-coordinate conversion, geographic/FDID catalogs, new native-policy guesses, unrelated player-map behavior, group lifecycle redesign and `GetBestMapForUnit` expansion. Current-profile native coordinates require actual client evidence; internal explicit input does not claim a native backing mechanism.
