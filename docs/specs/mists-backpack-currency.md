# Mists backpack currency

The Mists legacy backpack-currency globals share watched flags with `C_CurrencyInfo` currency-list state. VERIFIED native evidence from Mists 5.5.4.70631 covers the four-value getter, numeric enable flag and absence of a native three-slot cap. The Mists-only legacy publication lives in `src/c_api/item_spell/c_currency.rs`; [the investigation](../wiki/investigations/mists-r21-classic-apis.md) records native evidence and original TokenUI selection.

## What it must do

- [ ] `SetCurrencyBackpack(currencyListIndex, enable)` uses a 1-based currency-list index and treats numeric zero as disabled and nonzero as enabled.
- [ ] `GetBackpackCurrencyInfo(index)` accepts a numeric or numeric-convertible 1-based index and returns `name, count, icon, currencyID` for a watched slot, or four nils for an empty or out-of-range slot.
- [ ] The getter reflects `SetCurrencyBackpack` changes and has no native three-slot cap.
- [ ] The selected Mists TokenUI continues to display only its first three watched slots while the API can return a fourth.

## How it works

- [The Mists R21 native evidence and selected-consumer proof](../wiki/investigations/mists-r21-classic-apis.md)
- [Currency display-list state](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/c_api/item_spell/c_currency.rs` — Mists watch-flag mutation, list projection and four-value legacy getter.
- `src/c_api/currency_list.rs` — watched overrides shared with the currency-list projection.
- `src/lua_api/env_init/mod.rs` — registers the Mists legacy globals after profile bootstrap.

## Tests asserting this spec

- `tests/mists_r21_currency.rs` — four watched currencies, tuple/nil behavior, numeric-convertible indices and the original selected TokenUI consumer.

## Known gaps (current cycle)

- [ ] The focused Mists currency tests must pass against the original selected consumer.

## Out of scope

- VERIFIED native behavior, not modeled here: `SetCurrencyBackpack` can send a flag update to the server when connected. This simulator surface models local flags only and adds no network dispatch.
- Native currency catalog contents, quantities, and any server-side cap.
- Era, Anniversary and other profile behavior; this implementation is Mists-only.
