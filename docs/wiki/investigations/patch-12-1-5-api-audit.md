# Patch 12.1.5 API Audit

PTR `12.1.5.69594` adds or changes a broad generated API surface relative to `12.1.0.69587`. The frozen register contains 449 semantic occurrences. Current manifest disposition is 74 best-effort and 375 evidence-required rows, with no untriaged rows; this remains an incomplete conformance audit.

## Source Boundary

- Base Gethe commit: `a89e9d0ceb7f6cd31e8fc5ca7df1a338ac0b1b58` (`12.1.0.69587`).
- Target Gethe commit: `49b69918fcdc77e109813281e4f537d45ec7dcbf` (`12.1.5.69594`).
- Register SHA-256: `89e58d77b6f02cd7d8f08b5ac7d434573a2cc035253026bac5bfc2f03b691ddb`.
- Occurrences: 262 added, 185 changed, 2 removed.
- Categories: 75 functions, 78 methods, 20 enums, 198 enum members, 14 events, 16 structures, 43 fields, 2 script objects, 1 callback, and 2 predicates.

The source boundary is immediate generated `*Documentation.lua` files. It excludes FrameXML-only helpers, CVars, GlobalStrings, patch-note behavior, runtime values, and intermediate builds. Documentation prose is ignored; ordered arguments, returns, fields, defaults, security metadata, and symbolic enum/constant expressions are retained.

## Coverage Matrix

| Surface | Current evidence | Disposition | Missing proof |
|---|---|---|---|
| PTR table contracts | Commits `1d11c0176` and `f0ae4a96a`; focused tests cover extensions plus `getcountinfo` and required `create` hints | 10 best-effort / behavioral | Secret-key/value propagation, invalid arguments, sparse edge cases, capacity observability, and native error semantics |
| `C_Intl` and `LuaLocaleContext` | Generated signatures only; no simulator namespace or locale object | evidence-required / unsafe | Unicode normalization, segmentation, collation, formatting, locale state, nil/error cases, and taint/secret semantics |
| `C_Weather`, `Enum.WeatherType`, `WEATHER_CHANGED` | Generated declarations only; no weather model or event producer | evidence-required / unsafe | Native weather values, intensity, event timing/payload, and state transitions |
| `Enum.FragmentID` and metadata | Commit `5e5e46dd9` with focused PTR and earlier-retail exact-table tests | 45 best-effort / behavioral rows | Gameplay meaning, consumers, validation, coercion, and security/native semantics |
| Other new enums and metadata | Numeric generated values are available | evidence-required / unsafe | PTR-only publication tests and preservation on earlier profiles |
| Existing timed-signal, rounding, and math work | Prior focused tests or implementations exist for selected contracts | evidence-required pending row linkage | Exact occurrence-to-test/commit evidence and explicit limits |
| Removed declarations | Two generated removals | evidence-required / unsafe | PTR absence and excluded-profile preservation |

## Confirmed High-Priority Gaps

- Missing publication: `CreateFrameWithOptions`, `C_Intl`, `C_Weather`, new locale/weather enums, and `WEATHER_CHANGED`.
- Remaining table gaps: `table.freeze` and `table.isfrozen` need a frozen-table state model; `string.trim` remains a separate 12.0.0 mismatch rather than a 12.1.5 occurrence.
- Missing additive helpers: documented string extensions, selected `C_ActionBar`, `C_PvP`, `C_LFGInfo`, `C_UnitAuras`, aura-option normalization, and script-bucket throttle limits.
- Native evidence remains required for weather values, Training Grounds IDs, active LFG state, castbar token behavior, Unicode/locale semantics, and protected/security behavior.

## Implementation Order

1. Link already-proven behavior to exact occurrence rows without widening claims.
2. Add focused PTR publication tests for deterministic enum/event/global surfaces.
3. Correct small source-defined contracts such as table count migration and profile-gated removal.
4. Keep stateful, Unicode-heavy, timing-sensitive, and security-sensitive rows evidence-required until native evidence exists.
5. Revalidate manifest/checklist/inventory parity after every row update.

## Sources

- [`data/patch-api/sources/12.1.5-register.json`](../../../data/patch-api/sources/12.1.5-register.json) — frozen normalized source occurrence register.
- [`data/patch-api/12.1.5.json`](../../../data/patch-api/12.1.5.json) — machine audit manifest.
- [Patch API audit manifest specification](../../specs/patch-api-audit-manifest.md) — generator and evidence contract.

## See Also

- [[patch-12-1-5-occurrence-inventory]] — all 449 exact rows and current dispositions.
- [[patch-12-1-api-audit]] — preceding 12.1.0 audit.
- [[client-profiles]] — PTR profile/source selection.
