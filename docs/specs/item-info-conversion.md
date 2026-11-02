# ItemInfo conversion

C_Item queries share item identity conversion. Retail build 70688 evidence is recorded in the WM23 item-a handoff; the pinned declaration for `C_Item.GetItemIDForItemInfo` is `Blizzard_APIDocumentationGenerated/ItemDocumentation.lua:559`. That document also declares ItemInfo arguments for `GetItemIconByID` (589), `GetItemMaxStackSizeByID` (794), `GetItemNameByID` (824), `GetItemQualityByID` (884), `IsItemDataCachedByID` (1483), and `RequestLoadItemDataByID` (1673).

## What it must do

- [ ] Truncate finite in-range numeric inputs to signed 32-bit IDs, including zero and negative IDs.
- [ ] Resolve item links and numeric item strings to IDs. The numeric-string form preserves existing simulator behavior; native coercion is not established.
- [ ] Resolve item names against the generated item catalog. An unresolved string resolves to ID zero.
- [ ] Share the conversion for C_Item queries that consume item identity, including equippable and consumable classification.

## How it works

- Generated catalog: `crates/sim-data/data/items.rs`.

## Implementation inventory

- `src/c_api/item_spell/c_item.rs`: signed conversion and generated-name index.

## Tests asserting this spec

- `tests/item_info_conversion.rs`: signed numeric, item-link, generated-name and unresolved-string cases.

## Known gaps (current cycle)

- [ ] Exact native name matching, duplicate-name selection, locale and malformed-link behavior remain unknown. The simulator uses case-insensitive matching and the lowest generated ID for duplicate names.
- [ ] Exceptional numbers, full coercion rules and secret restrictions require further native proof.
- [ ] Classic and Forever native conversion remains unverified; WM23 evidence is Retail build 70688.
- [ ] The existing C_Item.IsItemInRange placeholder ignores item identity; it does not yet use the shared conversion.

## Out of scope

- Server-side item lookup and cache loading.
