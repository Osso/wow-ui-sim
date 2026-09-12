# Tooltip money lines

Loaded Blizzard tooltip code owns money-line rendering. See [[tooltip-money-line]] for load ownership and the removed bootstrap shim.

## What it must do

- [ ] Loaded `GameTooltip_AddMoneyLine(self, rawCopper, useRedLineColor)` formats money with the Blizzard money formatter.
- [ ] A zero copper value produces the loaded tooltip formatter's single-space line.
- [ ] `false` or omitted uses highlight color; `true` uses red.
- [ ] Mail tooltip consumers add their label before enclosed-money or unaffordable-COD money lines.

## How it works

- [[tooltip-money-line]]
- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs` — no longer defines a pre-load `GameTooltip_AddMoneyLine` substitute.
- Cached `Blizzard_GameTooltip` and `Blizzard_SharedXML` runtime files — own the loaded helper and formatter.

## Tests asserting this spec

- Pending: grouped loaded-mail consumer coverage; exact file and test names await the focused test slice.

## Known gaps (current cycle)

- [ ] Add focused loaded Blizzard tooltip/mail assertions for formatting, zero output, colors, and ordering.
- [ ] Reconcile `added:GameTooltip_AddMoneyLine` only after those assertions pass.

## Out of scope

MoneyFormatter redesign, vendor edits, native locale/rendering guarantees, mail-system redesign, and pre-addon helper availability.
