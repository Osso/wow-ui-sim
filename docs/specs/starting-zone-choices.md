# Starting-Zone Choices

`C_CharacterCreation.GetStartingZoneChoices` supplies the choices consumed by the Glue character-creation screen. The simulator stores host inputs with the character-creation state.

## What it must do

- [ ] Retail with no host input returns two values: a choice table with `zoneName = ""`, `zoneImageAtlas = nil`, and `isNPE = false`, then nil.
- [ ] Retail with a normal-zone input and no restriction returns that first choice and a second choice with `zoneName = ""`, `zoneImageAtlas = nil`, and `isNPE = true`.
- [ ] Retail with a normal-zone record and the NPE restriction active returns the default first-choice fields with `isNPE = true` and no second choice.
- [ ] Forever returns a choice table and nil; `isNPE` is always false.
- [ ] PTR uses the Retail projection. PTR applicability is **INFERRED** from Retail 12.1.5.70688; no PTR-specific behavior is asserted.

## How it works

- [C API and Lua API architecture](../lua-api.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/c_character_creation.rs`: registers and publishes the starting-zone choices.
- `src/c_api/mod.rs`: exposes the character-creation host-input types.
- `src/lua_api/state/sim_state.rs`: stores the host inputs.
- `src/lua_api/state.rs`: initializes the inputs to none.

## Tests asserting this spec

- `tests/glue_character_select.rs::character_creation_default_starting_zone_reaches_customize_screen`
- `tests/glue_character_select.rs::retail_character_creation_two_starting_zone_choices_reach_forward_consumer`
- `tests/glue_character_select.rs::retail_character_creation_npe_restriction_skips_zone_choice`

## Known gaps (current cycle)

- [ ] PTR behavior has no separate native evidence; use the shared Retail projection only as the labelled inference above.

## Out of scope

- The native atlas lookup and its exact strings are unknown. The simulator keeps `zoneImageAtlas` nil unless a host input supplies a string.
- Mists and Wrath have no `GetStartingZoneChoices` consumer in their checked runtime caches, so this API is not registered for those profiles.
- Era and Anniversary are not part of this change.
