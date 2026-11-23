# Currency transfer activity query

`C_CurrencyInfo.IsCurrencyTransferInProgress()` returns explicit environment-local transfer activity. Cached API metadata specifies one boolean; the cached transfer menu consumes it separately from request/readiness/transaction APIs. [C API architecture](../lua-api.md) describes ownership.

## What it must do

- [ ] Return exactly one non-nil boolean, modeled false when unsupplied.
- [ ] Project supplied false/true/false unchanged without modifying input or inferring activity from currency metadata/readiness.
- [ ] Read live state through retained function references; isolate existing and new environments.
- [ ] Drive unchanged cached immediate spinner/content methods with real frame state and the actual query, without mocking request/lifecycle APIs.

## How it works

- [Lua API and simulator state](../lua-api.md).
- [Client profiles](../wiki/systems/client-profiles.md).

## Implementation inventory

- `src/lua_api/state/sim_state.rs`: environment-local `bool` input for the C API query; not a separate transfer lifecycle model.
- `src/lua_api/state.rs`: false default.
- `src/c_api/item_spell/c_currency.rs`: existing namespace owner; live boolean query registered under `retail-12-0-0`.

## Tests asserting this spec

- `tests/currency_transfer_activity.rs`: three default/live/isolation and bounded cached-consumer cases.

## Known gaps (current cycle)

- [x] Execute genuine RED at `ff72f3c3b`: three failures reach the boolean contract, live true-state comparison and actual cached spinner-state assertion; no observed fixture failures.
- [x] Executed scoped Retail producer GREEN: 3/3 PASS at `6b522c595`; [investigation SSOT](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#executed-staged-queries-and-new-producers).
- [ ] Broader runtime/profile and default-check acceptance remain open; startup still fails with 12 SMOOTH setup errors, not clean.

## Out of scope

Transfer requests, transaction history, timers, completion/failure events and persistence remain out of scope. Account roster, balances and transfer costs are covered by the [account currency transfer contract](currency-transfer-eligibility.md). A supplied activity boolean does not implement the transfer lifecycle or prove native semantics. Native activity derivation and secret-return classification remain unknown. Executed bounded cached-consumer assertions cover immediate spinner methods, not full addon or timer behavior.
