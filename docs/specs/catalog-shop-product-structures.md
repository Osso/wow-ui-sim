# Catalog shop product structures

`C_CatalogShop.GetProductInfo(productID)` and `C_CatalogShop.GetCatalogShopProductDisplayInfo(catalogShopProductID)` expose explicit, per-environment product and display records. This bounded contract covers six [12.0.5 structure deltas](../../data/patch-api/sources/12.0.5-api-changes.txt), not shop restrictions or purchases. Cached retail `Blizzard_APIDocumentationGenerated/CatalogShopDocumentation.lua` supplies the complete DTO shapes: product at lines 727–776, display at 686–724, nested subitem/currency/quantity at 801–836. Cache declarations are shape evidence, not native execution proof.

## What it must do

### Stored records and exact DTOs

- [x] Read separate, empty-default product and display maps by exact product ID. Never manufacture products, derive display fields from product fields, or use legacy seeded IDs 2003/20031/20032 as fallback records.
- [x] Serialize every declared product field (45) and display field (34), with exact Lua names and types, and no extra fields. Preserve explicit boolean false and all numeric values; `number` fields use Rust `f64`, strings/texture kits/texture atlases use `String`, booleans use `bool`, nested records use typed structs, sequences use `Vec`, nullable fields use `Option`. `ItemQuality` is represented as an `i32` enum value. Rust members use snake_case, with `r#type` mapping to Lua `type`.
- [x] `additionalProductPMTURLs` is a required ordered string array, including when empty. A nonempty host array round-trips unchanged (`structures-CatalogShopProductDisplayInfo-631`).
- [x] `houseTextureAtlas` is an optional string, omitted when absent (`structures-CatalogShopProductDisplayInfo-632`).
- [x] `previewBGOverrideProductURL` and `previewSmallBGOverrideProductURL` are optional strings, omitted when absent (`structures-CatalogShopProductInfo-634`, `structures-CatalogShopProductInfo-635`).
- [x] `decorQuantity` is an optional table containing exactly required numeric `placedQuantity` and `storedQuantity`. Copy explicit counts, including zero; never infer them from housing totals (`structures-CatalogShopProductInfo-636`).
- [x] Never emit `consumableQuantity`, including in a fully populated product; observable `rawget(product, 'consumableQuantity')` is nil (`structures-CatalogShopProductInfo-637`).
- [x] Serialize all declared subitem fields (`name`, `itemID`, `itemAppearanceID`, `invType`, `quality`) and currency fields (`amount`, `currencyCode`); every array and nested row has exact shape. Omit every nullable field when `None`, not only the new fields.

### Explicit simulator policies

- [x] **INFERRED:** Each getter returns exactly one nil for an absent record. `GetProductInfo` declares nullable `productInfo` (226–239); `GetCatalogShopProductDisplayInfo` declares **nonnullable** `item` (85–98), so its requested nil-on-miss simulator policy differs from that declaration. No native miss behavior is claimed.
- [x] **INFERRED:** Each result and nested table is a fresh snapshot. Lua mutation cannot change host inputs or another result. Host replacement/removal affects subsequent queries only. Environments remain isolated. This is requested simulator ownership behavior, not native aliasing proof.
- [ ] **INFERRED:** Accept only public, exact `i32` numeric selectors; reject nil, nonnumeric, fractional, nonfinite and out-of-range values rather than truncating or coercing them. Cached arguments are nonnullable `number`, not an explicit 32-bit integer declaration. Secret values are conservatively rejected without unwrapping; native `AllowedWhenUntainted` acceptance is unmodeled.

## How it works

- [Lua API architecture](../lua-api.md)
- [Housing DTO precedent](housing-catalog-categories.md)

## Implementation inventory

- `src/c_api/c_catalog_shop_products.rs` — typed record maps, getters, field-by-field stack-rooted snapshots and nested serializers.
- Wired in `src/c_api/mod.rs`, `src/lua_api/globals/missing_surface.rs`, `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`.
- Matching seeded getters and their now-unused copy helpers in `src/lua_api/workarounds/temporary/housing_catalog_state.lua` must be removed; unrelated seeded catalog APIs are not replaced here.

