# Patch 11.0.7 publication sweep

Probe retained Warcraft Wiki page 609319, revision 6726777, against unmodified cached Game UI. [Audit](../wiki/investigations/patch-11-0-7-api-audit.md) owns source provenance, implementation rationale and semantic boundaries. Default retail carries 12.1.0, not historical 11.0.7.

## What it must do

- [x] Probe all 98 inventory occurrences, applying eleven later registers (11.1.0 through 12.1.0) chronologically. Latest add/remove wins; changes preserve publication. Keep original direction and supersession IDs.
- [x] Persist observations before exact gap-ID comparison. P1107_SWEEP_OUT selects results; P1107_SWEEP_REGISTER selects a same-sized negative-control register. Changing C_AccountStore.BeginPurchase added → removed must add exactly one gap and resolve none.
- [x] Keep three C_ArrowCalloutManager WorldLootObject callout members and C_WorldLootObject.GetCurrentWorldLootObjectSwapInventoryType absent after repeated raw/ordinary lookup. Preserve AcknowledgeCallout and both cached LFG deprecation surfaces.
- [x] RemoveRaidTargets clears all GUID-keyed markers, returns no values and notifies after mutation; allow reassignment afterward. Callback timing on repeated/empty clearing follows the existing simulator SetRaidTarget policy, not native timing proof. Cached SecureActionButton_OnClick clear-all dispatch must reach this producer without vendor substitution.
- [x] Validate exhaustive 181-row page ledger, source hashes, extract/register reproduction, retained inputs and all proof artifacts.

## How it works

- [Source, supersession and bounded fixes](../wiki/investigations/patch-11-0-7-api-audit.md#root-causes-and-bounded-fixes)
- [Coverage matrix](../wiki/investigations/patch-11-0-7-api-audit.md#coverage-matrix)
- [Profile/epoch architecture](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/patch_retired_members.rs` — suppress retired-member autostub fabrication on supported retail epochs, not classic profiles.
- `src/lua_api/globals/real/raid_targets.rs` — remove modeled GUID raid markers and publish the global.
- `src/lua_api/globals/{real/mod.rs,register.rs}` — compile/register modeled global for supported 12.x retail epochs.
- `tools/extract_patch_non_inventory.py` — preserve non-inventory text, build transition and explicit unexpanded profiler-enum transclusion.
- `data/patch-api/sources/11.0.7-*` — pinned source/register/extract/provenance and patch-page-coverage/v1 ledger.

## Tests asserting this spec

- `tests/patch_11_0_7_publication_sweep.rs` / `tests/common/publication_sweep.rs` — exact publication/absence and chronological supersession.
- `tests/patch_11_0_7_publication_fixes.rs` — repeated retirement absence; multi-unit clear-all, callback-visible state, repeated empty operation and reassignment.
- `tests/patch_11_0_7_cached_raid_targets.rs` — full cached secure-action path after preload, with no new Lua errors.
- `tools/test_extract_patch_non_inventory.py` — retained transclusion reference, stable row ID, build-context classification; unknown templates still fail explicitly.
- [Artifact validator](../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-validate.py) — accounting/source/fixture/proof contract, not native parity.

## Known gaps (current cycle)

- [ ] 28 exact publication gaps: [per-ID producer/model/policy boundaries](../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-gap-review.json).
- [ ] 69 non-inventory contract rows: [exhaustive scout](../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-extract-scout.md); no behavior credit from names, registration or empty DTOs.
- [ ] Native restricted-action authorization and raid-marker event timing; cached documentation declares restrictions but does not define authorization/timing policy.

## Out of scope

Historical 11.0.7 emulation, 11.x epoch features, native signature/output/security/behavior parity, external linked-page/transclusion expansion, full suite, agents/models, push and merge. Explicit publication may still be a placeholder; generic namespace autostubs are not explicit publication. Cached deprecation wrappers remain unchanged; their two accepted source/identity probes are not strict absence or successor DTO proof. Active cached consumers must not be retired.

## Local proof

[Proof ledger](../../data/patch-api/evidence/11.0.7-session-2026-10-06/p1107-proof.json) records exact command/cwd/revision/outcome and failed/superseded scopes. Runtime revision `9b4119c9e`; final cached-consumer fixture `e29e8b98f`. All commands/build artifacts use p1107-page; no sibling target reuse or copying. Later sources/fixtures/coverage ledgers remain unchanged from `6aba97a94`. Local development proof only, not independent/native acceptance.

| Isolated sweep | Rows | OK | Exact gaps | Result |
|---|---:|---:|---:|---|
| 11.0.7 | 98 | 70 | 28 | PASS |
| 11.1.0 | 116 | 97 | 19 | PASS |
| 11.1.5 | 125 | 89 | 36 | PASS |
| 11.1.7 | 48 | 40 | 8 | PASS |
| 11.2.0 | 162 | 135 | 27 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 414 | 94 | PASS |
| 12.0.0 | 1010 | 989 | 21 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

Each sweep runs alone through `cargo test --test integration <filter> -- --nocapture --test-threads=1`. Negative control: exactly one new gap, no resolutions, 28 → 29, expected exit 101. Two new behavior tests GREEN after RED; three isolated prefork cases pass (cached secure clear-all and two deprecated-LFG surfaces). An initial fixture tried SECURE_ACTIONS, a local cached table; corrected fixture uses the exported consumer entry point. Failed evidence retained with explicit invalidation reason.

`cargo fmt` / `cargo fmt --check`, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded exit-0 startup [] pass. Zero non-vendor warnings; six existing iced manifest deprecations plus vendor summary remain unsuppressed. Changed Rust lines manually reviewed for readability. Sixteen extractor/register fixtures pass (13 extractor, three register); deterministic extract reproduction, all twelve byte-identical register regenerations and artifact validation pass at `45e5d932a`. Portable artifact validator passes at `7c6765826` using tracked warning/test summaries, without requiring ignored local logs. Later evidence/docs edits do not invalidate those scopes. No full suite.

181 unique IDs: 56 partial-development-green, 11 bounded-coverage (nine strict removals/two cached aliases), 97 audit-pending, 17 metadata-only. Three later reversals receive metadata-only credit. Publication accounting is not full behavioral parity; ledger stays in-progress.
