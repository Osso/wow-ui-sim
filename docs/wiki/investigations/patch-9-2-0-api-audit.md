# Patch 9.2.0 API page audit

Page 496546, revision 4788278 (June 6, 2022, 01:53:21 UTC), retrieved October 7, 2026. Current retail carries 12.1.0. Branch `p920-page` starts from master `572a77d84`.

## Source accounting

80 inventory occurrences: 55 Global API (51 added, four removed), four widgets, thirteen events, seven CVars and one command. Header counts match. Initial parser treated unbolded Commands as a CVar and omitted HTML CVar defaults. Behavioral fixture reproduces both errors; parser now retains defaults and command identity. Existing source/register inputs stay unchanged. Extract has fourteen nonblank rows; source/build caption remains separately accounted.

## Runtime findings

Initial cached sweep records 28 publication gaps. Two unused removed C_PvP members (`GetSpecialEventDetails`, `GetSpecialEventInfo`) were fabricated by namespace lazy lookup. Mark them retired using the existing retail-epoch module; no classic profile loads it. Qualified and bare-name cached Lua scans have zero consumers; whole src/tests scan has no pre-existing callers. Deprecated vendor wrappers untouched. Repeated lookup behavioral regression reproduces the failure.

26 publication gaps retained; added lookup-only stubs do not qualify as modeled producers. C_CharacterServices.GetCharacterServiceDisplayOrder stays a gap: no qualified consumer, but bare name is defined/called at Blizzard_GlueXML/Mainline/CharacterSelect.lua:1493/1542; conservatively retained under the task rule, not claimed as the same namespace API. Removed global GetBattlefieldFlagPosition is already absent; current qualified C_PvP consumer in BattlefieldFlagDataProvider.lua:43 is its successor, not a reason to remove the successor.

## Coverage ledger and boundaries

[Ledger](../../../data/patch-api/sources/9.2.0-page-coverage.json) accounts for all **95 unique IDs**: 80 inventory, fourteen nonblank extract and one historical build caption. Twelve bounded-coverage rows prove current absence (including two closed retirements); 42 partial-development-green rows prove publication/registration only; 32 audit-pending rows comprise 26 publication gaps plus six summary contracts; nine metadata-only rows receive no runtime credit. [Gap review](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-gap-review.json) gives every retained ID its literal, observation and specific missing-model/scope reason. [Extract scout](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-extract-scout.json) maps all fourteen extract rows back to exact source lines.

Pending summary contracts: click-to-cast routing, mouseover target/modifier routing, device LED transitions, notch-aware layout, Cosmic combat-log school payload and raid-marker 9–18 rejection. No unrelated test or static declaration receives behavior credit. CMAA2Quality remains current default `3` versus historical page `2`; this is a separately reported default mismatch, not a new publication gap or justification to overwrite current retail state.

## Verification

Compiled source/test revision `3d2de0d1b`. **28 publication sweeps plus existing animation-factory case pass (29/29)**, covering 6,459 inventory rows with exact per-patch fixtures. Own sweep: 80 rows, 54 OK, 26 gaps. Bare repeated-lookup regression and cached PvP retirement prefork pass; Mists legacy lookup regression passes. Whole src/tests scan before and after finds no pre-existing callers of the two changed members; no caller migration or additional affected surface tests needed. Retirement module is gated by retail-12-0-0, absent under classic profiles. No vendor deprecation wrapper deleted or patched.

