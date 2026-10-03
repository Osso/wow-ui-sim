# Housing basic-mode pending decor request

Bounded pending-request model for 12.0.5 `C_HousingBasicMode.StartPlacingNewDecor`. Reuses the [catalog full variant identity](housing-catalog-variants.md), not a decor instance. Retained [change source](../../data/patch-api/sources/12.0.5-api-changes.txt) rows `global api-C_HousingBasicMode-StartPlacingNewDecor-278` and `-279` rename argument 1 to `catalogEntryVariantID` and change its type to `HousingCatalogEntryVariantID`. These rows do not establish placement or validation semantics. Bounded independent pending-request acceptance is recorded below; both source rows remain audit-pending. No full-placement claim.

## What it must do

Pending lifecycle, eligibility, missing/malformed field and numeric-domain policies below are **bounded simulator inferences**, not native-verified behavior. The cached `AllowedWhenUntainted` declaration separately grounds the secret-input authorization boundary; it does not establish native parsing or placement semantics. Checked requirements mean accepted bounded simulator pending/input proof only, not native or complete-placement acceptance. Main accepts independent574 PASS for producer `c685f487`, with source-backed runtime equivalence through import-only `d83666b13`; historical compiled checkpoints remain separate below.

- [x] A fresh `HousingState` has no pending request. Its `pending_new_decor` is an `Option<HousingCatalogEntryVariantID>` using the existing C API-owned full `(recordID, entryType, variantIdentifier)` type. It carries no GUID, instance, transform, stock reservation or selection data.
- [x] `StartPlacingNewDecor(catalogEntryVariantID)` authenticates the original selector and all three original fields before parsing any field, checking domains or consulting the model, then validates the entire integer selector before mutation. An existing exact variant with positive explicitly supplied `num_stored` sets pending to that full identity. No base entry, seeded catalog or alternate selector lookup is required. Identical valid requests are repeatable; valid replacement requests replace record, type and variant identity. Changing the caller's table afterward does not change pending.
- [x] `IsPlacingNewDecor()` returns exactly one ordinary boolean reflecting pending presence. Start and cancel return zero values. `CancelActiveEditing()` clears this pending request; repeated cancel is harmless. This bounded cancel inference does not expand or replace existing placed/customize/preview selection behavior.
- [x] Unknown full keys and nonpositive stored counts are inferred no-ops, preserving any existing pending request. Destroyable count is independent of eligibility: positive stock with zero destroyable count can start; zero stock with positive destroyable count cannot. No stock, destroyable count, dye or base catalog mutation occurs on start, replacement, rejection or cancel.
- [x] Missing/malformed selectors, noninteger fields and out-of-range numbers fail explicitly and atomically under the inferred parser policy. Plain rejection tests require neither fixture input nor existing pending. Separate populated tests assert prior pending survives rejection.
- [x] `AllowedWhenUntainted`: secure callers may supply actual secret NUM fields or a secret-wrapped TABLE carrying the full selector; authorized valid inputs must produce the same meaningful pending identity as public inputs. Tainted callers must receive authentication denial before wrong type, malformed/missing public fields, field-domain errors or model misses can hide a later denied secret. A secure secret NUM used as the top selector authenticates, then fails the TABLE type check. Authentication/acceptance/denial never clears or adds caller taint, modifies catalog state or emits events. Underlying guarded-table access checks still apply after authorized unwrap. Conservative secure-secret rejection is incompatible with this boundary, not native parity.
- [x] Pending requests are per environment: starting/replacing/canceling in one environment cannot alter another, including an empty catalog environment.
- [x] Existing `GetSelectedDecorInfo`, `IsDecorSelected`, placed list, customize info and preview state/count stay unchanged. Never synthesize `GetSelectedDecorInfo` from a catalog variant: its declared return is selected **instance** info. No new `decorGUID`, placed instance, 3D transform, selection event, storage event or placement success/failure event is fabricated.
- [x] `FinishPlacingNewDecor` remains the existing no-op; this input checkpoint and future bounded pending producer must not claim commit/placement completion. Pending survives finish until cancel or a valid replacement request.
- [x] The later replacement preserves unconditional registration of the replaced BasicMode surface; no new profile gate. Tests here target only `retail-12-0-5`; registration requirement is not an all-profile execution claim.

