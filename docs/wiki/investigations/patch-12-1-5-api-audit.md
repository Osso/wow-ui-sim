# Patch 12.1.5 API Audit

PTR `12.1.5.69594` adds or changes a broad generated API surface relative to `12.1.0.69587`. The frozen register contains 449 semantic occurrences. Current manifest disposition is 326 best-effort and 123 evidence-required rows, with no untriaged rows; this remains an incomplete conformance audit.

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
| PTR string extensions | Commit `65704757d` with focused PTR byte-preserving literal/case matching and one-sided byte-set trimming tests, plus earlier-retail absence proof | 5 best-effort / simulator behavioral rows | Byte-set/literal/case/explicit-nil choices are simulator assumptions; Unicode, locale, coercion, `AllowedWhenUntainted`, security, and native semantics remain unproven |
| `C_Intl` locale-context storage | Commit `61bf36927` with focused PTR opaque-userdata storage, current-locale consistency, mutation/failure atomicity, and earlier-retail absence proof | 5 best-effort / simulator behavioral rows | Context algorithms, segmentation, collation, formatting, canonicalization, identifier validation, and taint/secret semantics remain unproven |
| `C_Intl.Length` / `LuaLocaleContext.Length` | Commit `d491728e1` with focused PTR scalar-count tests across locale contexts, UTF-8 validation, and earlier-retail absence | 2 best-effort / simulator behavioral rows | Scalar unit, errors, cstring/NUL, MayReturnNothing, secret/taint, coercion, and native semantics remain unproven |
| `C_Intl.Normalize` / `IsNormalized` | Commit `33ff9c993` with focused PTR ICU 2.1.1 NFC/NFD/NFKC/NFKD normalization and predicate tests plus earlier-retail absence | 2 best-effort / simulator behavioral rows | Invalid-input/no-return policy, Unicode data-version equivalence, cstring embedded-NUL, `AllowedWhenUntainted`, coercion, and native semantics remain unproven |
| `C_Intl.TransformLocale` / `LuaLocaleContext.TransformLocale` | Commits `15a52023b` and `1b287e67e` with focused PTR ICU 2.1.1 eight-selector transform tests plus earlier-retail absence | 2 best-effort / simulator behavioral rows | WoW-tag translation, BCP-47 output, empty components, variant ordering, syntactic parent policy, errors, `MayReturnNothing`, CLDR/Unicode-version equivalence, cstring, security, coercion, and native semantics remain assumptions or unproven |
| `C_Intl.FindBreaks` / `LuaLocaleContext.FindBreaks` | Commits `d86f5c177`, `2744e14ba`, and `e30cf16b0` with focused PTR ICU 2.1.1 grapheme, word, sentence, and line byte-boundary tests plus earlier-retail absence | 2 best-effort / simulator behavioral rows | Zero-based endpoints, empty-input handling, locale-independent segmentation, validation, `MayReturnNothing`, Unicode-version equivalence, cstring embedded-NUL, `AllowedWhenUntainted`, coercion, and native semantics remain simulator assumptions or unproven |
| `C_Intl` / `LuaLocaleContext` casing | Commits `f94a5be6d`, `0081d03f0`, and `bd3d6a330` with focused PTR ICU 2.1.1 lower, upper, full-fold, and per-word titlecase tests across global and context APIs plus earlier-retail absence | 8 best-effort / simulator behavioral rows | Titlecase segmentation/default options, locale-tag translation, accepted identifiers, invalid-input and `MayReturnNothing` policy, Unicode-version equivalence, embedded-NUL, `AllowedWhenUntainted`, coercion, and native semantics remain unproven |
| `C_Intl` / `LuaLocaleContext` collation | Commit `974e38888` with focused PTR ICU 2.1.1 five-strength comparison and binary sort-key tests across global and context APIs plus earlier-retail absence | 4 best-effort / simulator behavioral rows | Comparison signs, raw key bytes/termination, locale parsing, errors, `MayReturnNothing`, Unicode/CLDR-version equivalence, embedded-NUL, `AllowedWhenUntainted`, coercion, and native semantics remain unproven |
| PTR locale option enums | Commit `50958d938` with focused client-profile exact-map/metadata and earlier-retail absence proof for `BreakType`, `CollationStrength`, `CurrencyNameStyle`, `DateTimeStyle`, `LocaleTransform`, `NormalizationForm`, `NumberStyle`, and `PluralType` | 45 best-effort / behavioral rows | `C_Intl`, Unicode, locale algorithms, formatting/collation/normalization behavior, coercion, and security/native semantics |
| `Enum.WeatherType` | Commits `286d0d3f7` and `17c085065` with focused PTR exact-map/metadata and earlier-retail absence tests | 6 best-effort / behavioral rows | Weather state, events, intensity, rendering, coercion, security, and native semantics |
| `SimpleScriptRegion` rounding controls | Commit `fd2ab64d6` adds earlier-retail Frame/Texture/FontString absence proof; existing PTR capture replay proves flag/default, dirty-layout, and captured geometry behavior | 2 best-effort / behavioral rows | Protected and secret enforcement, inheritance, untested widget types, half-pixel ties, rendering, hit testing, coercion, and native-edge behavior |
| Player-data flag enums | Commit `96b91aa4b` with focused PTR exact one-member maps/metadata and earlier-retail absence tests | 4 best-effort / behavioral rows | Player-data, logging, gameplay, consumers, coercion, security, and native semantics |
| `C_Weather`, `WeatherInfo`, `WEATHER_CHANGED` | Commits `c77963250`, `42457f475`, and `b4943dcf6`; focused PTR simulator-state/snapshot and explicit event-injection tests plus earlier-retail absence | 5 best-effort / simulator behavioral rows | Native weather values, initial intensity/range, automatic transitions, event timing/payload, rendering, and security semantics |
| Duration core | Commits `9aa4a1eb7`, `89a71308d`, `7c3a41b67`, and `5149f5d46`; focused manual/default-clock progression, rewind, endpoint, reset, validation, and cooldown-proxy consumer tests | 7 best-effort / simulator behavioral rows | Rate/modifier formulas, clock/default/reset/zero rules, validation, Seconds/FrameTime identity, secret/protected/forbidden behavior, and native timing semantics |
| `C_ActionBar.IsMacroActionWithShowTooltip` | Commits `80d312c5a`, `e53298cce`, and `96c186c1c`; focused PTR macro-body, assignment/edit/move/replacement/deletion tests and earlier-retail absence | 1 best-effort / modeled behavioral row | Directive case/line/token parsing and validation are simulator assumptions; macro conditional/token resolution, secret/taint/protected behavior, and native valid-slot semantics remain unproven |
| `C_LFGInfo.GetActiveLFGDungeonName` | Commits `cc15512ab` and `f96557ce2`; focused PTR instance-ID/catalog mutation, inactive/proposal-only, unknown-ID recovery, arity, existing-LFG-query, and earlier-retail absence tests | 1 best-effort / modeled behavioral row | Instance-ID-only selection, empty inactive result, unknown-ID error, and validation are simulator choices; queue/proposal precedence, native inactive/error behavior, security, taint, coercion, and native-edge semantics remain unproven |
| `Enum.FragmentID` and metadata | Commit `5e5e46dd9` with focused PTR and earlier-retail exact-table tests | 45 best-effort / behavioral rows | Gameplay meaning, consumers, validation, coercion, and security/native semantics |
| Cooldown threshold publication | Commit `88c705aa7` with focused PTR and earlier-retail numeric threshold tests | 3 best-effort / behavioral rows | Native defaults, rendering/display effects, units/conversion, type/coercion, and security/native semantics |
| Caster-name aura options | Commit `894333d48` with focused PTR default/boolean normalization and earlier-retail absence proof | 4 best-effort / behavioral rows | Caster-name rendering, realm formatting, class-color display, coercion, and security/native semantics |
| `Enum.BonusStatIndex` reserved range | Commit `bc786d5e5` with focused PTR exact-range and earlier-retail preservation tests | 60 best-effort / behavioral rows | Stat/gameplay meaning, consumers, validation, coercion, and security/native semantics |
| `Enum.CurioRarity` | Commit `867bcc8ea` with focused PTR and earlier-retail exact-map/metadata tests | 2 best-effort / behavioral rows | ItemQuality mapping, Curio/gameplay meaning, consumers, coercion, and security/native semantics |
| `Enum.TieredEntranceType` | Commit `16a89f7b2` with focused PTR and earlier-retail exact-map/metadata tests | 3 best-effort / behavioral rows | Placeholder meaning, consumers, validation, coercion, and security/native semantics |
| `Enum.TransmogIllusionFlags` | Commit `abcd763cb` with focused PTR exact-map/metadata and actual earlier-retail drift-preservation tests | 2 best-effort / numeric publication rows | Pinned base expects `1/2/4` and `1/4/3`; actual earlier retail intentionally remains `1/2` and `1/2/2`, so this is not base-conformance proof. Gameplay, security, and native semantics remain unclaimed |
| `Enum.TooltipDataLineType` | Commits `bb72d4031` and `28330eedd` with focused profile publication test | 3 best-effort / numeric publication rows | PTR publishes the exact 52-member target and metadata `0/51/52`; actual earlier retail intentionally retains 47 members with stale `0/43/44` metadata before/after bootstrap, not the pinned 50-member base. This is not base-conformance proof; tooltip rendering, gameplay, security, and native semantics remain unclaimed |
| `Enum.HousingResult` | Commits `e6ea692ba` and `950b8fb28` with focused profile exact-map/metadata tests | 43 best-effort / numeric publication rows | PTR publishes the exact 113-member target with `MessageTooLong = 71`, 41 shifted values, and metadata `0/112/113`; earlier retail retains the exact 112-member base table and metadata `0/111/112` before/after bootstrap. Housing gameplay, security, and native semantics remain unclaimed |
| `CreateFrameWithOptions` and `CreateFrameOptions` | Commit `e6712ab29` with focused PTR structured-constructor, lifecycle, validation-assumption, and earlier-retail absence tests | 9 best-effort / simulator adapter rows | PTR adapter publishes the constructor only on PTR, applies declared fields through existing allocation/template paths, and preserves earlier-retail absence. Lifecycle and validation choices are simulator assumptions, not native conformance; security, coercion, global structure publication, and native edges remain unclaimed |
| Other new enums and metadata | Numeric generated values are available | evidence-required / unsafe | PTR-only publication tests and preservation on earlier profiles |
| `C_Timer.NewTimedSignalMap`, `TimedSignalMap`, 8 methods, and callback | Commits `3bff28497`, `1be3abaa0`, `45b39beed`, and `ad23581f2`; focused PTR behavior and earlier-retail factory absence proof | 11 best-effort / behavioral rows | `RequiresTimedSignalMapAccess`, `TimedSignalMapEntry`/fields, empty-map `GetNextSignal`, FrameTime identity, coercion, security, and native semantics |
| Removed declarations | Two generated removals | evidence-required / unsafe | PTR absence and excluded-profile preservation |

