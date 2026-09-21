# Special action-bar indices

The action-bar index queries in `src/lua_api/globals/action_bar_api.rs` expose configured page indices independently of whether a special bar is active. This supports consumers that build paging conditions before the bar becomes available.

## What it must do

- [ ] `C_ActionBar.GetOverrideBarIndex`, `GetVehicleBarIndex`, and `GetTempShapeshiftBarIndex` return their configured numeric indices with inactive or active bars.
- [ ] Toggling the corresponding `Has*ActionBar` state changes availability predicates without erasing or changing configured indices.
- [ ] Blizzard's legacy wrappers forward the same values as the namespace methods.
- [ ] Ellesmere's inactive-bar paging-condition construction succeeds without nil concatenation.

## How it works

- Existing special-bar state supplies the indices; availability predicates remain separate.
- [Action-bar subsystem](../wiki/index.md) provides broader project context.

## Implementation inventory

- `src/lua_api/globals/action_bar_api.rs` — configured index getters and independent availability predicates.
- `src/lua_api/globals/action_bar_api/registration.rs` — existing namespace registration, unchanged.

## Tests asserting this spec

`tests/c_action_bar_state_globals.rs`, in the existing grouped `integration` target:

- `special_bar_indexes_build_paging_conditions_independently_of_availability`
- `special_bar_indexes_legacy_vendor_wrappers_share_the_numeric_producer`
- Existing active vehicle, override, temporary-shapeshift and bonus-bar tests.

## Evidence

- Forever 1.60.1.69913 `Blizzard_APIDocumentationGenerated/ActionBarFrameDocumentation.lua` declares all three returns as `luaIndex`, `Nilable = false`.
- EllesmereUI v9.2.2, CurseForge file `8936131`, `EllesmereUIActionBars.lua:2448`, constructs an override paging condition during initialization before special-bar activation.
- `Blizzard_DeprecatedActionBar/Deprecated_ActionBar.lua` forwards the legacy calls to `C_ActionBar`.
- Development ledger: `/tmp/ellesmere-forever/special-bar-ledger.json`.

## Known gaps (current cycle)

- [ ] Targeted GREEN and parent-owned final verification pending.

## Out of scope

- Native numeric defaults are not inferred. Tests use explicit configured values, not claimed native defaults.
- Secure paging/snippet enforcement, special-bar activation policy, and other Ellesmere startup failures are unchanged.
