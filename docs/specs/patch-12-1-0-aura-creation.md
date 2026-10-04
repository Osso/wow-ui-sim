# Retail 12.1.0 aura-creation — C01

Bounded simulator proof for [prose-undated-009; prose-2026-06-18-037; prose-2026-07-23-207](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L9/L37: intrinsic container/button creation retains nondefault parent, size and visibility; real CustomAuraButtonTemplate initializer also passes.
- [x] L207: tainted addon creates container/button during combat and retains caller taint.
- [ ] Managed automatic aura presentation/security integration and pixels not established.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/xml/types_elements.rs — intrinsic XML types`
- `src/lua_api/globals/create_frame/ — creation`

## Tests asserting this spec

- `tests/patch_12_1_0_aura_creation.rs::patch_12_1_0_aura_creation_in_combat_preserves_parent_and_frame_state`
- `tests/forbidden_aspect_creation.rs::aura_button_icon_and_overlay_border_follow_real_initializer_order`

## Known gaps (current cycle)

- [ ] Managed automatic aura presentation/security integration and pixels not established.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
