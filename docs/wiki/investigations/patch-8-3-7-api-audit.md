# Patch 8.3.7 API audit

Warcraft Wiki **Patch 8.3.7/API changes**, page **160588**, revision **1572710** (2021-04-23T01:19:54Z), retrieved 2026-10-08. This first requested page exists, so 8.3.0 was not selected. Default retail carries 12.1.0; this is complete publication accounting, not historical reconstruction.

## Source and markup boundary

[Source response](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-fetch.json) retains exact raw wikitext. Direct HTTP API attempts for both candidate titles returned 403; browser navigation to 8.3.7 raw source and revisions API succeeded. A blocked HTTP response is not evidence that 8.3.0 is absent.

The page has **two API/New bullet additions**, no consolidated table or numerical headers: global `GetAreaText`, CVar `MouseUseLazyRepositioning`. Existing generator produced zero inventory entries. New **opt-in `--simple-api-list`** retains both names, sections, directions and original source lines. [Provenance](../../../data/patch-api/sources/8.3.7-api-changes.provenance.json) records generator flags. Behavioral fixture failed before parser implementation, then all **42** generator/extractor fixtures passed. Shared extractor and publication classifier remain unchanged.

## Coverage and gaps

| Identity | Current expectation | Observation | Proof boundary |
|---|---|---|---|
| GetAreaText | published | raw=nil; lookup=nil | Exact retained publication gap; area-text selection remains undefined |
| MouseUseLazyRepositioning | absent after later removal | no value/default | Bounded current-retail absence, not historical CVar semantics |

No runtime APIs changed or retired; **zero gaps closed, one retained**. [Gap review](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-gap-review.json) preserves literal source and observation. [GetAreaText reference](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-GetAreaText-reference.wikitext) documents only a string return. Existing zone/subzone/minimap producers do not establish which state this distinct area-text API selects. Inventing an alias or placeholder would not be modeled behavior; closure needs a defined selection contract and concrete behavioral fixtures.

[Occurrence ledger](../../../data/patch-api/sources/8.3.7-page-coverage.json) accounts for **ten unique IDs**: two inventory plus eight extract IDs. Statuses: **one bounded-coverage, one audit-pending, eight metadata-only**. Extract has six navigation/headings/build/resource contexts plus two renderings of inventory references. Duplicate renderings link to inventory IDs and earn no independent runtime credit. No separate substantive summary statements or numerical header captions are omitted. [Extract scout](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-extract-scout.json) records every literal and classification.

## Verification and preservation

Acceptance runtime/test revision **5c27f3b4c**; subsequent evidence/docs/validator changes do not alter that scope. **34 register sweeps plus animation factory regression pass (35/35)**, covering every exact fixture. Negative control changes GetAreaText added → removed: gaps **1 → 0**, exactly one stale known-gap identity, no new gaps, same row IDs and expected test exit one. [Negative receipt](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-negative-result.json).

`cargo fmt`, `cargo fmt --check`, default `cargo check`, Mists `cargo check --no-default-features --features sound,gui,casc,client-mists --tests`, separate retail build and bounded startup all pass. Startup exits zero and returns **`[]`**. Mists emits **zero non-vendor warnings**; inherited iced manifest deprecations remain unsuppressed. No changed runtime surface requires caller migration or classic behavioral regression tests. New parser behavior has RED/GREEN coverage; changed Rust is input-only and has [readability review](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-readability.md).

All **173 pre-existing source/register/ledger inputs remain byte-identical**. All **66 pre-existing extraction-mode exit/stdout outcomes remain unchanged**, including sixteen inherited failures and both modes of 12.0.5/12.0.7/12.1.0; own extract passes both modes. All **34 registers regenerate byte-identically**, using recorded flags where present and explicitly identified historical flags otherwise. [Reproduction receipt](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-register-reproduction.json). No earlier sources or provenance rewritten.

[Proof ledger](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-proof.json) retains commands, code revisions/scopes, cwd, target, exits, environments and log hashes; discovery RED is not acceptance. [Artifact validator](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/validate.py) derives counts, statuses, register/result sets and sweep totals from current fixtures/sources/results, not hard-coded integration receipts.

## Sweep table

Publication/absence only, exact retained-gap fixtures, not signatures or behavior parity. [Machine-readable table](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-sweep-summary.json).

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 8.3.7 | 2 | 1 | 1 | pass |
| 9.0.2 | 77 | 51 | 26 | pass |
| 9.0.5 | 28 | 15 | 13 | pass |
| 9.1.0 | 179 | 128 | 51 | pass |
| 9.1.5 | 169 | 122 | 47 | pass |
| 9.2.0 | 80 | 54 | 26 | pass |
| 9.2.5 | 84 | 50 | 34 | pass |
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

## Integration and scope

[Read-only 9.0.1 intersection](../../../data/patch-api/evidence/8.3.7-session-2026-10-08/p837-901-supersessions.json) finds **no add/remove intersection** with retained `wt-global-api-GetAreaText-9`; no predicted 9.0.1 gap supersessions. One-line placeholder starts the later-register list; integrating main thread inserts 9.0.1 when merged. Existing 9.0.2 through 12.1.0 registers already supersede historical publication expectations.

Every command explicitly used `/home/osso/.worktrees/wow-ui-sim-p83x-page` cwd and its own target directory. Branch was verified `p83x-page` before first commit. No canonical/sibling working-file changes, vendor/cache/Wowless writes, Blizzard monkey-patches, agents/models, push or merge.

## Sources

- [Pinned raw source](../../../data/patch-api/sources/8.3.7-api-changes.wikitext), [extract](../../../data/patch-api/sources/8.3.7-api-changes.txt), [register](../../../data/patch-api/sources/8.3.7-wikitext-register.json).
- [Specification](../../specs/patch-8-3-7-publication-sweep.md).
- [Prior audit template](patch-9-0-2-api-audit.md).

## See Also

- [[patch-9-0-2-api-audit]] — occurrence accounting and chronological publication expectations.
