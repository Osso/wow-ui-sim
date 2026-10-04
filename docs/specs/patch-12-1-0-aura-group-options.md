# Retail 12.1.0 aura-group-options — C09

Bounded simulator proof for [prose-2026-07-07-127; prose-2026-07-07-128; prose-2026-07-07-129](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L127: three real native aura icons occupy increasing x positions in AuraInstanceIDOnly normal order; reversing direction changes actual icon order from 11/22/33 to 33/22/11.
- [x] L128: initializeFrame runs exactly once for every actually allocated AuraButton, retains addon closure taint and creates live size/icon/anchor state.
- [x] L129: two addon XML templates apply size 37x29 and alpha 0.5 before each callback; base CustomAuraButtonTemplate icon methods remain usable.
- [ ] Original additional-template KeyValue probe expected public fields and failed; no claim of public/private arbitrary-field publication. Reproduction archived; size/alpha proof establishes actual template application, not the field ownership contract.
- [ ] Other sort methods, tie-breaking, combat-secret ordering and lifetime beyond tested initialization not covered.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `cached Blizzard_AuraContainer/Blizzard_AuraContainerFrameProviders.lua — real callback/provider`
- `cached Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua — sorting`
- `src/loader/ — actual XML template loading`

## Tests asserting this spec

- `tests/patch_12_1_0_aura_group_options.rs — initialize_each_native_button_without_extra_templates; sort_direction_changes_actual_icon_positions; apply_additional_template_size_and_alpha`

## Known gaps (current cycle)

- [ ] Original additional-template KeyValue probe expected public fields and failed; no claim of public/private arbitrary-field publication. Reproduction archived; size/alpha proof establishes actual template application, not the field ownership contract.
- [ ] Other sort methods, tie-breaking, combat-secret ordering and lifetime beyond tested initialization not covered.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
