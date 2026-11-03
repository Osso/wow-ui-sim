# Binary decoder inputs

`C_EncodingUtil` decoders accept byte strings and ordinary numbers. See [binary encoding](../wiki/systems/binary-encoding.md) for the implementation and evidence limits.

## What it must do

- [x] Convert ordinary numeric inputs to strings using Lua number formatting for `DeserializeCBOR`, `DecodeBase64` and `DecompressString`.
- [x] Reject nil and tables with their Lua type in the CBOR error; Base64 and decompression report their respective usage strings.
- [x] Reject truncated CBOR headers with `attempted to deserialize an incomplete or ill-formed data item`.
- [x] Return zero values for empty CBOR and one empty string for empty Base64; raise `DecompressString(): internal decompression error` for empty compressed input. These native outcomes are inferred from static analysis, not observed end-to-end.

## How it works

- [Binary encoding](../wiki/systems/binary-encoding.md)

## Implementation inventory

- `src/c_api/encoding_util.rs` — argument handling and decoder dispatch.
- `src/c_api/encoding_util/cbor.rs` — CBOR decoding.

## Tests asserting this spec

- `tests/encoding_util_binary.rs` — decoder input and error contracts.

## Known gaps (current cycle)

- [ ] Native end-to-end return and rethrow connections remain unverified for the empty-input cases.

## Out of scope

- Native behavior for all other malformed CBOR forms is not established by this evidence.
