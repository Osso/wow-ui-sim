# Retail `C_ColorOverrides`

This spec defines the Retail `C_ColorOverrides` API and its state model. See the [color overrides system page](../wiki/systems/color-overrides.md) for the implementation and evidence boundaries.

## What it must do

### Quality colors

- [ ] For Poor and Epic quality, `GetDefaultColorForQuality` returns the matching `GlobalColor` colors. It does not read overrides.
- [ ] `GetColorForQuality` uses overrides for two backed quality/type pairs and returns defaults after removal or clear.
- [ ] A missing default row returns the native magenta fallback. This branch is not reachable with the current generated catalog.

### Override records

- [ ] `SetColorOverride` accepts RGBA table components, clamps finite values to `[0, 1]`, converts them to byte channels, and stores two distinct types.
- [ ] `GetColorOverrideInfo` returns one record for each stored type and exactly one nil for an absent type. The returned RGB color has no `a` field.
- [ ] `RemoveColorOverride` removes selected records and accepts a missing type without changing other records.
- [ ] `ClearColorOverrides` removes all stored records.
- [ ] The original Retail `ColorManager` consumer uses the model's default and overridden colors.

## How it works

- [Color overrides system](../wiki/systems/color-overrides.md)
- [GlobalColor catalog](../wiki/systems/global-color-catalog.md)
- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_color_overrides.rs`: Retail API registration, in-memory override state, validation, and color publication.
- `src/c_api/c_ui_color.rs`: shared `ColorMixin` copying and packed-color channel conversion.
- `src/lua_api/env_init/mod.rs`: ordinary-environment registration.
- `src/lua_api/workarounds/temporary/color_defaults.rs`: no longer supplies the two temporary `C_ColorOverrides` defaults.

## Tests asserting this spec

- `tests/blizzard_ui_unit.rs::original_color_manager_tracks_globalcolor_defaults_and_color_overrides`
- `tests/blizzard_ui_unit.rs::original_color_manager_bag_quality_uses_default_without_override`
- `src/loader/tests/wow_api_globals/startup_namespaces.rs::test_bootstrap_registers_color_override_members`

## Known gaps (current cycle)

- [ ] Add a supplied `playerColorOverrides` value only after its exact setting encoding and save/load triggers are verified. The native setting name is known, but this simulator does not claim reload or logout parity.
- [ ] Verify the native callback-to-event mapping and the exact `overrideColorString` byte order. Current event names and packed-string formatting are inferred; tests do not assert them.

## Out of scope

- Secure-environment flag-3 publication; a separate lane owns that change.
- A reload/logout chain or storage behavior beyond the `playerColorOverrides` seam while its native lifecycle remains unknown.
