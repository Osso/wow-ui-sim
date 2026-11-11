# Covenant anima info

`C_CovenantSanctumUI.GetAnimaInfo()` reports the anima currency ID and reservoir limit. This spec covers the Retail no-currency-row fallback only. See [Covenant state model](../wiki/systems/covenant-state.md) for implementation details.

## What it must do

- [ ] When the local currency list has no entry for anima currency 1813, return exactly two values: `1813, 0`.
- [ ] Preserve the existing non-Retail simulator scenario. This is not a native-parity claim.

## How it works

- [C API boundary](../wiki/index.md)
- [Covenant state model](../wiki/systems/covenant-state.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/campaign_covenant_defaults.rs` — profile-specific no-data fallback.
- `tests/c_campaign_covenant_defaults_probes.rs` — Retail no-data result and non-Retail scenario coverage.

## Tests asserting this spec

- `tests/c_campaign_covenant_defaults_probes.rs::retail_missing_anima_record_returns_currency_id_and_zero_limit`
- `tests/c_campaign_covenant_defaults_probes.rs::non_retail_covenant_sanctum_scenario_keeps_previous_reward`

## Known gaps (current cycle)

- [ ] The supplied state does not model an active currency row, override reservoir limit, effective-limit calculation, bag anima, or contained items.
- [ ] `CanDepositAnima` remains false without a player/input model. `DepositAnima` does not submit native interaction work.
- [ ] The selected UI files are admitted through the Covenant Sanctum load-on-demand TOC, but runtime loading of that addon is not proved here.
- [ ] The Retail API test is not rerun; this result remains unverified.

## Out of scope

- Deposit eligibility with a populated player inventory, action submission, server response, events, and parity beyond the inspected Retail 12.1.5 build.
