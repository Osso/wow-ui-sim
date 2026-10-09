# Patch 4.0.1 API audit

Pinned historical retail page 129645, revision 1271877, timestamp 2012-09-06T23:25:23Z; supplied HTTP 200 receipt on October 9, 2026. No network requested or used.

## Source coverage

The page contains 416 labeled inventory occurrences plus three API-linked breaking references: 258 added/83 removed globals, 60 added/15 removed events, one changed global and two changed events. Eight retained non-inventory rows cover navigation, heading, four breaking statements and two automatically generated build contexts. No explicit structure or enumeration sections. Only explicit signature assertion: GetItemCooldown numeric item IDs; no inferred signatures for inventory-only names.

Existing colon/indented-list and combat-restriction flags do not parse NEW/REMOVED inventories under commented spaced headings with correctly typed event references. New `--cataclysm-labeled-inventory` handling is opt-in in both tools; no existing source flags change. Targeted parser/extractor fixtures pass 75/75 at 6c041621f. Default-feature cached retail RED sweep recorded all 419 observations and 119 mismatches. Headless build failed on 17 pre-existing generated registry references to GUI-gated modules; no unrelated registry fix. Exact command/revision/log receipts are retained.

## Bounded runtime change and remaining limits

Complete 197-symbol expected-absence scans cover 2,551 cached retail Lua files plus complete src/tests Rust/Lua/XML, including Lua strings. Fully qualified and bare spellings coincide for these global/event names. `rg` is unavailable; recorded Python regex uses ASCII whole-word boundaries and retains all matching lines, file hashes and paths in `retirement-scans.json` (1.1 MB).

Only CollapseSkillHeader/ExpandSkillHeader are retired: explicit source removals, no cached retail/test callers, only two no-op registrations. Factory RED proves both were callable. A separate 4.0.1 list gates modern-retail registration exclusion; Classic unchanged. Targeted GREEN at d2dfdb4a4: retail factory 1/1, retail prefork 2/2 (own sweep plus cached retirement), Mists factory 1/1. Only the two expected source rows change; gaps 119 → 117. Mists cached prefork is blocked by the target's required client-retail feature, not executed; no other Classic runtime proof.

The historical 117 publication gaps have individual observations and domain/consumer reasons in the sealed page ledger; current accounting after actual 4.x successor integration is 304 matches/115 gaps across the same 419 rows. Current consumers preserve GetQuestLogRewardHonor and later-retired CVar bitfield aliases; existing state APIs with tests are not swept into the unused-no-op change. Inventory identities alone provide no primary call signatures or state contracts for adding missing gameplay models. No new constants, aliases, shims or fallback behavior.

Four historical breaking statements remain separate native limits: implicit script globals; legacy per-power event non-emission and replacement payload/order; numeric-only GetItemCooldown; protected buff/weapon-enchant cancellation. C_Item.GetItemCooldown currently ignores its itemInfo and returns constants: no item cooldown backing model exists here, so narrowing its current retail input based on a 2010 statement would be unsound. Dedicated signature row preserves this gap. No native client, structure or enum parity claim.

## Targeted evidence

[Command receipts](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/) retain exact argv, revisions, environment, scope and logs. Negative scratch register changes one published symbol without changing row count: exact gaps 117 → 118, own sweep fails as required. Ledger derives 428 source IDs: 302 bounded publication/absence rows, 117 publication gaps, four prose limits, one separate signature gap and four metadata rows. No structure/enumeration statements exist in this pinned page.

Inherited iced manifest deprecations remain unsuppressed. No check/lint/type/readability/coverage, broad/all-publication suite, startup smoke, full-suite or final acceptance gate. Main owns integration and final verification.

## Historical validator

Dedicated own `validate.py` uses [compact historical pins](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/historical-pins.md): 84 source/tool/test/runtime/successor inputs, recorded code commit/tree/blob identities and sealed targeted logs. Archives total 2,083,184 bytes; each file below 5 MB. Existing object/tree helper contract is loaded from pinned bytes, avoiding dependence on old commit objects or mutable future registers. Counts derive from retained register/ledger/results; no current-head or runtime re-execution claim. At a0a29ee21, own validator fixtures pass 5/5 with Git unavailable in child PATH: sealed acceptance, source-response tamper rejection, own-green-log tamper rejection, archive tamper rejection, and unrelated later inventory acceptance without scope expansion. Tampering touches disposable copies only; originals remain intact. Own register/extract byte reproduction is covered inside those fixtures. [Validator command ledger](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/validator-command-ledger.md) retains RED/GREEN argv, code/test/context identities and logs. This is targeted validator development proof, not main's final gate.

## Successor boundary

Actual retail 4.1.0/4.2.0/4.3.0/4.3.4 registers replace the placeholders before 5.0.1 and later retail registers; Classic successors remain excluded. Only CanTransform/Transform additions become superseded by 4.1 removal, producing 304 current matches/115 gaps across 419 rows. Supersession is not model credit. CollapseSkillHeader/ExpandSkillHeader retirement remains modern-retail only; Mists registration is unchanged. Four source breaking statements and the separate GetItemCooldown numeric-only signature remain UNPROVEN. [Historical source intersections](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/source-overlaps.json) retain their captured scope, not current successor coverage. No native historical behavioral credit follows from modern retail publication or deprecation wrappers.

## Current integration receipts (2026-10-09)

[Current discovery receipts](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/integrated/) are separate from historical receipts: sealed 302/117 accounting and negative 118 remain unchanged, as do 42 archive/evidence seals. Main publication passes 70/70; Mists check/build exit 0. Built startup is not yet checked.

Independent formatting scope covers 212 files; Python fixtures 75/75 and own v2 validator fixtures 5/5 pass. All 74/74 register outputs reproduce with every process exit captured; 74/77 extracts reproduce. Three known inherited failures remain: 12.0.5/12.0.7 mismatch and 12.1.0 error/no output. Shared portable gate, current negative control, runtime smoke, CI and full suite remain pending; these receipts do not establish all-green acceptance.

[Portable validator v2 note](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/portable-validator-v2.md) records the minimal removal of the ROOT/target temporary-workspace dependency. Fresh-root/no-target/no-Git fixtures pass 5/5. Exact v1 validator/context bytes remain preserved; current self-seal metadata is new, not a retroactive rewrite of historical invocations or results.

## Sources

- [Pinned source](../../../data/patch-api/sources/4.0.1-api-changes.wikitext).
- [Source pin](../../../data/patch-api/evidence/4.0.1-session-2026-10-09/source-pin.json).
- [Spec](../../specs/patch-4-0-1-publication-sweep.md).

## See Also

- [[patch-4-1-0-api-audit]] — actual first retail successor; CanTransform/Transform removal supersession.
- [[patch-5-0-1-api-audit]] — later integrated retail register.
- [[patch-5-0-4-api-audit]] — substantive later retail publication.
