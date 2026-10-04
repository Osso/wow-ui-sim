# Tiered entrance PDEID — row260

`C_DelvesUI.GetTieredEntrancePDEID()` reads an explicit per-environment host scalar. The [12.0.5 delta](../../data/patch-api/sources/12.0.5-api-changes.txt), row260, removes `MayReturnNothing`; the cached retail `DelvesUIDocumentation.lua:315–320` declares no arguments and one nonnil numeric `pdeID`. This supplements [Delves API inputs](delves-api-inputs.md); declarations are not native execution evidence.

## What it must do

- [ ] Read `SimState.tiered_entrance_pde_id` live and return exactly one public number, including when the stored value is zero.
- [ ] INFERRED empty-state policy: initialize the scalar to zero. Zero is not a native-verified sentinel and does not trigger a fallback or missing result.
- [ ] INFERRED storage domain: accept host `u32` values unchanged, including `u32::MAX`; all values convert exactly to a Lua number.
- [ ] Queries must not mutate the scalar; updates in one environment must not affect another.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/state/sim_state.rs` — public host scalar.
- `src/lua_api/state.rs` — explicit zero initialization.
- `src/lua_api/globals/missing_surface/delves_ui.rs` — existing registered producer, now reads state; registration ownership is unchanged.

## Tests asserting this spec

`tests/pdeid_specialbar.rs`: `pdeid_default_is_exactly_one_public_numeric_zero`, `pdeid_reads_live_scalar_including_zero_and_storage_boundary`, `pdeid_updates_are_environment_local`. Authored only; not executed.

## Known gaps (current cycle)

- [ ] Compile and run the authored behavioral tests; no RED/GREEN execution is claimed.
- [ ] Cached difficulty-picker callers use the PDEID as a CVar-table key. Source inspection supports numeric zero; loaded UI and CVar serialization behavior are not runtime-verified by this authoring task.

## Out of scope

Native entrance-ID acquisition, native empty-state meaning, other Delves API changes, vendor edits and audit-row closure. No native/all-profile parity claim. No compatibility fallback.
