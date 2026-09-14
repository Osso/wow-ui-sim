# Explicit transmog set filters

`C_TransmogSets.GetSetsFilter` and `SetSetsFilter` retain explicitly written filter values. Pinned 12.0.0 contracts are recorded in `data/patch-api/sources/12.0.0-register.json`; implementation lives in `src/c_api/c_transmog_sets.rs`.

## What it must do

- [x] Retain explicit boolean writes for ordinary filter indices 1–4 independently, including repeated replacement.
- [x] Return one boolean for a stored filter and zero values from the setter.
- [x] Keep distinct environments independent in both directions.

Simulator policy: storage starts empty, and an unset getter returns zero values, consistent with the pinned may-return-nothing contract but not proof of native defaults. Numeric indices and strict boolean values are required. Native coercion, bounds and validation remain unproven; proof is restricted to explicitly written indices 1–4.

## How it works

- [C API audit architecture](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/c_transmog_sets.rs`: merges getter/setter into the existing namespace.
- `src/c_api/mod.rs`, `src/c_api/registration.rs`: publication gated at `retail-12-0-0`.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment filter map and initialization.

## Tests asserting this spec

Three `transmog_sets_filter_*` tests at `4fb364410` reached RED 0/3: after explicit writes, the getter had the wrong type and the setter returned nonzero values. Runtime `5cded1d0b` passes the focused retail 12.0.0 proof 3/3. Development ledger: `/tmp/transmog-sets-filter-development-ledger.json`; GREEN ledger: `/tmp/transmog-sets-filter-green-ledger.json`. Independent verification is pending.

## Known gaps (current cycle)

- [ ] Native default/unset behavior, validation, ranges and coercion.
- [ ] Persistence, events, actual set filtering and loaded consumer execution.

## Out of scope

Default/reset APIs, filtering effects and security remain unchanged. Ordinary stored-state proof does not establish native behavior in those domains.
