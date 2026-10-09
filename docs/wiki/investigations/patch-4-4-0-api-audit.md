# Cataclysm Classic 4.4.0 source audit

Verified source accounting 2026-10-08, not runtime compatibility. Pinned **CLASSIC4.4.0**, page `580953`, revision `6163701`, parent `6028368`, timestamp `2024-10-29T16:26:26Z`, literal TOC **40400**. Source acquisition commit `0c7a007e91e13c14b80209104dbb4c12533e10da`; no new network retrieval claimed. Read merged [4.4.2 audit](patch-4-4-2-api-audit.md), its spec/validator, repository rules and [handoff](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/HANDOFF.md). This task's source-only constraints exclude the handoff's runtime/delegation/integration workflow.

## Exact occurrence/capability proof matrix

[Ledger](../../../data/patch-api/sources/4.4.0-page-coverage.json) retains each nonblank source line literally. Wikitext and plaintext line numbers coincide; blank lines 2, 7, 11 and 13 remain in both files, without capability rows.

| Source ID suffix / line | Literal subject | Status / exact proof limit |
|---|---|---|
| `source-context-001` / 1 | 4.4.0 navigation: prev 3.4.3, next 4.4.1 | Metadata; no supersession proof |
| `source-context-003` / 3 | Summary heading | Metadata; editorial only |
| `prose-undated-004` / 4 | New `_Cata` client-specific TOC suffix | UNPROVEN: no Cata profile/cache or native selection/precedence measurement; linked TOC format unexpanded |
| `prose-undated-005` / 5 | Comma-delimited Interface versions, multiple versions/flavors | UNPROVEN: comma parser exists, but Cata/native compatibility acceptance, selection and rejection unmeasured |
| `prose-undated-006` / 6 | `C_ChatInfo.SendAddonMessage`: all traffic throttled per prefix; enum instead of boolean if unable to queue | UNPROVEN: no Cata/native queue probe, prefix independence, traffic limits or failure-result types measured. Two contracts; no enum members/thresholds inferred; linked 10.2.7 details unexpanded |
| `source-context-008` / 8 | Resources heading | Metadata; editorial only |
| `source-context-009` / 9 | TOC 40400 | Metadata; source identity, not loaded native build |
| `source-context-010` / 10 | Two 3.4.3..4.4.0 external diff links | Metadata; unexpanded, no inferred members |
| `source-context-012` / 12 | Breaking changes heading | Metadata; editorial only |
| `prose-undated-014` / 14 | Legacy `_Classic` behavior changed | UNPROVEN: no old/new Cata/native selection observation proves transition |
| `prose-undated-015` / 15 | `_Classic` now all Classic flavors, previously Era only | UNPROVEN: no Cata/native all-flavor or prior Era-only proof; supported-profile lists insufficient |
| `prose-undated-016` / 16 | Use `_Vanilla` instead for Era-only TOC | UNPROVEN: no native Era/Cata selection/exclusion successor proof; existing preference alone insufficient |

Totals: **12 rows = 6 metadata + 6 UNPROVEN prose rows, 7 contracts**. One named API occurrence in prose, zero enumerated inventory entries/header counts, zero removal statements, zero runtime/native credit. No publication register or modeled gap count. The changed `_Classic` statement and its nested semantics remain distinct literal occurrences, not deduplicated.

## Source identity and rejection proof

Returned response: **1646 bytes**, SHA-256 `3563e32d0abf16e7fa0e71d1d4b5cc4a8b62cba718ac91b5e73debdb32ee2497`. Main-slot content exactly equals pinned wikitext: **1348 bytes**, SHA-256 `f8598e92092793a0621ea26d5620fd910c178dcb1fccd0b0aa313b83a6b56d5e`. Plaintext: **964 bytes**, SHA-256 `e424ca462d923241d977198ad11f910afc9d7a85dcf172068f17bf4cba22de4d`.

[Validator](../../../data/patch-api/evidence/4.4.0-session-2026-10-08/validate.py) checks returned page/title/revision/parent/timestamp, source TOC, exact row coverage, all reasons via ledger seal, plaintext reproduction and [source receipt](../../../data/patch-api/evidence/4.4.0-session-2026-10-08/source-proof.json). Nine development tests pass, including fabricated revision/credit, every omitted row and owned-byte/receipt/rationale tampering. These external serialized-data checks are source proof only. Own inputs have live SHA-256 seals; context uses three historical Git tree pins, never shared live hashes or verbose listings. Validator is root-relative and does not bind current HEAD or shared tree state.

## Runtime boundary and implementation observation

[ClientProfile](../../../src/client_profile.rs) has no Cata variant/cache selection. No Cata runtime or native client was loaded. No retail/Mists stand-in, Cargo build or runtime test used.

At the source commit, [TocFile::interface_versions](../../../src/toc/mod.rs) splits comma-delimited Interface values into parsed integers. [active_profile_toc_suffixes](../../../src/loader/mod.rs) lists `_Classic` for Wrath/Mists/Era/Anniversary and `_Vanilla` preference for Era/Anniversary, with no `_Cata` profile arm. These are read-only implementation observations, not tests or native proof for any ledger contract. No runtime/parser/profile/classifier redesign performed.

External references retained only where literally present: TOC format, linked 10.2.7 messaging, Classic Era and both GitHub comparisons. None fetched or expanded. No unlisted enum/API members introduced. Navigation does not establish supersession; pending 4.4.1 integrates newer-first under coordinator ownership. No rebase/push/merge here.

## Sources

- [Wikitext](../../../data/patch-api/sources/4.4.0-api-changes.wikitext), [plaintext](../../../data/patch-api/sources/4.4.0-api-changes.txt), [ledger](../../../data/patch-api/sources/4.4.0-page-coverage.json).
- [Response](../../../data/patch-api/evidence/4.4.0-session-2026-10-08/source-response.json), [provenance](../../../data/patch-api/evidence/4.4.0-session-2026-10-08/source-pin.json), [spec](../../specs/patch-4-4-0-source-accounting.md).

## See Also

- [[patch-4-4-2-api-audit]] — merged source-only precedent, unsupported Cata boundary.
- [[client-profiles]] — existing profiles, not native Cata support.
