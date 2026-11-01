# Retail inspect specialization cache

`C_SpecializationInfo.GetInspectSpecialization` reads one host-supplied inspect response for the requested unit identity.

## What it must do

- [ ] Require a unit argument and return the supplied specialization ID only when the resolved identity matches the cached identity; otherwise return zero.
- [ ] Apply the same cache match to the player as to other units.
- [ ] `NotifyInspect` clears the existing Retail cache before a subsequent response is supplied.
- [ ] Preserve the simulator's identity-restriction output policy.

## Best-guess basis

`SpecializationInfoDocumentation.lua:97–109` declares the unit and numeric result. `Blizzard_PlayerSpells/Blizzard_PlayerSpellsFrame.lua:292` consumes the API. Retail build 70688 static evidence supports the one-cache decision and request clearing; end-to-end numeric publication and runtime response timing remain unobserved.

## How it works

- [Lua API](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/c_spec.rs` reads the inspect cache.
- `src/lua_api/globals/inventory_probes.rs` clears it on `NotifyInspect`.
- `src/lua_api/state/sim_state.rs` holds host-supplied identity and specialization.

## Tests asserting this spec

- `tests/patch_12_1_0_publication_sweep.rs::inspected_spec_cache_is_read_by_player_spells` exercises two matching values, a mismatch and request clearing.

## Known gaps (current cycle)

- [ ] Response arrival and `INSPECT_READY` ordering are not modeled by the supplied cache.
- [ ] Numeric unit conversion, exact argument errors and full secret-value parity require further proof.

## Out of scope

Network inspection and inspection cache expiry are not simulated.
