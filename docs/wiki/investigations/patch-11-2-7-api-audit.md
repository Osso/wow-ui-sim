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

At `9cc60a03d`, six isolated sweeps pass with exact baselines: 11.2.7 387/508 OK, 12.0.0 987/1010, 12.0.1 222/225, 12.0.5 352/363, 12.0.7 171/174, 12.1.0 773/778. All later baseline files remain unchanged. One unsuperseded `C_BattleNet.InstallHighResTextures` row flipped to removed yields exactly one new gap and one changed observation (121 → 122; expected exit 101). Final formatting, retail check, Mists test check and startup pass; Mists has no non-vendor warnings and startup exits 0 with `[]`. Results are retained in the proof ledger/spec.

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


## Publication-gap follow-up — 2026-10-06

At runtime `0523068b2`, 27 of the original 121 gaps close: 25 host/state-backed producers and two permanent unsupported-3D methods; 94 remain explicit gaps. See [per-ID outcomes](../../../data/patch-api/evidence/11.2.7-session-2026-10-06/p1127-gap-outcomes.json), [cached contract/consumer evidence](../../../data/patch-api/evidence/11.2.7-session-2026-10-06/p1127-gap-contracts.json), and [contract/proof](../../specs/patch-11-2-7-publication-sweep.md#follow-up-acceptance--runtime-0523068b2). Earlier counts above are historical; page-coverage ledgers remain frozen.

| Exact surface | Outcome | Proof level / boundary |
|---|---:|---|
| HouseEditor availability/activity/entry/leave | 8 modeled | Host status/results and deferred replies; no server eligibility/approval fabrication |
| Dye category/color/ownership queries | 6 modeled | Explicit catalog, carried/bank consumables and fresh rooted swatches; no native catalog acquisition |
| Housing location/tracking, exterior door hover, room count | 6 modeled | Host location plus existing state; GUID selection/geometry excluded |
| Binding contexts | 3 modeled | Independent idempotent active-set queries; no routing/priority/migration |
| Neighborhood invitation preference globals | 2 modeled | Existing per-environment boolean now Retail-gated; no persistence |
| ModelSceneActor collision-bound preference | 2 permanent workarounds | No-op setter, false getter; 3D intentionally unsupported |
| Remaining original IDs | 94 still-gap | Reasons and exact cached signatures/consumers retained per ID |

The remaining event is a concrete source conflict, not a cheap registration omission: current cached `HousingUIDocumentation.lua:673` publishes `HOUSE_LEVEL_CHANGED`; current blueprint/editor Lua registers it, while the later page register expects removal. Acquisition failures are also retained: cached objects are world-associated ScriptObjects, not proof of a generic `CreateFrame` factory.

Six isolated sweeps pass: 414/508, 987/1010, 222/225, 352/363, 171/174, 773/778 OK. Only the 11.2.7 exact-gap fixture changes (121 → 94); later fixtures remain unchanged. Eleven new behavior tests have passing proof; existing housing-pending 20 and keybinding 73 pass both before/after. Mists has no non-vendor warnings, default check/formatting pass, startup exits 0 with `[]`. One initial dye test assumed an unavailable bare-env SetRGB helper; master comparison proves the assumption error, and direct mutable DTO fields now exercise the real isolation boundary. No native parity or full-project-suite claim.

Readability audit retains one advisory long serializer; no warning suppression, deep nesting, vendor patch or unmodeled placeholder added. All API producers stay in `src/c_api/` except the real non-C invitation preference and documented permanent 3D widget compatibility.

Ledger credit (merged at `33527cc6c`): capability `gap-closures-11-2-7` moves the 27 closed rows from audit-pending to partial-development-green (25 modeled) or bounded-coverage (2 permanent no-ops). Post-merge main-thread proof: seven sweeps 11.2.5–12.1.0, dye_color 2, house_editor 8, housing 291, p1127 11, Mists `cargo check --tests` clean. Ledger now 353 green / 11 bounded / 59 metadata / 104 pending.
