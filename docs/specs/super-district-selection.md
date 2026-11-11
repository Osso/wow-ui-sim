# Superdistrict Selection and Information

The Glue character-select screen reads the selected superdistrict through `C_Login` and retrieves display records through `C_DistrictUtils`. The source-backed simulator inputs live in `SimState`.

## What it must do

- [ ] Forever and PTR expose `C_Login.GetSelectedSuperDistrictID` on Glue screens only; Retail does not expose it.
- [ ] The selected-ID query takes no arguments and returns one signed 32-bit number. Its initial and login-start value is `0`.
- [ ] Forever exposes `C_DistrictUtils.GetSuperDistrictInfo` on Glue only; PTR exposes it on Game and Glue; Retail does not expose it.
- [ ] The host supplies district records. The default list is empty; do not invent district entries.
- [ ] A known ID returns one table with `superDistrictID`, `displayName`, `description`, `iconFileDataID`, `disallowLogin`, `noRealmsAvailable`, `overrideBgAtlas`, and `overrideBgGlowAtlas` fields.
- [ ] An unknown ID, including `0` with no matching record, returns zero values.
- [ ] The original cached `CharacterSelectList.lua` consumer displays an empty string when the lookup has no record and uses the host record's `displayName` when it does.
- [ ] Forever and PTR expose the global `SelectSuperDistrict(id)` on Glue only; it returns no values and changes selection only when the host list contains the ID.
- [ ] Both ID inputs accept numbers and numeric strings, check the signed 32-bit range before truncating fractions toward zero, and report the verified usage or range-error detail.
- [ ] Missing or nil `SelectSuperDistrict` input reports `Usage: SelectSuperDistrict(superDistrictID)`; an invalid value reports `outside of expected range -2147483648 to 2147483647 - Usage: SelectSuperDistrict(superDistrictID)`.
- [ ] Missing or nil `GetSuperDistrictInfo` input reports `Usage: local info = C_DistrictUtils.GetSuperDistrictInfo(superDistrictID)`; an invalid value reports `outside of expected range -2147483648 to 2147483647 - Usage: local info = C_DistrictUtils.GetSuperDistrictInfo(superDistrictID)`.
- [ ] Do not emit a selected-ID-changed event or infer district data.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)
- [Superdistrict system](../wiki/systems/super-district-selection.md)

## Implementation inventory

- `src/c_api/c_login.rs`: registers the selected-ID query on Glue and resets the host selection on login-screen start.
- `src/c_api/c_district_utils.rs`: stores the host district record shape and registers its profile-specific Game/Glue lookup.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`: store and initialize selected-ID and district-record inputs.
- `src/lua_api/env_runtime.rs`: updates the screen-specific API surfaces.
- `src/lua_api/workarounds/temporary/forever_glue_display_defaults.rs`: keeps the separate empty `GetAvailableSuperDistricts` default.

## Tests asserting this spec

- `tests/glue_character_select.rs::super_district_title_uses_host_selection_and_info`
- `tests/glue_character_select.rs::retail_does_not_register_super_district_glue_apis`

## Known gaps (current cycle)

- [ ] Native clear/reset lifecycle paths are not exposed as separate simulator operations; `ScreenKind::Login` models login-start reset.

## Out of scope

- Realm-list transport, Battle.net last-played-character messages, district catalog data, and list-update events.
