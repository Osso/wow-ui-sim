# PTR ICU collation

PTR `C_Intl` and `LuaLocaleContext` comparison methods use ICU 2.1.1 collation for valid UTF-8. The simulator models five strengths, locale-context selection, comparison signs, and raw binary sort keys; those outputs are not native WoW observations.

## Model

- Global calls use the current client locale; context calls use each userdata's stored locale.
- Strength values `0` through `4` map to ICU primary through identical strength.
- Comparisons return simulator signs `-1`, `0`, or `1`.
- Sort keys are raw ICU binary Lua strings. Their bytewise order matches the modeled comparison for the same locale, strength, and ICU version.
- Stored locale identifiers stay opaque; casing-time locale parsing and WoW-tag translation are reused only for collation operations.

## Boundary

ICU/CLDR version, signs, key bytes, termination, locale parsing, errors, `MayReturnNothing`, secret access, coercion, and native WoW equivalence remain unverified.

## Sources

- [PTR Intl collation](../../specs/intl-collation.md) — modeled contract, dependency, proof, and exclusions.
- [patch 12.1.5 API audit](../investigations/patch-12-1-5-api-audit.md) — occurrence status and evidence boundary.

## See Also

- [[locale-context-storage]] — userdata locale state.
- [[patch-12-1-5-api-audit]] — audit inventory.
