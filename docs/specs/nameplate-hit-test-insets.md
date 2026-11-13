# Nameplate hit-test inset configuration

The Retail `C_NamePlateManager` API stores four hit-test inset values for each `Enum.NamePlateType`. The simulator models this configuration and getter only; it does not model nameplate objects or world hit testing. The lane findings are in the [EllesmereUI startup investigation](../wiki/investigations/full-addon-startup-2026-10-10.md#nameplate-inset-contract-r88a-2026-11-13).

## What it must do

- [ ] Publish `C_NamePlateManager.GetNamePlateHitTestInsets(type)` and `SetNamePlateHitTestInsets(type, left, right, top, bottom)` for Retail. The pinned local API documentation declares Friendly=0 and Enemy=1; its exact client build is UNKNOWN.
- [ ] **VERIFIED (pinned local Retail API documentation; exact client build UNKNOWN):** Return four values in left/right/top/bottom order and keep each type's tuple separate.
- [ ] **VERIFIED (Retail 12.1.5 build 70688):** The setter rejects a missing or nil type. For a non-nil type, failed numeric conversion selects type 0; successful conversion produces a signed 32-bit integer, and the parser accepts low-byte values 0 and 1. Values 256 and 257 therefore select Friendly and Enemy.
- [ ] **VERIFIED (Retail 12.1.5 build 70688):** Each inset must pass the native number check, which accepts numbers and values accepted by the native numeric coercer. The setter converts each value to single precision, multiplies by `1/1024`, then by single-precision `0.8`, and stores the float.
- [ ] **VERIFIED (Retail 12.1.5 build 70688):** A tainted caller is not rejected when no restriction-state pointer exists. The public setter must complete without a Lua error for ordinary inset numbers in this context.
- [ ] **INFERRED (simulator policy):** Each type starts with `(0, 0, 0, 0)`. This is not a native-observed default.
- [ ] Leave `C_NamePlate.GetNamePlateForUnit` nil and `GetNamePlates` empty. Do not create plate frames or clamp offsets against fabricated bounds.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [EllesmereUI startup investigation](../wiki/investigations/full-addon-startup-2026-10-10.md#nameplate-inset-contract-r88a-2026-11-13)

## Implementation inventory

- `src/c_api/c_nameplate_manager.rs`: per-type inset values, setter parsing and namespace registration.
- `src/c_api/mod.rs`, `src/c_api/registration.rs`: feature-gated API registration.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment inset state and inferred zero default.

## Tests asserting this spec

- `tests/nameplate_hit_test_insets.rs::nameplate_hit_test_insets_store_scaled_values_per_type`
- `tests/nameplate_hit_test_insets.rs::nameplate_hit_test_insets_errors_preserve_configuration`
- `tests/nameplate_hit_test_insets.rs::retail_tainted_nameplate_hit_test_insets_succeeds_without_restriction_state`
- `tests/nameplate_hit_test_insets.rs::retail_nameplate_hit_test_insets_matches_native_input_parsing`

## Known gaps (current cycle)

- [ ] **VERIFIED (Retail 12.1.5 build 70688):** The `HasRestrictions` gate blocks only when source metadata and a restriction-state pointer exist and the permission is false; a blocked call reports a blocked action and returns zero Lua values. **UNKNOWN (Retail 12.1.5 build 70688):** The permission's meaning is not established. **UNKNOWN in simulator:** this restriction gate and its state are not modeled.
- [ ] **VERIFIED (Retail 12.1.5 build 70688):** Secret access depends on the argument mark, restriction-state pointer and mode. A clear mark or null pointer allows access; with a mark and pointer, mode 0 denies, mode 1 denies only for nonzero source metadata, and other modes allow. **VERIFIED (Retail 12.1.5 build 70688):** When the native `NotAllowed` gate denies an argument, its exact reason is `Usage: C_NamePlateManager.SetNamePlateHitTestInsets(type, left, right, top, bottom). Secret values are not allowed for this argument.` **UNKNOWN in simulator:** this conditional secret-argument policy is not modeled. Tests must not assert its denial.
- [ ] **UNKNOWN (Retail 12.1.5 build 70688):** The native setter writes four fields before a pending-error check, but the path from that check to normal return is not established. The simulator's normal return count is not a native-parity claim.

## Out of scope

World hit testing, nameplate catalogs and bounds, camera queries, simplified/size setters, and non-Retail publication. The documentation's clamping statement does not provide numeric bounds for a simulator model.
