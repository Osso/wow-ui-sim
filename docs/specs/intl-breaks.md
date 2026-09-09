# Intl break boundaries

PTR `C_Intl.FindBreaks` and `LuaLocaleContext:FindBreaks` return Unicode break boundaries. Source: `src/c_api/c_intl/breaks.rs`; [Intl casing architecture](../wiki/systems/intl-casing.md).

## What it must do

- [x] Accept required UTF-8 text and BreakType 0–3; return one fresh numeric array of byte offsets.
- [x] Support grapheme clusters, words, sentences, and line breaks with existing ICU 2.1.1 compiled data.
- [x] Preserve multibyte byte offsets, combining sequences and emoji boundaries.
- [x] Validate context receivers without interpreting or changing their opaque locale identifiers.
- [x] Reject malformed UTF-8, nonstrings, and invalid break types without mutation.
- [x] Keep the APIs absent on earlier retail and preserve existing Intl behavior.

Simulator assumptions, not native conformance: offsets are zero-based UTF-8 byte boundaries including zero and the final byte length. Empty input follows ICU's single zero boundary. All contexts use locale-independent default segmentation. Valid input returns a table; native `MayReturnNothing` conditions remain unknown.

## How it works

- [Intl casing architecture](../wiki/systems/intl-casing.md)
- [Locale context storage](../wiki/systems/locale-context-storage.md)

## Implementation inventory

- `src/c_api/c_intl/breaks.rs` — default ICU segmentation and Lua result arrays.
- `src/c_api/c_intl.rs` — global and userdata registration.
- `src/c_api/c_intl/text.rs` — strict UTF-8 input validation.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_breaks.rs`

## Known gaps (current cycle)

- [ ] Confirm native indexing/endpoints, locale-specific segmentation, Unicode data version, optional-result conditions and validation behavior.

## Out of scope

- Secret/taint enforcement and native compatibility claims: require separate evidence.
- New dependencies and other segmentation APIs: existing direct `icu_segmenter =2.1.1` (`compiled_data`, `auto`) supplies all four modes.
