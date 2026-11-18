# `UnitInSubgroup`

`UnitInSubgroup` classifies a unit against the selected HOME or INSTANCE owner roster. This contract is based on Retail evidence in `pr46-49-decomp-answers.md` §R88bb.

## What it must do

- [ ] Return one Boolean after checking no more than four identities in the selected subgroup sequence. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [ ] Select HOME for index 1 and INSTANCE for index 2; with no index, prefer INSTANCE when present and otherwise select HOME without retry. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [ ] Compare full identities and skip empty roster slots. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [ ] Accept a string or number unit argument and apply the documented index conversion and error behavior. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [ ] Publish the global on Retail. (INFERRED: consumer-required publication; `pr46-49-decomp-answers.md` §R88bb)
- [ ] Represent HOME and INSTANCE rosters as explicit host inputs, empty by default. (INFERRED: simulator mapping required because the existing roster model has no owner/subgroup inputs; `pr46-49-decomp-answers.md` §R88bb)

## How it works

- [Unit and group query system](../wiki/systems/unit-and-group-queries.md)

## Implementation inventory

- `src/lua_api/game_data.rs` — host-supplied owner roster records.
- `src/lua_api/state/sim_state.rs` — HOME and INSTANCE roster inputs.
- `src/lua_api/globals/group_queries_relationships.rs` — API behavior.

## Tests asserting this spec

- `tests/unit_in_subgroup.rs` — original Retail unit-position consumer and API contract.

## Known gaps (current cycle)

- [ ] Implement the consumer-backed API and test its native selection and identity behavior.

## Out of scope

- Native behavior on profiles other than Retail; the available evidence covers Retail 12.1.5 only.
