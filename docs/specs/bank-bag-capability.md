# Bank player-bag capability

`C_Bank.ShouldUsePlayerBagsInBank` reports the inspected client profile's bank-panel capability. The selected original `BankPanelMixin.OnLoad` consumer uses it to decide whether to create the bank-tab pool. R44-Q10 inspected PTR 12.1.7.70668 and WoW Forever 1.60.1.70707 only.

## What it must do

- [x] On PTR 12.1.7.70668, return false. The original bank panel creates both the bank-tab pool and item-button pool.
- [x] On WoW Forever 1.60.1.70707, return true. The original bank panel skips the bank-tab pool and creates the item-button pool.

## How it works

- See the [C API and Lua API boundaries](../wiki/index.md).

## Implementation inventory

- `src/c_api/item_spell/c_currency.rs` — registers the profile-specific `C_Bank` capability.

## Tests asserting this spec

- `tests/native_defaults_ingame.rs::approved_bank_panel_onload_uses_ptr_capability` — loads the selected original PTR bank-panel consumer.
- `tests/forever_startup_gaps.rs::forever_bank_panel_onload_uses_player_bag_capability` — loads the selected original Forever bank-panel consumer.

## Known gaps (current cycle)

- [ ] Native capability values for Retail, Mists, Era, Anniversary, and Wrath remain unverified.

## Out of scope

- Do not infer purchased bank-tab data or other bank capability values from this API.
