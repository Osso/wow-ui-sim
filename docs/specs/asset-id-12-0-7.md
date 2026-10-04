# Retail 12.0.7 UI asset IDs

B01, `global api-C_UIFileAsset-GetFileID-049` in the [retained 12.0.7 source](../../data/patch-api/sources/12.0.7-api-changes.txt) define this bounded authoring slice. Current generated declarations may postdate 12.0.7; declarations are not native behavior evidence. Bounded tests pass on the default cumulative retail build; strict historical and older-epoch builds were not run. See [Lua API architecture](../lua-api.md).

## What it must do

- [x] Resolve numeric IDs 123, 136243 and u32 maximum unchanged, and exact bundled Trade_Engineering path aliases to 136243; unknown paths return exactly one nil. Bundled catalog contents are real input, not complete client-asset coverage.
- [x] Read known slash/backslash and ASCII case variants without inventing IDs for numeric strings, absent paths or loose assets.
- [x] **INFERRED** positive integral u32 numeric domain; zero, negatives, fractions, nonfinite values and overflow return one nil. Nonnil number-or-string input is mandatory; other representations error. No native coercion/error-policy claim.
- [x] Authenticate arg1 and EVERY supplied extra with VM unwrap_secret before validating arg1. Untainted secret IDs/paths work; tainted secrets (including secret false/nil in extras) fail before malformed public arg1 validation.
- [x] Preserve rooted secret identity, caller taint and secrecy across GC; public recovery remains usable. **INFERRED** public output and ignored public extras.

## How it works

- [Lua API architecture](../lua-api.md)
- [C API signature audit](../c-api-signature-audit.md)

## Implementation inventory

- `src/c_api/c_ui_file_asset.rs` — existing bundled catalog and authenticated GetFileID producer.
- `src/limited_listfile.rs` — existing shipped limited asset catalog.

## Tests asserting this spec

`tests/patch_12_0_7_b01_b04.rs` — ONE module in the auto-generated integration harness; first-line retail-12-0-7 cfg. Tests prefixed B01 assert the bounded values and policies above. Default-build B01/B02/B03 tests passed in this integration round.

## Known gaps (current cycle)

- [ ] Strict epoch build remains unexecuted; native numeric/coercion policy and historical declaration epoch remain unverified.
- [ ] IsKnownFile/IsLooseFile have separate assigned rows; their unauthenticated current behavior is not covered.

## Out of scope

Whole client catalog, CASC discovery, IsKnownFile/IsLooseFile delta rows and native output security. No new input field is required: the existing bundled path catalog is the bounded input.
