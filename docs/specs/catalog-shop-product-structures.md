# Catalog shop product structures

`C_CatalogShop.GetProductInfo(productID)` and `C_CatalogShop.GetCatalogShopProductDisplayInfo(catalogShopProductID)` expose explicit, per-environment product and display records. This bounded contract covers six [12.0.5 structure deltas](../../data/patch-api/sources/12.0.5-api-changes.txt), not shop restrictions or purchases. Cached retail `Blizzard_APIDocumentationGenerated/CatalogShopDocumentation.lua` supplies the complete DTO shapes: product at lines 727–776, display at 686–724, nested subitem/currency/quantity at 801–836. Cache declarations are shape evidence, not native execution proof.

## What it must do

### Stored records and exact DTOs

- [ ] Read separate, empty-default product and display maps by exact product ID. Never manufacture products, derive display fields from product fields, or use legacy seeded IDs 2003/20031/20032 as fallback records.
- [ ] Serialize every declared product field (45) and display field (34), with exact Lua names and types, and no extra fields. Preserve explicit boolean false and all numeric values; `number` fields use Rust `f64`, strings/texture kits/texture atlases use `String`, booleans use `bool`, nested records use typed structs, sequences use `Vec`, nullable fields use `Option`. `ItemQuality` is represented as an `i32` enum value. Rust members use snake_case, with `r#type` mapping to Lua `type`.
- [ ] `additionalProductPMTURLs` is a required ordered string array, including when empty. A nonempty host array round-trips unchanged (`structures-CatalogShopProductDisplayInfo-631`).
- [ ] `houseTextureAtlas` is an optional string, omitted when absent (`structures-CatalogShopProductDisplayInfo-632`).
- [ ] `previewBGOverrideProductURL` and `previewSmallBGOverrideProductURL` are optional strings, omitted when absent (`structures-CatalogShopProductInfo-634`, `structures-CatalogShopProductInfo-635`).
- [ ] `decorQuantity` is an optional table containing exactly required numeric `placedQuantity` and `storedQuantity`. Copy explicit counts, including zero; never infer them from housing totals (`structures-CatalogShopProductInfo-636`).
- [ ] Never emit `consumableQuantity`, including in a fully populated product; observable `rawget(product, 'consumableQuantity')` is nil (`structures-CatalogShopProductInfo-637`).
- [ ] Serialize all declared subitem fields (`name`, `itemID`, `itemAppearanceID`, `invType`, `quality`) and currency fields (`amount`, `currencyCode`); every array and nested row has exact shape. Omit every nullable field when `None`, not only the new fields.

### Explicit simulator policies

- [ ] **INFERRED:** Each getter returns exactly one nil for an absent record. `GetProductInfo` declares nullable `productInfo` (226–239); `GetCatalogShopProductDisplayInfo` declares **nonnullable** `item` (85–98), so its requested nil-on-miss simulator policy differs from that declaration. No native miss behavior is claimed.
- [ ] **INFERRED:** Each result and nested table is a fresh snapshot. Lua mutation cannot change host inputs or another result. Host replacement/removal affects subsequent queries only. Environments remain isolated. This is requested simulator ownership behavior, not native aliasing proof.
- [ ] **INFERRED:** Accept only public, exact `i32` numeric selectors; reject nil, nonnumeric, fractional, nonfinite and out-of-range values rather than truncating or coercing them. Cached arguments are nonnullable `number`, not an explicit 32-bit integer declaration. Secret values are conservatively rejected without unwrapping; native `AllowedWhenUntainted` acceptance is unmodeled.

## How it works

- [Lua API architecture](../lua-api.md)
- [Housing DTO precedent](housing-catalog-categories.md)

## Implementation inventory

- `src/c_api/c_catalog_shop_products.rs` — typed record maps, getters, field-by-field stack-rooted snapshots and nested serializers.
- Integration pending in `src/c_api/mod.rs`, `src/lua_api/globals/missing_surface.rs`, `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs`; exact edits are supplied to the main session, not performed by this slice.
- Matching seeded getters and their now-unused copy helpers in `src/lua_api/workarounds/temporary/housing_catalog_state.lua` must be removed; unrelated seeded catalog APIs are not replaced here.

## Tests asserting this spec

- `tests/catalog_shop_product_structures.rs` — six row-specific fixtures; exact fully populated product/display snapshots; all nullable omissions and required empty arrays; unknown/legacy IDs; recursive snapshot independence with garbage collection; live replacement/removal; exact-map separation; environment isolation; zero counts; malformed public selectors.
- Parent test filter: `catalog_shop_product_structures::` in the generated `integration` binary under `retail-12-0-5`.
- No compilation, formatting or test execution performed by this slice. Requirements remain unchecked. Main session owns wiring and acceptance.

## Known gaps (current cycle)

- [ ] Wire state/module/registration and remove matching seeded publishers before running behavioral fixtures.
- [ ] Main session must format, compile, run focused fixtures and check startup behavior. Remaining seeded category/product-ID lists can refer to products absent from these new empty maps; no UI interaction acceptance is claimed.

## Out of scope

- `HasRestrictions` rows `global api-C_CatalogShop-GetProductInfo-247` and `global api-C_CatalogShop-PurchaseProduct-249`: declarations supply no restriction predicate. No restriction or purchase behavior is modeled or credited.
- `PurchaseProduct`, other catalog APIs, housing bundles and other structure rows; no native secret-argument parity, native validation/error-wording claims, other-profile execution or storefront panel acceptance.
