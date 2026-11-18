# Profession specialization path state

`C_ProfSpecs.GetStateForPath` and the path entry/currency lookups provide the state consumed by the Retail profession specialization tree. Native behavior is documented in `pr46-49-decomp-answers.md` §§R88ax.1 and R88ax.3.

## What it must do

- [ ] Register the getters in the Game screen on Retail 12.1.5 only.
- [ ] Require signed 32-bit path IDs, and also a config ID for `GetStateForPath`; accept numeric strings and fractions after range validation, and raise the documented Usage errors for invalid arguments.
- [ ] Return exactly one numeric state from `Enum.ProfessionsSpecPathState`: Locked 0, Progressing 1, or Completed 2. Missing path/config and rank below the required rank return Locked; otherwise equality with effective maximum returns Completed, and other ranks return Progressing. (R88ax.1, VERIFIED.)
- [ ] Return the selected type-7 spend entry, type-9 unlock entry, or first qualifying type-1 spend currency. Unknown paths return 0 for entry getters and nil for currency. (R88ax.3, VERIFIED.)
- [ ] **INFERRED (simulator mapping):** use empty-default host inputs for path entry ranks, per-config current/effective maximum ranks, entry IDs, spend currency IDs, and trait configuration IDs. Do not synthesize DB2 or runtime rows.

## How it works

- See [profession specialization path children](profession-spec-path-children.md) for ordered path-tree inputs.
- See [profession specialization reminders](profession-spec-reminders.md) for related `C_ProfSpecs` inputs.

## Implementation inventory

- `src/c_api/profession_spec_inputs.rs` — host-supplied path and configuration inputs.
- `src/c_api/c_prof_specs_path_queries.rs` — Retail path state and entry/currency queries.

## Tests asserting this spec

- `tests/c_prof_specs_get_state_for_path.rs` — state results and argument boundary.
- `tests/test_showuipanel_lod.rs` — original Retail specialization-tree consumer.

## Known gaps (current cycle)

- [ ] Trait-condition rows and their runtime evaluation are UNKNOWN (R88ax.1); keep them unsupplied by default. Supply effective maximum rank directly instead of inventing condition behavior.
- [ ] Perk getters remain outside this spec until their decomp evidence is filed.

## Out of scope

- Loading profession specialization DB2 or runtime trait configuration data.
- Other client profiles and Glue registration.
