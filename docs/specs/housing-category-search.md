# Housing category search

`C_HousingCatalog.SearchCatalogCategories` and `SearchCatalogSubcategories` must query explicit category/subcategory inputs in `src/c_api/c_housing/catalog.rs`. This bounded batch26 contract targets exact 12.0.5 audit row `structures-HousingCategorySearchInfo-661`: `withOwnedEntriesOnly` → `withStoredEntriesOnly`. Category snapshot getters from producer `bbbacf8f0` are unchanged. See [API architecture](../wiki/systems/lua-api.md) for subsystem context.

## What it must do

### Cached declaration contract

- [ ] Require a non-nil `HousingCategorySearchInfo` table; return a table of numeric category/subcategory IDs. Cached `HousingCatalogUIDocumentation.lua:385–412` declares both functions and `SecretArguments = "AllowedWhenUntainted"`.
- [ ] Default missing/nil `withStoredEntriesOnly` and `includeFeaturedCategory` to false. `HousingCatalogUIDocumentation.lua:594–601` declares `Default = false` for both booleans and optional `editorModeContext`.
- [ ] With `withStoredEntriesOnly = true`, restrict results to records with stored entries; an editor mode restricts results to associated records. Cached text describes stored entries as "the player has something stored under" and mode filtering as "categories associated with/used by this Editor Mode". Native associations are not supplied by this declaration.

### Explicit simulator policies, not native guarantees

- [ ] Query only the appropriate explicit map. Empty maps return fresh empty arrays, never former seed IDs or synthesized featured records.
- [ ] Use each record's explicit `any_stored_entries` boolean; never derive it from partial variants, names, subcategory lists, or parent records. False/default includes both stored and unstored records.
- [ ] Both category and subcategory records accept host-owned `editor_mode_contexts: Vec<i32>`. Existing literals receive empty vectors without changing other data. Empty means no known association, not association with every mode. No context (`None`/nil) applies no mode filter; a specified public integer matches exact vector membership. Explicit numbers are host inputs, not fabricated native enum associations.
- [ ] Exclude the existing featured category by default/false, include it when requested and all other filters match. Read the current runtime `Constants.HousingCatalogConsts.HOUSING_CATALOG_FEATURED_CATEGORY_ID`; do not hardcode its value. The requested `HousingConsts` namespace is not the observed namespace: cached `HousingCatalogConstantsDocumentation.lua:67–73` declares `HousingCatalogConsts`, matching runtime `src/lua_api/globals/enum_data/constants_values.lua`.
- [ ] **Inferred featured-subcategory policy:** treat a subcategory as featured solely when its explicit `parent_category_id` equals that constant. Do not require the parent category record to exist. General subcategory queries also impose no inferred graph/list-membership constraint.
- [ ] Sort results by `order_index`, then numeric map ID, independent of insertion order. This deterministic simulator policy is not a native ordering guarantee.
- [ ] Return fresh snapshots; caller result mutations do not affect inputs or future queries. Subsequent host input insertion, removal, predicate, association, or order changes affect new results, not previous arrays. Searches do not mutate catalog records, entries, or variants.

### Bounded parser/security and deprecated consumer

- [ ] Require a public table and preserve the VM table-access guard before reading fields. Supplied booleans must be public booleans; optional mode must be a public integral `i32`, not a coerced string, fractional number, boolean, secret, or out-of-range number. This strict simulator boundary is not complete native validation evidence.
- [ ] Allow ordinary public addon calls without clearing caller taint. Reject secret tables and each secret field in secure and tainted callers without unwrapping values or changing caller taint. Preserve host-installed VM secured-table rejection; the fixture installs real rilua policy because retail `settablesecurity` is a compatibility no-op.
- [ ] Raw new APIs ignore removed `withOwnedEntriesOnly`. The actual cached `Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua:113–121,196–208` wrapper maps the old field only when new is missing: explicit new false wins over old true. Consumer table mutation by that wrapper is intentional and distinct from raw query/input mutation.

## How it works

- [API architecture](../wiki/systems/lua-api.md)
- [Category DTO contract](housing-catalog-categories.md) — unchanged getter scope.
- [Catalog variant contract](housing-catalog-variants.md) — separate filter-free ordinary searcher scope.

## Implementation inventory