### Cached declarations and consumers — read 2026-10-01

Cache root: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`. This is cached source, not a historical 12.0.5 client probe; the [catalog provenance](housing-catalog-variants.md#cached-declaration-provenance-2026-10-01) identifies retail 12.1.0.69933.

- `Blizzard_APIDocumentationGenerated/HousingBasicModeUIDocumentation.lua:185–194`: `StartPlacingNewDecor`, `SecretArguments = "AllowedWhenUntainted"`, one non-nil `catalogEntryVariantID: HousingCatalogEntryVariantID`.
- Same file `:118–125`: `IsPlacingNewDecor` returns non-nil `hasPendingDecor: bool`; `:40–48` describes `GetSelectedDecorInfo` as "Returns info for the decor instance that's currently selected, if there is one", returning nullable `HousingDecorInstanceInfo`. Cancel `:11–14` describes canceling editing/returning unplaced decor, not this inferred no-stock-mutation policy. Finish `:26–28` has no documented commit contract.
- `Blizzard_HousingTemplates/Blizzard_HousingCatalogEntry.lua:687–700` selects preview separately and passes `self.entryVariantID` directly for ordinary new decor. `:703–704` expects a selection event for sound; this full UI behavior is outside the bounded request model.
- `Blizzard_HouseEditor/Blizzard_HouseEditorBasicDecorMode.lua:21–25,68–70` branches on pending to call finish; `:186–199` uses pending/selected flags to cancel on Escape. `:38–44` requests instance info only in selected-target event handling. No contradiction with a separate bounded pending request; full UI event/commit behavior remains missing.
- `Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua:212–219` documents the old/new selector wrapper. Legacy selector compatibility is not introduced by this slice.

## How it works

- [Compound catalog inputs/security](housing-catalog-variants.md).
- [Independent destroyable count](housing-destroyable-count.md).
- [Housing audit ownership](../wiki/investigations/patch-12-0-5-api-audit.md).

## Implementation inventory

- `src/lua_api/state/support_types.rs`: empty-default `pending_new_decor` slot; existing C API-owned variant ID reused.
- `tests/housing_pending_decor.rs`: twenty authored bounded behavioral expectations in the existing grouped `integration` target: thirteen retained controls and seven B71 secret-boundary cases replacing one combined conservative rejection case. No new Cargo target or global fixture state.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: only BasicMode start/query/cancel publishers removed. Finish remains a documented no-op; selected/customize/preview owners remain separate.
- `src/c_api/c_housing/basic_mode.rs` and `src/c_api/c_housing/pending_input.rs`: main-supplied producer `c685f487` uses a narrow `AllowedWhenUntainted` pending parser. It authenticates the original selector and all three original fields before parsing, roots the underlying table and retains `check_table_access`. Unconditional start/query/cancel publication and existing free-place feature gate remain; exact positive-stock pending identity, query and cancel retain their host-model behavior.
- `src/c_api/c_housing/catalog/input.rs`: shared catalog parser unchanged; B71 grants no other catalog-security credit. Stock, nonplacement/finish behavior, selected/customize/preview state and no-events policy remain unchanged. Inventory is supplied by main, not independently source-inspected here.

## Tests asserting this spec

Filter: `housing_pending_decor::` in existing target `integration`. Parent reports compiled RED at input commit `659f79a3c`: **1 PASS / 13 FAIL**, including empty pending and missing-selector acceptance failures (`batch22-red-build/run*`). Historical implementation checkpoint deferred GREEN to parent; the reconciled independent proof below supersedes that pending status.

| Cases | Observable boundary |
|---|---|
| `plain_default_and_cancel_without_input_guards`, `plain_missing_selector_errors_without_catalog_or_pending_setup`, `plain_invalid_selectors_error_without_input_guards` | Fresh environment, no seeded fixture/pending prerequisite; default, arity, malformed rejection |
| `start_and_cancel_record_only_pending_request`, `repeated_and_replacement_requests_preserve_full_identity`, `replacement_request_changes_record_identity` | Public pending boolean plus concrete Rust pending full ID, repeat/cancel/restart, caller-table independence |
| `unknown_and_zero_stock_noop_preserve_empty_or_existing_pending`, `nonpositive_stock_cannot_replace_pending_request`, `malformed_selector_errors_atomically_with_existing_pending` | Exact-key/no-stock no-op and error atomicity, explicit stock/destroyable/dye preservation |
| `public_addon_requests_and_cancel_preserve_taint`, `guarded_selector_rejects_tainted_access_without_replacing_pending` | Retained public addon access and actual VM table-access guard |
| `secure_secret_numeric_fields_accept_actual_full_variant_request`, `addon_secret_numeric_fields_deny_atomically_without_clearing_taint` | Actual host-secret NUM fields, individually and together; secure full-identity replacement vs addon authentication denial, empty/existing pending atomicity |
| `secure_secret_table_selector_accepts_and_copies_full_identity_across_gc`, `addon_secret_table_selector_denies_before_lookup_and_preserves_pending` | Actual TABLE wrapped by `wrap_secret`, underlying table rooted before allocation; first/second identity, caller-table independence, GC and environment isolation |
| `secret_numeric_top_selector_authenticates_before_secure_type_error`, `all_original_fields_authenticate_before_any_public_parse_domain_or_model_lookup`, `secret_wrapped_guarded_table_retains_underlying_access_constraints` | Secure wrong-top-type vs addon auth; malformed/missing/domain/model public inputs cannot mask later secret denial; real VM guarded-table constraints |
| `pending_requests_do_not_leak_between_environments`, `pending_lifecycle_preserves_instances_preview_and_emits_no_success` | Independent environments; complete existing public info snapshots, listeners and queued-event comparison, unresolved finish |

### B71 implementation — independent bounded acceptance

Scope: exact rows278/279 only; no coverage/count/source-row promotion. Main-supplied existing proof: fourteen tests pass at `361642437245548aaab32e74e6bc7faa843cef66` (ancestor `e7750b17d`); not independently rerun here. That proof includes conservative rejection of secure secrets and is **not** `AllowedWhenUntainted` parity. Historical GREEN below does not cover these revised expectations.

Current input file has **20 tests**: retain13, replace1 with7. New secure NUM-field and wrapped-TABLE acceptance cases require real pending first/second identity, not registration or permissive no-op success. Real host NUM payloads and TABLE identity are inspected through existing VM helper APIs; underlying tables are stack-rooted before wrapper allocation and published wrappers remain rooted. Secret cases compare the complete catalog fixture, existing instance/preview snapshots, listeners with zero events and the queued-event names; exercise collection, caller taint and independent environments. Guarded-table tests retain host-installed VM restrictions, not native guard-policy claims.

Cached primary source `HousingBasicModeUIDocumentation.lua:185–194` expressly declares `SecretArguments = "AllowedWhenUntainted"` and `catalogEntryVariantID: HousingCatalogEntryVariantID`. Cached declarations are not native historical12.0.5 probes. Pending/no-stock/no-events, missing/malformed inputs and numeric domains remain inferred; no native parsing probe, actual placement, finish/commit, UI or all-profile claim.

**Main-supplied actual compiled RED:** `4bec1394a`, after fixing the missing `LuaApi` test import, compiled in **220.479727s**, zero diagnostics; twenty selected tests produced **13 PASS / 7 FAIL** in **5.58s**. Artifacts: `/tmp/patch-12.0.5-batch71-red-fixed-build-result.json`, `/tmp/patch-12.0.5-batch71-red-run.json`, `/tmp/patch-12.0.5-batch71-red-run.stdout` and `/tmp/patch-12.0.5-batch71-red-run.stderr`. Input `df00d200` first failed compilation; that failure is **not behavioral RED**. Thirteen retained controls and seven secret-boundary expectations remain distinct; downstream assertions are not independently credited merely because a case failed.

Main accepts **independent574 PASS** ([report](/tmp/patch-12.0.5-housing-pending-secret-independent-proof.md), [JSON](/tmp/patch-12.0.5-housing-pending-secret-independent-proof.json)): **89 distinct Retail PASS = 20 pending + 69 catalog controls**, startup exit **0**, stdout **[]**, zero unique/occurrence errors. Saved GREEN compiled at `554efed4a` in **284.747327s**, with one historical unused-export warning. Independent source/root/security/all-Lua/readability/wiring inspection accepts meaningful pending identity, secure secret authorization, tainted denial before public parsing/model lookup, guarded-table access, atomicity, copied identity/GC, isolation and unchanged adjacent state/events. Shared catalog security is unchanged; historical conservative secure-secret rejection is superseded for pending only.

Final scoped format and default check exit **0**, zero diagnostics, check **21.503126679s** at `d83666b13`. Only the unused `read_selector`/`read_variant_id` export was removed; inspected executable behavior sources are unchanged from saved GREEN. Original check exit0 in **218.079007s** retains its one historical warning and included unmeasured build-lock wait. Import-only source equivalence permits saved runtime reuse, **not** a new final ELF or zero-diagnostic GREEN rebuild claim; default check does not compile integration tests. Advisory helper-length finding was rejected as unrelated refactor; no blocking counterexample.

Accounting commit `c7366a4a0` updates only the existing housing-pending-decor capability and notes278/279; separate accounting verification awaits a new agent. Exact278/279 **STATUS remains audit-pending**: real placement/finish/instance/stock/events remain unmodeled; no literal placement-delta credit. Counts unchanged: **177 pending / 164 bounded / 14 partial / 7 metadata**, 362 ordered IDs/77 capabilities. Native/type/acquisition/all-profile and isolated historical-epoch parity remain unverified. Dirty-combined/protected-path, global-format and historical-process limits remain; no blanket clearance. This reconciliation ran no source inspection, build/check/runtime gate or delegation.

### Proof ledger — historical input checkpoint

Base checkout `67b44f2c36c12c86cc1e10c334d010041239e7c9`, initially clean. No output producer, build, check, delegation or push authorized.

| Command / scope | Result | Limits / invalidation |
|---|---|---|
| Existing `target/debug/wow-sim --no-addons --no-saved-vars --exec-lua @/tmp/housing-pending-plain-red.lua lua-errors` under `timeout 90` | Exit 0: existing binary rejects nil selector | Not RED; binary does not represent checkout's unchanged no-op provider |
| Same executable with `/tmp/housing-pending-plain-state-red.lua` | Exit 0: empty query nil, pending false before/after | Plain existing-binary probe only, not new fixtures or checkout producer proof |
| `rustfmt --edition 2024 --config skip_children=true src/lua_api/state/support_types.rs tests/housing_pending_decor.rs` | Exit 0 | Formatting only; later Rust edits invalidate this scope |
| Same executable with `/tmp/housing-pending-source-plain-red.lua` | **Actual plain provider RED**, exit 1: `PENDING_SOURCE_PLAIN_NIL_ACCEPTED true`; assertion `StartPlacingNewDecor(nil) must error, current provider accepts it` | Executes complete unchanged checkout Lua provider via `loadstring` in separate BasicMode namespace, no catalog inputs/pending guards. Does not compile or exercise new Rust input fixtures. Prior `loadfile` attempt failed because loadfile unavailable; that harness failure is not RED |

Actual RED log: `/tmp/housing-pending-source-plain-red.log`. Executable SHA-256 `2e5a1629d63bda382231681cc0f15074a3ba11b76b22411826022785c805fa32`; checkout provider SHA-256 `2ed5af029fd8b140ad93b08072b2898834d7ba58a880fc1da6d8910d2fe7653f`. Script embeds the complete provider verbatim, sets its environment to a fresh BasicMode namespace inheriting runtime globals, executes it, then asserts plain nil rejection. This is executed behavior, not source-string matching. Provider changes invalidate this narrow RED evidence; new Rust fixture execution remains pending.

## Known gaps (current cycle)

- [x] Parent reports grouped compiled RED at `659f79a3c`: 1 PASS / 13 FAIL; separate from the earlier plain-provider probe.
- [x] Independent inspection accepts saved targeted GREEN for bounded C API start/query/cancel at `f59c03402`; no independent test reexecution.
- [ ] Finish/commit/placement lifecycle unresolved: existing finish no-op must remain visible as a gap.
- [x] Main reports B71 compiled RED at `4bec1394a`: 13 PASS / 7 FAIL after the missing-import fix; first compile failure is not behavioral RED.
- [x] B71 producer `c685f487`: main accepts independent574 bounded PASS, 20 pending + 69 catalog controls with import-only equivalence through `d83666b13`; historical fourteen-test proof does not cover revised secret boundaries.
- [ ] Native validation/no-op/cancel semantics, native secure-secret parity and all-profile execution unverified.

## Out of scope

Full placement/3D transforms/collision, GUID or instance allocation, stock reservation/consumption, dye/destruction mutation, preview placement, selected-instance synthesis, UI success/selection publication and full addon UI readiness. No native/full-row/all-profile completion claim. Exact source rows retain audit-pending status; bounded proof links do not establish full delta/domain acceptance.

## Reconciled bounded proof — 2026-10-01

[Independent report](/tmp/patch-12.0.5-housing-pending-decor-independent-proof.md) accepts only the explicit simulator pending-request contract at producer `f59c03402`. Compiled RED at `659f79a3c` is **1 PASS / 13 FAIL**: default/cancel passes; missing/malformed selector acceptance and missing pending prerequisites dominate failures, not thirteen independently exercised downstream boundaries.

Saved GREEN is **45/45 PASS**: 14 pending-request, 4 free-place, 2 decor, 1 customize and 24 catalog controls. Independent source/artifact inspection, not test reexecution. Fresh verifier default `cargo fmt --check` and `cargo check` exit **0** at the producer snapshot. Cumulative default retail features are not isolated historical 12.0.5 or all-profile execution. Parent batch22 startup exits **0**, stdout **[]**; saved matching binary/startup metadata is corroboration, **not independent compilation provenance** or verifier-run startup.

| Capability | Accepted proof / limit |
|---|---|
| Full pending variant identity, ordinary boolean/arity, repeat/replacement/cancel and environment isolation | Saved bounded GREEN and source inspection |
| Exact-key positive explicit stock eligibility; no stock/dye/count consumption; atomic malformed/secret/table-access rejection and caller taint | Saved GREEN; simulator policy inference, not native AllowedWhenUntainted parity |
| Existing instance/selection/preview snapshots and event queues unchanged | Saved GREEN and source; no fabricated instance, GUID, 3D transform, stock reservation or success/storage/selection event |
| Unconditional replaced registration | Source-only outside default profile |
| Finish/commit and real placement | Existing finish no-op; pending survives finish. Unmodeled, not accepted |

Rows `global api-C_HousingBasicMode-StartPlacingNewDecor-278` (argument rename) and `-279` (variant argument type) link this bounded full-selector evidence but **remain audit-pending**, like DestroyEntry. Partial inferred pending behavior does not establish the literal delta across the real placement boundary; no whole-domain promotion. Historical batch22 counts were **278 audit-pending / 70 bounded-coverage / 14 partial-development-green**; they are not current B71 accounting. At that checkpoint, all **362** IDs and source SHA-256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329` retained. Audit **IN PROGRESS**.

Native validation/cancel/eligibility, secure-secret parity, full UI readiness and all-profile execution remain unverified. Advisory helper length finding is not a reason for an unrelated refactor. No finish, instance allocation, 3D, stock mutation or event-production coverage claimed.
