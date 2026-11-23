# Account currency transfer queries

`C_CurrencyInfo` exposes account-transfer eligibility, source data and amount calculations. Retail eligibility uses supplied effective policy inputs. Retail and WoW Forever expose the account-source and amount APIs. This contract covers query behavior, not transfer execution or its lifecycle.

## What it must do

- [x] **VERIFIED (simulator tests):** `IsAccountTransferableCurrency` returns true for a supplied transferable currency and false for a nontransferable or unknown currency. **VERIFIED (generated data):** percentages and flags come from CurrencyTypes and CurrencyCategory data for 12.1.5.70391. **INFERRED (WM64):** the percentage field is `WarbondTransferPercentage`.
- [x] **VERIFIED (simulator tests):** Retail `CanTransferCurrency` returns a boolean and an optional result code for tested server-disabled, missing-source and eligible-source cases. It reads supplied effective policy inputs. **VERIFIED (simulator source):** the query does not request data or dispatch an event.
- [x] **VERIFIED (Retail and WoW Forever tests):** `GetMaxTransferableAmountFromQuantity` and `GetCostToTransferCurrency` require a currency ID. Missing or nil IDs raise each function's published Usage error; they do not return a currency result.
- [x] **INFERRED (R88es, Retail 12.1.5.70688 and Forever 1.60.1.70707):** on the percentage path, the maximum amount is `floor(p * 0.01 * Q)` using single-precision operations; the total cost is `ceil((100 / p) * q)` using single-precision division and multiplication. **VERIFIED (Retail simulator tests):** examples assert 76 from 100 at currency 1560's rate, costs 99 for 76 and 101 for 77. The original Retail TokenUI consumer asserts source-specific amounts 159/79 and costs 199/99 for supplied source quantities 200/100 at an 80% rate.
- [x] **VERIFIED (Retail simulator test):** when account data is ready, the original TokenUI source selector can read supplied character rows, select each source, set the maximum through its button, and observe the resulting text clamp and gross cost.
- [ ] Assert the public `FetchCurrencyDataFromAccountCharacters` result while account data is not ready, including its nil result and subsequent ready-state row shape.

## How it works

- [Lua API system](../wiki/systems/lua-api.md#retailforever-account-currency-transfer) describes the Rust state and C API registration.
- [Admin economy inputs](../admin-api/economy.md) describes the supplied server, rule and account-source data.
- [Client profiles](../wiki/systems/client-profiles.md) describes profile selection.

## Implementation inventory

- `src/c_api/currency_list.rs` stores account-source rows, transfer rules and readiness state.
- `src/c_api/item_spell/c_currency.rs` registers the C_CurrencyInfo queries and calculates Retail eligibility and transfer amounts.
- `src/lua_api/globals/admin_currency_transfer.rs` reads simulator-admin inputs.
- `src/lua_api/state/sim_state.rs` owns the profile-gated currency-list state.

## Tests asserting this spec

- `tests/currency_transfer_arguments.rs` asserts required currency IDs and exact Usage errors.
- `tests/currency_unused_mutations.rs` asserts supplied eligibility policy, account sources and original TokenUI projections.
- `tests/blizzard_token_ui_loads.rs` asserts the original TokenFrame and CurrencyTransferMenu path with two source quantities.

## Known gaps (current cycle)

- **INFERRED (WM66):** the shared server control defaults to disabled, matching cold-client zero fill; its ordinary-session value is UNKNOWN and may be supplied. Transfer-in-progress defaults to idle. **INFERRED:** in-world context defaults to present as the active local player. Currency and transaction loading do not gate `CanTransferCurrency`; the host supplies account rows and response timing.
- **VERIFIED (simulator source):** when required for a decision, missing faction keys, award-condition results, effective quantity or weekly-cap results, and current-character GUID produce explicit not-modeled errors. **UNKNOWN (WM64/WM66):** public faction-key names, condition internals, effective cap overrides, protected-context guards, and login/server timing.
- The live server enablement, account context, transfer activity, faction conditions, effective caps and account-source writers are not modeled as native systems; callers supply effective inputs. Generated currency data predates the inspected Retail 12.1.5.70688 build.
- The not-ready public `FetchCurrencyDataFromAccountCharacters` result still needs a direct assertion. R88es documents a comparison shortcut, but the comparison operand and its reachability for ordinary currency data remain UNKNOWN.

## Out of scope

Transfer requests, transaction history, persistence, timers, completion/failure events and native lifecycle behavior are separate from these query contracts. The host-provided account rows and policy inputs do not implement those systems.
