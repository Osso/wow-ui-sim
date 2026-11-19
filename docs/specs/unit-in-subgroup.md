# `UnitInSubgroup`

`UnitInSubgroup` classifies a unit against the selected HOME or INSTANCE owner roster. This contract is based on Retail evidence in `pr46-49-decomp-answers.md` §R88bb.

## What it must do

- [x] Return one Boolean after checking no more than four identities in the selected subgroup sequence. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [x] Select HOME for index 1 and INSTANCE for index 2; with no index, prefer INSTANCE when present and otherwise select HOME without retry. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [x] Compare full identities and skip empty roster slots. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [x] Accept a string or number unit argument and apply the documented index conversion and error behavior. (VERIFIED, `pr46-49-decomp-answers.md` §R88bb)
- [x] Publish on Retail, PTR, Mists, Era, Anniversary, and Forever. (INFERRED (consumer-required publication; API documentation lists it); cached Mainline consumers in PTR/Forever and Classic consumers in Mists/Era/Anniversary.)
- [x] Represent HOME and INSTANCE rosters as explicit host inputs, empty by default. (INFERRED: simulator mapping required because the existing roster model has no owner/subgroup inputs; `pr46-49-decomp-answers.md` §R88bb)
- [x] Apply Retail 70688 owner/slot semantics to Forever and Mists. (INFERRED (R88bb Retail 70688 semantics applied to Forever/Mists).)
- [x] Apply the same semantics to PTR, Era, and Anniversary. (INFERRED: cached API references support publication, but do not verify native semantic parity.)

## How it works

- [Unit and group query system](../wiki/systems/unit-and-group-queries.md)

## Implementation inventory

- `src/lua_api/game_data.rs` — host-supplied owner roster records.
- `src/lua_api/state/sim_state.rs` — HOME and INSTANCE roster inputs.
- `src/lua_api/globals/group_queries_relationships.rs` — API behavior.
- `src/lua_api/globals/group_queries.rs` — publication for profiles with cached API references.
- `src/lua_api/workarounds/temporary/secure_legacy_unit_globals.rs` — profile registration metadata.
- `src/lua_api/workarounds/temporary/inert_global_defaults.rs` — removes the guessed fallback and checks profile publication.

## Tests asserting this spec

- `tests/unit_in_subgroup.rs` — original Retail and Forever unit-position consumers and API contract.

## Known gaps (current cycle)

None.

## Out of scope

- Native behavior is VERIFIED only for Retail; non-Retail semantic parity remains INFERRED.
