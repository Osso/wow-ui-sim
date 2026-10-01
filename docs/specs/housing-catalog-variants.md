# Housing catalog variants

Bounded 12.0.5 catalog identity contract from [retained changes](../../data/patch-api/sources/12.0.5-api-changes.txt) and cached HousingCatalog/Searcher declarations. Empty-backed producers now replace overlapping temporary catalog/search outputs after actual parent RED 0/11 at input `05ca7dff0`. Producer `76f2ac88a`; follow-up restricts registration visibility to its owning `c_housing` module so the re-export is legal. Parent compilation and repaired bounded GREEN passed at `157d15cef`; saved startup exits 0 with `[]`. Independent report 178 remains pending; accounting acceptance is not awarded. [Ownership findings](../wiki/investigations/patch-12-0-5-api-audit.md#Housing catalog variant producer — parent GREEN, independent pending) record integration boundaries.

## What it must do

### Explicit inputs and identity

- [ ] `HousingState.catalog` starts empty in each environment. Production catalog records must not be invented; only explicit supplied inputs populate it.
- [ ] Base entries use `(recordID, entryType)`; variant stacks use the full `(recordID, entryType, variantIdentifier)` key. A base entry is not variant zero.
- [ ] Filter-free `GetAllSearchItems` exposes all source variant IDs. After explicit `RunSearch`, `GetCatalogSearchResults` exposes matching variant IDs. Two variants of one record remain distinct in both getters; source and results remain distinct containers with independently published IDs even when their contents coincide. No order requirement.
- [ ] Fresh independently created searchers work without loaded Blizzard addons, callback registration, global fixture state or release helpers. Preserve the existing callback, parameter setter/getter and progress method surface. Callback compatibility controls are added; automatic update and native snapshot timing remain unverified.
- [ ] `GetCatalogEntryVariantInfo(entryVariantID)` reads explicitly different `numStored` and `dyeSlots` for each compound key; no invented product/name/legacy `variantID` fields. `numStored` is storage input, never a destroyability policy.
- [ ] `GetAllVariantInfosForEntry(entryID)` lists the fixture's distinct variant stacks; fixture replacement coverage, not an additional 12.0.5 signature-change claim.
- [ ] `GetCatalogEntryInfo(entryID)` remains base info: explicit record/type/item/name/trophy fields, no synthesized `entryID`, `entryVariantID`, `variantIdentifier` or variant `numStored`. This is a bounded field subset, not a complete entry DTO.
- [ ] **Inferred simulator lookup policy:** absent full variant keys return nil; absent base entries return nil and variant lists are empty. Wrong type must not resolve solely by record ID; no seed/numeric/variant-one fallback. Native invalid-enum, malformed-input, coercion and error behavior remain unknown.

### Security

- [ ] Ordinary selectors remain callable from tainted addon code without clearing caller taint, consistent with cached `SecretArguments = "AllowedWhenUntainted"` on the three info/list queries.
- [ ] Nested secret selector fields reject safely in both secure and tainted callers until access is modeled. Do not unwrap secrets, clear taint or bypass secured-table access. Parent host-secret and repaired host-installed guarded-table controls passed; independent report 178 remains pending. This conservative simulator limit is stricter than `AllowedWhenUntainted`, not native security parity. Selectors currently require ordinary tables with raw integer fields; native coercion, metatable lookup and numeric ranges remain unverified.

The two no-argument search getters have no `SecretArguments` annotation in the inspected cache. This is not proof that returned identities are always public in native WoW.

## How it works

- [Housing ownership and pending proof](../wiki/investigations/patch-12-0-5-api-audit.md#Housing catalog variant producer — parent GREEN, independent pending).
- [Independent existing free-place contract](housing-free-place-state.md).

## Implementation inventory

- `src/c_api/c_housing/catalog.rs` and `catalog/queries.rs`: empty typed inputs and exact compound-key queries; registration remains unconditional like the replaced Lua surface, with no new profile gate.
- `catalog/snapshot.rs` and `catalog/searcher.lua`: stack-rooted native serializers and retained Lua searcher lifecycle, owned by `c_api`. Each refresh publishes separate source/results snapshots from explicit variant keys. Filter setters store parameters only; the old seeded filter matcher is removed. Sorting, async and automatic updates are not implemented. Existing row-length count methods remain compatibility placeholders, not accepted owned-instance semantics.
- `src/lua_api/state/support_types.rs`: `HousingState.catalog` references the C API-owned input model. Existing house reset replaces `HousingState` with its derived default; catalog reset behavior is untested.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: removes exactly the replaced three info/list keys, CreateCatalogSearcher, seeded search publisher and variant-copy/list-match helpers. Shared seeds still serve excluded legacy ByItem/ByRecordID and destroyable-count providers; storefront/cart/exterior/customize owners remain untouched. These seeds are never read by the new catalog queries or search publisher.
- `tests/housing_catalog_variants.rs`: fourteen grouped fixtures; `tests/housing_catalog.rs` retains storefront/cart coverage and adds an unconditional empty-surface/lifecycle control. No new Cargo target.

## Tests asserting this spec

Parent compiled input `05ca7dff0` and ran all original eleven `housing_variant_*` fixtures against the unchanged Lua provider: **0 passed, 11 failed**. Failures include seeded default records, wrong variant counts and incompatible output fields. Parent compiled producer `76f2ac88a` plus visibility fix `45f0d8b21` successfully, initially observing **13 passed, 1 failed** across fourteen variant fixtures. The failure was the guarded-selector fixture described below, corrected without production changes at `157d15cef`. Parent repaired run passes **14/14 variants**, with **4 cart + 4 free-place + 1 customize + 2 decor controls PASS**; saved startup exits **0**, output **[]**. Earlier zero-match filters provide no evidence; the later actual controls do. Independent report **178 pending**; no accounting acceptance awarded. No build/check/delegation/push authorized in this docs audit. Contract checkboxes remain acceptance-pending, not implementation absence.

| Contract | Cases |
|---|---|
| Default empty source/results | `default_search_source_is_empty`, `default_search_results_are_empty` |
| Full IDs with independent searchers | `search_source_preserves_compound_ids`, `search_results_preserve_compound_ids` |
| Explicit storage/dye data and base separation | `query_preserves_distinct_stack_fields`, `list_uses_explicit_fixture_records`, `entry_info_remains_base_info` |
| Missing selectors and environment isolation | `default_queries_have_no_seed_records`, `missing_selectors_do_not_use_legacy_seeds`, `inputs_do_not_seed_another_environment` |
| Ordinary addon selector security | `public_selectors_allow_tainted_callers` (now all three queries) |
| Independent nested snapshots/source and result containers | `snapshots_do_not_alias_model_or_search_containers` |
| Conservative secret/access rejection | `nested_secrets_reject_without_unwrapping`, `secured_selector_preserves_access_guard` |
| Unconditional empty surface and retained callback/parameter controls | `housing_catalog_empty_surface_remains_registered` in `housing_catalog.rs` |

Replacement mapping: old `housing_catalog_market_and_variant_methods_use_seeded_state` checked variant 2 and a two-element list using temporary `variantID`/`productID`/`name` extensions. The new query/list fixtures assert the declared `entryVariantID`, stored and dye fields instead. Old itemID=1001 and isUniqueTrophy=false assertions are preserved under explicit base-info inputs. Featured products, bundle preview/viewed state, all market/cart operations and the two standalone cart tests remain unchanged in `housing_catalog.rs`. No source-only substring/shape tests substitute for behavioral fixtures.

### Proof ledger and producer gate

| Command / scope | Result | Invalidation |
|---|---|---|
| Baseline `git status --short`, `git rev-parse HEAD` | Clean `5b0644b3373244574ee4a7b1a1958fc82f5529b5` | Later edits not covered |
| `rustfmt --edition 2024 --config skip_children=true src/c_api/c_housing/catalog.rs src/c_api/c_housing.rs src/lua_api/state/support_types.rs tests/housing_catalog.rs tests/housing_catalog_variants.rs` | Exit 0 on input/tests-only working scope; formatting only | Later Rust changes invalidate formatting scope |
| Parent `cargo test --test integration --no-run --message-format=json` at `05ca7dff0866fb2572b92717c5d3203a26615e68` | Exit 0, 252.50s; `/tmp/patch-12.0.5-batch18-red-build*` | Input fixtures only, not current producers |
| Parent `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c housing_catalog_variants:: --nocapture --test-threads=1` | Exit 101, **0/11**; `/tmp/patch-12.0.5-batch18-red-run.log` and `.json`; binary SHA-256 `4d6e4297ca7a680f192c5c3c133126a184bb87ead83dd8482f76a6f586725957` | Same input revision; not current producers/new controls |
| Producer `76f2ac88a`: `rustfmt --edition 2024 --config skip_children=true` on `c_housing.rs`, `catalog.rs`, `catalog/{queries,snapshot}.rs`, `tests/housing_catalog{,_variants}.rs` | Exit 0; formatting only, no compilation or behavioral proof | Registration visibility follow-up invalidates only `queries.rs` scope |
| Follow-up `rustfmt --edition 2024 --config skip_children=true src/c_api/c_housing/catalog/queries.rs` | Exit 0 on registration visibility fix; formatting only | Later edits to that file invalidate scope |

Actual compiled behavioral RED precedes this producer replacement. Parent `/tmp/patch-12.0.5-batch18-green-run-0.log` records 13/14 variant PASS at `45f0d8b21`; this historical partial proof is superseded by repaired parent 14/14 below, not independent or native acceptance.

Parent repaired build at `157d15cef796ccd11b3e6c04f80b57d0700728cb`: `/tmp/patch-12.0.5-batch18-guard-fixture-build-result.json`, exit 0, 93.36s. Variant artifacts `run.json` / `run.log` records exit 0, 14 PASS, 9.48s; integration SHA-256 `713a8f9abb3c123184aae8c9d23787d043ad82f700e235fe176ac99ae99a37eb`. Artifact prefix throughout is `/tmp/patch-12.0.5-batch18-guard-fixture-` (variant metadata is `run.json`). `controls.json` and `control-{0,1,2}.log` record 4 free-place, 1 customize, 2 decor PASS on that binary. Earlier `/tmp/patch-12.0.5-batch18-green-run-1.log` supplies 4 cart PASS, not a repaired-revision cart rerun. `startup-run.json` records exit 0, 4.86s, normal binary SHA-256 `7e673f2a0365ed5cc2e3127d26cd38f033844005d61e3cca674c185d79843578`; `startup.json` is `[]`. Saved parent evidence only; independent report 178 and acceptance pending.

### Guarded-selector fixture root cause

Retail `src/ptr/compat_bootstrap.lua` defines `settablesecurity` as a no-op. `src/lua_api/env_init/mod.rs` installs real rilua table-security globals only under `client-wowforever`. The failing fixture created its selector securely and stamped the reader closure `HousingCatalogFixture`, but calling the retail placeholder never set the table's restriction. Its expectation that the source was guarded was false; catalog access rejection was not disproven.

Pinned rilua `6044544b960cd68b4b0c58bb3373412757c2caee` sets option 0's table flag only through real `set_table_security`; `check_table_access` rejects flagged tables whenever any live call frame is tainted, without a table-owner exception. Catalog `read_selector` already invokes that check before its raw integer-field reads. Neither production code nor the guard is changed.

The repaired test explicitly registers pinned VM table security in its own environment. Named assertions require secure source indexing/rawget and all three catalog queries to succeed, tainted source indexing/rawget and each query to reject with the table-access error, caller taint to persist after rejection, and secure access/source data to remain intact after the addon returns. Lua `rawget` itself invokes the VM guard; no bypass reads are added. This proves only host-installed guarded selectors, not retail registration, native policy or Forever parity. Revised fixture compilation/execution passed in the saved parent run at `157d15cef`; independent report 178 remains pending. Repair formatting: `rustfmt --edition 2024 --config skip_children=true tests/housing_catalog_variants.rs` exited 0; only this changed Rust file is covered. No builds, checks or tests ran during the fixture edit itself; later parent build/run evidence is recorded above.

## Known gaps (current cycle)

- [ ] Parent producer compilation, repaired 14/14 grouped GREEN and saved zero-error startup passed; independent report 178 and accounting acceptance remain pending. Actual prerequisite RED and historical 13/14 fixture failure are retained above.
- [ ] Distinct source/results are implemented; real filtering, sorting, async updates and owned-instance count semantics remain unmodeled. Existing filter method names do not establish matching behavior.
- [ ] Complete base metadata/aggregates and secure access to secret selectors are missing. Strict public raw integer selectors and conservative secret rejection are simulator limits, not native range/coercion/access claims.
- [ ] Overlapping variant/base seed assertions were replaced by explicit fixtures at `05ca7dff0`; coverage is retained, not deleted. Historical deprecated tests target the excluded ByRecordID wrapper; current retail cache no longer ships that addon. No other test references the replaced seeded query outputs. Customize selection remains a separate temporary fixture.
- [ ] ByItem/ByRecordID and category-name providers remain outside this slice. Retained seed providers do not supply fallback data to new queries/searchers.

## Out of scope

- Destroy mutation, destroyable counts, aggregate/search count policies, storage events and placement selection: source deltas establish types, not mutation/emission/count/placement policy.
- All search filters, category/subcategory ownership predicates and sorting: intentionally filter-free fixtures; no invented matching semantics.
- Storefront/cart/exterior/customize-selection behavior and legacy wrapper migration: unrelated owners retained, not declared authoritative catalog data.
- Native parity, loaded UI acceptance, broad checks/builds, delegation, push and source-row completion: not authorized or established in this cycle.

## Source and accounting boundary

Retained source SHA-256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`; [362-row register](../../data/patch-api/sources/12.0.5-register.json) and [coverage register](../../data/patch-api/sources/12.0.5-page-coverage.json) are unchanged. This slice changes no register statuses/IDs and awards no row credit; concurrent unrelated audit commits are outside its proof scope.

| Exact accounting IDs | This slice |
|---|---|
| `scriptobjects-HousingCatalogSearcher-GetAllSearchItems-524`, `scriptobjects-HousingCatalogSearcher-GetAllSearchItems-525`, `scriptobjects-HousingCatalogSearcher-GetCatalogSearchResults-527`, `scriptobjects-HousingCatalogSearcher-GetCatalogSearchResults-528` | Producer implemented after RED; Parent full-ID GREEN; independent acceptance pending, not filter proof |
| `structures-HousingCatalogEntryInfo-648`, `structures-HousingCatalogEntryInfo-649`, `structures-HousingCatalogEntryInfo-652` | Bounded producer implemented after RED; Parent GREEN; independent acceptance pending, no full entry-info claim |
| `structures-HousingDecorDyeSlot-663` | Explicit `dyeColorName` publication parent GREEN; independent acceptance pending |
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

Cached wording: GetAllSearchItems is the "source collection of what's being searched"; GetCatalogSearchResults returns "the most recent search result entries". GetSearchCount counts owned instances, not result rows. The replaced Lua provider aliased both getters to `state.searchResults`. Current source/results use distinct containers. Row-length count methods are retained unchanged as compatibility placeholders; owned-instance semantics are not implemented or adopted as a requirement.
