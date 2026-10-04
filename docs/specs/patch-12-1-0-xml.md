# Retail 12.1.0 xml — C05

Bounded simulator proof for [prose-2026-06-18-064; prose-2026-06-18-065; prose-2026-06-18-066; prose-2026-06-18-067](../../data/patch-api/sources/12.1.0-api-changes.txt), captured 2026-10-04. No native-client parity or live-service claim.

## What it must do

- [x] L64: local KeyValue preserves private table identity and live mutation on literal and runtime instances without cross-addon/global leakage.
- [x] L65/L66: Mixins block applies local nested entries in sequence; later entry wins.
- [x] L67: nested qualified mixins resolve in both global attribute and local block, and resulting methods execute on concrete frames.
- [ ] Exhaustive invalid-path/type diagnostics and security delegation beyond ordinary addon loading not asserted.

## How it works

- [Lua API](../lua-api.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/loader/xml_frame/ — XML mixin and key-value loading`
- `src/lua_api/globals/create_frame/ — runtime template application`

## Tests asserting this spec

- `tests/patch_12_1_0_xml.rs::patch_12_1_0_xml_preserves_private_identity_nested_mixins_and_addon_isolation`

## Known gaps (current cycle)

- [ ] Exhaustive invalid-path/type diagnostics and security delegation beyond ordinary addon loading not asserted.

## Out of scope

- Historical intermediate publication timing, native parity, live services and visual rendering beyond asserted frame state.
