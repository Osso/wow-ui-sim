# Patch 3.4.1 Wrath Classic API audit

Bounded frozen-source audit, verified 2026-10-09 on `p341-source`, base `e7eb38c362be230b00af4096ac8092b9dd147200`. Page `379792`, revision `3656581`, timestamp `2023-05-04T13:15:35Z`, TOC **30401**. Caption compares 3.4.0 build 45435 to 3.4.1 build 47612, January 11, 2023. This is Wrath Classic, not retail Wrath history.

## Literal coverage matrix

The [ledger](../../../data/patch-api/sources/3.4.1-page-coverage.json) retains **376 nonblank raw source rows**: 333 inventory occurrences, two substantive prose statements and 41 metadata/table/source-context rows. Six blank lines carry no capability. Source occurrence IDs include exact raw line numbers; no linked pages/diffs were expanded.

| Section/direction | Header | Parsed | Runtime/native proof |
|---|---:|---:|---|
| Global API added / removed | 146 / 65 | 146 / 65 | UNPROVEN publication/removal; 211 identities, no signatures |
| Events added / removed | 35 / 9 | 35 / 9 | UNPROVEN availability, triggers, dispatch, ordering, payloads |
| CVars-table added / removed | 63 / 15 | 63 / 15 | Added includes 62 CVars **and one command**, `LogFps`; 15 removed CVars |

**333 inventory occurrences = 211 globals + 44 events + 77 CVars + one command.** Directions: 244 added, 89 removed. Removed here means a literal historical page claim, not an executed retirement or verified current absence. All inventory/prose rows remain UNPROVEN: 335 substantive rows, zero closed runtime gaps. Source publication claims are explicitly accounted, not tested by a retail or Mists surrogate.

**Signature ledger:** all 211 global-function occurrences have `arguments: null`, `returns: null`, UNPROVEN. Zero explicit function signatures. Event rows lack payloads/trigger contracts; `LogFps` lacks command grammar/output contract. CVar rows retain all literal template metadata including default/scope/category/description where present: **62 explicit defaults**. Defaults/descriptions are source assertions, not measurements of persistence, effects or simulator parity. No page-established callable input/output or event/state contract justifies a cheap modeled implementation. No runtime, shim, fallback, retirement, vendor, Wowless or cache change.

| Raw line | Statement | Limit |
|---:|---|---|
| 4 | “a subset of API changes” from retail 10.0.2 | UNPROVEN subset membership; linked member documentation not reconstructed |
| 235 | “same Widget API” as retail 10.0.0 | UNPROVEN linked equivalence; zero local widget identities/signatures/native observations |

Stock extraction with `--text-only --canonical-patch-navigation` reproduces six extract rows (five metadata, one pending summary). It **omits raw line 235**, inside an inventory section. Full raw `source_rows` plus `prose_ledger` explicitly preserve that statement; plaintext is not claimed as complete prose. No shared extractor flag or default behavior changes.

## Supported profile and separate history

[Client profile](../../../src/client_profile.rs) supplies supported `client-wrath`, `Wrath`, `wrath/AddOns`, configured interface **38001**. Source TOC **30401** is distinct. [Historical observation](../../../data/patch-api/evidence/3.4.1-session-2026-10-09/profile-observation.json) pins the base code hash; no cache inspection/sync, native probe, runtime load or compilation occurred. Supported profile is not absent; this bounded audit does not establish publisher coverage or native behavior.

Only Wrath Classic 3.4.2 and 3.4.3 are queued. Agent196 owns separate 3.4.2; parent integrates it before this audit. Unmerged 3.4.3 reference `48ab8e1c3` establishes no explicit member-level contract/supersession (zero enumerated APIs). `later_registers` stays empty. Retail 10.0.2/10.0.0 links are not Wrath successors; Cata Classic 4.4.x is a separate client transition. No default-retail publication register/harness is used: existing shared classifier/generator client-line support is not extended merely for source accounting.

## Historical proof boundary

Before any copy, response JSON identity, returned literal wikitext, bytes and manifest hashes were checked. Response **21755 bytes**, SHA-256 `8fd09b4b807c4191b62843d4e2dbfc18a5313e15796a6e284e24016694e99afe`; raw **21020 bytes**, SHA-256 `b336b152bea05e3609de06ac3971a98e7d2584129e4e3b74a5c333c6fc060ed1`. Exact manifest row retained in `source-pin.json`; frozen cache untouched.

[Validator](../../../data/patch-api/evidence/3.4.1-session-2026-10-09/validate.py) derives rows, kinds, directions, statuses, header counts, defaults and signature limits from sealed own serialized source/ledger and historical generator/extractor. No global register discovery, later tool dependency, live cache requirement, Git lookup or fixed evolving receipt totals. [Tests](../../../data/patch-api/evidence/3.4.1-session-2026-10-09/test_source_accounting.py) reject omitted inventory/prose/signature/source rows, fabricated contracts/credit, altered metadata/hashes/profile and foreign supersession. These test accounting, not simulator API execution.

Own development RED fails the missing 333-occurrence result against a temporary empty validator; GREEN **7/7** passes. Recorded-flag extraction and own source replay exit 0. Exact command/log/content scope retained in [proof ledger](../../../data/patch-api/evidence/3.4.1-session-2026-10-09/source-proof.json). Python manually formatted (ruff/black unavailable); no Rust edits/formatter. Parent owns integration and broad/final gates. No publication sweep/check/build/readability/startup/smoke/final gate, delegation, push/merge, provider/model/retry change or other-worktree write.

## Sources

- [Raw source](../../../data/patch-api/sources/3.4.1-api-changes.wikitext), [reproducible limited extract](../../../data/patch-api/sources/3.4.1-api-changes.txt), [pin/response/evidence](../../../data/patch-api/evidence/3.4.1-session-2026-10-09/).
- [Tracked accounting spec](../../specs/patch-3-4-1-source-accounting.md).

## See Also

- [[client-profiles]] — actual Wrath support and cache selection.
- [[patch-4-4-2-api-audit]] — separate Cata source boundary, not Wrath supersession.
