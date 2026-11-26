# CurrencyTransfer nil currency during EditBox initialization

Retail and Forever startup reached the original CurrencyTransfer input handler before a currency was selected. The simulator now returns no maximum for that nil currency input. The native nil-argument behavior remains inferred pending R88g8 evidence.

## Content

### Symptoms

The stock Lua-error baselines recorded two entries from `Blizzard_CurrencyTransfer.lua:488`: the `OnTextChanged` Usage error from `GetMaxTransferableAmountFromQuantity(nil, ...)` and its `Blizzard_ScriptErrors` echo.

### Evidence and cause

The original consumer initializes `CurrencyTransferAmountSelectorMixin`, resets its input box with `SetNumber(0)`, and validates changed text before a currency is selected. **VERIFIED (simulator source):** new frame text is `None`, and `SetNumber(0)` stores `"0"`; the simulator therefore sees a changed value. The pinned API declaration marks `currencyID` non-nilable, but the original consumer calls the query without a nil guard and uses `or 0` for a missing maximum.

**INFERRED — R88g8 (native evidence pending):** the native max query returns nil for a nil currency ID. The UI consumer is the available evidence for this choice; the declared signature does not prove nil acceptance. R88et-b leaves hidden-box delivery UNKNOWN. This fix does not add a visibility gate or change EditBox dispatch.

### Fix and proof

`src/c_api/item_spell/c_currency.rs` returns nil for argument 1 equal to nil in `GetMaxTransferableAmountFromQuantity`. `GetCostToTransferCurrency` keeps its required-ID Usage error. The targeted API test checks the nil case and positive supplied results for currencies 1560 and 1792. The original TokenUI amount-selector test checks its normal amount/cost consumer path. The [currency-transfer contract](../../specs/currency-transfer-eligibility.md) records the API expectation.

**VERIFIED (Retail and WoW Forever captures):** both stock `lua-errors` outputs are `[]` with exit 0, so both CurrencyTransfer records are gone from both baselines. **VERIFIED (build receipts):** Retail, WoW Forever, Mists and Wrath `--lib --test integration --no-run` builds pass with zero warnings, and PTR `cargo check --lib --tests` passes with zero warnings.

## Sources

- [Currency-transfer contract](../../specs/currency-transfer-eligibility.md) — nil inference, amount arithmetic and required cost-query argument.
- [EditBox text and selection contract](../../specs/editbox-text-position-selection.md) — simulator notification behavior and unknown native hidden-box delivery.
- [Retail Lua-error baseline](../../baselines/retail-lua-errors.json) — the prior startup diagnostic and refreshed clean capture.
- [Currency C API implementation](../../../src/c_api/item_spell/c_currency.rs) — simulator argument policy and amount query.

## See Also

- [[lua-api]] — C API registration and currency-transfer state.
- [[event-system]] — EditBox text-change dispatch boundaries.
