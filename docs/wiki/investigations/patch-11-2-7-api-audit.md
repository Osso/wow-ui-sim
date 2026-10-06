# Patch 11.2.7 API page audit

Retained page 649551, revision 6726771 (May 25, 2026), captured October 6, 2026. Audit pipeline accounts for 508 inventory occurrences and 19 supplemental extract rows. Publication evidence observes the default retail 12.1.0 surface; no historical 11.2.7 runtime or behavioral parity is claimed.

## Source and parser defects

This revision starts its Global API table directly below Consolidated changes, without a Global API subsection, and contains a separate Commands table. The existing generator silently omitted both. Behavioral RED fixture reproduces missing global/command rows; parsing now retains them, maps Command occurrences to the shared cvars/command classifier, and preserves all ten header counts. All five existing registers regenerate byte-identically.

The non-inventory extractor previously leaked the unheaded inventory and errored on CVar templates. Start inventory exclusion at its wikitable and resume at Structures as well as Enums. Repeated fixtures preserve existing 12.0.0/12.0.1 extraction; this page contributes two prose statements and eight structure parent/member statements, plus nine editorial rows.

## Runtime defects and fixes

`JoinBattlefield` remained explicitly registered after its 11.2.7 removal. `C_CharacterServices.RPEResetCharacter` and `C_ReturningPlayerUI.AcceptPrompt`/`DeclinePrompt` were raw-absent but fabricated by namespace lookup. Stop the global publisher and mark retired namespace keys from the earliest supported retail epoch, 12.0.0. Classic profiles retain their independent contracts. Repeated raw/ordinary lookup regression and the existing `C_PvP.JoinBattlefield` queue producer pass; no successor behavior is invented.

Ten 11.2.7 console command records were absent from the modeled catalog. Add exact names as Command records for supported retail epochs, preserving classic catalogs and later 12.0.7 records. Catalog publication is not command execution, neighborhood service or native metadata proof.

## Coverage matrix

| Statements/features | Count | Proof level / remaining boundary |
|---|---:|---|
| Inventory add/change publication | 328 | Partial-development-green; publication only |
| Unsuperseded source removals | 9 | Bounded absence; no earlier/native behavior claim |
| Superseded inventory OK | 50 | Metadata-only; latest later add/remove expectation observed |
| Inventory gaps | 121 | 110 namespace members, six globals, two scriptobject acquisition failures, two intentionally unsupported 3D methods, one superseded event removal |
| Extract statements | 10 | Audit-pending; five ranked batches, no behavioral credit |
| Extract editorial rows | 9 | Metadata-only |

527 unique source IDs: 328 partial, nine bounded, 131 pending, 59 metadata. All 135 initial gaps reviewed; 14 close. `HOUSE_LEVEL_CHANGED` is superseded by 12.0.0 removal yet still accepted; existing later sweep also retains this gap. Acquisition failures are not proof that housing scriptobjects are natively absent.

## Supersession and epochs

Apply all five later registers, oldest first. The shared sweep stores latest later add/remove per symbol; changed entries preserve publication. An older-than-all input needs no index-dependent code change. Fifty-one rows have reversed later expectations; 50 observe the new expectation, one event remains a gap. 11.2.7 removals are baseline for every supported retail epoch, **not** every client profile. No new 11.x feature exists or is added.

## Development proof

At `9cc60a03d`, six isolated sweeps pass with exact baselines: 11.2.7 387/508 OK, 12.0.0 987/1010, 12.0.1 222/225, 12.0.5 352/363, 12.0.7 171/174, 12.1.0 773/778. All later baseline files remain unchanged. One unsuperseded `C_BattleNet.InstallHighResTextures` row flipped to removed yields exactly one new gap and one changed observation (121 → 122; expected exit 101). Final checks and startup results are retained in the proof ledger/spec.

No agents/models, push, merge, vendor edits, Blizzard monkey-patches or existing page-coverage edits. This is main-thread simulator evidence, not independent/native acceptance.

## Sources

- [Publication contract](../../specs/patch-11-2-7-publication-sweep.md)
- [Source provenance](../../../data/patch-api/sources/11.2.7-api-changes.provenance.json)
- [Coverage ledger](../../../data/patch-api/sources/11.2.7-page-coverage.json)
- [Per-ID review](../../../data/patch-api/evidence/11.2.7-session-2026-10-06/p1127-gap-review.json)
- [Extract scout](../../../data/patch-api/evidence/11.2.7-session-2026-10-06/p1127-extract-scout.md)
- [Proof ledger](../../../data/patch-api/evidence/11.2.7-session-2026-10-06/p1127-proof.json)

## See Also

- [[patch-12-0-1-api-audit]] — later-page pipeline and publication limits.
- [[console-command-registry]] — catalog versus execution.
- [[client-profiles]] — profile/epoch boundaries.
