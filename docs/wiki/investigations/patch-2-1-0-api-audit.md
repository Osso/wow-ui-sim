# Historical retail Patch 2.1.0 SOURCE audit

Frozen 279373/6767102, timestamp 2026-07-09T22:15:03Z, checked against exact cached response and manifest before copying. Page describes 2007 retail; revision timestamp is not patch release date. Branch `p210-source`, base `44df100f1b553c5d717af9d8c14d3405823ecfa7`. No network, runtime or native measurement.

## Literal coverage

| Source | Accounting | Proof limit |
|---|---|---|
| Full raw/default rendering | 176 lines, 150 nonblank; 19 named headers, no numerical inventory headers | Raw text authoritative: default renderer strips `<name>` command placeholders and navigation attribution; neither omission loses raw evidence |
| Named occurrences | 92: 56 globals, six widgets, eight template references, two CVars, 18 command occurrences, two event-order references; 55 added/29 changed/eight removed | Publication claims distinguished from contextual references; no inferred API membership or runtime availability |
| Signatures | 124 records: 44 literal call-like spans + 80 callable/command boundaries; 31 have a matching span, 49 have none | Literal returns retained where explicitly labeled; observed names stay unspecified; examples and wildcard family removal are not full signatures |
| Source prose | Every substantive raw row UNPROVEN | No shortcut from callable presence or existing generic model to behavior |
| Seals/replay | 18 original seals, 101 archived snapshots; 350,913-byte compressed archive, all files below 5 MB | Targeted copied-process GREEN pending; main owns integration/native/final gates |

Duplicate `IsFeignDeath` removals at lines 46/85 retained independently; `likely` rename/replacement qualifications remain literal. `GameTooltip:Set*CompareItem(...)` is one wildcard family, never expanded to guessed methods. `SetId`, `timeused`, `throtting`, `effciently`, `aucion`, malformed quote/parenthesis text and literal escaped name quotes are not corrected. Repeated `/equip` references and source attribution/comment survive. `unit/name` is not slash command `/name`.

## Contract matrix

| Area | Literal contract | Missing proof |
|---|---|---|
| Profiling/template handlers | Shared inherited closure identity/setfenv warning; per-addon cached KB, one-byte precision; CPU seconds/about-microsecond precision, optional subtree/count aggregation, reset; disabled profiling with persisted `scriptProfile`/reload activation | No measured attribution, clock/profiler/cache/GC state or lifecycle; 50% highlight not benchmark evidence |
| Frames/tooltips/APIs | Per-object alpha clamp and inherited product; secure hook equivalence; unitid/comparison index 1/2; taint name/macro empty string; merchant cursor, login event boundary, group feign death, whisper channel, aura castability/time ordering, gem ambiguity and spam line/mail context | No signature acceptance, tuple/state transitions, native retirement or secure enforcement observations |
| Secure templates/macros/raid | Item ID/link, attribute feedback, saved-state push/pop/swap, state-driver example, group/pet derivation and show attributes; flyability/focus substitutions, OR clauses, bag-slot/item-ID commands, roles/current-target/secure attributes | Linked header usage unexpanded; unnamed protected raid API stays unnamed; no guessed grammar or model |
| Loading/default UI/layout | Ordered first-success LoadManagers/load-on-demand; reagent counts/top-three memory; `scriptErrors` and error sink/slash cache; UIParent attribute defaults 384/-104/0/80 and panel layout attributes | Attribute defaults are not CVar defaults; no cache/service/vendor changes or UI/loader/combat acceptance |
| Protection/fixes/world state | Undetermined whisper throttle; GM-ticket protection; persistence, Cooldown alpha, item self-cast, this typo, auction tokens, BG event ordering, ScrollChild parent assignment; eleven WorldState return names with uiType first | No guessed throttle; event producer/payload/order unmeasured; no sort/render/native tuple parity |

Zero model closures, runtime observations, new models, retirements, shims or fallback changes. Static backing review/probes unnecessary for SOURCE-only scope. No claims of simulator absence from names-only evidence.

## Development history

Fixtures committed before RED at `0af6dc600`: expected missing opt-in/validator failures plus a fixture-path error for the separate 3.4.0 template. Corrected template path/header count at `d33481554`: RED 4 tests, two expected failures, two controls pass. The original failing log is retained honestly; it is not called a clean feature RED. No Python formatter installed; own Python formatting manual before commit. No Rust changes or check/lint/readability/coverage/broad/startup/final gates.

## Original sealed boundary

[Historical manifest](../../../data/patch-api/evidence/2.1.0-session-2026-10-09/historical-inputs.json) seals exact raw/response/pin/provenance, original ledger/gaps/register/extract/counts, both default controls, command ledger and four physical RED/GREEN logs. Compressed snapshots retain full manifest and 101-page handoff registry ending 1.0.0, selected tools/tests, current static profile source, template recorded flags/outputs and actual/queued successor inputs. Manifest SHA-256 anchored in [validator](../../../data/patch-api/evidence/2.1.0-session-2026-10-09/validate.py); integrity controls are not externally authenticated signatures.

Actual 74 ordered retail registers (3.1.0/3.2.0/3.3.0 then retained 3.3.3 onward) contain 18 exact same-symbol occurrences; these are source comparisons, not automatic semantic closures or native retirements. Seven ordered main-owned placeholders: 2.2.0, 2.3.0, 2.4.0, 2.4.2, 3.0.2, 3.0.3, 3.0.8. Their raw inputs remain archived; no guessed membership or queued supersession. Classic 2.5/3.4/4.4/5.5 never supersedes retail. `PLAYER_LOGIN` timing stays literal in raw line 44, separate from the two event-order inventory references; no inferred new-event publication. All raw UI attributes and contextual methods also remain literal, not new publication inventory.

Parser/default/template GREEN 3/3 at `3f32a7629`; source accounting and portable replay GREEN pending. Original command receipts include expected portable missing-validator RED at `322cac662`. No rerun of already-proven parser controls is required for validator-only changes. Later receipts remain outside original seals.

## Sources

- [Exact source](../../../data/patch-api/sources/2.1.0-api-changes.wikitext), [register](../../../data/patch-api/sources/2.1.0-wikitext-register.json), [ledger](../../../data/patch-api/sources/2.1.0-page-coverage.json).
- [Pin](../../../data/patch-api/evidence/2.1.0-session-2026-10-09/source-pin.json), [spec](../../specs/patch-2-1-0-source-accounting.md).

## See Also

- [[patch-3-0-3-api-audit]], [[patch-3-0-8-api-audit]], [[patch-3-1-0-api-audit]] — read-only retail templates.
- [[patch-3-4-0-api-audit]] — separate Classic source template; no supersession or behavioral credit.
