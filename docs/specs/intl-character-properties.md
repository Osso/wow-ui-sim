# Intl character properties

PTR `C_Intl.GetCharacterProperties(text)` returns Unicode properties for the first scalar of valid UTF-8 text. The pinned contract is in `data/patch-api/sources/12.1.5-register.json`; implementation is in `src/c_api/c_intl/character_properties.rs`. Shared context: [locale context storage](../wiki/systems/locale-context-storage.md).

## What it must do

- [x] Return one fresh table with exactly seven fields: numeric `codePoint`; boolean `isAlphabetic`, `isDigit`, `isWhitespace`; string `generalCategory`, `scriptCode`, `blockCode`.
- [x] Use ICU Alphabetic and White_Space properties; `isDigit` means General_Category Decimal_Number (`Nd`), not every numeric character.
- [x] Return the General_Category short name (such as `Lu`) and ISO script short name (such as `Latn`), with the block's canonical display name (such as `Basic Latin`).
- [x] Return `No_Block` for a scalar outside every block. This is an explicit model value, not lookup against alternative data.
- [x] Inspect the first scalar, return zero results for empty input, reject malformed UTF-8 anywhere in the input, and reject missing/non-string arguments.
- [x] Keep returned tables independent, without publishing `CharacterProperties` as a global.
- [x] Preserve earlier-retail namespace absence before and after bootstrap and existing Intl behavior.

### Simulator assumptions

First-scalar selection, property-name conventions, empty-input behavior and validation errors are chosen simulator semantics. The generated documentation names the seven fields and permits no result but does not establish those native details. Valid trailing scalars are ignored; malformed trailing bytes are rejected because the entire input must be valid UTF-8. Unicode scalar values are returned numerically without locale dependence.

### Dependency and data versions

- Direct `icu_properties =2.1.2`, with only `compiled_data`, uses the already-locked `icu_properties` and `icu_properties_data` **2.1.2** packages. It supplies binary properties, General_Category, Script and their short names. Their bundled Unicode data release was not identified in the inspected released package sources; it remains **unknown**, not assumed to be Unicode 17 or the same as the other ICU crates.
- Direct `unicode-blocks =0.1.10` supplies block lookup and canonical display names because this slice needs a block lookup independent of ICU's exposed property surface. The released crate's `src/lib.rs` and `src/unicode_blocks.rs::VERSION` explicitly identify **Unicode 17.0.0**; the generated dataset cites Unicode `Blocks.txt`.
- ICU and block data version agreement is therefore **unverified**. The providers may classify newly assigned characters differently. This does not trigger substitution between datasets.
- `Cargo.lock` adds only `unicode-blocks 0.1.10`; no existing package versions change. Both direct dependencies are exact-pinned for predictable behavior.

## How it works

- [Locale context storage](../wiki/systems/locale-context-storage.md)

## Implementation inventory

- `src/c_api/c_intl/character_properties.rs`: first-scalar property query and result construction.
- `src/c_api/c_intl/text.rs`: existing strict UTF-8 input validation.
- `src/c_api/c_intl.rs`: PTR registration; earlier-profile namespace exclusion.
- `Cargo.toml`, `Cargo.lock`: explicit Unicode properties/block dependencies.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_character_properties.rs`: ASCII, Arabic-Indic decimal digits, nondecimal numeric characters, combining marks, whitespace, emoji, Han, no-block scalar, input selection/validation, fresh tables and retail absence.
- Existing `intl_` library tests: regression coverage for context storage, normalization, length, casing, titlecase, segmentation, locale transforms, collation and plurals.

## Known gaps (current cycle)

- [ ] Native first-character selection, naming, empty/no-result conditions, invalid-input handling and Unicode-version equivalence remain unverified.
- [ ] Identify the bundled ICU properties Unicode data version; agreement with Unicode 17.0.0 block data is not established.
- [ ] `AllowedWhenUntainted` enforcement is not established by ordinary-value tests.

## Out of scope

Locale-dependent classification, native error compatibility, secret/taint enforcement, documentation-only structure globals and unrelated Unicode dependency upgrades.
