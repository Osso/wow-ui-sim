# C_MythicPlus.GetOwnedKeystoneChallengeMapID

Status: implemented simulator policy, not native parity.
Authority: pinned declaration plus the client-handler decomp answer.

## What it must do

- [ ] `GetOwnedKeystoneChallengeMapID()` publishes one number: the challenge-mode map ID of the key the player owns, or `0` when no key is owned.
- [ ] It publishes a value in every state. It does not return nothing.
- [ ] Publication is Retail, PTR and WowForever only.

## Data source

The client reads a cached client-side keystone context, not the player's bags: it walks a fixed-stride table inside that context and takes the map ID from the first entry whose tag byte marks it as the keystone (decomp answer, 1.60.1). A context without a matching entry publishes `0`.

The simulator supplies that cached state as `MythicPlusState::owned_keystone_challenge_map_id`, next to `owned_keystone_level`. `0` means no key, and the reader publishes it. No bag item is scanned.

## Not proved

- [ ] The null-context arm: the answer covers a context with no matching entry, not an absent context. The simulator has no separate absent state, so the no-key case publishes `0`.
- [ ] The publication bridge itself was only partially admitted in the reconstruction.

`GetOwnedKeystoneLevel` keeps its own no-value contract; this reader does not change it.

## Declarations

`MythicPlusInfoDocumentation.lua`: Retail and PTR `76-84`, WowForever `76-84`, all with `MayReturnNothing`. Mists, Era and Anniversary declare the same member at `75-83` and do not publish it here.

## Implementation inventory

- `src/lua_api/globals/missing_surface/mythic_plus.rs` — `get_owned_keystone_challenge_map_id`.
- `src/lua_api/state_types/mythic_plus_scenario.rs` — `MythicPlusState::owned_keystone_challenge_map_id`.

## Tests asserting this spec

`tests/c_mythic_plus_probes.rs`: zero with one published value without a key, and the supplied map ID next to the supplied level.
