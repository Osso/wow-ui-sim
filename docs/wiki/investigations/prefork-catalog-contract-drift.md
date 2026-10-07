# Prefork catalog contract drift

Three full-UI cases retained assumptions from removed housing wrappers and seeded storefront data. Correct the test inputs/contracts rather than restoring obsolete runtime behavior. Investigated against `ed1822abf` and the read-only retail 12.1.0 cache.

## Housing category and entry cases

`git log -S Blizzard_DeprecatedHousingCatalog -- data/blizzard-ui-files/retail.txt` identifies `bf6c3e3ea` as the manifest transition removing both the TOC and `Deprecated_HousingCatalog.lua`. `27f3215d2` updated discovery/source tests to assert that removal but left the two wrapper-field assertions unchanged. This is the first source-contract incompatibility for both cases, not a runtime producer defect. Current cached `Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua` does not install either wrapper.

Later `346c7e1be` (entry selectors) and `bbbacf8f0` (category getters) replace seeds with empty-default host maps. The old cases now also query absent records. The documented contracts require explicit host records, `totalNumStored`/`totalNumPlaced` and `anyStoredEntries`; neither legacy super-IDs nor `anyOwnedEntries` belongs to the current DTOs.

Replace the two cases with `blizzard_housing_catalog_entry_info_preserves_current_fields` and `blizzard_housing_catalog_category_info_preserves_current_fields`. They run after actual cached full-UI startup with deprecation fallbacks enabled, populate explicit records, assert exact current values and absence of every previously asserted legacy field, and remove records to prove no seed fallback. Both true and false stored predicates are checked. Entry snapshots are independently mutable.

Historical qualification: manifest history establishes the wrapper-removal commit. A runtime pass/fail bisect across that transition would require the old Blizzard cache; the supplied current cache no longer contains the old addon, and cache/vendor changes are excluded. Do not label the current-cache execution of an old revision as historical native-wrapper proof.

## Storefront population case

Adjacent-revision bisect in an isolated temporary worktree confirms `a0e23199d` as first bad: parent `9a7a0291b` passes the unmodified `catalog_shop_loads_and_populates_navigation_and_products`; `a0e23199d` fails `product_provider_empty`. That commit removes seeded product/display getters in favor of explicit empty-default maps. Cached `Blizzard_CatalogShop_Products.lua:467–470` skips missing products. The test never supplied them.

`9ff0ff93c` later migrates sections to host inputs; [section enumeration](patch-12-0-1-api-audit.md#catalog-section-enumeration-regression) already fixes its separate provider-assignment failure. Population fixtures now also need a host section. Keep the original navigation/provider/product-name assertions and supply explicit product, display, section and bundle records in the test. Do not restore runtime seeds or monkey-patch cached Lua.

## Sources

- [Housing category contract](../../specs/housing-catalog-categories.md)
- [Housing base selectors](../../../tests/housing_catalog_base_lookups.rs)
- [Product and section contract](../../specs/catalog-shop-product-structures.md)
- [Full-UI housing cases](../../../tests/blizzard_deprecated_housing_catalog_loads.rs)
- [Full-UI catalog case](../../../tests/catalog_shop.rs)
- Read-only retail cache: `Blizzard_Deprecated`, `Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua`, `Blizzard_CatalogShop/Blizzard_CatalogShop_Products.lua`.

## See Also

- [[patch-12-0-1-api-audit]] — independent section enumeration regression and proof.
