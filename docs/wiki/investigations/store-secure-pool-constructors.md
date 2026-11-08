# Store Secure Pool Constructors

Retail Store cards rendered as blank/red placeholders and left the Store stuck behind the "Connecting to Shop" dialog because secure Store code used stale simulator pool constructors.

## Content

The Store addon runs with `UseSecureEnvironment: 1`, so `StoreFrame_OnLoad` resolves constructors through `__secureenv`. The simulator installed fallback pool constructors during bootstrap before `Blizzard_SharedXMLBase` loaded the real Blizzard pool surface. `_G.CreateFramePoolCollection` was later replaced by Blizzard's proxy-backed constructor, but `__secureenv.CreateFramePoolCollection` still pointed at the fallback implementation.

That stale secure constructor produced a private collection shape with an exposed `pools` table and no `GetPool` method. Store product cards then came from the fallback collection instead of Blizzard's pool proxy path, which broke normal card setup and made the visible Store card area look corrupt even though the Blizzard source files were valid.

The original simulator fix synced the real pool/factory constructors from `_G` into `__secureenv` immediately after `Blizzard_SharedXMLBase` loaded. Retail probing later showed `secureenv` has no `_G` fallback, so the replacement model is to load Blizzard Lua library files in the secure environment pass as well. That makes secureenv receive the Blizzard constructors directly instead of copying them from `_G` after the fact. The runtime Store data fallback also avoids relying on Store enum globals that `Blizzard_EnvironmentCleanup` removes, and keeps Store secure state in a local table so namespace fallbacks cannot turn `_state` into a function.

Regression coverage:

- `test_frame_pool_collection_returns_proxy_surface` asserts both global and secure `CreateFramePoolCollection()` return Blizzard-style proxy collections.
- `store_tree` exercises Store loading with the discovered Blizzard addon dependency closure instead of a hand-picked partial addon list.
- `text_manifest_entries_reject_binary_cache_data` guards against binary data entering text cache paths.
- `wow-sim --no-addons --no-saved-vars lua-errors` and a Store-open `lua-errors` probe both return `[]`.

## Sources

- [runtime_surface_bootstrap.lua](../../src/lua_api/env_init/runtime_surface_bootstrap.lua) — Store fallback state, enum restoration, and secure Store defaults
- [pool_constructor_defaults.rs](../../src/lua_api/workarounds/temporary/pool_constructor_defaults.rs) — fallback pool constructors for isolated partial loads
- [store_tree.rs](../../tests/store_tree.rs) — Store regression coverage
- [pool_api.rs](../../tests/pool_api.rs) — pool collection surface regression coverage

## See Also

- [[addon-load-order]] — addon load timing can require post-load repairs
- [[lib-test-failure-sweep-2026-06]] — records why post-load repairs must not rely on `hooksecurefunc(C_AddOns, "LoadAddOn")`

## Current bounded profile evidence — 2026-10-14

[Owning revision/gate matrix](../../specs/store-product-group-shape.md#seven-profile-saved-runtime-matrix--2026-10-14) separates Retail/PTR/Mists Store runtime PASS from Mists overall authentication PARTIAL, retained cleanup failures and native/render/full-parent gaps. It also scopes actual WoWForever TBC record-consumer evidence without expanding public ID projection or claiming older Store runtime acceptance. Historical checkpoints below remain revision-bound.

## Store group shape — source checkpoint 2026-10-14

`85d8d7bd4` registers the modeled `C_StoreSecure.GetProductGroups` query under `src/c_api/`. Registration precedes runtime bootstrap; the getter reads the already-published `__wow_store_state.productGroups` at call time. Mists receives numeric IDs; other profiles retain the existing record table. Both bootstrap Lua definitions of this getter are removed. Demo backing, other queries and transactions remain legacy bootstrap Lua; no additional state bridge.

[Owning contract/status](../../specs/store-product-group-shape.md) separates cached-consumer inference and simulator demo policy from native evidence. At this source checkpoint new Rust execution, postproducer runtime preservation and Store UI population were unproved. Historical pool-constructor proof above does not cover this producer.

## Temporary Store selection trace — source checkpoint 2026-10-14

`ffb6d7d7d` observes before/after binding counts around existing trust filtering. [Diagnostic limits](../../debug-tools.md#temporary-store-script-selection-trace) define default-off activation and retirement; counts do not establish callback success, filter-specific cause or populated cards. Mists root/runtime proof remains pending.

## Featured display metadata — source checkpoint 2026-10-14

[Owning source checkpoint](../../specs/store-product-group-shape.md#featured-display-metadata--source-checkpoint-2026-10-14) records the single legacy Lua initializer field at `bf1b61f10`, shared `e472` assertion-first normal Mists447 numeric-display RED (CLI1), and observed original TBC nil-equality. Numeric zero is regular-group simulator policy, not native enum evidence; no aliases retained. Independent `bf1b61f10` full-stream gate passes Store2 unique/item4/corrected bag1 and original public lifecycle with one shown entry1003 card. [Diagnostic retirement](../../specs/store-product-group-shape.md#diagnostic-retirement--source-checkpoint-2026-10-14) at `bc5cecffb` requires separate current-profile preservation; earlier query PASS/UI FAIL and historical Retail Show PASS keep their old-source scopes.

## Current bounded Retail receipt — 2026-10-14

[Public historical packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/store-macro-20261014/README.md) retains Store receipts and failures; [owning status](../../specs/store-product-group-shape.md#current-bounded-retail-receipt--2026-10-14) owns their scope and separate accepted copy/privacy limits. No later item/display, PTR, current UI, native/render or full-parent acceptance follows.

[Owning current status](../../specs/store-product-group-shape.md#current-bounded-retail-receipt--2026-10-14) records independent existing-artifact PASS at `a6b572732`: normal public Show, record-contract/completion markers, one shown/visible Apprentice Rider Bundle card with positive reported bounds inside the shown/visible Store, CLI0/CLEAN/zero errors. Startup workarounds remain present; this is not vendor-free, native, rendering or parent proof. Mists query PASS remains separate from public Store UI FAIL; PTR preservation pending.
