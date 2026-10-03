# Housing bundle structures

Retail 12.0.5 row `structures-HousingBundleInfo-641` adds required `nonDecorProducts` to `HousingBundleInfo` in [source notes](../../data/patch-api/sources/12.0.5-api-changes.txt). Cached retail `Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua` declares nullable `C_HousingCatalog.GetBundleInfo` (68–81), required bundle-array `GetFeaturedBundles` (252–259), and bundle/decor-entry fields (501–521). Declarations provide shape evidence, not native execution proof. Follow the [catalog product DTO precedent](catalog-shop-product-structures.md): typed host inputs, empty defaults, explicit serializers.

## What it must do

- [ ] Start with no bundle or featured records. Never synthesize legacy bundle 5001 or fall back to Lua seeded records.
- [ ] Serialize exactly required numeric `productID`, `price`, required numeric array `nonDecorProducts`, required typed `decorEntries` array, and boolean `canPreview`; omit optional numeric `originalPrice` when absent.
- [ ] Preserve non-decor product order and numbers, including fractional values; each nested decor row contains exactly numeric `decorID` and `quantity`, preserving explicit zero counts.
- [ ] Emit required arrays even when empty. Bundles containing non-decor products cannot be previewed, as the cached `canPreview` documentation states; otherwise preserve host preview eligibility, including false.
- [ ] Both single and featured getters publish the same exact DTO shape. **INFERRED:** featured records are an independent ordered host-supplied sequence; no inferred relationship to the lookup map.
- [ ] **INFERRED:** an absent ID returns exactly one nil; return fresh deep snapshots, isolate environments, and expose host updates/removal only to subsequent calls.
- [ ] **INFERRED:** accept exact public i32 numeric selectors; reject missing, nonnumeric, fractional, nonfinite or out-of-range inputs without truncation/coercion. Secret inputs are rejected without unwrapping; native `AllowedWhenUntainted` acceptance is not modeled.

## How it works

- [Lua API architecture](../lua-api.md)
- [Catalog shop typed DTO precedent](catalog-shop-product-structures.md)

## Implementation inventory

- `src/c_api/c_housing_bundles.rs` — typed maps/featured records, getters, stack-rooted nested serializers.
- Main-session edits required in `src/c_api/mod.rs`, `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`, `src/lua_api/globals/missing_surface.rs` and `src/lua_api/workarounds/temporary/housing_catalog_state.lua`; exact replacements/removals are in the scratchpad handoff.

## Tests asserting this spec

- `tests/housing_bundle_structures.rs` — exact fields/types/order, required empty arrays, nullable omission, absent/legacy IDs, preview constraint, fresh snapshots and GC, host replacement/removal, featured order, malformed selectors, environment isolation.
- Generated integration binary filter: `housing_bundle_structures::`.
- Tests authored before implementation; no compilation or RED/GREEN execution authorized. All requirements remain unchecked.

## Known gaps (current cycle)

- [ ] Main session must apply handoff wiring and remove both seeded bundle getter publishers and their unused copy helper.
- [ ] Main session must compile, run focused fixtures and check startup; no storefront panel acceptance claimed.

## Out of scope

- Purchases, market actions, product availability, catalog-entry structure deltas, native secret/coercion/error-wording parity, other-profile execution and 3D previews. Remaining seeded market operations are not made state-backed by this DTO slice.
