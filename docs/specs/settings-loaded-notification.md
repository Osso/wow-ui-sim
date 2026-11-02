# Settings-loaded notification

## What it must do

- [ ] Retail `C_SettingsUtil.NotifySettingsLoaded()` synchronously dispatches `SETTINGS_LOADED` to registered listeners and returns zero values.
- [ ] The original `SettingsRegistrar` invokes the notification after both `VARIABLES_LOADED` and `PLAYER_ENTERING_WORLD`.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/SettingsUtilDocumentation.lua:11` declares the function without a Returns section; its event declaration at line 31 marks `SETTINGS_LOADED` synchronous. `Blizzard_Settings_Shared/Blizzard_SettingsRegistrar.lua:19` is the original caller. WM24 build 70688 identifies an optional notification callback and no direct entry security check; forwarding it to the simulator event dispatcher is the supported local effect. Native callback restrictions and callback-induced effects remain unknown.

## Implementation inventory

- `src/c_api/c_settings_util.rs`: Retail registration and event dispatch.
- `src/lua_api/globals/state_backed_queries.rs`: synchronous event dispatcher.

## Tests asserting this spec

- `tests/settings_loaded_notification.rs`: original registrar waits for both startup events and emits the notification; direct notification returns zero values.

## Known gaps

- [ ] Native callback restrictions, whole-wrapper security/hardware requirements, and runtime client transitions are not established. Other client profiles are not covered by this contract.
