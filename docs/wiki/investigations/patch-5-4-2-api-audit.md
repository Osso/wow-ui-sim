# Patch 5.4.2 API audit

Pinned from scratch on 2026-10-08: Warcraft Wiki pageid 262849, revision 2543251 (2014-02-20). Parent pageid 151415 revision 6441253 states December 10, 2013 release, TOC 50400 and latest build 17688; the API tables compare 5.4.1.17538 → 5.4.2.17688. This is the 2013 retail chain, not Mists Classic 5.5.x. Neither page is a redirect or navigation stub.

## Source and accounting

The page contains 68 inventory occurrences: seven new/two removed global APIs, two FrameXML additions, 55 events and two slider methods. All caption counts match. The opt-in `--mists-automated-diff` generator preserves bare removals and widget ownership; the extractor retains three prose contracts and five enum values instead of dropping the Lua Enums table. Saved sources use recorded flags; no default parser behavior changed.

The ledger accounts for all 85 identities: 68 inventory plus 17 retained extract rows. Discovery found 39 publication gaps: 37 glue/auth/patcher events, fastrandom, and the IsOnGlueScreen name collision. No new runtime registrations, models or shims were added. Existing publication/absence covers 29 inventory rows; it does not prove historical behavior. Five historical numeric rows remain pending because current documented/deprecation-alias values are 0–4, not 1–5. Nine extract rows are metadata; eight extract rows remain pending.

## Behavior and problematic cases

| Contract | Current proof | Remaining boundary |
|---|---|---|
| Explicit guild name-realm | Cached query preserves Arthas-Silvermoon, follows Jaina-Proudmoore replacement, nil on roster removal | GuildMember stores only name/rank/online; no separate realm or local identity source. Bare Jaina stays bare. Automatic qualification unmodeled. |
| Autocomplete priorities | Current cached LE_* aliases equal documented Enum values 0–4 | Pinned 2013 values 1–5 differ. No historical numeric/ranking/lifecycle parity claim. |
| Slider methods, full-name/ambiguation/color APIs | Inventory publication; existing slider tests pass 15/15 | No 2013 signature/native parity claim. |
| 37 missing events | Each rejected registration recorded | No authentication/network/launcher/patcher backend supplies lifecycle/payload. Registering arbitrary names would be a shim. |
| RNG and secure environment | fastrandom absent; raw prose retained verbatim | Fast/secure generator separation, performance and secure-environment exclusion unmodeled. Prose redirects random/Math.random to securerandom while diff removes that exported name; no alias inferred. |
| IsOnGlueScreen | Later 6.0.2 removal meets current boolean export; whole-word consumers retained | Boolean is consumed by current Blizzard UI; do not delete or patch consumers. |

## Retirements and ordering

No new retirements. StartUnratedArena and securerandom already have nil raw/ordinary lookup. Whole-word `/usr/bin/grep -R -n -w` cached scans exclude *Documentation* and source/test scans retain all lines, including possible pcall and conditional references; both removed globals have zero matches. Pinned a9d7c9566, p547-page 531ada0628ac9100228f228cce1fb96080ac4f16 and p548-page 2caa653ace4e52ae3b918bba8ad322bb80b06671 register trees contain no re-additions. Grep evidence is untruncated. Cache inventory/provenance is recorded, not treated as empty by assumption.

Queued 5.4.7 then 5.4.8 placeholders start later_registers, followed by 6.0.1, 6.0.2 and the remaining merged retail chain. Replace placeholders during ordered integration. No 5.5.x Classic registers included. C_ProductChoice.GetNumSuppressed absence comes from existing 8.3.0 supersession; this audit does not retire it again.

## Verification

Discovery initially fails on the exact 39 unaccounted gaps. All publication sweeps pass 56/56; own cached cases 3/3, guild queries 22/22 and sliders 15/15 pass. All six Python fixture scripts pass (86 fixtures). All 55 registers and 52 extracts reproduce; three inherited extract failures (12.0.5, 12.0.7, 12.1.0) are unchanged, not repaired or hidden. Format and Mists tests check pass with zero non-vendor warnings; six inherited iced manifest warnings remain. Negative control adds exactly one gap (39 → 40) and fails as expected. Portable validator gate passes 37/37 in the clean committed checkout and 38/38 after synthetic later changes (including the synthetic validator). Own sealed-log tampering fails; the original log bytes are restored. The eight pending extract rows and 39 publication gaps are limitations, not passing behavior claims. No full integration suite. Runtime/src files are unchanged, so a shared-runtime startup-regression comparison is not required for this audit.

## Sources

- [Pinned provenance](../../../data/patch-api/sources/5.4.2-api-changes.provenance.json), [page](../../../data/patch-api/sources/5.4.2-api-changes.wikitext), [extract](../../../data/patch-api/sources/5.4.2-api-changes.txt), [register](../../../data/patch-api/sources/5.4.2-wikitext-register.json), [ledger](../../../data/patch-api/sources/5.4.2-page-coverage.json).
- [Session evidence](../../../data/patch-api/evidence/5.4.2-session-2026-10-08/).
- [Spec](../../specs/patch-5-4-2-publication-sweep.md).

## See Also

- [[patch-6-0-2-api-audit]] — later removal and register/reproduction template.
- [[patch-audit-validator-portability]] — clean/later-audit gate and historical pins.
