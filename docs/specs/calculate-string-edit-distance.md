# CalculateStringEditDistance

`CalculateStringEditDistance` returns a byte-based edit distance for two strings. Its Retail and WoW Forever behavior is an inferred model; native evidence is pending R88g9.

## What it must do

- [x] For string inputs, return the minimum insert, delete, and substitute operations needed to transform the first byte string into the second.
- [x] Return zero for equal strings, including two empty strings.
- [ ] Confirm native handling of non-string inputs and non-ASCII strings (INFERRED R88g9 pending).

## How it works

- [Lua API system](../wiki/systems/lua-api.md)

## Implementation inventory

- `src/lua_api/globals/real/string_extensions.rs` — Retail and Forever Rust provider.
- `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs` — legacy-profile fallback.

## Tests asserting this spec

- `tests/calculate_string_edit_distance.rs`

## Known gaps (current cycle)

- [ ] Native evidence for argument validation and byte-versus-character semantics.

## Out of scope

- Unicode grapheme distance; no native contract or consumer evidence supports it.
