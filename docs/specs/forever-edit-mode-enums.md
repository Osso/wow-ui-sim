# Forever Edit Mode enums

Forever 1.60.1.69913 publishes finite additions from its `EditModeManagerConstantsDocumentation.lua` through `src/c_api/forever_edit_mode_enums.rs`. See [client profiles](../wiki/systems/client-profiles.md).

## What it must do

- [x] Publish the six missing GroupFinder, MainActionBarEndCap and SwingTimer setting/index/visibility namespaces and their source metadata.
- [x] Publish documented LossOfControl and RaidWarning settings, Gamepad preset 2, systems 24–29, account settings 33–37, minimap IconScale, micro-menu DeprecatedEyeSize and unit-frame DebuffIconSize/BuffIconSize with updated metadata.
- [x] Reuse the existing RaidDispelOverlayType values and metadata.
- [x] Execute the actual Forever preset and setting-display consumers without enum lookup failures.
- [ ] Keep these additions scoped to Forever without enabling a retail epoch.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)
- [Running patch report](../wowforever-1.60.1.md)

## Implementation inventory

- `src/c_api/forever_edit_mode_enums.rs` — finite documented additions and metadata.
- `src/c_api/mod.rs` — profile-scoped compilation.
- `src/lua_api/env_init/enums.rs` — publication after shared enum initialization.
- `src/lua_api/globals/enum_data/combat_system.rs` — reused raid dispel definitions.

## Tests asserting this spec

- `tests/wowforever_edit_mode_enums.rs` — exact source values, metadata, actual preset and display consumers.

## Known gaps (current cycle)

- [ ] Full Edit Mode UI loading and interactions remain parent integration work.
- [ ] Earlier-profile isolation verification remains pending.

## Out of scope

Native-game behavior, arbitrary enum generation, retail epoch activation, and vendor changes.
