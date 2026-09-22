# Player facing

Forever `GetPlayerFacing()` exposes nullable player orientation from simulator state, independently of widget/model transforms. Source: `src/lua_api/globals/real/player_facing.rs`; see [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Return nil while player facing is unknown, and reflect each configured finite angle without normalization.
- [ ] `A_Admin.SetPlayerFacing(number | nil)` updates or clears player facing. Invalid types and non-finite numbers fail before changing state.
- [ ] Model-widget facing and other simulator environments remain independent of player facing.
- [ ] Real frame OnUpdate consumers can read two successive angles and apply texture rotation; an unknown angle leaves the consumer's prior rotation unchanged.

## How it works

- [Lua API architecture](../lua-api.md)
- [Player identity admin surface](../admin-api/player-identity.md)

## Implementation inventory

- `src/lua_api/state_types/character_world.rs` — nullable Forever player-facing field; derived default is unknown.
- `src/lua_api/globals/real/player_facing.rs` — modeled getter and validated admin input.
- `src/lua_api/globals/real/mod.rs`, `globals/register.rs`, `globals/admin.rs` — Forever-only publication.
- `tests/player_facing.rs` — grouped state, validation and texture-consumer regressions.

## Tests asserting this spec

- `player_facing_tracks_nullable_state_independently_of_model_widgets`
- `invalid_player_facing_inputs_preserve_previous_state`
- `player_facing_drives_texture_rotation_through_on_update`

## Known gaps (current cycle)

- [ ] Targeted GREEN and independent verification pending. Frozen `2c5bf78c7` RED: `/tmp/forever-addon-audit/player-facing-red-ddxm2ugk/ledger.json`.
- [ ] Parent-owned unchanged CustomMinimapArrow `8909385` startup and two-orientation replay remain pending.
- [ ] Native Forever probes unavailable. Cached `PlayerScriptDocumentation.lua:761–768` proves only a nullable numeric return. Unknown initial state, admin validation and non-normalization are explicit simulator policies, not native-default or angle-domain conformance.

## Out of scope

Camera/3D rendering, hardware input, world-position changes, inferred angle wrapping, and publication changes to other profiles. The admin setter changes simulator state, never the addon or its getter function.
