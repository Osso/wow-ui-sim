# Retail 12.0.7 total free bag slots

B03, `global api-C_Container-CalculateTotalNumberOfFreeBagSlots-028` in the [retained 12.0.7 source](../../data/patch-api/sources/12.0.7-api-changes.txt) define this bounded authoring slice. Current generated declarations may postdate 12.0.7; declarations are not native behavior evidence. Bounded tests pass on the default cumulative retail build; strict historical and older-epoch builds were not run. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Return exactly one numeric sum of empty slots from concrete capacity/occupancy inputs, including bag5: fixture totals 4, 6, 7, 3, 0 reflect capacity change, item removal and bag removal live.
- [x] Missing bags and empty metadata contribute zero; out-of-capacity item entries do not consume valid slots.
- [x] **INFERRED existing bounded policy** sum bags0..5 including nonzero family32; ignore bank bag-1 and bag6. No native hidden/family eligibility inference.
- [x] Read-only query retains metadata and observable inventory values; independent environments produce their own totals.
- [x] No declared inputs: **INFERRED** ignore secret/public extras for either caller context without changing taint. This is NOT an AllowedWhenUntainted API declaration.

## How it works

- [Lua API architecture](../lua-api.md)
- [C API signature audit](../c-api-signature-audit.md)

## Implementation inventory

- `src/c_api/item_spell/c_container.rs` — existing total producer, unchanged.
- `src/c_api/bag_info.rs` — existing capacity/occupancy input and zero-for-missing-bag behavior.
- `src/lua_api/state/sim_state.rs` — existing host bag metadata/items.

## Tests asserting this spec

`tests/patch_12_0_7_b01_b04.rs` — ONE module in the auto-generated integration harness; first-line retail-12-0-7 cfg. Tests prefixed B03 assert the bounded values and policies above. Default-build B01/B02/B03 tests passed in this integration round.

## Known gaps (current cycle)

- [ ] Native hidden-bag/family semantics remain unverified; strict older-profile execution was not performed.

## Out of scope

Native bag eligibility, UI-hidden bags, family-specific exclusions and bank aggregation. Existing cross-profile model/defaults are untouched.
