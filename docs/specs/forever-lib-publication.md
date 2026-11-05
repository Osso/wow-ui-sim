# Forever 70707 library publication

The Forever 1.60.1.70707 generated API documentation declares library endpoints
that must read supplied simulator state, rather than disappear behind Retail
version gates. Runtime loading uses the profile-pinned documentation described
in [client profiles](../wiki/systems/client-profiles.md).

## What it must do

- [x] Publish `C_MerchantFrame.GetItemInfo(index)` against the existing merchant
  inventory model (`MerchantFrameDocumentation.lua:27–39`). Empty inventory
  returns no item. Existing modeled free, unlimited single-item offers remain
  simulator policy, not native price/stock evidence.
- [x] Publish `C_TransmogSets.GetSetInfo` and `HasAvailableSets` against the
  existing supplied set catalog (`TransmogSetsDocumentation.lua:349–356` for
  availability). Availability retains the existing inferred character-valid,
  hidden-until-collected policy; no native eligibility is derived here.
- [x] Publish `C_CooldownViewer.GetGroupBuffItems` as fresh ordered snapshots of
  supplied records with exactly `spellID`, `displaySpellID`, `name`, `iconID`,
  `flags`, and `isKnown` (`CooldownViewerDocumentation.lua:43–50,149–159`).
  **EMULATED:** `cooldown_viewer_group_buffs` starts empty until supplied. No
  native empty-catalog claim, rank selection or spell-data derivation is made.
- [x] Preserve the [Forever outfit lookup contract](outfit-catalog-lookups.md#forever-70707-singular-lookup)
  and [fifteen-bit forbidden enum](forever-forbidden-aspects.md).

## How it works

- [C API boundary and Lua environment](../lua-api.md)
- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_merchant_frame.rs`: existing merchant getter publication.
- `src/c_api/c_transmog_sets.rs`: existing set-model query publication.
- `src/c_api/c_cooldown_viewer.rs`: six-field supplied group-buff DTO and getter.
- `src/lua_api/state{,/sim_state}.rs`: per-environment group-buff input list.

## Tests asserting this spec

`src/c_api/forever_lib_baseline_tests.rs` exercises two distinct records and
mutations through each endpoint, including fresh group-buff/outfit snapshots.
Existing original library tests remain enabled without weaker assertions.

## Known gaps (current cycle)

- [ ] Native group-buff catalog population, ordering and rank derivation.
- [ ] Native merchant pricing/stock and transmog availability policies.

## Out of scope

New market/catalog inputs, outfit lifecycle, texture/animation restriction
enforcement, and complete startup/native parity are separate contracts.
