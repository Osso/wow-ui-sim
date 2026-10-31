# Supplied stable pet slots

Retail/PTR slot queries use explicit environment-local observations. The [contract](../../specs/stable-pet-info.md) defines the supported boundary. Forever has a separate [build 70707 stable-read contract](../../specs/forever-stable-read-state.md): flagged-record counts, a raw slot byte, supplied cost rows, and non-nil empty pet tables. Its empty-table fields and refresh-event relationship remain unknown.

## State and projection

`SimState.stable_pet_slots` maps a positive integral slot to `Option<PetInfo>`. An absent key means unknown, a present `None` means explicitly empty, and a present record contains complete supplied fields. `A_Admin.SetStablePetInfo` validates before replacing one observation; `ClearStablePetInfo` removes it. Neither reads active-list membership.

`c_stable_info/pet_info.rs` gates registration with `client_profile::ACTIVE`. It reuses `active_list.rs` decoding and projection: input tables and ability arrays are copied, and every query allocates a fresh record and arrays. Empty slots return exactly one nil. Unknown slots raise a named `native-unknown: C_StableInfo.GetStablePetInfo` error containing the slot. This is an explicit simulator uncertainty policy, not a claim about native errors.

## Original consumer boundary

The selected Retail root and PTR Standard StableUI create the original hidden `StableFrame`. Primary tests call its original `RefreshSelectedPetData`; secondary tests observe original `ActivePetList.Refresh` after secondary-button `SetPet`. These are bounded interactions, not full stable-window or 3D acceptance. Retained setup diagnostics remain visible.

## Verification — 2026-10-26

Merged GREEN6f1f29349 / RED2c66b42ec use byte-identical lane fixtures. Each profile has three exact original-consumer RED failures at the intended missing-query boundaries and seven exact GREEN passes. The primary selection routes cover varied slots, complete records, updates, explicit empty, unset, clearing, recovery and environment isolation. The secondary route proves original SetPet consumption only; both existing active-list fixtures also pass.

| Profile | Raw stock RED → GREEN | Per-key delta |
| --- | --- | --- |
| Retail | 0/0 → 0/0 | 0 |
| PTR | 122/138 → 122/138 | 0 |

All seven full-feature all-target checks pass with zero Cargo JSON/plain compiler warnings. The approved Mists-only fixture repair is3af7fab2a; it changes no production source and leaves Retail/PTR runtime evidence applicable. Formatting/readability pass. All100 guarded launches (44 merged) preserve the original CVar file stat/hash. Pinned declarations and selected consumer hashes are unchanged.

Outside Retail/PTR, this lane provides source gating and compilation proof, not newly executed raw-publication/runtime acceptance. That remains with I1. PTR setup diagnostics and stock errors are retained; this is not a claim of complete StableUI cleanliness or native ownership/default policy.

## Sources

- [Contract](../../specs/stable-pet-info.md) and `tests/stable_pet_info.rs`.
- Pinned `Blizzard_APIDocumentationGenerated/StableInfoDocumentation.lua`: Retail 70448 function 66–79, record 193–214; PTR 70493 function 88–101, record 215–241. All fields are supplied, including uiModelSceneID.
- [Consumer selection](../investigations/consumer-selection-and-red-proof.md).

## See Also

- [Active-list contract](../../specs/stable-active-list.md) — independent observations, not inferred slot mappings.
