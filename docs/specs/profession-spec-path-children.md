# Profession specialization path children

`C_ProfSpecs.GetChildrenForPath` returns the ordered child path IDs used by the Retail profession specialization tree. Its contract follows `pr46-49-decomp-answers.md` §R88as.

## What it must do

- [ ] Register the query in Game only on Retail 12.1.5.
- [ ] Require one int32 path ID. Accept numbers and ordinary numeric strings, check the signed int32 range before truncating toward zero, and raise the documented `Usage:` argument error for invalid input.
- [ ] Return exactly one table. Preserve the supplied child order; return an empty table for an unknown path or a leaf.
- [ ] **INFERRED (simulator mapping):** use empty-default host inputs for path ID to group ID and group ID to ordered child node IDs.

## How it works

- See [profession specialization reminders](profession-spec-reminders.md) for related `C_ProfSpecs` inputs.

## Implementation inventory

- `src/c_api/profession_spec_inputs.rs` — supplied path and group mapping inputs.
- `src/c_api/c_prof_specs_get_children_for_path.rs` — API registration and query behavior.

## Tests asserting this spec

- `tests/prof_specs_get_children_for_path.rs` — conversion, error, return-shape, empty-case, and ordering contract.
- `tests/test_showuipanel_lod.rs` — original Retail Blizzard specialization-tree consumer.

## Known gaps (current cycle)

- [ ] The host must supply path and group mappings; native DB2 tables are not loaded here.

## Out of scope

- Loading native `ProfTraitPathNode` or `TraitNodeGroupXTraitNode` data.
- Other client profiles and Glue registration.
