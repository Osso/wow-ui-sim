# Currency hyperlink builder — `GetCurrencyListLink` and `GetCurrencyLink`

Producer: `src/c_api/item_spell/c_currency.rs` (`build_currency_link`, `c_currency_get_list_link`,
`c_currency_get_currency_link`). Destination-buffer state: `SimState.currency_link_buffer`.

Contract source: decompiled client, `H/pr46-49-decomp-answers.md` Round 2 Q1 (Retail 12.1.5.**70688**,
Forever 1.60.1.**70707**, PTR 12.1.7.**70668**). See [C API boundary](../lua-api.md) for the module split.

## What it must do

Both getters publish the link built by **one shared builder**, because native both call the same function
with the same two-argument layout: a signed 32-bit currency id and a signed 32-bit amount (proved in all three
builds).

- [x] A resolved currency id formats `|cnIQ<n>:|Hcurrency:<id>:<amount>|h[<name>]|h|r`, where `<n>` is the
  record quality, `<id>` the requested id, `<amount>` the requested amount, and `<name>` the record's plain
  name. The colour is the **named** quality token, never `|cffRRGGBB`.
- [x] `GetCurrencyListLink(index)` passes a constant amount of zero, so every list-built link reads
  `currency:<id>:0` regardless of the held quantity.
- [x] `GetCurrencyLink(type[, amount])` forwards the optional amount and passes zero when the argument is
  absent.
- [x] `GetCurrencyLink` is published only where the pinned `Blizzard_APIDocumentationGenerated`
  `CurrencyInfoDocumentation.lua` declares it: Retail, PTR, Forever. Mists, Era and Anniversary pin no
  declaration, so the member stays unpublished there. Wrath pins no generated documentation at all.
- [x] An index at or past the displayed count publishes exactly one nil. The native getter does not call the
  builder on that path.
- [x] There is no header-row test: a header row takes the same builder path with its own id (zero in this
  model) and therefore republishes the shared buffer rather than nil.
- [x] An id the builder cannot resolve (zero, or absent from the currency records) publishes the destination
  buffer itself: the empty string until the first link is built, then the last built link. The result is
  always a string, never nil.
- [x] The destination buffer is per environment, initially empty, and only the success path writes it.

Inferred, not proved: the record byte read as `<n>` is the currency database `Quality` column (the schema
descriptors are built at run time, so no static mapping exists). This spec uses the record quality because
that is the only quality the model holds. The stale-link observable is a construction argument, not a
runtime observation.

## How it works

- [C API and Lua boundary](../lua-api.md)
- [Displayed currency indices](currency-displayed-indices.md) — the list projection both getters index into.

## Implementation inventory

- `src/c_api/item_spell/c_currency.rs` — shared builder, both getters, registration per profile.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs` — the per-environment link buffer.

## Tests asserting this spec

- `src/c_api/item_spell/c_currency.rs` (`mod tests`): native token format, header-row buffer, header row
  repeats the previous link, amount forwarding, unresolved-id buffer reuse, nil past the list.
- `tests/currency_info_probes.rs::get_currency_list_link_matches_the_displayed_row`: original consumer reads
  `currency:<id>` out of the link on every profile with a pinned declaration.

## Known gaps (current cycle)

- [x] Link text and publication replaced the modelled `|c<hex>` best guess.

## Out of scope

Destination-buffer capacity, the RGB a named quality token resolves to, the PTR publisher tail, and builds
70575 / 70493 are not modelled or proved.
