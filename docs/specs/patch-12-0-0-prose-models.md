# 12.0.0 prose model proof

Unassigned PROSE-MODELABLE extract rows from the [scout](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-extract-scout.md). [Per-source report](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-b8-prose-report.md) owns results.

## What it must do

- [x] COMBAT_LOG_EVENT and COMBAT_LOG_EVENT_UNFILTERED script registrations error on addon-tainted RegisterEvent/RegisterUnitEvent paths, preserve existing registrations and receive no OnEvent delivery after rejection. Callback classification remains distinct.
- [x] Revalidate all 21 legacy retirements and available successors through the existing data-driven native/cached proof, distinguishing historical execution from current Retail.
- [ ] Establish operation-specific secret-value behavior only with a documented/native operation oracle; predicate publication is not that proof.

## How it works

- [Event system](../event-system.md).
- [Retirement successors and scope](retirement-successors.md).
- [Secret-number ordering limitations](secret-number-ordering.md).

## Implementation inventory

- `src/event/valid_events.rs`: 12.0.0 script-registration exclusion; both events remain valid callback names.
- `tests/patch_12_0_0_prose.rs`: tainted registration and no-delivery boundary.
- `tests/patch_12_0_0_deprecated.rs`: all21 row assertions untouched. Canonical cached case `patch_12_0_0_deprecated_retirement_and_successors` belongs to separate `prefork_full_ui` target; the prose integration filter does not select these21 rows.
- Cast model/module/registration from Retail1200 onward: [count inventory](spell-count-outputs.md#implementation-inventory). Display/max/secret125 boundaries unchanged; exact1200 bounded GREEN recorded below.

## Tests asserting this spec

`prose_combat_log_registration_errors_without_delivery`, `prose_callback_event_mechanism_is_separate_from_script_registration`, `patch_12_0_0_deprecated_retirement_and_successors`, historical `patch_12_0_0_deprecated_native_retirement`.

## Known gaps (current cycle)

Historical native/cached registration and retirement results remain revision-scoped in the per-source report; they do not establish current exact-12.0.0 acceptance. Source inspected 2026-10-10: `on_update.rs` guards `private_aura_sounds::playback::tick` with `retail-12-0-5`; `Cargo.toml` already supports exact epoch selection with `profile-retail,retail-12-0-0` and default features disabled. The older feature-gate failure is historical, not the current blocker.

Historical proof was **storage-blocked**. Main unit `wow-retail-1200-prose-20261010t154144z`, submitted for `9e7af4de09c607adb1841a7335e9b0b69672be1d`, reached compilation but failed writing `target/debug/deps/rustcXtklhi/lib.rmeta`: `No space left on device (os error 28)`; `build-finished.success=false`. Worker final receipts are missing and `source-after.json` is zero bytes, so source equality is not established. No usable test artifact or selected test execution is established; this is neither behavioral RED nor compile PASS nor native parity.

Inspected receipts under `/home/osso/.local/state/wow-ui-sim/verification/retail-1200-prose-current/`: `storage-failure-main.json`, `20261010T154144Z/compile.stdout`, and `20261010T154144Z/submission.json`. The main receipt's later observation records target cache absent and 78,797,254,656 free bytes (~73 GiB); neither explains the earlier ENOSPC or establishes why the target was removed. Historical failure, RED and ledger records remain unchanged. See the [exact-1200 wiki audit](../wiki/investigations/patch-12-0-0-api-audit.md#current-prose-proof--test-boundaries-and-count-model-gate-2026-10-10).

Actual retry `20261010T163526Z` under the same receipt root records `compile_exit=101`, `source_equal=true`, and `selected_execution_performed=false` in `outcome.json`. `compile.stdout` contains five errors: unresolved `iced::Point`, configured-out `AppearanceSourceInfo`, configured-out PTR helper, unavailable `spell_cast_counts` field, and unavailable `transmog_appearance_sources` field. Current blocker is test feature boundaries plus the actual cast-count model gate, not storage.

Source inspection of `ee21cd9cc` only: tuple coordinates replace `Point`, preserving all game-menu tests in headless builds; the later-only transmog state fixture is gated at `retail-12-0-5`; the earlier encounter-record test removes the inappropriate `retail-12-1-0` PTR helper while retaining its assertions. No post-repair compile or selected runtime proof is claimed: **runtime PENDING**. At this revision `spell_cast_counts` exists in `src/lua_api/state/sim_state.rs:409`, gated by `retail-12-0-5`; it is not absent from the checkout. Production count state/model changes belong to a separate stage and receive no fix or acceptance credit here.
- [ ] General secret operations lack an operation-specific authoritative oracle in the captured prose. VM/dependency changes outside this worktree are unauthorized.

## Authentic count RED and source-present repair — 2026-10-10

Independent audit of stage `3c36b24546c0901837d4987b86ad33c634a8f671`, receipt `20261010T164918Z`: compile exit0/source_equal true; exact sealed `patch_12_0_0_prose::prose_spell_cast_count_reads_live_explicit_inputs` execution exit101/artifact_unchanged true, **actual1 FAIL / 0 PASS**, expected7 got0 for19750. Both bare environments initialized; arity/numeric/non-secret checks preceded the failing value assertion. Later isolation, charge independence and live mutation/removal/clear assertions were not reached. This is authentic behavioral RED, not a compiler failure, timeout or zero selection. The auditor's PASS means receipt audit passed, not producer runtime acceptance.

After `84dcf2723`, inspected cast implementation/module/registration and inverse shim gates are present from1200 onward. Ordinary pre125 output is simulator policy, not native proof. Display/max/secret125 behavior remains unchanged. Older API presence does not require a specific internal native backing model. **Historical checkpoint: post-repair GREEN was PENDING; superseded only within the bounded receipts below.** All21 retirement row assertions remain untouched; cached canonical case requires the separate prefork target, not prose-filter coverage. Historical failed compilations above remain separate epochs.

[Sanitized receipts and independent audit](../../data/patch-api/evidence/12.0.0-session-2026-10-05/count-red-20261010/independent-report.md), with adjacent SHA-256 manifest, retain this RED without binaries, source snapshots or vendor stderr. Hashes establish retained bytes and the audit's recorded scope, not hermetic native provenance.

## Bounded count receipt PASS — 2026-10-10

[Retained independent audit](../../data/patch-api/evidence/12.0.0-session-2026-10-05/count-green-20261010/independent-report.md) and adjacent manifest/stdout/check receipts establish actual **5/5 exact1200 + 40/40 current-default PASS**, zero failures. Relevant code is `84dcf27231db5f06d8bf39cb6ba2ddaaf91fdc7c`; submission/check revision `095081101f04254270dee6805ac6b8cbb8e9ea3b` changes only docs/evidence. Both compilations, both fmt checks and exact/default Cargo checks exit0; source manifests match before/after and audited relevant bytes. Not warning-free: exact headless scope retains 10 preexisting project warnings; both scopes retain six iced manifest deprecations. No warnings repaired or suppressed.

| Selected scope | Actual PASS | Boundary |
| --- | ---: | --- |
| Exact1200 prose | 3/3 | callback distinction, combat-log rejection, live explicit cast inputs |
| Exact1200 early policy | 1/1 | supplied count remains ordinary despite restriction flag |
| Exact1200 BARE21 | 1/1 | absence assertions for21 rows ONLY; not21 tests or cached successor acceptance |
| Current-default count outputs | 30/30 | cast/display simulator contract |
| Current-default maximum applications + metadata defaults | 8/8 + 2/2 | default includes125 and later features; not isolated exact125 |

Separate historical cached21 case `patch_12_0_0_deprecated::patch_12_0_0_deprecated_retirement_and_successors` is actual PASS at **c6bc87c120**, saved full-suite log line13804; [retained exact line/provenance](../../data/patch-api/evidence/12.0.0-session-2026-10-05/count-green-20261010/cached21-historical.json). This supplements the audit's unresolved cached-case lookup, without changing its original text: neither count receipt selected that cached case, and c6bc does not cover later84dcf2723 acceptance. BARE21 does not replace it.

Ordinary pre125 output and display policies remain inferred simulator behavior; native secrecy/input permission remains unknown or unmodeled. Historical RED/storage/feature failures remain separate epochs. No native, full-suite, all-profile or hermetic provenance claim. Later AutoHide test963 is outside count proof scope.

## Out of scope

No vendor overrides, callback security redesign, native secret-operation parity invention or rilua source changes. Existing older-epoch and Classic registration policies remain unchanged.
