# Housing basic-mode pending decor request

Bounded pending-request model for 12.0.5 `C_HousingBasicMode.StartPlacingNewDecor`. Reuses the [catalog full variant identity](housing-catalog-variants.md), not a decor instance. Retained [change source](../../data/patch-api/sources/12.0.5-api-changes.txt) rows `global api-C_HousingBasicMode-StartPlacingNewDecor-278` and `-279` rename argument 1 to `catalogEntryVariantID` and change its type to `HousingCatalogEntryVariantID`. These rows do not establish placement or validation semantics. Bounded independent pending-request acceptance is recorded below; both source rows remain audit-pending. No full-placement claim.

## What it must do

All lifecycle and validation policies below are **bounded simulator inferences**, not native-verified behavior. Checkboxes describe requirements, not native or complete-placement acceptance. Bounded compiled proof is recorded below; inferred policies remain qualified.

- [ ] A fresh `HousingState` has no pending request. Its `pending_new_decor` is an `Option<HousingCatalogEntryVariantID>` using the existing C API-owned full `(recordID, entryType, variantIdentifier)` type. It carries no GUID, instance, transform, stock reservation or selection data.
- [ ] `StartPlacingNewDecor(catalogEntryVariantID)` validates the entire public integer selector before mutation. An existing exact variant with positive explicitly supplied `num_stored` sets pending to that full identity. No base entry, seeded catalog or alternate selector lookup is required. Identical valid requests are repeatable; valid replacement requests replace record, type and variant identity. Changing the caller's table afterward does not change pending.
- [ ] `IsPlacingNewDecor()` returns exactly one ordinary boolean reflecting pending presence. Start and cancel return zero values. `CancelActiveEditing()` clears this pending request; repeated cancel is harmless. This bounded cancel inference does not expand or replace existing placed/customize/preview selection behavior.
- [ ] Unknown full keys and nonpositive stored counts are inferred no-ops, preserving any existing pending request. Destroyable count is independent of eligibility: positive stock with zero destroyable count can start; zero stock with positive destroyable count cannot. No stock, destroyable count, dye or base catalog mutation occurs on start, replacement, rejection or cancel.
- [ ] Missing/malformed selectors, noninteger fields, out-of-range numbers and secret selectors fail explicitly and atomically. Plain rejection tests require neither fixture input nor existing pending. Separate populated tests assert prior pending survives rejection. Public secure/tainted callers preserve caller taint; host-installed guarded-table restrictions remain enforced. Conservative secret rejection in secure callers is stricter than `AllowedWhenUntainted`, not native parity.
- [ ] Pending requests are per environment: starting/replacing/canceling in one environment cannot alter another, including an empty catalog environment.
- [ ] Existing `GetSelectedDecorInfo`, `IsDecorSelected`, placed list, customize info and preview state/count stay unchanged. Never synthesize `GetSelectedDecorInfo` from a catalog variant: its declared return is selected **instance** info. No new `decorGUID`, placed instance, 3D transform, selection event, storage event or placement success/failure event is fabricated.
- [ ] `FinishPlacingNewDecor` remains the existing no-op; this input checkpoint and future bounded pending producer must not claim commit/placement completion. Pending survives finish until cancel or a valid replacement request.
- [ ] The later replacement preserves unconditional registration of the replaced BasicMode surface; no new profile gate. Tests here target only `retail-12-0-5`; registration requirement is not an all-profile execution claim.

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
- `tests/housing_pending_decor.rs`: fourteen bounded behavioral expectations discovered into the existing grouped `integration` target. No new Cargo target or global fixture state.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: only BasicMode start/query/cancel publishers removed. Finish remains a documented no-op; selected/customize/preview owners remain separate.
- `src/c_api/c_housing/basic_mode.rs`: unconditional `register_pending` publishes start/query/cancel; existing free-place registration retains its original feature gate. Start uses the shared catalog parser/VM table-access guard, then stores the exact ID only for an existing variant with positive stock. Query reads presence; cancel clears only pending. No events or other housing mutations.
- `src/c_api/c_housing/catalog/input.rs`: shared selector and full-variant parsing exposed within the housing module; validation behavior unchanged.

## Tests asserting this spec

Filter: `housing_pending_decor::` in existing target `integration`. Parent reports compiled RED at input commit `659f79a3c`: **1 PASS / 13 FAIL**, including empty pending and missing-selector acceptance failures (`batch22-red-build/run*`). Historical implementation checkpoint deferred GREEN to parent; the reconciled independent proof below supersedes that pending status.

| Cases | Observable boundary |
|---|---|
| `plain_default_and_cancel_without_input_guards`, `plain_missing_selector_errors_without_catalog_or_pending_setup`, `plain_invalid_selectors_error_without_input_guards` | Fresh environment, no seeded fixture/pending prerequisite; default, arity, malformed rejection |
| `start_and_cancel_record_only_pending_request`, `repeated_and_replacement_requests_preserve_full_identity`, `replacement_request_changes_record_identity` | Public pending boolean plus concrete Rust pending full ID, repeat/cancel/restart, caller-table independence |
| `unknown_and_zero_stock_noop_preserve_empty_or_existing_pending`, `nonpositive_stock_cannot_replace_pending_request`, `malformed_selector_errors_atomically_with_existing_pending` | Exact-key/no-stock no-op and error atomicity, explicit stock/destroyable/dye preservation |
| `public_addon_requests_and_cancel_preserve_taint`, `secret_selectors_reject_atomically_in_secure_and_tainted_callers`, `guarded_selector_rejects_tainted_access_without_replacing_pending` | Public addon access; host-rooted secrets and actual VM table-access guard |
| `pending_requests_do_not_leak_between_environments`, `pending_lifecycle_preserves_instances_preview_and_emits_no_success` | Independent environments; complete existing public info snapshots, listeners and queued-event comparison, unresolved finish |

### Proof ledger — input checkpoint

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
- [ ] Native validation/no-op/cancel semantics, secure secret access and all-profile execution unverified.

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

Rows `global api-C_HousingBasicMode-StartPlacingNewDecor-278` (argument rename) and `-279` (variant argument type) link this bounded full-selector evidence but **remain audit-pending**, like DestroyEntry. Partial inferred pending behavior does not establish the literal delta across the real placement boundary; no whole-domain promotion. Counts remain **278 audit-pending / 70 bounded-coverage / 14 partial-development-green**, all **362** IDs and source SHA-256 `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329` retained. Audit **IN PROGRESS**.

Native validation/cancel/eligibility, secure-secret parity, full UI readiness and all-profile execution remain unverified. Advisory helper length finding is not a reason for an unrelated refactor. No finish, instance allocation, 3D, stock mutation or event-production coverage claimed.
