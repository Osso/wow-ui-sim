# Loot and buyback tooltip data

Retail and PTR tooltip getters select supplied loot and buyback slots and publish item data for the original tooltip handler.

## What it must do

- [ ] `GetLootItem` resolves a one-based slot in supplied loot state.
- [ ] `GetBuybackItem` resolves a one-based slot in supplied buyback state.
- [ ] The original tooltip handler displays the selected item's data.
- [ ] Missing slot behavior is **EMULATED** as no returned values. WM82 leaves native missing-result arity **UNKNOWN**; no native parity is claimed.

## How it works

- [Tooltip and Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_loot.rs`: supplied item selection and missing-result seam.
- `src/lua_api/globals/missing_surface/tooltip_info/`: item tooltip publication.

## Tests asserting this spec

- `tests/tooltip_data_loot_consumer.rs`
- `tests/tooltip_data_buyback_consumer.rs`

## Known gaps (current cycle)

- [ ] The original MerchantFrame hover also calls the missing `ShowBuybackSellCursor`; native evidence is pending.

## Out of scope

- Native missing-result arity and unsupplied item metadata.
