# B96 independent verification

Target: a0e23199d. Read-only verification started; no cargo, repo edits, agents, or model CLIs.

## Observed command: raid_roster_unknown_name::
Revision a0e23199d; source/test diff empty, clean worktree. Exit 0.
```

running 4 tests
test raid_roster_unknown_name::cache_arrival_is_live_and_does_not_change_other_members ... ok
test raid_roster_unknown_name::cached_name_keeps_existing_roster_tuple ... ok
test raid_roster_unknown_name::missing_indices_and_inactive_group_still_return_twelve_nils ... ok
test raid_roster_unknown_name::uncached_name_is_public_unknown_with_unchanged_tuple ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10157 filtered out; finished in 0.37s


```

## Observed command: catalog_shop_product_structures::
Revision a0e23199d; source/test diff empty, clean worktree. Exit 0.
```

running 16 tests
test catalog_shop_product_structures::additional_product_pmt_urls_preserve_order_and_string_type ... ok
test catalog_shop_product_structures::all_nullable_fields_are_absent_and_required_arrays_exist ... ok
test catalog_shop_product_structures::consumable_quantity_is_absent_from_populated_product ... ok
test catalog_shop_product_structures::decor_quantity_round_trips_nested_numbers ... ok
test catalog_shop_product_structures::environments_are_isolated ... ok
test catalog_shop_product_structures::fully_populated_display_has_exact_declared_shape ... ok
test catalog_shop_product_structures::fully_populated_product_has_exact_declared_shape ... ok
test catalog_shop_product_structures::host_replacements_affect_new_snapshots_only ... ok
test catalog_shop_product_structures::house_texture_atlas_round_trips ... ok
test catalog_shop_product_structures::malformed_public_selectors_do_not_alias_stored_products ... ok
test catalog_shop_product_structures::maps_use_exact_keys_without_product_display_cross_lookup ... ok
test catalog_shop_product_structures::preview_bg_override_product_url_round_trips ... ok
test catalog_shop_product_structures::preview_small_bg_override_product_url_round_trips ... ok
test catalog_shop_product_structures::returned_tables_and_every_nested_array_are_independent ... ok
test catalog_shop_product_structures::unknown_ids_and_legacy_seed_ids_return_nil ... ok
test catalog_shop_product_structures::zero_decor_counts_are_present_not_nil ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 10145 filtered out; finished in 1.34s


```

## Interim artifact findings
Roster provider/state wired; five production construction sites and seven test construction sites explicitly initialize cache true. Prose lines 123/180 match transient Unknown behavior. Catalog typed getters pass all 16 focused tests. Cached storefront product insertion checks nil and skips missing seeded IDs; housing base display lacks nil guard, but its seeded 5001/91001/91003 IDs were absent from old getters too. Distinguish pre-existing housing gap from this commit. Feature-gated new getters plus unconditional deletion remove API presence on non-12.0.5 profiles.

# Final verification report — a0e23199d

## Proof ledger

HEAD: `a0e23199dc60d4056add5f5a15b68f23dcac38cc`. Worktree clean; `git diff a0e23199d HEAD -- src tests` empty before execution and at final inspection. Authorized prebuilt binary `target/debug/deps/integration-8ea324359263a4d2` run once per filter with `--test-threads=1` and a 90-second timeout:

| Filter | Observed result | Exit |
|---|---|---|
| `raid_roster_unknown_name::` | 4 passed, 0 failed, 0 ignored; 10157 filtered out | 0 |
| `catalog_shop_product_structures::` | 16 passed, 0 failed, 0 ignored; 10145 filtered out | 0 |

No cargo, repository edits, git mutation, panels, agents, or model CLIs. Binary build provenance not independently rebuilt. Supplied RED roster result (2 fail / 2 pass) and startup `lua-errors []` are caller evidence, not independently observed runs.

## 1. Raid roster — ACCEPT

`data/patch-api/sources/12.0.5-api-changes.txt:123,180` both specify that uncached names return “Unknown” instead of nil. `src/lua_api/globals/group_queries.rs:331–340` reads `name_cached` live, retains cached-name secret marking, and returns the existing `UNKNOWN` global for uncached members. Registered at `group_queries.rs:49`; the twelve-result producer remains intact.

All 12 tracked Rust construction sites set `name_cached = true`: five production sites (`game_data.rs:491`, `globals/admin_api/units.rs:389`, `globals/admin_party_target_helpers.rs:60`, `globals/group_queries.rs:305`, `globals/group_verbs.rs:50`) and seven named test fixtures. Admin defaults store the literal name “Unknown”; marking that explicitly stored placeholder cached preserves existing admin behavior, rather than claiming an automatic cache lookup. Tests explicitly toggle cache availability false/true.

**Provably earned spec checkboxes:** `docs/specs/raid-roster-unknown-name.md:7–11` — cached tuple; public uncached sentinel with unchanged eleven other returns/arity; live cache arrival and peer/player isolation; invalid/inactive twelve-nil results; explicit constructor initialization (static proof plus behavioral cached fixtures).

