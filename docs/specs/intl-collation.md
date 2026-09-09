# Intl collation

PTR global and locale-context comparison/sort-key APIs use ICU collation. Source: `src/c_api/c_intl/collation.rs`. See [locale context storage](../wiki/systems/locale-context-storage.md) for the surrounding context model.

## What it must do

- [x] Publish `C_Intl.CompareStrings(left, right, strength)` and `GetSortKey(text, strength)`, plus corresponding context methods, only on PTR.
- [x] Require strength: 0 Primary, 1 Secondary, 2 Tertiary, 3 Quaternary, 4 Identical. Reject missing, nonnumeric, fractional, or out-of-range strengths.
- [x] Return comparison results as -1, 0, or 1. Return one raw binary Lua string for a sort key, without encoding or appending a terminator.
- [x] For each strength, bytewise key ordering agrees with comparison across distinct inputs; canonically equivalent strings compare equally.
- [x] Respect accent/case strength distinctions and Swedish versus German ordering. Context locale edits affect subsequent calls; other contexts remain independent.
- [x] Use the current client locale globally and the stored locale for context calls, retaining shared WoW-tag translation and explicit ICU locale parsing errors without silent fallback.
- [x] Reject malformed UTF-8 without replacement and reject invalid receivers. Preserve existing Intl APIs and retail absence.

### Modeled choices and dependency

`icu_collator = "=2.1.1"` with `compiled_data` supplies locale-aware collation and compatible binary keys; scalar or lexical comparisons cannot supply this behavior. Required new lockfile packages are `icu_collator`, `icu_collator_data`, `utf16_iter`, and `write16`; existing package versions remain unchanged. ICU's remaining collation options use its defaults.

Comparison signs, raw unterminated ICU key bytes, locale translation, and error behavior are simulator contracts, not native WoW observations. Keys depend on ICU/Unicode/CLDR data and must not be treated as durable cross-version identifiers. Only keys produced with the same locale, strength, and implementation version are comparable.

## How it works

- [Locale context storage](../wiki/systems/locale-context-storage.md)

## Implementation inventory

- `src/c_api/c_intl/collation.rs`: strength mapping, comparisons, binary key publication.
- `src/c_api/c_intl.rs`: registration, shared current/context locale and locale parsing.
- `src/c_api/c_intl/text.rs`: strict UTF-8 input handling.
- `Cargo.toml`, `Cargo.lock`: pinned collation dependency and data.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_intl_collation.rs`: strengths, key ordering, canonical equivalence, locale changes, invalid inputs, profile absence.
- Existing `intl_` grouped library tests: context, normalization, length, casing, titlecase, segmentation, and locale-transform regressions.

Focused proof: `/tmp/intl-collation-red.log` (two missing-API failures), `/tmp/intl-collation-green-ptr.log` (16 passed), `/tmp/intl-collation-green-retail.log` (8 passed).

## Known gaps (current cycle)

- [ ] Confirm native comparison signs, key bytes/options/version, and failure behavior. Pinned functions declare `MayReturnNothing`; conditions for no return remain unknown.
- [ ] Enforce/verify `SecretArguments = AllowedWhenUntainted` separately.

## Out of scope

Custom collation options, stable persisted keys, native Unicode-version equivalence, taint/security enforcement, and unrelated Intl algorithms are not established by these tests.
