# Merchant item-link publication

R88l.2 (Retail 12.1.5.70688) supplies `GetMerchantItemLink` validation,
result arity and link formatting. Previously the global was absent and original
Retail `MerchantFrame_Update` stopped at `MerchantFrame.lua:329`.

## Contract and model boundary

The Retail-only publication reads argument one, accepts numeric strings using
the existing numeric parser, range-checks the inclusive signed 32-bit bounds
before truncation, and subtracts one with wrapping arithmetic. Invalid arguments
raise `Usage: GetMerchantItemLink(index)`; overflow uses the existing six-decimal
`integer overflow attempting to store %f` diagnostic. Unselected/ineligible,
unsupported, absent and missing-required-record paths return zero Lua values.
A selected builder with no text returns one nil; text returns one string.

The existing `merchant_items` collection remains the host's **selected offer
ordering**, not a newly invented replica of raw native storage. `merchant_links`
adds explicit host availability/rejection inputs and per-slot builder records.
It does not infer the native raw-row filtering/renumbering algorithm or server
writer. An empty override map preserves the existing item-only collection;
there are no seeded currency/spell offers. The two special spell paths are
represented separately; the resolving path requires the host's resolved record.
No event or completion callback is installed.

Item records must exist in the shared item database. The shared item-link
formatter now accepts unnamed positional fields, compacts complete zero fields,
and supplies the existing quality token/name. Its old helper still supplies its
historical level/spec fields unchanged for existing callers. The merchant route
instead supplies twelve empty fixed fields when it has no extra field data,
never the historical hardcoded level80/spec70. A host can supply actual positional
fields; this is not a claim to know their complete names or native writers.
Currency and spell formatting follows the literal native templates using only
host-supplied record data. Missing record and missing builder text are distinct.

## Verification scope

`tests/merchant_item_link.rs` covers argument diagnostics, truncation, extra
arguments, result arity, missing/ineligible records and explicit builder text.
These are API tests of VERIFIED publication facts, not native producer tests.
`merchant_item_link_original_consumer_stores_distinct_item_links` invokes the
original selected Retail Game `MerchantFrame_Update` through normal startup and
checks two distinct existing item IDs in the original item button's link field.
It rejects a constant/nil/no-op producer. The earlier affordability test no longer
expects the missing-global boundary at line329 and checks the stored link.
These tests do not claim unrelated later MerchantFrame presentation is complete.
No Blizzard Lua, native-unknown behavior or other-profile publication is changed.

Final receipts must be checked against the eventual immutable candidate. This
page does not itself claim a build/runtime pass.

## Sources

- Tracker lt-5d7d, native clean-room answer R88l.2, comment1063.
- [Merchant implementation](../../../src/lua_api/globals/real/merchant_item_link.rs)
- [API tests](../../../tests/merchant_item_link.rs)
- [Original consumer](../../../tests/reset_set_merchant_filter.rs)

## See Also

- [[merchant-lfg-r88i]] — earlier affordability behavior and downstream boundary.
