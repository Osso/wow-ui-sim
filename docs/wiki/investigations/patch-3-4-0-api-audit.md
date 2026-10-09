# Patch 3.4.0 Wrath Classic source/model-contract audit

Frozen page 16896, revision 165668, timestamp 2022-11-19T13:43:58Z; audited 2026-10-09 against base `34c303c108d55e6c9d4919878abbb9a0efc08c00`. Source TOC **30400**, caption 2.5.4 build 44400 → 3.4.0 build 45435, September 1, 2022. Wrath Classic is separate from original retail Wrath and Cata Classic.

## Literal coverage matrix

[Ledger](../../../data/patch-api/sources/3.4.0-page-coverage.json): **386 nonblank raw rows**, 331 inventory occurrences, four summary contracts, 51 metadata/markup rows; six blank lines. Every raw occurrence retains its line/text. No linked API pages or diffs reconstructed.

| Section | Added | Removed | Proof limit |
|---|---:|---:|---|
| Global API | 228 | 27 | Identities only, no callable signatures |
| Widgets | 10 | 0 | Method identities, no inputs/outputs/state contracts |
| Events | 43 | 3 | Availability claims, no payload/trigger/dispatch contracts |
| CVars table | 19 | 1 | Added includes 18 CVars and command `SetRaidDifficulty` |

331 = 255 globals + 10 widget methods + 46 events + 19 CVars + one command. Directions 300 added / 31 removed. Eighteen literal defaults and all CVar metadata retained. `SetRaidDifficulty` is explicitly under Commands despite its `type=cvar` template: outside-instance restriction stated; command syntax/output/state transition unprovided. `TTSUseCharacterSettings` removal is a bare-name source occurrence, not omitted. All 335 substantive raw rows remain **UNPROVEN**; zero runtime observations, native observations or closed runtime gaps.

**Signatures:** 265 callable inventory occurrences have unknown arguments/returns. Separate `UnitAura` partial return contract yields 266 signature-limit rows: named `shouldConsolidate` immediately before variable returns; numeric slot, type/value derivation and prior/variable tuple unspecified. Zero complete explicit signatures. Summary occurrences are not double-counted as table publication inventory.

| Line | Source contract | Model/behavior limit |
|---:|---|---|
| 4 | Subset of retail 9.2.5 changes; UnitPopup breaking changes | No explicit subset membership or UnitPopup transitions; not a Wrath successor |
| 5 | `_Wrath` and alternative `-WOTLKC` TOC suffixes | Static loader candidates support `_Wrath`/`-Wrath`, not `-WOTLKC`; collision precedence not established. Candidate loader gap, not executed failure/native proof |
| 6 | `UnitAura.shouldConsolidate` before variable returns | Existing aura model has no consolidation input and tuple has ten fixed returns; do not invent boolean or numeric slot |
| 7 | `CURSOR_UPDATE` replaced by `CURSOR_CHANGED` | Both table identities preserved; no trigger/payload/order contract or executed retirement |

## Supported profile and bounded model decision

`src/client_profile.rs` supports `client-wrath`, cache `wrath`, interface **38001**: configured 3.3.5-era architecture, not native Classic TOC30400 proof. [Profile observation](../../../data/patch-api/evidence/3.4.0-session-2026-10-09/profile-observation.json) is static at the exact base. No cache inspection/write or runtime load.

[Model context](../../../data/patch-api/evidence/3.4.0-session-2026-10-09/model-context.json) preserves historical code snippets with file hashes and line ranges. `src/loader/mod.rs:189-266` exposes the suffix candidate gap. `src/lua_api/globals/auras.rs:680-697,808-848` and `src/lua_api/game_data.rs:78-100` establish current tuple and missing consolidation state. No source-established backing value for `shouldConsolidate`; changing the shared tuple would affect other profiles and guess unavailable signature details. The page literally states: “An alternative suffix of” `-WOTLKC` “is also supported for consistency with other legacy TOC suffixes.” Loader alias eligibility for Classic30400 versus configured38001 remains parent integration scope; this source-only slice does not widen that runtime target. No runtime implementation, shim, fallback, retirement, vendor/Wowless or cache mutation.

Only Wrath Classic 3.4.1 (`61fd58823`, 333 source inventory / zero runtime observations), 3.4.2 (`990dae19c`, 155 source inventory), 3.4.3 (`48ab8e1c3`, zero inventory / seven raw rows) queued. No actual successor register applied; `later_registers` empty. References are historical source proof only. Agent205's uncommitted 3.4.2 factory/classifier work is neither copied nor credited. Parent owns current factory/publication measurement and integration after 3.4.1; endpoint 1.0.0 unchanged.

## Historical evidence and development proof

Before copying, checked manifest identity, response identity, literal response content, bytes and hashes. Raw 17530 bytes, SHA-256 `080a68a5f4f24e10962f4a63f3db6cb025038729c83ddb9c081735943b6a6ac2`; response 18301 bytes, SHA-256 `750bc9d2a3e9d3605718488f73890051308671bd612c385b5372914ce79f359e`. Exact manifest row retained. Historical own generator/extractor copied from the base; shared tooling unchanged. Limited text extract uses `--text-only --canonical-patch-navigation`; complete raw ledger remains authoritative.

[Dynamic validator](../../../data/patch-api/evidence/3.4.0-session-2026-10-09/validate.py) derives counts/statuses/kinds/directions/headers/signature limits from sealed own inputs, without Git, cache access or global register discovery. Own serialized development RED rejected missing accounting (`None != 331`); GREEN **8/8** at `2497ec7a8` includes 987 individual row-omission controls and fabricated partial-signature/profile/credit controls. Recorded-flag extraction reproduces nine rows (five metadata/four pending). [Proof ledger](../../../data/patch-api/evidence/3.4.0-session-2026-10-09/source-proof.json) retains exact commands, revisions, scope hashes and logs. Python manually formatted; no Rust edits. [Portable controls](../../../data/patch-api/evidence/3.4.0-session-2026-10-09/portable-proof.json) at `30071d654b69f7377cfbe4a9690ff1a0c0fce167`: serialized ledger and GREEN-log tampering each exit 1 at the exact seal; originals byte-restored/hash-checked. Relocated Git-free archive replay exits 0 with 17 historical sealed inputs, identical source summary and no runtime/native credit. Archive SHA-256 `af8cb1165ea0f7b79e42d5f48c5a60583b7ce770f74248ace9c8d8f23cc5f606`; complete member hash mapping in `portable-controls.log`. Evidence plus source fixtures 509611 bytes at controls, below 5MB. Later receipt/doc/seal additions do not alter tested source/validator/test/historical-tool/model/profile/control bytes. These prove source accounting only, never API execution. Parent owns all publication/check/build/readability/smoke/final gates. No delegation, push/merge, cwd switch, provider/model/retry change.

## Sources

- [Raw](../../../data/patch-api/sources/3.4.0-api-changes.wikitext), [limited text](../../../data/patch-api/sources/3.4.0-api-changes.txt), [evidence](../../../data/patch-api/evidence/3.4.0-session-2026-10-09/).
- [Tracked accounting spec](../../specs/patch-3-4-0-source-accounting.md).

## See Also

- [[client-profiles]] — actual runtime profile boundary.
- [[patch-4-4-2-api-audit]] — separate Cata Classic source line, not a successor.
