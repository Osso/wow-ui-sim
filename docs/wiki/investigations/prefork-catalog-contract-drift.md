# Prefork catalog contract drift

Three full-UI cases relied on runtime seeds replaced by explicit empty-default host maps. Retain the assertions and provide host fixtures. A populated housing entry also exposes a missing producer field required by Blizzard's cached deprecation wrapper.

## Root causes and first bad commits

Adjacent-revision execution bisects used a separate temporary worktree and the unchanged retail cache. Each parent passed its original unmodified case and each named commit failed it. Logs remain in the task scratchpad.

| Original case | Passing parent | First bad commit | Cause / decision |
| --- | --- | --- | --- |
| `blizzard_deprecated_housing_catalog_wraps_category_info_with_legacy_field` | `0e9ab5c83` | `bbbacf8f0` | Category/subcategory getters now read empty-default host maps; test never supplies records. Add explicit fixtures, preserve both mirror assertions, additionally prove false mirrors. |
| `blizzard_deprecated_housing_catalog_wraps_get_catalog_entry_info_with_legacy_fields` | `d672038d2` | `346c7e1be` | Base selectors stop returning seeded entries. Add a host entry; this then reproduces arithmetic-on-nil `remainingRedeemable` in the cached wrapper. Model that independent host count and serialize it for all three selectors. Preserve every legacy-field assertion. |
| `catalog_shop_loads_and_populates_navigation_and_products` | `9a7a0291b` | `a0e23199d` | Product/display getters now read empty-default maps; cached product-provider construction skips missing products. Supply explicit product/display inputs and host sections, without restoring runtime seeds or weakening populated-provider/name assertions. |

`bf6c3e3ea` removes the standalone `Blizzard_DeprecatedHousingCatalog` addon, but **does not remove its behavior**: wrappers moved to `Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua`. The initial contrary migration was reverted in `b3727ab81`; populated-fixture failures falsified it. Assertions must follow loaded behavior, not the old addon path or only the latest deprecation file.

## Producer boundary

`HousingCatalogEntryRecord.remaining_redeemable: Option<u32>` mirrors the existing explicit aggregate-input policy: no sum, inferred zero or seed fallback; omitted input remains a documented simulator data gap. `Some(0)` publishes numeric zero. The shared entry serializer publishes `remainingRedeemable`, so record-ID, item and compound-ID queries all work through the unmodified cached wrappers when host inputs are complete. Raw DTOs still omit legacy fields; cached Lua adds them.

Raw integration coverage checks a redeemable count of 13 independent of zero stored / 11 placed and variant counts 3/5, snapshot isolation and subsequent explicit zero. Full-UI coverage checks actual wrapper `showQuantity` for stored-only, redeemable-only and empty counts, all three selectors and fresh results. No complete DTO/native parity claim.

The separate [section enumeration regression](patch-12-0-1-api-audit.md#catalog-section-enumeration-regression) remains fixed by `46e83785f`; current population tests must now explicitly supply sections as well as products. No cache/vendor edits or Blizzard monkey-patches.

## Verification

Fixes: `c5e930fdd` supplies housing fixtures and the redeemable producer; `c2db360a6` supplies storefront host fixtures. All commands below ran locally in the fix worktree at `c2db360a6`; later changes only record documentation. Every test command was captured once to a scratchpad log and inspected. Temporary bisect worktree removed; no push or merge.

| Command / scope | Result |
| --- | --- |
| `cargo test --test prefork_full_ui -- <each original exact case name above>` (three separate runs) | 1/1 each |
| `cargo test --test prefork_full_ui -- catalog` | 27/27 |
| `cargo test --test prefork_full_ui -- housing` | 203/203 |
| `cargo test --test integration catalog_shop` | 24/24 |
| `cargo test --test integration housing` | 292/292, including independent redeemable snapshot regression |
| `cargo fmt`, post-commit `cargo fmt --check`, `git diff --check` | exit 0 |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | exit 0; zero non-vendor warnings, six existing iced vendor-manifest deprecations |

Changed-line readability audit found no new violations. Serializer cognitive/cyclomatic complexity is 4/4; new raw snapshot test is 0/1. Full-UI macro bodies are manually reviewed because the metrics parser does not expand their contents. Proof ledger/logs are under the task's session scratchpad; no native-client, full-project-suite or complete metadata-parity claim.

## Sources

- [Housing aggregate and redeemable contract](../../specs/housing-catalog-aggregates.md)
- [Housing category contract](../../specs/housing-catalog-categories.md)
- [Product/section contract](../../specs/catalog-shop-product-structures.md)
- [Housing full-UI cases](../../../tests/blizzard_deprecated_housing_catalog_loads.rs)
- [Catalog full-UI case](../../../tests/catalog_shop.rs)
- Read-only retail cache: `Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua:76–111,130–150,184–194`, `Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua:555–557`, `Blizzard_CatalogShop/Blizzard_CatalogShop_Products.lua:467–470,567–581`.

## See Also

- [[patch-12-0-1-api-audit]] — independent section enumeration regression.
