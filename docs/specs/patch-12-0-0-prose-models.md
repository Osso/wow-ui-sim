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

Current native/cached registration and retirement proof passes; exact-12.0.0 execution remains blocked by the master-reproduced `on_update.rs:61` / `private_aura_sounds` feature-gate error. See the per-source report for revision scope.
- [ ] General secret operations lack an operation-specific authoritative oracle in the captured prose. VM/dependency changes outside this worktree are unauthorized.

## Out of scope

No vendor overrides, callback security redesign, native secret-operation parity invention or rilua source changes. Existing older-epoch and Classic registration policies remain unchanged.
