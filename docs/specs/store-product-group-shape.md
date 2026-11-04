# Store product group query shape

`C_StoreSecure.GetProductGroups` exposes existing simulator demo groups in the shape consumed by each audited profile. Demo backing and other queries remain in `src/lua_api/env_init/runtime_surface_bootstrap.lua`; the modeled group getter lives in `src/c_api/c_store_secure.rs`. See [Store source checkpoint](../wiki/investigations/store-secure-pool-constructors.md#store-group-shape--source-checkpoint-2026-10-14).

Evidence is inferred from original cached consumers plus explicit simulator policy, **not native proof**. Cached official StoreSecure declarations are absent. Retail/PTR cached shared Store consumers read `groupID`/`parentGroupID`; the Mists TBC consumer treats each entry as an ID.

## What it must do

- [ ] Retail/PTR: return one demo record in a fresh simulator with `groupID=501`, `parentGroupID=0`.
- [x] Mists: return one demo numeric ID in a fresh simulator, `501`; no record adapter in the consumer. Two Rust cases and full normal-session contract passed at compiled3e0cba6d4/restore04d83; this is not Store UI acceptance.
- [ ] Resolve the returned actual ID through `GetProductGroupInfo` to `groupName="Featured"` and through `GetProducts` to the single entry `1003`.
- [ ] Fresh featured group publishes numeric `displayType=0`, an explicit regular-group simulator policy, not a claim about native legacy enum numbers. Absent metadata must not match absent legacy layout enum fields.
- [ ] Resolve returned entry `1003` through `GetEntryInfo` to `entryID=1003`, `productID=2003`, and `sharedData.name="Apprentice Rider Bundle"`.

Concrete fresh demo values above are explicit simulator policy, not claims about a live/native catalog.

## How it works

- [Lua API architecture](../lua-api.md)
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/env_init/runtime_surface_bootstrap.lua`: existing demo Lua table remains published as `__wow_store_state`; replaced Lua GetProductGroups definitions are removed.
- `src/c_api/c_store_secure.rs`: modeled group query reads the existing table and projects IDs only on Mists; other profiles retain their record path. No duplicated catalog or initialization bridge.

## Tests asserting this spec

- `tests/store_group_contract.rs`: active-profile API contract in a fresh `WowLuaEnv` and after original cached EnvironmentCleanup loading, then explicit post-load workarounds; a separate case checks that restoration preserves an existing getter. Auto-discovered by the integration harness. The second post-load check was unreached in retained cleanup RED.
- `tests/fixtures/store_group_contract.lua`: shared behavioral assertions; directly executable via normal `--exec-lua` with explicit mode prefix. New numeric/zero display-type assertions are authored before changing backing; earlier group-shape passes do not cover them.

## Known gaps (current cycle)

- [x] Shared Mists preproducer normal-Lua fixture fails at `Mists GetProductGroups[1] must be numeric` (CLI1), before group/product lookups. Authored Rust case remains unexecuted. Independent existing-artifact audit read all 18 selected artifacts: Mists first-shape RED and Retail/PTR complete baseline lookups accepted. Source-audit ENOENT was a separate agent visibility limitation; main read the delivered report.
- [x] Retail post-restoration normal-session record contract and public Show witness accepted at `a6b572732`, scoped below. Earlier Retail compile failed before runtime at a private-module import; that historical failure is not the current Retail result.
- [ ] PTR post-restoration record/lookup preservation remains pending. Preproducer modern preservation is not fresh producer proof.
- [x] Historical Mists API GREEN still left eight cards hidden with ID0 (146×209) and failed original public Show/GetProductList non-vacuity. Conditional positive-ID/size/rect assertions were unreached; CLI1/two records represent one assertion failure. At compiled `bf1b61f10`, numeric display metadata closes this bounded population failure: one shown card1003 and completed positive-ID/size/rect assertions. Native/render and current post-retirement preservation remain pending.

## Source checkpoint — 2026-10-14

Producer `85d8d7bd4`, tests `f199e9424`: modeled getter ownership and initialization order are source-inspected, not runtime acceptance. Mists no-run compilation finished successfully with 741 JSON records and no compiler-message records; stderr retains six manifest deprecations and one `UnitRole` dead-code warning. Both new copied executables passed independent once-per-copy authentication. Fresh Mists Rust API case passed (1 case); the full normal-session fixture failed at its first data assertion, `GetProductGroups must return a list` (CLI1, two error records for the same assertion). Later length/shape/group/product/entry assertions were unreached; Store UI was not exercised. Preproducer modern preservation does not cover this producer.

Source inspection identifies an original lifecycle gap: cached Mists `Blizzard_EnvironmentCleanup` clears `C_StoreSecure`; simulator restoration reruns the Lua bootstrap without re-registering the modeled getter. That is source-grounded explanation, not proof of the exact dynamically returned value/dispatch. Commit `d9c7c6aa1` adds a real cached-cleanup regression, checking the shared contract after original addon loading and post-load workarounds; Mists execution reached the first post-cleanup contract and failed at `GetProductGroups must return a list` (one unique Rust case, exit101); the second post-load assertion was unreached. Initial admission denial was zero-launch, not RED. No getter override, artificial namespace clearing or forced vendor handler.

`04d83e2a5` re-registers the same Rust `GetProductGroups` implementation immediately after runtime-bootstrap restoration, through the existing namespace registrar. Both immediate cleanup and later post-load restoration call this path. This is a source repair intended to close lost registration, not proof either runtime path passes. Recreating the public namespace does not itself republish it into the secure environment; post-cleanup secure/public identity and vendor dispatch remain unproved. Independent original-cleanup RED gate confirms one unique failure and separate zero-launch denial. Three initial Mists compile admissions were denied without launching; later changed headroom admitted the compiled3e0cba6d4 build (741 JSON records, six manifest deprecations plus UnitRole warning). Both new copied executables passed independent once-per-copy authentication. The initial runtime setup failed before launch because verifier receipts used `authentication-*` names; whole-text `receipt-*` aliases corrected the helper basename without rehashing.

Independent restored-Mists runtime gate: **two unique Rust cases**, fresh query and real cached cleanup with the second post-load assertion reached; one timeout child is not a third case. Full normal session completes all numeric-ID/group/product/entry/name assertions, marker `ids`, `[]` and CLEAN/zero observed errors. Source label3e includes restore04d83 and passive target-input0b; not later macro source acceptance. At that checkpoint Retail/PTR fresh preservation remained pending; current bounded Retail receipt status follows below, while PTR remains pending. Original public Mists Store population separately fails as recorded above; no native/render/whole-parent acceptance.

## Featured display metadata — source checkpoint 2026-10-14

`bf1b61f10` adds only `displayType = 0` to the existing featured `productGroupInfo` record initializer in legacy `runtime_surface_bootstrap.lua`. This is explicit regular-group simulator policy, not a new typed model, catalog or native enum-value claim. No legacy enum aliases are retained; Blizzard/vendor Lua and renderer behavior are unchanged.

Shared fixture assertions at `e472` preceded this source change. The actual preproducer normal Mists447 run reached `featured group displayType must be numeric` as its first failure (CLI1); the zero-value and later product/entry assertions were unreached. Original TBC `SetCategory` at line720 compares missing group metadata with missing legacy `Splash`/`DoubleWide` fields: observed post-Show metadata made both nil comparisons true. Explicit numeric zero avoids that nil-equality; it does not establish native layout semantics.

Mists integration compiled at `bf1b61f10` (741 JSON records, 1m41; six manifest deprecations and one UnitRole warning). Recorded execution at `bf1b61f10` reaches the shared contract and original public lifecycle completion: one shown entry1003 card, rectangle `(615,633.5,146,209)`, seven hidden ID0 cards; CLI0, CLEAN/zero errors. Independent full-stream gate passed Store **2 unique**, item **4**, corrected bag **1**, and the original public lifecycle; all four launcher exits0. Startup loaded142 addons with1094/1094 cache hits; simulator workarounds and read-only EditMode cache remain in use. This proves bounded numerical population/geometry, not containment, native/render/full-parent acceptance. `bc5cecffb` retires temporary Store selection/query logging and its private flag; generic event tracing, group projection, restored registration and `displayType=0` remain. Current post-retirement Retail/PTR/Mists preservation is separate and pending. Earlier Mists query PASS/UI FAIL and historical Retail public Show PASS at `a6b572732` retain their old-source scopes and do not cover these new display assertions or this initializer change.

## Diagnostic retirement — source checkpoint 2026-10-14

`bc5cecffb` removes temporary Store selection/query diagnostics only. Independent `bf1b61f10` runtime PASS predates retirement; current-source Mists preservation and Retail/PTR display/query/lifecycle proof remain pending. Historical failures and earlier Retail acceptance retain their revision scopes.

## Current bounded Retail receipt — 2026-10-14

[Public historical Store/macro packet](../../data/patch-api/evidence/store-macro-lifecycle-2026-10-14/README.md) retains the recorded Mists query PASS/public Store UI FAIL and Retail query/public Show receipt through `a6b572732`, including earlier compile/setup/admission failures. Accepted copy/privacy verification covers 170/170 candidate byte pairs and bounded known-literal privacy only; it adds no executable/source authentication or behavioral acceptance. Later item/display changes, PTR preservation, current UI closure, native/render and full-parent acceptance remain excluded.

Independent existing-artifact gate at source `a6b572732`: **PASS for recorded Retail simulator execution**; the independent gate inspected saved outputs without rerunning it. Normal-session record-contract marker `records` and completion marker accompany one original public `StoreFrame:Show()` invocation, without forced callbacks or probe getter overrides. Store and its single active `Apprentice Rider Bundle` card are both shown/visible. Reported card GetRect `(613,609.5,277,224)` lies inside Store `(400,351.5,800,537)` with positive sizes; this is geometry, not render or parent correctness.

CLI0, final `[]`, CLEAN and zero observed Lua errors; 291 Blizzard addons and 1569/1569 bytecode cache hits. Startup simulator workarounds ran, including catalog product-card defaults twice. CASC, sound, third-party addons and SavedVariables disabled; read-only WTF EditMode cache loaded. This does not prove pristine/vendor-free behavior, callback source purity, native parity, rendering, PTR/Mists UI or whole-parent acceptance. Mists query PASS and original public Store UI FAIL above remain separate; PTR pending.

## Seven-profile saved runtime matrix — 2026-10-14

Source association is report metadata, not authenticated source inputs. Source/check checkpoint is `18a49459a014d1e212b72d4163f9151b96d71fa1`; the separate compiled revisions below are not interchangeable with that checkpoint or later documentation commits.

| Profile | Compiled source association | Item / vehicle | Store / bag / selected-unit | Independent saved gate |
|---|---|---|---|---|
| Retail | `d288c2701aa880648db73bba473db5dcc35c521b`, production delta `bc5cecffb`; test-only `4d` excluded | 4 / 5 PASS | 2 / 1 / 2 PASS; normal public Store PASS | `store-display-retail-runtime-current/report.md`: bounded runtime PASS |
| PTR | `bc5cecffbd89d719aa68390f4f12246059f013c5` | 4 / 5 PASS | 2 / 1 / 2 PASS; normal public Store PASS | `store-display-ptr-runtime-current/report.md`: bounded runtime PASS |
| Mists | `4d5572f241cce3d99a7b486709186a2638566a56`, production delta `bc5cecffb` | 4 / 5 PASS | 2 / 1 / not selected; normal public Store PASS | `store-display-mists-final-runtime-current/report.md`: runtime PASS, copied-byte receipts PASS, authentication/overall PARTIAL |
| Wrath | `18a49459a014d1e212b72d4163f9151b96d71fa1` | 4 / 5 PASS | not selected | `item-macro-wrath-era-runtime-current/report.md`: 9/9 PASS per profile |
| Era | `18a49459a014d1e212b72d4163f9151b96d71fa1` | 4 / 5 PASS | not selected | same Wrath/Era report: 9/9 PASS per profile |
| Anniversary | `18a49459a014d1e212b72d4163f9151b96d71fa1` | 4 / 5 PASS | not selected | `item-macro-anniversary-runtime-current/report.md` now exists: 9/9 PASS |
| WoWForever | `18a49459a014d1e212b72d4163f9151b96d71fa1` | 4 / 5 PASS | not selected | `item-macro-forever-runtime-current/report.md`: 9/9 PASS |

Report basenames above are local verification evidence, not published packet links. Store Rust scopes are 2×3; corrected ready-bag scope is 1×3; selected-unit preservation is 2×2 (Retail/PTR only). Item scope is 4×7 and vehicle scope 5×7. The named runtime gates record **76 profile/test outcomes at these scopes**, not 76 unique tests. Separate normal Store scenarios are not additive Rust cases; nested cleanup children repeat an outer case. Aggregate acceptance remains PENDING; Mists authentication remains PARTIAL.

Retail/PTR each retain one cleanup `AuraContainerUtil` nil failure reported twice. Separate normal CLEAN results do not erase that failure. Historical RED, compile failures, denial, zero-launch setup failures and earlier Mists query PASS/UI FAIL remain recorded above. Mists retains main strict-control evidence and accepted copied hashes; original-normal numeric pre-copy stat association is not independently established. Runtime PASS does not upgrade its overall PARTIAL or establish source authenticity.

`4d` expands test eligibility only; it adds no production behavior or native/older-profile proof by itself. Caller-accepted `store-item-final-check-source-current/report.md` records formatting/default check PASS with six manifest warnings and three synthetic sealer cases PASS: source-only, not runtime authentication. No reruns were performed for this update. Native behavior, rendered pixels, globally clean startup, all-registry coverage and full-parent acceptance remain unproved.

### Classic Store applicability and publication boundary

Local handoff `classic-store-consumer-applicability-current/report.md` records actual WoWForever TOC-selected TBC consumers reading group records; a separate secure helper projects those records to IDs internally. This supports the existing non-Mists public record path, not blanket ID-producer expansion. Wrath/Era/Anniversary cache absence is an evidence limit, not a native API gap; manifest entries alone are not runtime consumer proof. WoWForever cleanup source absence leaves that lifecycle unproved. Store acceptance is only the three named runtime profiles above.

Public Store/macro 170-copy packet is CLOSED and unchanged. The separate [item/display/profile packet](../../data/patch-api/evidence/item-display-profile-preservation-2026-10-14/README.md) retains495 whole texts after independent private-copy integrity and bounded decoded privacy PASS. Independent public-copy gate at `342b64c3824c13b069fc84c42896086c6b6380ca` passes495/495 whole-byte pairs,497/497 tracked files, new links and bounded decoded privacy. Packet CLOSED; reuse Git/stat metadata without reopening accepted payloads. Source authenticity, Mists overall PARTIAL, universal privacy/unlinkability and broader parent limits are not upgraded.

## Out of scope

- Other Classic profiles: preserve existing record behavior; WoWForever cached TBC consumer evidence and older cache limits are scoped in the matrix above, not Store runtime acceptance.
- Native parity claims: no official cached StoreSecure declarations or native execution evidence.
- Whole Store migration, new catalog defaults, transaction/settlement semantics, forced handlers, spies, getter overrides, and nil public category helpers.