## Confirmed High-Priority Gaps

- Remaining locale modeling: `C_Intl` has bounded opaque context storage, simulator scalar counting, and four-form ICU normalization; collation, formatting, canonicalization, segmentation, and other context methods remain unresolved. Native length units/errors, normalization failure behavior, Unicode-version equivalence, embedded-NUL cstrings, and security remain unproven. Stateful PvP and aura APIs remain missing. `C_Weather` has a bounded PTR simulator-owned state model; native weather values, transitions, intensity behavior, and event timing remain unresolved. `C_LFGInfo.GetActiveLFGDungeonName` has a bounded instance-ID/catalog model; native selection and inactive/error behavior remain unresolved. `CreateFrameWithOptions` has a bounded PTR simulator adapter; native lifecycle and validation remain unresolved. Duration core has bounded ordinary simulator behavior; native timing and security semantics remain unresolved.
- Remaining table gaps: `table.freeze` and `table.isfrozen` need a frozen-table state model; `string.trim` remains a separate 12.0.0 mismatch rather than a 12.1.5 occurrence.
- Missing additive helpers: selected `C_PvP`, remaining `C_LFGInfo`, `C_UnitAuras`, remaining aura-option normalization, and script-bucket throttle limits.
- Native evidence remains required for weather values, Training Grounds IDs, active-LFG selection/error behavior, castbar token behavior, Unicode/locale semantics, and protected/security behavior.

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
