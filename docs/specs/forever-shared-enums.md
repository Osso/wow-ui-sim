# Forever Shared Enums

Forever publishes three existing enum contracts through `src/lua_api/globals/enum_data/forever_shared.rs`; see [client profiles](../wiki/systems/client-profiles.md).

## What it must do

- [x] Publish all source-documented `BattleNetFriendLevel` (1–3), `VisualAlertType` (1–10), and `CooldownViewerSound` (0–93) fields and matching metadata on Forever.
- [x] Let vendor AccountUtil rank friend levels and compare their ordering.
- [x] Let vendor cooldown sound data retain numeric sound enums and concrete sound-kit IDs.
- [ ] Preserve existing exposure on earlier profiles without enabling a retail epoch for Forever.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/lua_api/globals/enum_data/forever_shared.rs` — existing values grouped behind one profile decision.
- `src/lua_api/globals/enum_data/mod.rs` — exports the group alongside base arrays.
- `src/lua_api/env_init/enums.rs` — registers both groups.

## Tests asserting this spec

- `tests/c_api_surface.rs::forever_shared_enums` — compares published fields/metadata against Forever API documentation and executes vendor AccountUtil and cooldown sound data.

## Known gaps (current cycle)

- [ ] Full Forever startup and native conformance are not established by these enum fixtures.

## Out of scope

- Other enums, API namespaces, or wholesale retail epoch enablement.