- `src/c_api/c_housing/catalog.rs` — explicit category/subcategory maps and new editor-mode association vectors only.
- `tests/housing_catalog_categories.rs` — existing four literals gain empty association vectors; getter assertions remain unchanged.
- `src/c_api/c_housing/catalog/category_search.rs` — public parser/access guard, runtime featured constant lookup, explicit-map predicates, ordered fresh ID arrays.
- `src/c_api/c_housing/catalog/queries.rs` — unconditional registration of both category searches; getters and ordinary searcher unchanged.
- `tests/housing_category_search.rs` — twelve grouped integration expectations; saved GREEN independently inspected below.
- `build.rs` / `tests/integration.rs` — existing discovery includes the new file in the single integration target; no new Cargo target.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua` — only the two seeded category search publishers removed; remaining publishers unchanged.
- Cached `Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua` — actual deprecated wrapper loaded in one fixture; not copied, rewritten, or monkey-patched.

## Tests asserting this spec

Parent's saved compiled RED at inputs `94a8f966b799b27bb13b4a3cf66fc92b0864b5f4`: **0 PASS / 12 FAIL**, build exit **0** in **129.005s**, test exit **101** in **1.524s**. Artifacts: `/tmp/patch-12.0.5-batch26-red-build-result.json`, `-red-build.json`, `-red-revision.txt`, `-red-run.json`, `-red-run.log`. Producer read these artifacts; did not rerun commands. Saved GREEN and independent bounded acceptance are reconciled below.

| Fixture in `tests/housing_category_search.rs` | Contract |
|---|---|
| `empty_maps_return_fresh_empty_arrays_without_seed_ids` | Empty/fresh/no synthesis |
| `stored_filter_uses_explicit_boolean_and_raw_api_ignores_removed_key` | Defaults, stored restriction, raw rename |
| `featured_filter_only_selects_existing_records_and_explicit_parent_ids` | Runtime constant, existing records, inferred featured-parent policy, combined filters |
| `context_filter_matches_exact_host_membership_without_parent_or_label_inference` | Exact/multiple associations, absent association excluded, unknown mode empty, None unfiltered |
| `ordering_is_order_index_then_numeric_id_after_state_changes` | Order index plus genuinely numeric tie-break |
| `fresh_snapshots_follow_current_inputs_without_aliasing_prior_results` | Freshness, host changes, prior arrays unchanged |
| `queries_do_not_mutate_records_or_derive_stored_predicates_from_variants` | Whole catalog state unchanged, independent explicit predicates |
| `required_public_table_and_strict_supplied_field_types_reject_invalid_inputs` | Required table and strict types/range |
| `public_addon_queries_preserve_caller_taint` | Public addon allowed |
| `secret_table_and_fields_reject_without_unwrapping_or_clearing_taint` | Real VM secrets, GC rooting, secure/addon rejection |
| `guarded_search_table_preserves_real_vm_access_policy` | Real host-installed secured-table guard |
| `cached_deprecated_wrapper_maps_old_only_when_new_field_is_missing` | Actual cached bridge, new false wins |

Parent's exact RED filter before fresh producer work:

```text
cargo test --test integration housing_category_search:: -- --nocapture
```

Default features include `retail-12-0-5`; the file uses that feature gate. Saved parent execution used the compiled integration binary with `housing_category_search:: --nocapture --test-threads=1` under `timeout 90`. Producer checkpoint performed formatting only; subsequent saved parent executions and independent gates are reconciled below. No native or whole-row acceptance.

## Producer checkpoint — 2026-10-01

Both APIs now parse a required public table through the existing VM access guard, strictly parse public boolean/integer fields, read the modeled runtime featured constant, filter explicit stored/mode inputs, sort `(order_index, numeric ID)`, and publish fresh ID arrays. No secret unwrap, taint clear, removed-key fallback, parent join, guessed associations, or synthesized IDs. Only the two old search publishers were removed; registration remains unconditional. Source implementation is committed before parent GREEN/verifier gates. Bounded simulator requirements are accepted below; cached/native guarantees remain open.

## Reconciled batch26 bounded proof — 2026-10-01

[Independent proof](/tmp/patch-12.0.5-housing-category-search-independent-proof.md) accepts producer `a608db343`: saved **12 search + 12 category + 12 aggregate + 14 base + 24 variants/count + 4 cart = 78 PASS**. Parent normal startup exits **0**, JSON `[]`; saved executions independently inspected, not rerun. Fresh independent default `cargo fmt --check` and `cargo check` exit **0** at producer scope, without warnings. Current producer source equality and saved compiler/runtime hashes agree; historical compilation metadata is not a cryptographic source attestation.

Exact row **661** gains bounded new-filter-name coverage: raw APIs ignore old `withOwnedEntriesOnly`, use explicit `withStoredEntriesOnly`, and actual cached deprecated consumer maps old-only input while explicit new false wins. Empty/explicit maps, stored predicates, fresh arrays, public addon taint preservation, secret rejection and real VM parameter-table guard have saved behavioral proof. Featured-parent classification, `(order_index, numeric ID)` ordering, strict parsing and explicit mode association policy remain simulator inferences, not native semantics. Exhaustive numeric edges, constant tampering/global-table security and GC at every population remain source-only or untested.

Native `AllowedWhenUntainted`, full DTO, ordinary-searcher filtering, housing panel safety and all-profile execution remain unclaimed. Batch27 appendix separately accepts tests-only aggregate coverage at `9cdb13a59`; it is not part of this 78-test execution. Audit **IN PROGRESS**.

## Known gaps (current cycle)

- [x] Saved targeted GREEN and independent bounded acceptance at `a608db343`; saved input RED is recorded above.
- [ ] Native ordering, featured-subcategory behavior, mode associations, and exact parser behavior remain unknown; policies above are explicit simulator inferences.
- [ ] Native `AllowedWhenUntainted` secret acceptance remains incomplete. Secure secret rejection here intentionally does not claim parity with that declaration.

## Out of scope

- Getter changes, other serializers, other seed publishers, or registration feature-gating.
- Ordinary `HousingCatalogSearcher` filtering: these two category queries establish no ordinary-searcher filtering claim.
- Shared wiki, coverage manifests, PLAN, existing category spec, native probes, builds/checks/tests, delegation, push, deployment: excluded by batch26 authorization.
