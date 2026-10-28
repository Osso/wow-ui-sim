# Supplied stable pet specialization catalog

Retail/PTR accept supplied `PetSpecInfo` lists for `C_StableInfo.GetAvailablePetSpecInfos`. The model belongs in `src/c_api/c_stable_info/pet_spec_infos.rs`; see the [consumer investigation](../wiki/investigations/consumer-selection-and-red-proof.md).

## What it must do

- [x] Publish the getter and `A_Admin.SetAvailablePetSpecInfos` on Retail/PTR using ACTIVE. Pinned Retail `StableInfoDocumentation.lua:24–31,216–224` and PTR70493 `:28–35,243–251` declare zero arguments and a nonnil array of records with nonnil `specID:number`, `specIndex:luaIndex`, `specializationName:string`.
- [x] Copy complete supplied records into environment-local state, preserve supplied order and return fresh records/arrays. Replacement is atomic. Validate finite specID, positive integral u32 specIndex and UTF-8 name as simulator input policy, not native validation parity.
- [x] Accept explicitly supplied empty lists; nil clears an observation. Unset/cleared queries return an empty list under the [user-approved simulator default](user-approved-native-defaults.md), not a native catalog claim.
- [x] Original selected PTR specialization OnLoad responds to empty/single/multiple supplied catalogs. Original Retail/PTR Refresh projects supplied names, indexes and selected spec IDs across changes and independent environments.
- [ ] Keep Forever and Wrath/Mists/Era/Anniversary publication unchanged. Static ACTIVE gating is present; absence tests are compiled-not-executed in this lane; runtime coverage awaits I1 frozen-SHA.

## How it works

- [Original-consumer selection and proof](../wiki/investigations/consumer-selection-and-red-proof.md).
- [Separate supplied active-pet list](stable-active-list.md).

## Implementation inventory

- `src/c_api/c_stable_info/pet_spec_infos.rs`: supplied record validation, admin input and profile-gated getter.
- `src/c_api/c_stable_info.rs`: stable namespace registration.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: environment-local optional observation.
- `src/lua_api/globals/admin.rs`: simulator input registration.

## Tests asserting this spec

- `tests/pet_spec_infos.rs`: input, original callbacks, copying, atomicity and isolation.
- `tests/pet_spec_infos_absence.rs`: other-profile publication preservation.

## Known gaps (current cycle)

Earlier supplied-state proof covered PTR installed OnLoad1205 and PTR/Retail Refresh1217/1229. The previous unset/clear error policy is superseded by the user-approved empty default. Four supplementary input/copy cases pass. PTR stock122/138 remains122/138 with exactly two attributed nil-member→native-unknown replacements, count1 each, and no surviving count drift. Retail stock remains0/0 with byte-identical raw output. These are supplied observations, not native catalog/default policy.

All seven canonical all-target checks at source-equivalent docs revision3cb0627e2 pass with zero Cargo JSON/plain compiler warnings. Formatting/readability pass; all36 runtime CVar guards and27 pinned input hashes remain unchanged. Other-profile raw absence runtime remains deferred to the I1 frozen-SHA full suite, as noted above.

## Out of scope

Native catalog members, acquisition/filter/order/default policy, native validation, specialization commands/events, secondary pet queries and full StableUI cleanliness are not modeled. Forever70532 declares the getter at `StableInfoDocumentation.lua:28–35,243–251`, but selected Camelot StableUI has no consumer; its implementation remains a declared-but-unconsumed gap. Do not force-load cached Standard UI.
