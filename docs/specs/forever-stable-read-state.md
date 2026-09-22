# Forever stable read state

`src/c_api/c_stable_info/forever.rs` supplies the four state-backed reads used by cached Camelot StableUI's money-event update. See [C API architecture](../lua-api.md). Cached `Blizzard_APIDocumentationGenerated/StableInfoDocumentation.lua` defines return types and PetInfo fields; `Blizzard_StableUI/Camelot/Blizzard_StableUI.lua` defines the consumed one-based slot layout.

## What it must do

- [x] Forever exposes numeric `GetNumStableSlots`, `GetNextStableSlotCost`, and `GetNumStablePets`; the first two read configured state, and pet count derives from stored records.
- [x] Explicit simulator scenario **guess**, not native evidence: two owned stable slots, empty pet storage, next purchase unavailable with cost zero. Ownership is not deduced from the capacity constant.
- [x] `GetStablePetInfo(index)` uses public one-based slots (current pet 1, stable slots 2 and 3), returns nil for absent slots, and returns every documented PetInfo field for configured pets. `slotID` matches the storage key; ability arrays use Lua indices.
- [x] Returned tables are snapshots: edits do not mutate stored pets. Separate environments have independent slot, cost, and pet state.
- [x] Inferred count policy includes all stored pets, including current slot 1. Missing/wrong-type required index arguments fail through the existing argument conversion; absent nonpositive/out-of-range slots return nil. Native coercion details remain unproven.
- [x] Existing `IsAtPetStable` remains tied to `pet_stables_open`.
- [ ] Non-Forever profiles retain their existing surface (profile-gated source; independent profile verification pending).
- [x] Actual cached Camelot `PLAYER_MONEY` dispatch completes with zero Lua errors, hides the purchase button in the fully unlocked scenario, and populates all three empty-slot tooltips.

## How it works

- [Lua API state and dispatch](../lua-api.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/c_api/c_stable_info.rs` — existing open probe and profile-gated registration.
- `src/c_api/c_stable_info/forever.rs` — read state, documented pet records, and four Lua query producers.
- `src/lua_api/state/sim_state.rs` — per-environment state field.
- `src/lua_api/state.rs` — explicit default scenario initialization.

## Tests asserting this spec

- `tests/wowforever_stable_reads.rs` — grouped integration tests for defaults/restoration, configured fields and snapshot isolation, absent slots, derived count/environment isolation/open probe, and actual simulator cached-UI money-event execution.
- Development proof: 4/4 pass at `a6fbf432e`; two identical Lua regression bodies failed against frozen pre-model `fffb25ae`. Build `gui,client-wowforever` completed with unchanged source hashes and no warnings; provenance and frozen simulator recorded in `/tmp/forever-addon-audit/stable-read-development-ledger.json`. This is targeted development proof, not final acceptance.

## Known gaps (current cycle)

- [ ] Parent-owned independent verification and unchanged Aurarium replay.

## Out of scope

- Purchase, swapping, favorites mutation, food queries, and gameplay/event producers: not required by this bounded read path.
- Native Forever conformance: native probes unavailable; defaults and total-count interpretation are explicit simulator guesses.
- Persistence and other-profile stable models: no changes requested.
