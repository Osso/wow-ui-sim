# Patch 4.3.4 API audit

Pinned historical retail page **389302**, revision **3743181**, timestamp **2021-08-22T03:09:40Z**. Browser response and exact **866-byte** wikitext committed at `85b757d2d`, based on integrated main `3d64fedad`. Audit only this page, not linked contracts or the entire handoff.

## Coverage matrix

| Scope | Accounting | Proof boundary |
|---|---|---|
| Inventory | 10 added globals, one removed global; both headers match | Default generator, no parser changes |
| Non-inventory | One navigation context | Default extractor; zero prose/signature/enum/structure statements |
| Successors | Literal 5.0.1 redirect, separate 5.0.4 destination and later retail chain | No Classic successor credit |
| Publication | Historical discovery: 7 missing, 3 later removals, 1 absence; current fixture: 6 missing and 1 modeled clock | Historical observations remain sealed; current coordinator proof is separate |
| Models/retirements | Retail client-open clock; zero retirement edits | Separately pinned API contract supplies units/origin; native precision/event delivery unproven |

## Historical caller evidence

[Complete scans](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/caller-scans.json) cover 1,652 source files, 2,150 test files (including Lua strings), and 2,551 cached-retail Lua files. Full and bare names coincide for this page's unqualified globals. Cache has `GetSessionTime` consumer at `Blizzard_BNet/Mainline/BNet.lua:362`, and generated documentation names for that API and `GetSecondsUntilParentalControlsKick`. Documentation mentions are not runtime consumers. No `ComplainChat` hits, but absence-by-scan is not a runtime proof. No retirement edits; Classic and Blizzard wrappers unchanged.

## Historical development proof

Own discovery at `5c3bccdf7` fails at the expected empty-gap boundary: seven missing globals. [Results](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/discovery-results.json) account for every inventory row. [Coverage ledger](../../../data/patch-api/sources/4.3.4-page-coverage.json) records exact missing backing systems: account parental-control deadline, login-session clock, restoration service, ticket availability/throttle and report submission/target lifecycle. The change page supplies no units, reset/sentinel semantics or signatures. Initial identity-only development left the clock unmodeled; coordinator subsequently checked its separate primary API contract and existing client-start state.

`GetNumSoRRemaining` is superseded by 8.2.5; `GMSubmitBug` and `GMSubmitSuggestion` by 9.0.1. Unqualified `ReportPlayer`/`SetPendingReportTarget` are not their similarly named retired C_* members. `ComplainChat` is absent without any new retirement. Reviewed seven-ID fixture committed before GREEN.

| Targeted proof | Revision | Result |
|---|---|---|
| Own publication sweep | `04f6f5ce1` | 1 passed, 0 failed; seven reviewed gaps, no positive types |
| Same-cardinality fabricated-global control | `04f6f5ce1` | Expected failure: exactly one new gap, 7 → 8 |
| Own register/extract reproduction | `04f6f5ce1` | Both exit 0; byte-identical, default flags |

[Proof ledger](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/README.md) retains exact argv/revisions/log seals. [Data-derived accounting](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/accounting.json): **12 ledger IDs = 7 publication gaps + 3 superseded-publication + 1 absence-only + 1 metadata-only**, zero prose/signature contracts, fixes or retirements. No unmodeled parity claim. Implementer ran no broad publication/check/lint/type/readability/coverage/startup/full-suite gate; coordinator owns integration acceptance. The separately sealed historical validator replays these original artifacts, not current modeled behavior.

[Compact historical pins](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/historical-input-pins.json) archive the sparse own inputs and **64 retail successors** as exact gzip JSON bytes (**274,982 bytes**), with SHA-256/Git blob IDs. Runtime tree IDs identify code only; no historical executable/relocated-runtime proof. Later audits may add coordinator receipts without overwriting sealed historical evidence.

## Coordinator client-clock model

[Primary contract](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/integrated/session-clock/contract-target-response.json): `API:GetSessionTime`, page 254254, revision 6809914. It specifies numeric seconds since opening the game client and Korean-locale warning usage, not locale-gated availability or integer granularity. The precursor `API_GetSessionTime` redirect remains supplemental documentation, not the primary contract. Its Added 4.3.2 note is retained separately; this 4.3.4 inventory is not rewritten.

`SimState.start_time` is an `Instant` initialized by `EmptyRuntimeState::new()`. Retail registration publishes that existing elapsed client-state clock; no new clock subsystem, constant shim, Classic surface change, or Blizzard Lua patch. A new client state starts fresh; login/character-screen changes preserve its origin. `GetTime` implementation is untouched: sharing its current backing primitive does not assert equivalence with its native computer-uptime contract.

`tests/patch_4_3_4_session_clock.rs` covers seconds, independent client instances and login/character-screen transitions. Both tests reproduced nil-call RED ([log](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/integrated/session-clock/clock-red.log), [scope receipt](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/integrated/session-clock/clock-red.proof.json)); launch HEAD was not recorded, and the receipt identifies runtime scope `d937c30d6`. GREEN is 2/2 at pre-rebase `160f8209c` ([log](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/integrated/session-clock/clock-green.log), [receipt](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/integrated/session-clock/clock-green.proof.json)). Binding and test code are unchanged at `575a93e30`; test SHA-256 remains `d20854086e4748dda76b8652f88bc4e02bd1ee6ea92ed91b409d371def15436e`. Assertions exercise elapsed outputs and state transitions, not function identity. Broad publication/Mists/check/build acceptance remains pending and coordinator-owned; the original seven-gap receipts and archived inputs are immutable. Current ledger has one bounded clock row, six remaining publication gaps, three supersessions, one absence and one metadata row. Native precision, session-warning event generation and Classic parity are not claimed.

## Sources

- [Publication spec](../../specs/patch-4-3-4-publication-sweep.md).
- [Pinned wikitext](../../../data/patch-api/sources/4.3.4-api-changes.wikitext), [response](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/source-response.json), [pin](../../../data/patch-api/evidence/4.3.4-session-2026-10-09/source-pin.json).
- [Exact page-agent instructions](../../../data/patch-api/evidence/handoff-laptop-to-agent-server/page-agent-prompt.md).

## See Also

- [[patch-5-0-1-api-audit]] — literal redirect, no destination reconstruction.
- [[patch-5-0-4-api-audit]] — separately owned successor page.