No in-scope implementation defect found. Localization-global identity and native invalid-index behavior remain unverified, correctly labeled inferred. Automatic cache population remains excluded. Documentation at spec lines 23,28 still describes integration as unapplied/no execution proof; stale relative to this commit and observed 4/4 result. Formatting/build claims not verified.

## 2. Catalog product DTOs — ACCEPT WITH QUALIFICATIONS

**Six audit rows earned:** 631,632,634,635,636,637 have stored-record behavioral fixtures, not merely publisher-presence tests. Registration `src/lua_api/globals/missing_surface.rs:249–250`, module `src/c_api/mod.rs:43–44`, state `src/lua_api/state/sim_state.rs:329–330`, initialization `src/lua_api/state.rs:297–298` are wired under `retail-12-0-5`.

DTO comparison against cached `Blizzard_APIDocumentationGenerated/CatalogShopDocumentation.lua:686–836`: exactly 45 product fields, 34 display fields, 5 subitem fields, 2 currency fields, 2 DecorQuantity fields; no missing/extra names. Types/nullability match the bounded spec. Tests exercise exact populated shapes, nullable omissions, required empty arrays, false/numeric values, zero quantities, recursive snapshot independence/GC, replacement/removal, separate maps, environment isolation and legacy misses.

**Provably earned spec checkboxes:** `docs/specs/catalog-shop-product-structures.md:9–16,20–21`; wiring/removal at line43. Line22 is earned for malformed public selectors, but not fully earned: no behavioral test supplies a secret selector. Native secret acceptance, native miss/aliasing/validation parity remain excluded. Formatting/build/startup/panel gate at line44 is not earned by this verification.

### Integration defects and risks

1. **Non-12.0.5 API-presence regression:** `missing_surface.rs:249–250` gates both replacement getters, while old getter deletion from `housing_catalog_state.lua` is unconditional (current catalog publisher starts at line558). That Lua file still executes on every profile via `workarounds/mod.rs:196` and `env_init/mod.rs:58`; its exterior installer condition does not gate catalog publication. Full source search found no other C_CatalogShop getter publisher. Therefore Wrath/Mists/Era/Anniversary/Forever and historical retail feature combinations without `retail-12-0-5` lose these callable APIs while retaining seeded lists. `Cargo.toml:149–155` confirms default Retail/PTR retain the feature, Forever/classic do not. Static regression proven; other-profile runtime/UI impact not executed. Do not present cross-profile replacement as verified.
2. **Seeded storefront now empty:** `housing_catalog_state.lua:558–574` still returns 2003/20031/20032; new maps default empty and getters return nil (`c_catalog_shop_products.rs:156–188`). Cached `CatalogShopUtil.GetProductInfo` at `Blizzard_CatalogShopSharedUtil/Blizzard_CatalogShop_SharedUtil.lua:1284–1293` nil-checks the product and returns nil. `Blizzard_CatalogShop/Blizzard_CatalogShop_Products.lua:466–470,505–510` skips each missing product; lines141–145 handle an empty provider. Thus this normal category-list path loses cards but does not establish a new nil-dereference on opening. This disappearance follows the explicit no-seeded-fallback spec, not an unfulfilled DTO requirement.
3. **Housing/card nil hazards exist; new crash not proven:** cached `Blizzard_HousingTemplates/Blizzard_HousingMarketProductDisplay.lua:46` passes the getter result unguarded into SetProductInfo; shared card `Blizzard_CatalogShop_SharedProductCards.lua:149–151,254–255` dereferences missing product/display data. Housing small cards guard nil at lines153–155; non-decor tooltip entries at245–246, cart templates469–471, and HouseEditorStorageFrame531–532 also guard products. Seeded housing bundle5001 and small products91001/91003 were absent from the old product table too, so their nil hazard is pre-existing, not demonstrated as introduced by this commit. Product-only host insertion likewise triggers the utility's explicit missing-display error; bundle-child display lookup at SharedUtil1327–1329 dereferences without a nil check. Panel-open behavior remains unexecuted; startup success cannot discharge it.
4. **Stale acceptance documentation:** catalog spec32,39 still says integration pending/no test execution; roster spec23,28 likewise stale. Keep inferred policies distinct from native compatibility proof.

## Artifact checks / merge risk

EXIST PASS: both implementations, specs and test files present. SUBSTANTIVE PASS: live roster state and typed product/display maps with nonconstant behavioral outputs. WIRED PASS on Retail12.0.5+; cross-profile getter-presence regression noted above. ANTI-PATTERN PASS: newly added reviewed Rust code/tests contain zero TODO/FIXME/HACK/XXX markers; no empty implementation/error swallowing found.

Bounded Retail audit credit is justified for both prose IDs and all six structure IDs. Whole integration readiness is qualified by lost non-12.0.5 API presence and absent storefront interaction proof. No new housing-market opening crash is claimed without reproduction.
