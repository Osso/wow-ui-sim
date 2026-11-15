# Retail `C_ColorOverrides`

This spec defines the Retail `C_ColorOverrides` API and its state model. See the [color overrides system page](../wiki/systems/color-overrides.md) for the implementation and evidence boundaries.

## What it must do

### Quality colors

- [x] For Poor and Epic quality, `GetDefaultColorForQuality` returns the matching `GlobalColor` colors. It does not read overrides.
- [x] `GetColorForQuality` uses overrides for two backed quality/type pairs and returns defaults after clear.
- [ ] A missing default row returns the native magenta fallback. This branch is not reachable with the current generated catalog.

### Override records

- [x] `SetColorOverride` accepts RGBA table components, clamps finite values to `[0, 1]`, converts them to byte channels, and stores two distinct types.
- [x] `GetColorOverrideInfo` returns one record for each stored type and exactly one nil for an absent type. The returned RGB color has no `a` field.
- [x] `RemoveColorOverride` removes two selected records and accepts a missing type.
- [x] `ClearColorOverrides` removes two stored records.
- [x] The original Retail `ColorManager` consumer uses the model's default and overridden colors.

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

- [ ] Exercise the missing-GlobalColor-row magenta fallback. The current generated catalog contains every required tag.
- [ ] Verify the native callback-to-event mapping, `overrideColorString` byte order, and mutation return arity. The current event and packed-string choices are inferred; tests do not assert them.

## Out of scope

- Secure-environment flag-3 publication; a separate lane owns that change.
- A reload/logout chain or storage behavior beyond the `playerColorOverrides` seam while its native lifecycle remains unknown.
