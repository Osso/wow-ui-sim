# Housing catalog variant inputs

Bounded 12.0.5 catalog identity contract from [retained changes](../../data/patch-api/sources/12.0.5-api-changes.txt) and cached HousingCatalog/Searcher declarations. This cycle adds inputs and grouped tests only; the temporary seeded Lua output provider remains unchanged. [Ownership findings](../wiki/investigations/patch-12-0-5-api-audit.md#Housing catalog variant inputs — pending RED) record integration boundaries.

## What it must do

### Explicit inputs and identity

- [ ] `HousingState.catalog` starts empty in each environment. Production catalog records must not be invented; only explicit supplied inputs populate it.
- [ ] Base entries use `(recordID, entryType)`; variant stacks use the full `(recordID, entryType, variantIdentifier)` key. A base entry is not variant zero.
- [ ] Filter-free `GetAllSearchItems` exposes all source variant IDs. After explicit `RunSearch`, `GetCatalogSearchResults` exposes matching variant IDs. Two variants of one record remain distinct in both getters; no order or table-aliasing requirement.
- [ ] Fresh independently created searchers work without loaded Blizzard addons, callback registration, global fixture state or release helpers. Only create → run → read is exercised; callback/automatic update/snapshot timing remain unverified.
- [ ] `GetCatalogEntryVariantInfo(entryVariantID)` reads explicitly different `numStored` and `dyeSlots` for each compound key; no invented product/name/legacy `variantID` fields. `numStored` is storage input, never a destroyability policy.
- [ ] `GetAllVariantInfosForEntry(entryID)` lists the fixture's distinct variant stacks; fixture replacement coverage, not an additional 12.0.5 signature-change claim.
- [ ] `GetCatalogEntryInfo(entryID)` remains base info: explicit record/type/item/name/trophy fields, no synthesized `entryID`, `entryVariantID`, `variantIdentifier` or variant `numStored`. This is a bounded field subset, not a complete entry DTO.
- [ ] **Inferred simulator lookup policy:** absent full variant keys return nil; absent base entries return nil and variant lists are empty. Wrong type must not resolve solely by record ID; no seed/numeric/variant-one fallback. Native invalid-enum, malformed-input, coercion and error behavior remain unknown.

### Security

- [ ] Ordinary selectors remain callable from tainted addon code without clearing caller taint, consistent with cached `SecretArguments = "AllowedWhenUntainted"` on the three info/list queries.
- [ ] Secret selector enforcement remains unverified: the annotation allows secret arguments only while untainted, but nested compound-field handling, propagation and secure-versus-tainted behavior need actual tests before claiming security parity. No unwrap/bypass or permissive secret policy is authorized here.

The two no-argument search getters have no `SecretArguments` annotation in the inspected cache. This is not proof that returned identities are always public in native WoW.

## How it works

- [Housing ownership and pending proof](../wiki/investigations/patch-12-0-5-api-audit.md#Housing catalog variant inputs — pending RED).
- [Independent existing free-place contract](housing-free-place-state.md).

## Implementation inventory

- `src/c_api/c_housing/catalog.rs`: plain typed entry/variant/dye inputs and empty-default maps; no producer, serializer, lookup implementation or registration.
- `src/c_api/c_housing.rs`: exports the input module; existing housing registrations unchanged.
- `src/lua_api/state/support_types.rs`: `HousingState.catalog` references the C API-owned input model. Existing house reset replaces `HousingState` with its derived default; catalog reset behavior is untested.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: unchanged temporary catalog/searcher and unrelated storefront/cart/exterior/selection providers.
- `tests/housing_catalog_variants.rs`: eleven self-contained cases in the existing generated `integration` target, not a new Cargo target.
- `tests/housing_catalog.rs`: preserves featured/bundle/market/cart coverage; variant and base-info seed assertions moved to explicit fixtures.

## Tests asserting this spec

All eleven `housing_variant_*` cases in `tests/housing_catalog_variants.rs` are **unrun**. No build/check/delegation authorized for this input-only cycle. No GREEN or actual fixture RED claim.

| Contract | Cases |
|---|---|
| Default empty source/results | `default_search_source_is_empty`, `default_search_results_are_empty` |
| Full IDs with independent searchers | `search_source_preserves_compound_ids`, `search_results_preserve_compound_ids` |
| Explicit storage/dye data and base separation | `query_preserves_distinct_stack_fields`, `list_uses_explicit_fixture_records`, `entry_info_remains_base_info` |
| Missing selectors and environment isolation | `default_queries_have_no_seed_records`, `missing_selectors_do_not_use_legacy_seeds`, `inputs_do_not_seed_another_environment` |
| Ordinary addon selector security | `public_selectors_allow_tainted_callers` |

Replacement mapping: old `housing_catalog_market_and_variant_methods_use_seeded_state` checked variant 2 and a two-element list using temporary `variantID`/`productID`/`name` extensions. The new query/list fixtures assert the declared `entryVariantID`, stored and dye fields instead. Old itemID=1001 and isUniqueTrophy=false assertions are preserved under explicit base-info inputs. Featured products, bundle preview/viewed state, all market/cart operations and the two standalone cart tests remain unchanged in `housing_catalog.rs`. No source-only substring/shape tests substitute for behavioral fixtures.

### Proof ledger and producer gate

| Command / scope | Result | Invalidation |
|---|---|---|
| Baseline `git status --short`, `git rev-parse HEAD` | Clean `5b0644b3373244574ee4a7b1a1958fc82f5529b5` | Later edits not covered |
| `rustfmt --edition 2024 --config skip_children=true src/c_api/c_housing/catalog.rs src/c_api/c_housing.rs src/lua_api/state/support_types.rs tests/housing_catalog.rs tests/housing_catalog_variants.rs` | Exit 0 on input/tests-only working scope; formatting only | Later Rust changes invalidate formatting scope |
| Future compiled `integration` filter `housing_catalog_variants::` | **Not run; actual RED pending** | Must cover committed inputs/tests against unchanged output provider |

**No output-producer edits before actual RED.** A compile failure, absent API, stale binary or predicted seeded mismatch is not behavior-level RED. A later authorized runner must compile these fixtures, execute them against the unchanged provider and preserve exact assertions/output/revision before replacing outputs. This cycle does not run an old binary and represent it as testing newly added typed state.

## Known gaps (current cycle)

- [ ] Actual compiled fixture RED; no query implementation or acceptance credit yet.
- [ ] Search-item source versus search-result collection must stay distinct even though they coincide in the filter-free fixtures. Real filtering, async updates and count semantics are not covered.
- [ ] Complete base-entry metadata/aggregate publication and secret-field enforcement are missing. Typed integer inputs cover fixtures, not verified native numeric ranges.
- [ ] Remaining historical seed consumers must be reconciled at producer replacement, not silently removed. Deprecated catalog tests exercise an old three-argument ByRecordID wrapper and seeded legacy fields; current retail cache no longer ships that addon. Existing customize selection remains a separate temporary fixture. Neither is replaced here.
- [ ] ByItem/ByRecordID and category-name providers remain outside this slice. Migration must remove only replaced Lua keys/helpers and preserve unrelated namespaces. No alternate fallback is a requirement.

## Out of scope

- Destroy mutation, destroyable counts, aggregate/search count policies, storage events and placement selection: source deltas establish types, not mutation/emission/count/placement policy.
- All search filters, category/subcategory ownership predicates and sorting: intentionally filter-free fixtures; no invented matching semantics.
- Storefront/cart/exterior/customize-selection behavior and legacy wrapper migration: unrelated owners retained, not declared authoritative catalog data.
- Native parity, loaded UI acceptance, broad checks/builds, delegation, push and source-row completion: not authorized or established in this cycle.

## Source and accounting boundary

Retained source SHA-256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`; [362-row register](../../data/patch-api/sources/12.0.5-register.json) and [coverage register](../../data/patch-api/sources/12.0.5-page-coverage.json) are unchanged. This slice changes no register statuses/IDs and awards no row credit; concurrent unrelated audit commits are outside its proof scope.

| Exact accounting IDs | This slice |
|---|---|
| `scriptobjects-HousingCatalogSearcher-GetAllSearchItems-524`, `scriptobjects-HousingCatalogSearcher-GetAllSearchItems-525`, `scriptobjects-HousingCatalogSearcher-GetCatalogSearchResults-527`, `scriptobjects-HousingCatalogSearcher-GetCatalogSearchResults-528` | Unrun full-ID fixtures for output name/inner-type changes; not filter proof |
| `structures-HousingCatalogEntryInfo-648`, `structures-HousingCatalogEntryInfo-649`, `structures-HousingCatalogEntryInfo-652` | Unrun bounded record/type/removed entryID assertions; no full entry-info claim |
| `structures-HousingDecorDyeSlot-663` | Explicit fixture `dyeColorName`; publication unrun |
| `structures-HousingCatalogEntryID-645`, `structures-HousingCatalogEntryID-646` | Base input omits legacy subtype fields; no runtime legacy-field absence test/credit |
| `structures-HousingCatalogCategoryInfo-643`, `structures-HousingCatalogSubcategoryInfo-659`, `structures-HousingCategorySearchInfo-661` | **Exact filter/ownership accounting IDs excluded**; renamed predicates/search input remain unresolved |
| `global api-C_HousingCatalog-GetCatalogEntryInfoByItem-284`, `global api-C_HousingCatalog-GetCatalogEntryInfoByRecordID-286` | Removed arguments outside scope |
| `global api-C_HousingBasicMode-StartPlacingNewDecor-278`, `global api-C_HousingBasicMode-StartPlacingNewDecor-279`, `global api-C_HousingCatalog-DestroyEntry-281`, `global api-C_HousingCatalog-DestroyEntry-282`, `global api-C_HousingCatalog-GetDestroyableInstanceCount-288`, `global api-C_HousingCatalog-GetDestroyableInstanceCount-289`, `events-HOUSING_STORAGE_ENTRY_UPDATED-555`, `events-HOUSING_STORAGE_ENTRY_UPDATED-556` | Entirely deferred; no policy inferred |
| `structures-HousingCatalogEntryInfo-650`, `structures-HousingCatalogEntryInfo-651`, `structures-HousingCatalogEntryInfo-653`, `structures-HousingCatalogEntryInfo-654`, `structures-HousingCatalogEntryInfo-655`, `structures-HousingCatalogEntryInfo-656`, `structures-HousingCatalogEntryInfo-657` | Aggregate/other removed fields untested; not completed by bounded base input |

### Cached declaration provenance (2026-10-01)

Read-only runtime cache `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`; provenance identifies retail `12.1.0.69933`, manifest `aa7dfec3fb3bc9440a8c737c2274363502cb3d22c7939b8a570a58e717202edd`. This cache corroborates field/function shape, not an exact historical 12.0.5 runtime or native behavior.

| File / relevant lines | SHA-256 |
|---|---|
| `HousingCatalogConstantsDocumentation.lua:77–99` — base versus compound IDs | `7a57d8fe99039b724aa8414e226084d022af11e22df68efab57b012bf2e1f1d5` |
| `HousingCatalogUIDocumentation.lua:54–64,125–198,537–580` — entry/list/variant signatures and DTOs | `8272fdceabfe5535fade6047fa7a6b9d0f11d0da9364a5b20e60fb7f6fc547c7` |
| `HousingCatalogSearcherAPIDocumentation.lua:10–34,255–267` — source/results and RunSearch | `0eeba7dfe9f68b547331538e8a54ccd6b4cb5da9be0a251c45e2c9e386e41ea3` |
| `HousingDecorSharedDocumentation.lua:23–36` — dye-slot fields | `13f88228e34e658d6a18a69bb846b1c527d35593cab48f955b3d22689dc32f7c` |

Cached wording: GetAllSearchItems is the "source collection of what's being searched"; GetCatalogSearchResults returns "the most recent search result entries". GetSearchCount counts owned instances, not result rows. Existing Lua aliases both getters to `state.searchResults` and returns its length for both count methods; neither aliasing nor count behavior is adopted as a requirement.
