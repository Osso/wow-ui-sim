## [2026-10-05] investigation | Retirement successor fallout

[Follow-up](investigations/patch-12-0-0-api-audit.md#retirement-fallout-follow-up) records all 17 reproduced failures and six reproduced successor gaps. Model tests use current APIs, legacy-only assertions retain epoch gates, and state-backed successors replace autostubs. [Contract and final proof](../specs/retirement-successors.md#verification--2026-10-05) define bounded scope, full-suite results, corrected post-suite controls and original-base reproductions. Page-coverage ledger unchanged.

## [2026-10-06] evidence | 12.0.0 DTO extract producer proof

[DTO follow-up](investigations/patch-12-0-0-api-audit.md#dto-extract-batches-b02b04b05b06) links 34 source-ID outcomes, full cached parent probes, LFG input roundtrips, host DTO snapshots and cast-driven recovery/GCD events. [Proof ledger](../../data/patch-api/evidence/12.0.0-session-2026-10-05/p1200-b2-proof.md) retains exact revisions, baseline comparisons, isolated sweep passes, startup `[]` and inferred/native limits. Page-coverage JSON unchanged.

## [2026-10-05] investigation | Cast duration rounding

[Investigation](investigations/cast-duration-rounding.md) records the captured one-ULP loss and deterministic RED. Cast state preserves configured spans; queries no longer recover duration from rounded deadlines. [Proof](investigations/cast-duration-rounding.md#verification--2026-10-05): targeted/module GREEN, 95 integration controls, three full parallel lib runs with only six known failures, startup `[]`, check/fmt pass. GC audit attribution updated.

## [2026-10-05] evidence | 12.0.0 enum/deprecated extract proof

[Follow-up](investigations/patch-12-0-0-api-audit.md#enum-and-deprecated-extract-proof) records loaded enum publication, falsified seed conflicts and deprecated retirement root causes. [Contract](../specs/patch-12-0-0-extract-enums-deprecated.md) links exact per-source outcomes, pending successor results, master comparisons and historical compilation limits; coverage ledgers unchanged.

## [2026-10-05] evidence | 12.0.0 non-inventory extraction and scout

[Supplement](investigations/patch-12-0-0-api-audit.md#non-inventory-extract-supplement) records additive pending rows, deterministic extraction and every-row proof planning. Source-derived enum conflicts and historical test reuse are distinguished from fresh runtime acceptance.

## [2026-10-06] system | Retail publication input queries

[Models](systems/retail-publication-inputs.md) link namespace query fields to the [remaining-row closure contract](../specs/patch-12-0-0-publication-sweep.md#remaining-namespace-closure--2026-10-06). Evidence retains every assigned outcome, exact revision/command proof and native limits; no page-coverage ledger changes.

## [2026-10-05] investigation | Plain-global publication attribution

[Investigation](investigations/plain-global-publication.md) records why direct cached Blizzard aliases lack Deprecated debug sources. [Contract](../specs/patch-12-0-0-publication-sweep.md#plain-global-closure-contract) owns state, epoch gates and two pinned-VM blockers; page-coverage files unchanged.

## [2026-10-05] system | C_Secrets publication queries

[Shared queries](systems/secrets-publication-queries.md): original selector authentication, same aura/cooldown/identity predicates as outputs, explicit INFERRED public defaults for absent backing models. [Final proof](../specs/secrets-publication-queries.md#verification--2026-10-05): six behavioral tests pass, four isolated publication sweeps pass, startup `[]`; unchanged pre-change master filter failures retained. Source page-coverage ledgers unchanged.

## [2026-10-05] investigation | Host Lua rooting audit

[Audit](investigations/gc-rooting-audit.md): 41 recorded candidates, 18 fixed sites, 23 false positives. Forced-GC DTO/event/cast/token negatives, 173 targeted passes, startup `[]`; final lib 1,972 pass/six baseline-reproduced failures. Original duration-flake cause remains unproven.

## [2026-10-05] investigation | Tooltip GC regression

[Investigation](investigations/patch-12-0-0-api-audit.md#tooltip-gc-regression-exposed-by-retirement-registration) records original failure, unchanged host line order, retirement-only allocation control, and forced-color-callback GC RED. Root item DTO/line tables across Lua callbacks; keep API retirements and stat assertions.

## [2026-10-05] evidence | Transmog selector and final proof

[Proof ledger](../specs/patch-12-0-0-publication-sweep.md#transmog-verification--2026-10-05) records integration/prefork filters, four isolated sweeps, startup `[]` and seven lib failures reproduced on master. [Consumer boundary](systems/transmog-outfits.md#consumer-boundary) distinguishes cached inventory selectors from outfit construction data. Page-coverage JSON unchanged.

## [2026-10-05] implementation | Retail transmog outfit state

[Outfit system](systems/transmog-outfits.md) documents catalog/pending/situation state, enum-slot topology, set imports, cursor actions and structured visual successors. [Contract](../specs/patch-12-0-0-publication-sweep.md#transmog-outfit-closure-contract) separates modeled behavior from inferred/native-unverified choices. Publication and final proof reconciliation remain pending at this entry.

## [2026-10-05] evidence | 12.0.0 wikitext publication sweep

[Audit supplement](investigations/patch-12-0-0-api-audit.md#wikitext-supplement) records revision 6747189, parser and classifier corrections, 64 exact namespace retirements, all 270 initial-gap reviews, 812 OK / 198 final known gaps, source comparisons and proposed statuses. Four isolated sweeps, one-gap negative control, local check/build, fmt and startup `[]` pass. Six vendor manifest warnings match master; ledger/coverage JSON unchanged.

## [2026-10-04] evidence | 12.0.5 pending non-sweep follow-up

[Audit follow-up](investigations/patch-12-0-5-api-audit.md#pending-non-sweep-follow-up--2026-10-04) records all24 row boundaries, leadership subset RED/master/GREEN, unchanged vendor mouse proof, 55 selected GREEN, checks/startup and pinned-rilua limits finding. Coverage JSON unchanged; aggregate blockers remain explicit.

## [2026-10-04] docs | 12.1.0 publication sweep accounted

778 inventory symbols swept: 631 match, 147 gaps baselined. [Audit page](investigations/patch-12-1-0-page-audit.md#round-2--publication-sweep--2026-10-04); 1,111 rows; 391 pending /151 bounded /505 partial /64 metadata.


## [2026-10-04] docs | 12.1.0 sweep gaps closed (round 3)

Sweep gaps 147 → 21: LoD bootstrap preload fix, 31 added Global API functions, widget/event/CVar surface. [Audit page](investigations/patch-12-1-0-page-audit.md#round-3--closing-sweep-gaps--2026-10-04); 265 pending /157 bounded /625 partial /64 metadata.

## [2026-10-04] docs | 12.1.0 extract rows round 4

Enum values fixed and proven (58), struct shapes (26), deprecated wrappers (9). [Audit page](investigations/patch-12-1-0-page-audit.md#round-4--extract-rows-enums-structs-deprecated-wrappers--2026-10-04); 156 pending /224 bounded /652 partial /79 metadata.

## [2026-10-04] docs | 12.1.0 strict removals fix and deprecation-fallback sweep rule

Post-startup strict removals deleted Blizzard deprecation wrappers; fixed at namespace setup. Sweep accepts deprecation-file fallbacks for removed symbols (gaps 25 → 11). [Audit page](investigations/patch-12-1-0-page-audit.md); 135 pending /239 bounded /658 partial /79 metadata.

## [2026-10-04] docs | 12.0.7 wikitext supplement

Extract missed 65 of 174 collapsed-table symbols; shared publication sweep added (147 OK, 27 gaps). [Audit page](investigations/patch-12-0-7-api-audit.md#wikitext-supplement--2026-10-04). 12.1.0 now 22 pending after leftovers; rilua pinned at a76ffa8 (intern GC fix).

## [2026-10-04] docs | 12.0.5 wikitext supplement

Crawler register missed 216 of 363 collapsed-table symbols; sweep 306 OK / 57 gaps. [Audit page](investigations/patch-12-0-5-api-audit.md#wikitext-supplement--2026-10-04).

## 2026-10-04 | Club membership model

Added [club membership](systems/club-membership.md) and contract; retired temporary management defaults. Verification pending.

## 2026-10-05 | Club host inputs

Added synchronous member snapshots/departures and stable historical chat authors to [club membership](systems/club-membership.md). New proof pending.

## 2026-10-05 | Guild opaque-ID regression

Cached GuildRoster rank menus require `guid` independently of member IDs. Added stable projected GUIDs; no vendor/UI patch. Retest pending.

## 2026-10-05 | Club membership proof

Recorded [development proof](../specs/club-membership.md#development-proof--2026-10-05): five host/member event rows, 12 behavior tests, club/guild/communities/bnet green, publication green, startup `[]`, fmt/check green. Chat failure reproduced on untouched master. No coverage ledger edits.

## 2026-10-05 — Console registry

Live CVar catalog extended with epoch-scoped Command records; shared command publication probing closes two 12.0.7 gaps. [System](systems/console-command-registry.md); [contract](../specs/console-command-catalog.md). Baseline console empty assertion was stale since `2c78bff73`.

## 2026-10-05 — Neighborhood initiatives

Added [host-backed initiative state](systems/neighborhood-initiatives.md), deferred request ordering and explicit diagnostic-workaround limits. Contract lives in [publication spec](../specs/patch-12-0-0-publication-sweep.md#housinginitiative-closure-contract).

## 2026-10-06 | 12.0.1 publication and extract audit

Added [page audit](investigations/patch-12-0-1-api-audit.md), retained revision 6747895, 225-row exact-gap sweep, four epoch-gated retirements and exhaustive 252-row extract scout. New 477-row ledger distinguishes publication-only, superseded, behavioral-pending and editorial scope. Existing page ledgers unchanged; 12.0.0 fixture reconciled against 12.0.1 supersession. [Contract](../specs/patch-12-0-1-publication-sweep.md) owns final proof.
