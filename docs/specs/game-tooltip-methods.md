# GameTooltip methods

This spec records the `GameTooltip` minimum-width and FontString-pair contracts implemented by `src/lua_api/frame/methods/widgets/tooltip/`; the [Lua API wiki](../wiki/systems/lua-api.md#game-tooltip-minimum-width-and-font-strings) describes the current model. Native evidence is R88n.7 for Retail 12.1.5.70688.

## What it must do

### Minimum width

- [ ] On Retail 12.1.5.70688, `GetMinimumWidth` uses `Usage: local width, forced = self:GetMinimumWidth()` and returns exactly the stored width as a number, then the stored forced state as a boolean.
- [ ] A new `GameTooltip` starts with minimum width `0` and forced state `false`; `SetMinimumWidth(width, force)` stores both values for the getter to return.
- [ ] A missing or nil getter receiver raises an error with the exact Usage text. Tested non-table, non-framescript and wrong-class conversions use the exact native error reasons recorded by R88n.7.

### FontString pairs

- [ ] `AddFontStrings` uses `Usage: self:AddFontStrings(leftFontString, rightFontString)`, reads a tooltip and two FontStrings, and returns zero values on success.
- [ ] A successful call stores the supplied left/right pair. Stored pair count is separate from `NumLines`; adding a pair does not add a populated tooltip line.
- [ ] Tested receiver and FontString conversion failures use the native reasons and Usage formatting recorded by R88n.7.

### Original consumer

- [ ] The selected Retail `GameTooltip_InsertFrame` consumer continues to use the first `GetMinimumWidth` result and can raise the stored width for an inserted frame.

## How it works

- [Lua API: GameTooltip minimum width and FontString pairs](../wiki/systems/lua-api.md#game-tooltip-minimum-width-and-font-strings)
- [Consumer selection and RED proof](../wiki/investigations/consumer-selection-and-red-proof.md#gametooltip-minimum-width-consumer)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/tooltip/line_data.rs` — minimum-width getter/setter and method argument conversion.
- `src/lua_api/frame/methods/widgets/tooltip/owner.rs` — `AddFontStrings` pair storage.
- `src/lua_api/tooltip.rs` — tooltip width flags and stored FontString pairs.

## Tests asserting this spec

- `tests/tooltip_minimum_width.rs` — API arity, defaults, round-trip, pair storage, `NumLines` separation and conversion errors.
- `tests/tooltip_minimum_width.rs::retail_shared_tooltip_insert_frame_preserves_minimum_width_consumer` — optional Retail consumer preservation test.

## Known gaps (current cycle)

- [ ] Run the targeted API and preservation tests, five zero-warning compile receipts, and the Retail `lua-errors` check before handoff.

## Out of scope

- `AddFontStrings` parent-change, forbidden-aspect and secret-state permission checks are not implemented by this slice; the tests do not claim their parity.
- The later layout, rendering, anchoring and lifetime use of stored FontString pairs is UNKNOWN. This slice stores the pair and does not invent those effects.
- `SetMinimumWidth` argument coercion beyond the existing implementation is not changed here.
