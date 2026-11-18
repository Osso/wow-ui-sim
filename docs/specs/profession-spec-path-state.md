# Profession specialization path state

`C_ProfSpecs.GetStateForPath` and the path entry/currency lookups provide the state consumed by the Retail profession specialization tree. Native behavior is documented in `pr46-49-decomp-answers.md` §§R88ax.1, R88ax.2, and R88ax.3.

## What it must do

- [ ] Register the getters in the Game screen on Retail 12.1.5 only.
- [x] `GetStateForPath` requires signed 32-bit path and config IDs; numeric strings are accepted, fractional values truncate after range validation, and invalid inputs raise the documented Usage and range errors. (R88ax.1, tests assert.)
- [x] Return exactly one numeric state from `Enum.ProfessionsSpecPathState`: Locked 0, Progressing 1, or Completed 2. Missing path/config and rank below the required rank return Locked; otherwise equality with effective maximum returns Completed, and other ranks return Progressing. (R88ax.1, VERIFIED.)
- [x] Return the selected type-7 spend entry, type-9 unlock entry, or first qualifying type-1 spend currency. Unknown paths return 0 for entry getters and nil for currency. (R88ax.3, VERIFIED.)
- [x] `GetPerksForPath` returns ordered perk records; `isMajorPerk` tests the `IsMajorBonus` flag (2). `GetUnlockRankForPerk` returns nil on a miss; `GetEntryIDForPerk` returns 0 on a miss. (R88ax.2, VERIFIED.)
- [x] `GetStateForPerk` returns one `Enum.ProfessionsSpecPerkState` number: Unearned 0, Pending 1, Earned 2. (R88ax.2, VERIFIED.)
- [x] **INFERRED (simulator mapping):** use empty-default host inputs for path entry ranks, per-config current/effective maximum ranks, entry IDs, spend currency IDs, perk groups/flags/entries/ranks, and trait configuration IDs. Do not synthesize DB2 or runtime rows.

## How it works

- See [profession specialization path children](profession-spec-path-children.md) for ordered path-tree inputs.
- See [profession specialization reminders](profession-spec-reminders.md) for related `C_ProfSpecs` inputs.

## Implementation inventory

- `src/c_api/profession_spec_inputs.rs` — host-supplied path and configuration inputs.
- `src/c_api/c_prof_specs_path_queries.rs` — Retail path state and entry/currency queries.

## Tests asserting this spec

- `tests/c_prof_specs_get_state_for_path.rs` — state results and argument boundary.
- `tests/c_prof_specs_perks.rs` — perk list, rank, entry, state, and argument contracts.
- `tests/test_showuipanel_lod.rs` — original Retail specialization-tree consumer.

## Known gaps (current cycle)

- [ ] Trait-condition rows and their runtime evaluation are UNKNOWN (R88ax.1); keep them unsupplied by default. Supply effective maximum rank directly instead of inventing condition behavior.
- [ ] `GetDescriptionForPerk` is only in scope if the consumer test reaches it; its formatter dependencies remain UNKNOWN (R88ax.2).

## Out of scope

- Loading profession specialization DB2 or runtime trait configuration data.
- Other client profiles and Glue registration.
