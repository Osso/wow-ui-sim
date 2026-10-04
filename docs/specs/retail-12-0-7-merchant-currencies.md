# Retail 12.0.7 merchant currencies — B19

Source row [037](../../data/patch-api/sources/12.0.7-api-changes.txt) adds `C_MerchantFrame.GetMerchantCurrencies`. Cached MerchantFrame documentation describes a number array; cached `Deprecated_12_0_7.lua` unpacks it for the retired global. Cache may postdate 12.0.7; no native historical proof.

## What it must do

- [x] Read ordered host currency IDs live, return exactly one public array, detach each snapshot, preserve environment isolation.
- [x] INFERRED: default/cleared input returns one empty table (cached MerchantFrame uses `#currencies` unconditionally); host order is authoritative, not derived from merchant items.
- [x] INFERRED: without a SecretArguments declaration, reject all secret arguments/extras for both untainted and tainted callers; ignore public extras.
- [x] Retired global is nil by ordinary lookup before loading the real cached wrapper; wrapper returns exactly the configured IDs, or zero results for empty input.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_merchant_frame.rs` — feature-gated producer, rooted detached array, secret guard.
- `src/lua_api/state.rs`, `src/lua_api/state/sim_state.rs` — explicit empty per-environment currency IDs.

## Tests asserting this spec

- `tests/p1207_merchant_currencies.rs` — four public API cases, including actual cached wrapper execution. Only that wrapper declaration is executed; unrelated menu migrations and the deprecation CVar gate are not claimed.

## Known gaps (current cycle)

- [ ] Historical policy provenance, absent-merchant MayReturnNothing behavior and native execution remain unproved.

## Out of scope

- Currency service/item-cost derivation, ordering rules, complete deprecated addon loading, other-profile proof.
- No vendor/cache edits; no nil default because cached consumers require an array.
