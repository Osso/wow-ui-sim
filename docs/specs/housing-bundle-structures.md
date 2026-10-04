# Housing bundle structures

Retail 12.0.5 audit row 641 adds required numeric array `nonDecorProducts` to `HousingBundleInfo`. Cached retail `Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua` declares nullable `GetBundleInfo` (68–81), bundle-array `GetFeaturedBundles` (252–259), `HousingBundleDecorEntryInfo` and bundle fields (501–521). `canPreview` documentation says: "Bundles containing non-decor items cannot be previewed". Declarations establish shape, not native execution behavior.

## What it must do

- [ ] Register both getters and `HousingMarketActionViewBundle` on every client profile; replace their unconditional Lua publishers without a fallback.
- [ ] Hold bundle content and viewed state in one typed per-environment record map. Featured IDs select those same records, never independent bundle copies.
- [ ] Preserve the existing seed: featured bundle 5001, price 500, absent original price, decor IDs 1001/1002 with quantity 1, empty non-decor products, preview enabled, initially not viewed.
- [ ] Serialize declared numeric `productID`, `price`, optional numeric `originalPrice`, required numeric `nonDecorProducts` array, required `decorEntries` array, and boolean `canPreview` for both getters. Preserve array order, fractional numeric values and explicit zero quantities. Each decor row contains only numeric `decorID` and `quantity`.
- [ ] Retain `entryIDs` and `wasViewed` as named simulator compatibility extensions, not native declaration fields. Derive `entryIDs` from ordered decor IDs rather than storing duplicate contents.
- [ ] Viewing an existing bundle returns true and marks that record viewed; both subsequent getters reflect it. Repeated views return true; missing IDs return false and create no records. This boolean return preserves existing simulator behavior despite the cached action declaration listing no returns.
- [ ] Required arrays remain tables when empty; absent `originalPrice` is omitted. Non-decor contents disable preview; otherwise preserve the host eligibility flag, including false.
- [ ] **INFERRED:** missing lookup IDs return one nil, featured missing/removed records are omitted without holes, snapshots are deeply fresh, host mutations affect subsequent reads, and environments remain isolated.
- [ ] **INFERRED:** selectors are exact public i32 numbers; reject missing, nonnumeric, fractional, nonfinite or out-of-range inputs without truncation. Malformed action inputs now error consistently with the getter instead of silently missing a Lua table key.

## How it works

[Lua API architecture](../lua-api.md) describes registration and stack dispatch. [Catalog shop products](../../src/c_api/c_catalog_shop_products.rs) supplies the typed input/explicit serializer and stack-rooting precedent; this slice deliberately retains the pre-existing bundle seed instead of copying that subsystem's empty defaults. `HousingBundles.records` owns data and viewed state; `featured_ids` owns only selection/order.

## Tests asserting this spec

`tests/housing_bundle_structures.rs` is auto-included in the single `integration` binary. All-profile cases cover seeded view behavior, featured reflection, missing/repeated action results, host updates/removal, fresh nested snapshots through GC, optional/empty output, environment isolation, preview eligibility, and invalid inputs without mutation. Only the exact 12.0.5 declaration-shape case is feature-gated.

Existing `tests/housing_catalog.rs::housing_catalog_storefront_and_market_methods_use_seeded_state` remains unchanged: seed 5001 still starts with two entry IDs, preview true, viewed false; the view action returns true and subsequent lookup reports viewed true. Its market/cart cases retain their independent decor-market state.

## Proof and gaps

Authoring only, 2026-10-03. No cargo, tests, git mutations, runtime acceptance, or audit-ledger changes authorized. Static anchor verification and scratch-only formatting are not compilation/RED/GREEN proof. Requirements stay unchecked until main-session acceptance.

Native `AllowedWhenUntainted` secret acceptance/coercion, action restrictions/events, purchases, actual storefront panel behavior, native snapshot aliasing, and native fidelity of the two simulator extensions remain unverified/out of scope. Other-profile availability is authored unconditionally; other-profile execution is not proved. No native action or full exact-structure parity credit from preserving simulator extensions.

## Development proof and independent bounded acceptance — 2026-10-03 (housing-bundle-structures)

Commit `4d142e325`. RED: 0 PASS / 6 FAIL. GREEN: 6/6. This section supersedes wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun), [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b99-verify-housing-bars.md) SHA256 `6cc70517c54675b995a442bc47d2fa681c21e457ea32e5dd0d9d582baa6fc65d`. Seed-only shape assertions are not RED evidence; all-profile execution not run. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): structures-HousingBundleInfo-641 bounded-coverage under capability `housing-bundle-structures`; **131 capabilities/362 IDs; 28 pending /269 bounded /30 partial /35 metadata**.
