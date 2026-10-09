# Pet-battle pet type

## What it must do

- `C_PetBattles.GetPetType(owner, petIndex)` reads the selected battle pet's existing `pet_type` state.
- Owner 1 selects player pets; owner 2 selects enemy pets. Slots are one-based.
- Mutations and removal of a pet are visible on the next read.
- Unknown owners and missing/nonpositive slots return nil, matching the existing species-identity lookup boundary.

## Implementation

`src/c_api/c_pet_battles.rs`; no added pet state, synthetic identity or Lua shim.

## Tests

`tests/patch_5_0_4_behavior.rs` checks two sides, changed type, cleared roster, slot zero and unknown owner in bare and cached environments.

## Limits

Fixture-backed read behavior only. No battle engine, species database inference, matchmaking, native invalid-input capture or 2012 parity claim.
