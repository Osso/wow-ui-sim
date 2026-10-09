# Patch 4.3.4 API audit

Pinned historical retail page **389302**, revision **3743181**, timestamp **2021-08-22T03:09:40Z**. Browser response and exact **866-byte** wikitext committed at `85b757d2d`, based on integrated main `3d64fedad`. Audit only this page, not linked contracts or the entire handoff.

## Coverage matrix

| Scope | Accounting | Proof boundary |
|---|---|---|
| Inventory | 10 added globals, one removed global; both headers match | Default generator, no parser changes |
| Non-inventory | One navigation context | Default extractor; zero prose/signature/enum/structure statements |
| Successors | Literal 5.0.1 redirect, separate 5.0.4 destination and later retail chain | No Classic successor credit |
| Publication | Own discovery pending | Type/absence only, no behavioral parity |
| Models/retirements | None | Page supplies identities only; no invented service behavior |

## Caller evidence

[Complete scans](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/caller-scans.json) cover 1,652 source files, 2,150 test files (including Lua strings), and 2,551 cached-retail Lua files. Full and bare names coincide for this page's unqualified globals. Cache has `GetSessionTime` consumer at `Blizzard_BNet/Mainline/BNet.lua:362`, and generated documentation names for that API and `GetSecondsUntilParentalControlsKick`. Documentation mentions are not runtime consumers. No `ComplainChat` hits, but absence-by-scan is not a runtime proof. No retirement edits; Classic and Blizzard wrappers unchanged.

## Development proof

Own sweep and accounting pending. No broad publication/check/lint/type/readability/coverage/startup/full-suite or final acceptance gate run by implementer.

## Sources

- [Publication spec](../../specs/patch-4-3-4-publication-sweep.md).
- [Pinned wikitext](../../../data/patch-api/sources/4.3.4-api-changes.wikitext), [response](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/source-response.json), [pin](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/source-pin.json).
- [Exact page-agent instructions](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/page-agent-prompt.md).

## See Also

- [[patch-5-0-1-api-audit]] — literal redirect, no destination reconstruction.
- [[patch-5-0-4-api-audit]] — separately owned successor page.