FIRST_FRAME_RENDERED added → removed negative control produces exactly **26 → 27 gaps**, zero resolved failures, unchanged 80 IDs; expected prefork exit 1. Parser RED reproduces Commands/default loss; GREEN passes twelve generator and 23 extractor fixtures. `cargo fmt --check`, default `cargo check`, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` pass, with **zero non-vendor warnings**. Six pre-existing iced manifest deprecations and their summary remain unsuppressed. Separate retail build and bounded startup exit zero with `[]`.

First startup invocation accidentally used the Mists executable rebuilt by the preceding Mists test in this worktree's own target. Saved classic errors are explicitly invalid retail proof; fingerprint confirmed client-mists. Rebuilding the retail binary restored correct profile and clean startup. No simulator/vendor change made to hide those classic startup errors, and no sibling target used.

[Proof ledger](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-proof.json) records commands, exact code scope/revision, exits, outputs and invalidation. Build-marker syntax failure was corrected before genuine retirement RED; its compiler failure is not credited as behavior proof. Changed Rust manually audited: flat retirement data and existing effect-revealing mark call; input-only sweep specification; bounded repeated-lookup tests. No readability violations, warning suppressions or new placeholders. Runtime proof remains valid after documentation/evidence-only updates.

## Sweep table

Publication/absence only, not full API compatibility. Every fixture matches exactly.

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 9.2.0 | 80 | 54 | 26 | pass |
| 9.2.7 | 3 | 3 | 0 | pass |
| 10.0.0 | 639 | 466 | 173 | pass |
| 10.0.2 | 416 | 267 | 149 | pass |
| 10.0.5 | 93 | 66 | 27 | pass |
| 10.0.7 | 70 | 44 | 26 | pass |
| 10.1.0 | 129 | 93 | 36 | pass |
| 10.1.5 | 101 | 69 | 32 | pass |
| 10.1.7 | 48 | 34 | 14 | pass |
| 10.2.0 | 150 | 120 | 30 | pass |
| 10.2.5 | 59 | 45 | 14 | pass |
| 10.2.6 | 220 | 200 | 20 | pass |
| 10.2.7 | 104 | 68 | 36 | pass |
| 11.0.0 | 495 | 329 | 166 | pass |
| 11.0.2 | 34 | 22 | 12 | pass |
| 11.0.5 | 48 | 38 | 10 | pass |
| 11.0.7 | 98 | 70 | 28 | pass |
| 11.1.0 | 116 | 97 | 19 | pass |
| 11.1.5 | 125 | 89 | 36 | pass |
| 11.1.7 | 48 | 40 | 8 | pass |
| 11.2.0 | 162 | 135 | 27 | pass |
| 11.2.5 | 163 | 118 | 45 | pass |
| 11.2.7 | 508 | 414 | 94 | pass |
| 12.0.0 | 1010 | 989 | 21 | pass |
| 12.0.1 | 225 | 222 | 3 | pass |
| 12.0.5 | 363 | 352 | 11 | pass |
| 12.0.7 | 174 | 171 | 3 | pass |
| 12.1.0 | 778 | 773 | 5 | pass |

## Preservation and integration

All **143 pre-existing source/register/ledger inputs remain byte-identical**. All 54 previous extract-mode outcomes are unchanged; fifteen pre-existing failures, including both modes for 12.0.5/12.0.7/12.1.0, remain untouched. Own extract passes both modes. Extractor unchanged; register parser changes are additive and covered by a concrete HTML/plain-label fixture. No prior register rewritten.

[Artifact validator](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/validate.py) checks every ID/count/literal/source hash, exact gap and negative-control identity, later fixtures, preserved bytes/mode outcomes and proof receipts without rerunning runtime tests.

Concurrent 9.2.5 register was read-only. **No retained gap has an add/remove supersession there**. Only `wt-global-api-C_LFGList.DoesEntryTitleMatchPrebuiltTitle-57` intersects: 9.2.5 changed row `wt-global-api-C_LFGList.DoesEntryTitleMatchPrebuiltTitle-145` corrects `Enum.LfgEntryPlaystyle` → `Enum.LFGEntryPlaystyle`, without altering publication. [Comparison](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-p925-intersections.json). One-line placeholder remains first in later-register list; main thread must insert 9.2.5 and recompute fixtures at integration.

All commands explicitly run with this worktree as cwd; own target only. No agents/models, push, merge, canonical working-file edits, sibling worktree edits, vendor/cache edits or hardware changes.

## Sources

- [Source response](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-fetch.json) — original revision and wikitext.
- [Register](../../../data/patch-api/sources/9.2.0-wikitext-register.json) — every inventory ID.
- [Spec](../../specs/patch-9-2-0-publication-sweep.md) — proof requirements.
- [Qualified scan](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-retirement-qualified.txt) and [bare scan](../../../data/patch-api/evidence/9.2.0-session-2026-10-07/p920-retirement-bare.txt) — read-only cached references.

## See Also

- [[patch-9-2-7-api-audit]] — adjacent later audit conventions.
- [[patch-10-0-0-api-audit]] — full publication accounting boundary.
