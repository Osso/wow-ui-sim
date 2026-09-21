# Aura container options

`C_AuraContainerUtil` normalizes the nine base aura presentation option structures in `src/c_api/c_aura_container_util.rs`. The shared `aura-containers` capability exposes the existing processors and native aura enums to Retail 12.1+ and Forever. Contracts come from each pinned client's `Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua` and `AuraContainerSharedDocumentation.lua`, plus their referenced duration-text, texture-slice and status-bar structures.

## What it must do

- [ ] Register the nine base processors before secure-environment copying for Retail 12.1+ and Forever, without enabling an entire Retail epoch for Forever.
- [ ] Publish `Enum.CustomAuraButtonDispelTypeStealableFilter` (`Stealable=0`, `NotStealable=1`) and its metadata in both environments. Preserve either option value, including zero.
- [ ] Publish `Enum.CustomAuraButtonUpdateMode` (`Assignment=0`, `Update=1`) and the five texture styles (`Border=0`, `BorderWithIcon=1`, `Icon=2`, `PreserveAsset=3`, `CustomAsset=4`), with exact membership and metadata in both environments.
- [ ] Require tables for tooltip backdrop, nine-slice, texture-slice, and application-bar options; accept nil for application-count, dispel-text, dispel-texture, duration-bar, and duration-text options.
- [ ] Return new structures and nested maps/arrays containing recognized fields; do not mutate input structures or return the input table unchanged.
- [ ] Validate required fields, primitive types, documented enum values, string-keyed maps, and duration-format component arrays. Preserve false and zero rather than replacing them with defaults.
- [ ] Apply only explicit structure defaults: zero tooltip offsets/insets and texture draw sublevel; texture coordinates `0,1,0,1`; false `useAtlasSize`; documented dispel visibility booleans and `BorderWithIcon` style.
- [ ] Copy color values into the existing ColorMixin representation. Retain formatter, duration-binding, and color-curve object references without implementing or cloning those objects here.
- [ ] Complete the real Blizzard CustomAuraButton public initializer's application-count setup without a missing-namespace error.

## How it works

- [Lua API](../lua-api.md)
- [Frame data flow](../frame-data-flow.md)

The module's local field lists describe only these documented structures. Primitive/enum validation, recursive structure/map/array copying, and object-interface validation share small helpers. Intermediate output tables remain rooted while color construction or object method lookup can enter Lua.

## Implementation inventory

- `src/c_api/c_aura_container_util.rs` — nine processors, field contracts, validation, copies, and color conversion.
- `src/c_api/mod.rs` — pre-secure-copy namespace registration.
- `Cargo.toml` and `src/lua_api/globals/enum_data/{mod,widget,addon_system}.rs` — shared capability and native enum publication.

## Tests asserting this spec

- `tests/aura_container_util.rs::aura_stealable_filter_enum_and_options_match_native_contract` — exact enum membership/metadata and both option values through public/secure processors; reproduces BetterBlizzFrames `auras.lua:796` missing-enum boundary.
- `tests/aura_container_util.rs` — grouped tests for defaults, public/secure availability, nested tooltip structures/colors, dispel maps, duration components/object identity, invalid values, and the real CustomAuraButton initializer.

## Known gaps (current cycle)

- [ ] Shared-capability grouped GREEN is pending; no Cargo was run for this publication slice. Existing Forever native-provider RED stops at missing `CustomAuraButtonUpdateMode`.
- [ ] Duration-binding, formatter lifecycle and XML argument projection remain separate dependencies. The duration-options test now requires non-nil modeled factory handles instead of allowing nil identity comparisons to pass.
- [ ] Pinned Forever documentation also declares `ProcessCustomAuraButtonCasterNameOptions` and application-bar `minApplications`. This slice leaves their existing `retail-12-1-5` gates unchanged: those native fields are unexpanded/unproven, not absent from the client.

- Targeted enum/options proof passed 1/1 in `/tmp/pi-aura-stealable-enum-green.*`; related duration/curve identity has separate coverage.
- Actual addon/SavedVariables startup returned `[]`, exit 0 in `/tmp/pi-accepted-final-startup.*`; it still records three loader warnings, so this is not a zero-warning claim.

This implementation does not establish exact native error wording or secret-argument enforcement.

## Out of scope

Formatter, duration-binding, and curve implementations; vendor/addon modifications; broader aura rendering or security-policy changes. Publishing option/enumeration data does not authorize or establish full access-restriction or secret-value enforcement. RGB colors use the existing three-channel `CreateColor` conversion rather than introducing an aura-specific alpha default.
