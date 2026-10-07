# Patch 9.2.7 API page audit

Page 542130, revision 5227425 (September 4, 2022, 12:19:59 UTC), retrieved October 7, 2026. Requested page exists; no 9.2.5 substitution. Branch `p927-page` starts from master `af9ae57c2`. Current retail carries 12.1.0, not reconstructed Shadowlands behavior.

## Source accounting

Three inventory occurrences: two added events, one changed event. Added header count matches; no removals or retirement candidates. All 26 later registers, 10.0.0 through 12.1.0, supersede this page. Initial extraction retained only eight rows and silently lost the enumeration and structure sections because the page has no Global API heading. Event-only fixture reproduces the loss; level-two Events recognition restores 23 nonblank extract rows without changing previous extraction-mode outcomes. The audited extract has nine metadata rows (including documentation relocation) and fourteen substantive pending rows.

[Ledger](../../../data/patch-api/sources/9.2.7-page-coverage.json) accounts for 27 unique IDs: three inventory, 23 extract and one historical build caption. Statuses: one bounded-coverage (current absence), two partial-development-green (event registration), fourteen audit-pending and ten metadata-only. The changed notification annotation stays attached to its inventory row with an explicit pending payload boundary. No metadata, static declaration or unrelated round-trip test receives runtime behavior credit.

## Proof boundaries

Event registration does not prove auction purchase/delivery state or the changed notification's `auctionID` payload. Five new enum tables, three AuctionHouseError additions, ReportType.PvPScoreboard, and ItemKeyInfo's itemID/battlePetSpeciesID additions need exact identity/populated DTO behavior proof. [Literal scout](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-extract-scout.json) assigns every extract occurrence a precise reason, literal line and original wikitext line. Enum tables/members exist in `src/lua_api/globals/enum_data/missing_enums.lua`; no page-specific value/flag parity test is credited. `registry_and_rows.rs:539-553` serializes GetItemKeyInfo name/icon/quality/commodity fields but omits itemID and battlePetSpeciesID. Existing round-trip tests assert those fields on **ItemKey**, not ItemKeyInfo. Populated item and pet DTO behavior remains pending rather than replaced with constants.

The read-only current native declaration confirms ItemKeyInfo's two non-nil fields at `Blizzard_APIDocumentationGenerated/AuctionHouseDocumentation.lua:1636-1637` and the notification's optional auctionID at `:1254`; declaration metadata is not execution proof. Documentation relocation is source metadata, not an API implementation. No placeholders, vendor patches or runtime surface changes introduced. One source-loss boundary fixed; zero runtime API gaps closed and zero publication gaps remain.

Whole `src/` and `tests/` scan retains 43 untruncated matches, including embedded Lua and function references. Cached retail scan retains 128 matches. No global/method changed, so no caller migration or classic gating is needed. No removed row exists; qualified/bare pre-retirement scanning is vacuous.

## Verification

Compiled source/test revision `cbc762757`. All 27 publication sweeps plus the existing animation-owner regression pass: 28/28 cases, covering 6,379 inventory rows with exact per-patch fixtures. Own sweep: three rows, three OK, zero gaps. AUCTION_HOUSE_PURCHASE_DELIVERY_DELAY_UPDATE correctly rejects registration because the 10.1.5 removal supersedes its historical addition; this is not a new retirement.

Negative control changes only AUCTION_HOUSE_PURCHASE_COMPLETED added → removed. Exactly one new failure, zero resolved failures, unchanged three row IDs; expected prefork exit 1. [Exact result](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-negative-result.json).

Extractor RED reproduces section loss; GREEN passes all 23 extractor and eleven register fixtures. Format/fmt --check, default cargo check, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup pass. Mists has zero non-vendor warnings; six pre-existing iced manifest deprecations plus their summary remain unsuppressed. Retail startup exits zero and prints `[]`. No global/method changed; no additional affected integration or prefork surface tests require migration. Changed Rust manually audited: flat input-only sweep specification, explicit source/fixture/env variables and chronological later inputs; no readability violations or warning suppressions introduced.

[Proof ledger](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-proof.json) records exact commands, revision/scope, output artifacts, exits and invalidation state. Cargo output captured once per command, saved and searched with rg; no rerun to recover output. All commands explicitly use this worktree as cwd and its own `target`; no sibling build reuse, agents/models, push, merge, vendor/cache or canonical working-file edits.

## Sweep table

All results are publication-only, not full API compatibility. Exact known-gap fixtures match every passing case.

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
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

## Preservation

All 138 pre-existing source/register/ledger inputs remain byte-identical. Every one of the 52 previous `--check --text-only` outcomes, with and without `--preserve-examples`, is unchanged. Fifteen pre-existing mode failures remain, including both modes for 12.0.5/12.0.7/12.1.0 and historical example-mode differences. Own page reproduces in both modes. Parser unchanged; no prior register rewritten. [Artifact validator](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/validate.py) checks exact IDs, counts, literals, source hashes, expectations, negative control, preserved inputs/outcomes, sweep receipts and Mists warning boundary without runtime reruns.

## Sources

- [Retained API response](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-fetch.json) — revision and original wikitext.
- [Register](../../../data/patch-api/sources/9.2.7-wikitext-register.json) — exact inventory identities and lines.
- [Spec](../../specs/patch-9-2-7-publication-sweep.md) — required proof boundaries.
- [Caller scan](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-whole-caller-scan.txt) — untruncated simulator callers.
- [Cached source scan](../../../data/patch-api/evidence/9.2.7-session-2026-10-07/p927-cached-source-scan.txt) — read-only retail references.

## See Also

- [[patch-10-0-0-api-audit]] — adjacent later publication register.
- [[patch-10-0-2-api-audit]] — accounting and proof conventions.
