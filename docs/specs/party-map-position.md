# Party map-position input

Frozen [8.0.1 source](../../data/patch-api/sources/8.0.1-api-changes.txt) and current Retail `Blizzard_APIDocumentationGenerated/MapDocumentation.lua:436–451` explicitly say `C_Map.GetPlayerMapPosition` works for the player and party members; the result is a nilable `vector2`. Explicit simulator party input is **bounded default-Retail GREEN**, verified 2026-10-10: 30 probe + 51 API = 81 actual PASS. This supplies the party-positive path from simulator input, not native coordinate acquisition. [Map API architecture](../lua-api.md) describes the surrounding surface.

## What it must do

- [x] A populated active party slot with supplied normalized coordinates for the requested known UI map returns one position vector; `GetXY()` yields that member's supplied coordinates, not the player's.
- [x] Updating the member's input changes subsequent reads. Different member records retain independent positions.
- [x] Preserve existing player-position behavior and the existing vector factory; do not redesign player projection or `GetBestMapForUnit`.
- [x] **Simulator input policy, not native parity:** no supplied matching position, inactive group, missing slot or unknown map yields the existing nil result. Do not invent geographic conversion or substitute player coordinates.
- [x] Position input belongs to the member record; ordinary new/replaced members start without a supplied position. No vendor behavior, new Lua setter or map catalog acquisition is required.

## How it works

- [Lua/C API boundary](../lua-api.md)
- [Historical 8.0.1 audit](../wiki/investigations/patch-8-0-1-api-audit.md)

## Implementation inventory

- `src/c_api/c_map.rs` — `UnitMapPosition { ui_map_id, position: (f64, f64) }` input record; party producer uses existing `unit_api::parse_party_index` and vector factory. State borrow ends before Lua vector allocation.
- `src/lua_api/game_data.rs` — `PartyMember.map_position: Option<UnitMapPosition>`; ordinary member construction initializes it to `None`.
- `tests/c_map_probes.rs` — unchanged player/target controls and original party-positive case; separate party-input/roster-reset control case. Both party cases actually passed in the bounded GREEN selection.

## Tests asserting this spec

`tests/c_map_probes.rs::get_player_map_position_uses_independent_active_party_member_input` activates two actual party slots, supplies distinct player/member coordinates and checks the real query's vector/`GetXY` and live independent updates. [Historical RED](../../data/patch-api/evidence/8.0.1-session-2026-10-08/party-map-red-20261010/assertion-summary.json) remains genuine: compile exit 0, exact case 1 FAIL, execution exit 101, `active party1 position must be present`. Original positive function body is byte-identical between RED and GREEN; no assertion weakening.

`tests/c_map_probes.rs::get_player_map_position_party_input_controls_and_roster_reset` covers absent input, known mismatched map 84, unknown map 999999, inactive group with populated input, and actual `A_Admin.SetPartySize(0)` / `SetPartySize(1)` removal/re-growth clearing old slot input. These nil policies are inferred simulator input policy, not native observations.

[Runtime audit](../../data/patch-api/evidence/8.0.1-session-2026-10-08/party-map-green-20261010/runtime-report.md): at submission `d25c1fdb444c2dd1c8b0c45dd4f8032b1ca75289`, actual named tests match source declarations exactly: probes 30/30 and API 51/51, both exit 0. Original worker expected 49 API cases incorrectly: the parent has 49 tests plus two ordinary `c_map_api::texture::` child-module tests. Its controller aborted before checks; this is not original-worker success. Later missing-only `cargo fmt --check` and default `cargo check --offline --locked -j 12` receipts exit 0 with source equality, without rerunning tests. Captured 3853-file source equality and sealed artifact/profile correspondence verified; inherited iced manifest deprecations remain, not warning-free.

## Known gaps (current cycle)

- [x] Bounded default-Retail runtime acceptance: genuine RED, unchanged positive body, 81 actual GREEN cases and later missing fmt/default-check receipts independently inspected.
- [ ] Native map hierarchy/projection, invalid-token behavior, raid distinctions, secrecy and historical parity remain unknown; positive supplied-input proof does not settle them.

## Out of scope

World-coordinate conversion, geographic/FDID catalogs, new native-policy guesses, unrelated player-map behavior, group lifecycle redesign and `GetBestMapForUnit` expansion. No native, all-profile, current full-suite or whole-HEAD acceptance claim. External dependency/source-to-artifact provenance, untracked files/index, inherited environment and runtime assets outside fixture inputs remain excluded. Current-profile native coordinates require actual client evidence; internal explicit input does not claim a native backing mechanism.
