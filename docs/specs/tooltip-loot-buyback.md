# Loot and buyback tooltip data

Retail and PTR tooltip getters select supplied loot and buyback slots and publish item data for the original tooltip handler.

## What it must do

- [x] `GetLootItem` resolves a one-based slot in supplied loot state.
- [x] `GetBuybackItem` resolves a one-based slot in supplied buyback state.
- [x] The original tooltip handler displays the selected item's data.
- [ ] Missing slot behavior is **EMULATED** as no returned values. WM82 leaves native missing-result arity **UNKNOWN**; no native parity is claimed.

## How it works

- [Merchant cursor presentation](merchant-cursor-presentation.md)

- [Tooltip and Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_tooltip_info_loot.rs`: supplied item selection and missing-result seam.
- `src/lua_api/globals/missing_surface/tooltip_info/`: item tooltip publication.

## Tests asserting this spec

- `tests/tooltip_data_loot_consumer.rs`
- `tests/tooltip_data_buyback_consumer.rs`

## Known gaps (current cycle)

- [ ] Native-compatible index coercion is not implemented: these getters retain strict integer parsing.
- [ ] D56 copied markup, packed RGB, present-side colors and nested identity publication remain incomplete. The current renderer uses simulator segment arrays; restoring copied markup requires moving markup parsing into the presentation layer first.

## Out of scope

- Native missing-result arity and unsupplied item metadata.
