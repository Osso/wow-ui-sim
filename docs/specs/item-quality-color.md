# Item quality colors

The item quality color API resolves named color globals at call time. The implementation is in `src/c_api/item_quality_color.rs`.

## What it must do

- [ ] Resolve qualities 0–8 in order from `ITEM_POOR_COLOR`, `ITEM_STANDARD_COLOR`, `ITEM_GOOD_COLOR`, `ITEM_SUPERIOR_COLOR`, `ITEM_EPIC_COLOR`, `ITEM_LEGENDARY_COLOR`, `ITEM_ARTIFACT_COLOR`, `ITEM_SCALING_STAT_COLOR`, and `ITEM_WOW_TOKEN_COLOR`.
- [ ] Return red, green, blue and a color string. Missing named colors and out-of-range qualities return magenta RGB.
- [ ] Make changes to the named globals visible to the original Mists UIParent consumer.
- [ ] Publish `C_Item.GetItemQualityColor` on Retail, PTR, Mists, Era, Anniversary and Forever; their pinned `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua` declares it at lines 899, 921, 628, 628, 628 and 921 respectively. Wrath's original `FrameXML/UIParent.lua:102` calls the legacy global.

## How it works

- [C API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/item_quality_color.rs`: shared API and global lookup.
- `src/c_api/item_spell/mod.rs`: registration.

## Tests asserting this spec

- `tests/item_quality_color.rs`: mutable globals, token, fallback and Mists UIParent consumer.

## Known gaps (current cycle)

- [ ] Native evidence establishes four returns but not the source or exact format of the fourth string. The current hexadecimal representation follows the simulator's `GenerateHexColor` convention; native parity is unverified.
- [ ] The native override hook is not modeled.
- [ ] Native behavior for nonnumeric inputs and malformed color globals is not established.

## Out of scope

- The separate `ITEM_QUALITY_COLORS` Lua table is not forced to match the getter.
