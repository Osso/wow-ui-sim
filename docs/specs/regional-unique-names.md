# Regional unique names

## Contract

- Forever publishes `RegionalUniqueNamesEnabled()` returning one boolean from `PlayerState.regional_unique_names_enabled`.
- The flag defaults to `false` as an explicit simulator policy, not a native-client default inference.
- Changing the flag affects subsequent calls without reinitializing Lua; separate environments retain independent policy.
- Other profiles do not gain this global. No Admin API or character-creation coupling is added.

## Evidence and verification

Forever 1.60.1.69913 `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua` documents the zero-argument boolean global. `Blizzard_FrameXMLUtil/Camelot/NameUtil.lua` uses it to choose whole-name versus first-name display.

`tests/wowforever_name_policy.rs` covers default/arity, false→true→false behavior through the unmodified naming consumer, environment isolation, and other-profile absence. Its explicit surname-separator fixture isolates naming policy from constants publication. Native naming rules and native default policy remain unverified.
