# Forever expansion identity

The simulator's Forever profile exposes a Classic current-expansion identity through `src/client_profile.rs`. This is an **inferred addon-compatibility policy**, not native Forever conformance: cached Angleur `8932166` explicitly selects its Camelot branch when `LE_EXPANSION_LEVEL_CURRENT == LE_EXPANSION_CLASSIC` under `WOW_PROJECT_MAINLINE`.

## What it must do

- [x] Forever publishes `LE_EXPANSION_LEVEL_CURRENT = 0` and `GetExpansionLevel() = 0` from one profile policy.
- [x] The unchanged Angleur version predicate selects Camelot (`4`), not retail (`1`).
- [x] Clearing and restoring the current-expansion constant after environment cleanup preserves that identity.
- [x] Other profiles retain their existing current constant (`11`) and legacy API value (`10`). This preservation does not claim those defaults are native-correct.
- [x] Existing previous/max/upgrade/account/client-display expansion values remain untouched.

## How it works

- [Client profiles](../wiki/systems/client-profiles.md)
- [Addon loading](../addon-loading-pipeline.md)

## Implementation inventory

- `src/client_profile.rs` — explicit Forever current identity and legacy API profile selection.
- `src/lua_api/globals/strings/string_data/core_strings.rs` — profile-selected constant, shared by initial registration and missing-global restoration.
- `src/lua_api/globals/enum_data/missing_constants.lua` — no second hardcoded current-expansion producer.
- `src/lua_api/workarounds/temporary/client_info_defaults.rs` — existing legacy API bootstrap substitutes the profile value; unrelated defaults remain temporary.

## Tests asserting this spec

- `tests/wowforever_profile.rs` — exact Angleur branch, restoration, other-profile policy values and non-Forever runtime preservation; existing grouped integration target.
- `src/loader/tests/wow_api.rs` — profile-aware current-expansion snapshot.
- `src/lua_api/workarounds/temporary/client_info_defaults.rs` — legacy API bootstrap result.

## Known gaps (current cycle)

- Focused Forever proof at revision `9e20a29d`: `expansion_identity` 2/2. The former `55ee20d7` RED reproduces `11`, API `10`, and retail branch `1`: `/tmp/forever-addon-audit/expansion-identity-red-e5_paz46/ledger.json`.
- [ ] Replay unchanged Angleur/Camelot startup beyond its corrected branch. The `b8f0982be` replay reaches a later unmodeled gamepad call, so it remains startup-failed: `/tmp/forever-addon-runtime/producer-b8-startup-ledger.json`.
- [ ] Native Forever numeric identity is unverified; user cannot run a native probe.
- [ ] Other Forever addons affected by expansion thresholds need separate replay; this patch is not inventory-wide acceptance.

## Out of scope

- Previous/max expansion values, account ownership/trial state, level caps, and other profiles' expansion corrections: no evidence or authorization for changing them in this slice.
- Vendor/addon edits or choosing a retail TOC instead of Camelot: incorrect fix layer.
