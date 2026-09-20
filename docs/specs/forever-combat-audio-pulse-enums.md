# Forever combat-audio pulse enums

Forever 1.60.1.69913 publishes the pulse-health enumeration described by its cached `Blizzard_APIDocumentationGenerated/CombatAudioAlertSharedDocumentation.lua`.

## What it must do

- [x] Publish `Enum.CombatAudioAlertPulsePercentValues`: `Off=0`, then `Under90Percent=1` through `Under10Percent=9`.
- [x] Publish corresponding `Meta`: `MinValue=0`, `MaxValue=9`, `NumValues=10`.
- [x] Allow the actual `TextToSpeechCommands.lua` pulse-health command block to register range 0–9, accept each integer in that range, and reject -1, 10, and nonnumeric input without changing the CVar.
- [ ] Restrict this publication change to Forever without enabling retail epochs.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/enum_data/forever_shared.rs`: exact-build values.
- `src/lua_api/globals/enum_data/mod.rs`: Forever-only registration.

## Tests asserting this spec

- `tests/c_api_surface.rs`: generated-documentation equality and actual vendor command-block execution in the existing grouped integration target.

## Known gaps (current cycle)

- [ ] Full TextToSpeechCommands startup and other-profile runtime isolation remain integration verification obligations.

## Out of scope

Native conformance, audio playback, other combat-audio enums, and retail epoch changes. The command test substitutes surrounding slash-command registration and label helpers; it does not establish complete addon loading.
