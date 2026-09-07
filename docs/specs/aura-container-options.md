# Aura container options

`C_AuraContainerUtil` normalizes the nine aura presentation option structures in `src/c_api/c_aura_container_util.rs`. The contract comes from the pinned retail `Blizzard_APIDocumentationGenerated/AuraContainerUtilDocumentation.lua`, plus its referenced duration-text, texture-slice, status-bar, and aura shared structures.

## What it must do

- [ ] Register all nine processors before the secure environment is copied, under the cumulative `retail-12-1-0` epoch.
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

## Tests asserting this spec

- `tests/aura_container_util.rs` — six grouped tests for defaults, public/secure availability, nested tooltip structures/colors, dispel maps, duration components/object identity, invalid values, and the real CustomAuraButton initializer.

## Known gaps (current cycle)

- [ ] Cargo execution is awaiting the authorized build slot; the newly added tests have not run.
- [ ] The duration formatter test depends on the separately implemented `C_StringUtil.CreateNumericRuleFormatter` object.

The existing full-addon failure at `Blizzard_CustomAuraButton.lua:61` is the production RED boundary (`/tmp/pi-addon-fixes-full-current.stdout.log`). This implementation does not establish exact native error wording, secret-argument enforcement, or opaque userdata identity validation beyond the object interfaces available in the simulator.

## Out of scope

Formatter, duration-binding, and curve implementations; vendor/addon modifications; broader aura rendering or security-policy changes. RGB colors use the existing three-channel `CreateColor` conversion rather than introducing an aura-specific alpha default.