## Tests asserting this spec

- `tests/catalog_shop_product_structures.rs` — six row-specific fixtures; exact fully populated product/display snapshots; all nullable omissions and required empty arrays; unknown/legacy IDs; recursive snapshot independence with garbage collection; live replacement/removal; exact-map separation; environment isolation; zero counts; malformed public selectors.
- Parent test filter: `catalog_shop_product_structures::` in the generated `integration` binary under `retail-12-0-5`.
- Compiled and executed by the main session; see the proof section.

## Development proof and independent bounded acceptance — 2026-10-03

Inputs and producer landed together in `a0e23199d`. RED with the producers withheld from the working tree: 0 PASS / 16 FAIL. GREEN: 16/16; the combined run was 396 PASS / 1 FAIL, the failure being `c_system_api::test_c_console_get_all_commands_empty` on an untouched console command count. `cargo fmt --check` exit0; startup `lua-errors` `[]`.

Main accepts an independent GPT-6.1-sol review: **ACCEPT WITH QUALIFICATIONS** (report SHA256 `caf669b3a28143b86dd45ae1f8eeca25e4b0a66992b5ce70640e3520ee1432bf`, scratchpad-only), own rerun 16/16 exit0. Secret-selector requirement stays unchecked (only malformed public selectors are tested). Panels were not opened. Checked requirements are bounded simulator proof on the tested fixtures, not native parity. No `cargo check`, broad suite or older-profile run.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): rows 631, 632, 634, 635, 636, 637 under new capability `catalog-shop-product-structures`; **107 capabilities/362 IDs; 77 pending /239 bounded /13 partial /33 metadata**.

## Known gaps (current cycle)

- [x] State, module and registration wired; the two seeded Lua publishers and their helpers removed.
- [ ] Storefront and housing panels were not opened. Seeded category/product-ID lists still name products absent from the empty maps; cached catalog code skips missing products by inspection, while `Blizzard_HousingMarketProductDisplay.lua:46` and shared card code dereference without a nil check (those housing IDs were absent from the old getters too).

## 12.0.1 extract bundle and section DTOs

`CatalogShopProducts` also stores explicit bundle child arrays and sections keyed by `(categoryID, sectionID)`. `GetSectionIDsForCategory` enumerates only stored section keys for that category, returning a fresh empty array when there are none. **INFERRED:** section IDs use ascending numeric order until host display-order metadata is modeled. Removing a host section removes it from subsequent enumeration; missing section queries still raise an error. Seeded category navigation must not advertise nonexistent sections. `GetProductIDsForBundle` preserves host array order and exact numeric `childProductID`, `displayOrder`, `quantityInBundle`; no synthetic quantity or seeded bundle fallback. `GetCategorySectionInfo` copies `ID`, `displayName`, optional parent ID/card type/grid size and required `shouldShowRecommendationOptOutDisclaimer`, including false. Both return fresh snapshots. Matching temporary Lua publishers are removed.

**INFERRED:** unknown bundles return an empty array; missing nonnullable sections raise a contextual host-input error. Public exact-i32 selector policy is shared with product queries. Native service availability, secret-selector acceptance and storefront navigation remain unproved.

`catalog_shop_patch_12_0_1_bundle_section_snapshots` tests two distinct quantities, nonsequential display orders, disclaimer transitions, optional omissions, independent snapshots and old `otherProductPMTURL` absence. Existing full product/display fixtures cover retained DTO fields.

## Out of scope

- `HasRestrictions` rows `global api-C_CatalogShop-GetProductInfo-247` and `global api-C_CatalogShop-PurchaseProduct-249`: declarations supply no restriction predicate. No restriction or purchase behavior is modeled or credited.
- `PurchaseProduct`, other catalog APIs beyond the bundle/section getters and section enumeration above, housing bundles and other structure rows; no native secret-argument parity, native validation/error-wording claims, other-profile execution or storefront panel acceptance.
