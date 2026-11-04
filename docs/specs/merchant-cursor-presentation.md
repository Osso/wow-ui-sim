# Merchant cursor presentation

Retail and PTR cursor presentation is separate from held cursor content. WM86 describes the native behavior; native gates and listener timing remain unknown.

## What it must do

- [x] Buyback hover selects Buy or UnableBuy from supplied buyback price and player money. Invalid slots preserve the display. Two supplied gates and a Point base must permit the operation before argument validation.
- [x] Repair clears held content, selects the Repair base and displays Repair when the supplied gates and merchant repair capability permit it.
- [x] `InRepairMode` reads the base. `SetCursor` and `ResetCursor` do not leave repair mode or clear held content.
- [x] A changed presentation notifies with held-content state only when supplied refresh and notification controls permit it. Repeated selectors do not notify.

## How it works

- [Held cursor model](cursor-model.md)

## Implementation inventory

- `src/lua_api/globals/real/cursor_presentation.rs`: presentation operations and notification.
- `src/lua_api/globals/real/cursor_model.rs`: held content and presentation storage.

## Tests asserting this spec

- `tests/merchant_cursor_presentation.rs`
- `tests/tooltip_data_buyback_consumer.rs`

## Known gaps (current cycle)

- [ ] The meanings and native defaults of the two gates and notification controls are unknown. Their simulator defaults are scenario inputs, not native claims.
- [ ] Native listener delivery timing and callback re-entry are unknown; the simulator currently uses its existing synchronous event dispatcher.
- [ ] Complete custom cursor asset selection is unknown and keeps the previous simulator result.

## Out of scope

- `ShowMerchantSellCursor`: WM86 found no native registration.
