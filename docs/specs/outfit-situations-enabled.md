# Global outfit-situations setting

`C_TransmogOutfitInfo` exposes a global boolean setting, not a per-outfit setting. Pinned signatures in `data/patch-api/sources/12.0.0-register.json` declare `GetOutfitSituationsEnabled() -> boolean` and `SetOutfitSituationsEnabled(enabled: boolean) -> no returns`.

## What it must do

- [x] Retain explicit false/true and repeated writes independently in each environment.
- [x] Return one boolean from the getter and zero values from the setter.
- [x] Preserve distinct values across bidirectional writes in two environments.

The simulator initializes the setting to `false`; this is an explicit simulator policy, not a verified native default. The setter requires a boolean; native coercion and exact error behavior remain unproven.

## How it works

- [C API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_transmog_outfit_info.rs`: boolean getter/setter merged into the existing namespace.
- `src/c_api/mod.rs`, `src/c_api/registration.rs`: registration gated at `retail-12-0-0`.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: environment-owned boolean and initial policy.

Existing lock functions and unrelated namespace defaults are unchanged.

## Tests asserting this spec

`tests/c_namespace_noop_replacements.rs` contains three `outfit_situations_enabled_*` tests committed at `14ac47769`. RED was 0/3: getter had the wrong type after an explicit write and setter returned nonzero values; later repeated-write/arity/isolation assertions were blocked. Runtime `457a8ae88` reached focused retail 12.0.0 GREEN 3/3; `/tmp/outfit-situations-enabled-green-ledger.json` retains the command, outputs and exit 0.

## Known gaps (current cycle)

- [x] Focused development GREEN after the committed runtime change.
- [ ] Independent profile and runtime verification, outside this implementation slice.

## Out of scope

Per-outfit state, pending situations, reset, persistence, events, UI behavior, native semantics and security are excluded from this bounded setting model. No consumer execution or audit credit is claimed.
