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
- `tests/patch_12_0_0_deprecated.rs`: existing 21-row retirement/successor proof; unchanged.

## Tests asserting this spec

`prose_combat_log_registration_errors_without_delivery`, `prose_callback_event_mechanism_is_separate_from_script_registration`, `patch_12_0_0_deprecated_retirement_and_successors`, historical `patch_12_0_0_deprecated_native_retirement`.

## Known gaps (current cycle)

Historical native/cached registration and retirement results remain revision-scoped in the per-source report; they do not establish current exact-12.0.0 acceptance. Source inspected 2026-10-10: `on_update.rs` guards `private_aura_sounds::playback::tick` with `retail-12-0-5`; `Cargo.toml` already supports exact epoch selection with `profile-retail,retail-12-0-0` and default features disabled. The older feature-gate failure is historical, not the current blocker.

Historical proof was **storage-blocked**. Main unit `wow-retail-1200-prose-20261010t154144z`, submitted for `9e7af4de09c607adb1841a7335e9b0b69672be1d`, reached compilation but failed writing `target/debug/deps/rustcXtklhi/lib.rmeta`: `No space left on device (os error 28)`; `build-finished.success=false`. Worker final receipts are missing and `source-after.json` is zero bytes, so source equality is not established. No usable test artifact or selected test execution is established; this is neither behavioral RED nor compile PASS nor native parity.

Inspected receipts under `/home/osso/.local/state/wow-ui-sim/verification/retail-1200-prose-current/`: `storage-failure-main.json`, `20261010T154144Z/compile.stdout`, and `20261010T154144Z/submission.json`. The main receipt's later observation records target cache absent and 78,797,254,656 free bytes (~73 GiB); neither explains the earlier ENOSPC or establishes why the target was removed. Historical failure, RED and ledger records remain unchanged. See the [exact-1200 wiki audit](../wiki/investigations/patch-12-0-0-api-audit.md#current-prose-proof--test-boundaries-and-count-model-gate-2026-10-10).

Actual retry `20261010T163526Z` under the same receipt root records `compile_exit=101`, `source_equal=true`, and `selected_execution_performed=false` in `outcome.json`. `compile.stdout` contains five errors: unresolved `iced::Point`, configured-out `AppearanceSourceInfo`, configured-out PTR helper, unavailable `spell_cast_counts` field, and unavailable `transmog_appearance_sources` field. Current blocker is test feature boundaries plus the actual cast-count model gate, not storage.

Source inspection of `ee21cd9cc` only: tuple coordinates replace `Point`, preserving all game-menu tests in headless builds; the later-only transmog state fixture is gated at `retail-12-0-5`; the earlier encounter-record test removes the inappropriate `retail-12-1-0` PTR helper while retaining its assertions. No post-repair compile or selected runtime proof is claimed: **runtime PENDING**. At this revision `spell_cast_counts` exists in `src/lua_api/state/sim_state.rs:409`, gated by `retail-12-0-5`; it is not absent from the checkout. Production count state/model changes belong to a separate stage and receive no fix or acceptance credit here.
- [ ] General secret operations lack an operation-specific authoritative oracle in the captured prose. VM/dependency changes outside this worktree are unauthorized.

## Out of scope

No vendor overrides, callback security redesign, native secret-operation parity invention or rilua source changes. Existing older-epoch and Classic registration policies remain unchanged.
