# Currency list field projection

Expose existing currency-list backing fields through `C_CurrencyInfo.GetCurrencyListInfo` and the Mists legacy tuple. Existing listed currencies project supplied `SimState.currency_info` display values; list structure remains static outside the separate [Retail displayed-index contract](currency-displayed-indices.md). Source lives in `src/c_api/item_spell/c_currency.rs` and `src/mists/compat_bootstrap.lua`.

## What it must do

- [x] Return non-nil boolean `isShowInBackpack`, initially from `CurrencyEntry.is_show_in_backpack`, preserving true and false. Explicit [Retail watched mutations](currency-watched-mutations.md) own subsequent overrides.
- [x] Return non-nil number `maxQuantity`, preserving zero and positive caps. Static header values come from `CurrencyEntry`; ordinary rows use supplied-state values.
- [x] Forward those fields directly through Mists `GetCurrencyListInfo` tuple positions 5 and 8.
- [ ] Preserve all other fields, unsupported tuple slots, catalog data, defaults, and registrations.

### Supplied-state display projection

- [x] For existing non-header list rows, project current name, quantity, icon, quality and maximum quantity from `SimState.currency_info` selected by the static currency ID.
- [x] Retail list rows publish numeric `currencyID` from the list entry, including the modeled header ID; `CurrencyInfoDocumentation.lua:681-689` declares it non-nil.
- [x] For existing watched rows, backpack info projects current name, quantity and icon from that same supplied row.
- [x] Observe successive supplied-state updates without changing list size/order, header records, currency IDs, static status flags or watched membership.
- [x] Do not resurrect static display values when a listed currency is removed from supplied state: list info returns nil and backpack info returns no values. Headers do not require a supplied map row.
- [x] Preserve invalid-index behavior and existing Mists supported tuple slots/arity. This is simulator policy, not a claim about native update timing.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/lua_api/globals/currency_data.rs` — existing `CurrencyEntry` fields and static list; unchanged.
- `src/c_api/item_spell/c_currency.rs` — namespaced list-info projection. Retail additionally publishes `currencyListDepth` through the displayed-index contract and `isTypeUnused`/`discovered` through the [unused-currency contract](currency-unused-mutations.md); non-Retail retains its existing field set.
- `src/mists/compat_bootstrap.lua` — existing Mists legacy tuple adapter.
- `tests/mists_currency_list.rs` — existing Mists namespace and legacy-tuple controls.
- `tests/currency_info_probes.rs` — shared supplied-state display and missing-row regressions.

## Tests asserting this spec

Two existing static-field tests in `tests/mists_currency_list.rs` (historical bounded GREEN, not supplied-state mutation proof):

- `currency_list_info_preserves_watched_flags_in_namespace_and_legacy_tuple`
- `currency_list_info_preserves_max_quantity_in_namespace_and_legacy_tuple`

Saved static Mists audit `20261010T051236Z`, compiled revision `a62f7a95ba22857ec48cd9e115fc15eb77df12e6`: five tests PASS, zero FAIL, exit 0. Field assertions prove strict boolean true/false at namespace `isShowInBackpack` and legacy slot 5, and strict numeric cap 90 at namespace `maxQuantity` and legacy slot 8; they do not prove zero-cap or mutation behavior. The other three tests cover numeric singleton list size, populated real TokenUI update, and injected missing-size failure after dependency loading. Warnings and loader diagnostics remain; no native, catalog, full-profile or full-suite acceptance.

Historical RED `0148437a7868b0a25d0ce2fca504378d4558b5ee` (`20261010T043019Z`) and supplemental RED `5184a8cf8feea9a820a15264604a6af303bb5a9d` (`20261010T045459Z`) each recorded one PASS / four FAIL / exit 101. Their field failures establish nil/false versus true/true and nil/nil versus 90/90. Earlier TokenUI fixture failures occurred before the intended size lookup, not successful missing-size reproductions. Private receipt: `/home/osso/.local/state/wow-ui-sim/verification/mists-currency-green-current/20261010T051236Z/audit/report.md`.

Four supplied-state regressions in `tests/currency_info_probes.rs`, introduced at `085f7d799` and decoder-corrected at `f4a6fb2e4`:

- `get_currency_list_info_reflects_successive_existing_currency_updates`
- `get_backpack_currency_info_reflects_successive_existing_currency_updates`
- `get_currency_list_info_returns_nil_when_existing_currency_row_is_removed`
- `get_backpack_currency_info_returns_nothing_when_existing_currency_row_is_removed`

Mists wrapper preservation probe in `tests/mists_currency_list.rs`:

- `legacy_currency_display_updates_preserve_static_rows_and_backpack_membership` — successive positive/zero display updates, exact legacy tuple arity/supported slots and static watched membership despite a conflicting supplied backpack flag; PASS in the bounded Mists capture.

## Known gaps (current cycle)

[Bounded proof packet](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/currency-supplied-state-2026-10-11/README.md): authentic default RED12PASS/fournewFAIL at `f4a6fb2e4`; default16/16 and Mists20/20 GREEN at `1dcc59c07`, independent gates, fmt/check0 and cached default stock `[]`/exit0. Initial unsupported test-decoder compile failure has zero executions. No broad test rerun supplies these targeted results.

- [ ] Exhaustive catalog/default/order acceptance and native parity remain unproved. Mutated Mists unsupported slots9/10 are not explicitly asserted; wrapper source is unchanged and header/unwatched baseline comparisons cover all12 slots.
[Seven-profile bounded proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/currency-seven-profile-2026-10-11/README.md): Retail16/Mists20/PTR16/Wrath16/Era16/Anniversary16/Forever16 independently accepted;116 GREEN executions across seven configurations,20 distinct named tests. Shared supplied-state proof, not native-profile catalogs.

- [x] Remaining Era/Anniversary/Forever probes: retry2 epoch `20261011T082636Z` records16/16 PASS each, independently accepted. [Earlier profile follow-up](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/currency-profile-followup-2026-10-11/README.md) preserves two zero-case Era resource blocks and initially unattempted Anniversary/Forever. Blocked vendor-input lanes are not reopened.
- [x] Reconcile completed `1dcc59c07` [saved full suite](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/currency-fullsuite-2026-10-11/README.md): 15,028 PASS/1 Garrison FAIL/19 skips/15,029 executions. Four new currency regressions PASS inside totals; source equality and accounting independently audited. Existing c75 FAIL remains history.
- [ ] Full-suite acceptance remains open: Garrison FAIL, skipped/native/other-profile behavior and external acceptance are not discharged by bounded currency proof.

## Out of scope

Native Mists catalog and comprehensive currency parity are not claimed. No catalog/default/registration changes, alternate paths, new fallbacks, vendor edits, or adjacent serializer refactors.
