# Retail currency tooltips

Retail 12.1.5 currency tooltips publish R84/R84b `C_TooltipInfo` data through the original `GameTooltip` consumers. The source lives in `src/c_api/c_tooltip_info_currency.rs`; see the [tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation) for implementation details.

## What it must do

### Inputs and routing

- [x] `GetCurrencyByID` passes the signed amount unchanged and uses zero when the amount is absent.
- [x] `GetLootCurrency` and `GetQuestLogCurrency` divide signed quantities by 100 only when CurrencyTypes Flags_0 bit 3 is set; division truncates toward zero.
- [x] A `currency:` hyperlink passes its optional quantity to the shared producer without that division; absent quantity is zero.
- [x] `GetQuestLogCurrency` accepts the full `reward`, `required` and `choice` strings with ASCII case folding, converts the public index from one-based to zero-based by subtracting before truncation, and uses the selected quest only when questID is absent.
- [x] Quest currency source results use the host input `(questID, canonical type, zero-based source index) -> (currencyID, raw quantity)`. Currency text rows, local values and preformatted display strings are also explicit inputs. The simulator does not infer source results from unrelated quest reward state.

### Payload and rows

- [x] Populated payloads use type 5 and a data-instance ID, include the currency ID only when it is nonzero, and omit top-level quantity.
- [x] The first supplied CurrencyContainer text row matching currency ID and signed quantity wins; bounds are inclusive and maximum zero is unbounded. No match uses ordinary currency text.
- [x] Rows are left-only. Account labels follow their verified flag/field gates and order. The description is the only wrapped row.
- [x] The XP row uses the getter's amount and ends the producer before spacer or cap rows.
- [x] A supplied local currency record with Flags_0 bit 22 clear permits the spacer and cap section; bit 22 suppresses both.
- [x] Weekly and total/season rows follow their verified maximum-source, scaling, flag and seasonal-exclusivity rules.

## How it works

- [Tooltip-data system](../wiki/systems/lua-api.md#tooltip-data-and-merchant-cursor-presentation).
- [Hyperlink requests](tooltip-hyperlink.md) — R82 marker grammar and request routing.
- R84/R84b native evidence: `~/.local/state/wow-ui-sim/handoff/pr46-49-decomp-answers.md` — Retail 12.1.5.70688.

## Implementation inventory

- `src/c_api/c_tooltip_info_currency.rs` — host inputs, Retail currency payloads and cap rows.
- `src/c_api/c_tooltip_info_loot.rs` — Retail `GetLootCurrency` dispatch.
- `src/lua_api/globals/missing_surface/tooltip_info/probes.rs` — `GetCurrencyByID`, `GetQuestLogCurrency` and `GetHyperlink` dispatch.
- `src/lua_api/globals/missing_surface/tooltip_info/builders.rs` — shared line and identity construction.
- `src/lua_api/state.rs` and `src/lua_api/state/sim_state.rs` — initialize and store explicit currency and quest selector inputs.

## Tests asserting this spec

- `tests/tooltip_currency_r84.rs` — original by-ID, hyperlink, quest, range-text, account-label, weekly and total/season consumers.
- `tests/tooltip_loot_currency.rs` — original loot hover, amount normalization and currency identity.
- `tests/tooltip_currency.rs` — currency name and top-level quantity assertions.

## Known gaps (current cycle)

- [ ] The host supplies selected quest source results; reward, required-objective and choice source lists are not inferred. Resolver-miss Lua results remain UNKNOWN.
- [ ] The balance-helper formula, runtime number-format mode, locale grouping and total-row prefix selection are not modeled; tests supply explicit values or formatted text.
- [ ] CurrencyContainer row content and amount-dependent description formatting are host-supplied. No 70688 text catalog is claimed.
- [ ] The conditional error row, recharge calculation and locale-table aliases remain unmodeled.

## Out of scope

- Non-Retail currency producer changes and other currency getters.
- Guessed quest-list sources, missing-result behavior, error-row predicates, balance formulas or formatter modes.
