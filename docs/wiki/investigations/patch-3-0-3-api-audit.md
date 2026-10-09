# Historical retail Patch 3.0.3 source audit

Frozen page 560990/revision 5407654, timestamp 2010-03-28T14:26:47Z; exact 431-byte response body validated against the committed manifest before copying. Historical retail, not Wrath Classic. Full handoff registry ends at 1.0.0.

## Literal accounting

| Contract | Accounted | Proof / missing |
|---|---|---|
| CVar names | Three added inventory occurrences on lines 4–6 | `syncronizeConfig`, `synchronizeBindings`, `synchronizeMacros`; publication UNPROVEN, not measured |
| Full page | Five nonblank raw/rendered lines: attribution/navigation, one named header, three definition lines | Eight total ledger IDs including inventory; two metadata, three publication-unproven, three semantic-unproven |
| Prose | 0/1 disables/enables synchronization of UI settings, key bindings, macros respectively | Three behavioral contracts UNPROVEN; range is not a default |
| Signatures | Zero explicit callable argument/return fragments | No fabricated signature, endpoint, service lifecycle, event producer, native or persistence contract |
| Later retail inputs | 71 actual registers, zero literal-symbol overlap | Queued 3.0.8 / 3.1.0 / 3.2.0 / 3.3.0 placeholders are ordered, main adds them; raw sources have zero exact-name mentions, not proof of no indirect change |

`--legacy-cvar-definitions` preserves semicolon definitions only under `New CVars`; malformed definition rows fail explicitly. Defaults and extractor are unchanged. Targeted fixtures were committed before RED; missing opt-in produces two expected failures while template reproduction passes. Targeted GREEN at `ce0a0e863` passes 3/3: literal definitions/default-byte isolation, section/malformed boundaries, and integrated template register/extract reproduction. No shared extractor behavior changed; no broad tool suite run.

## Model boundary

[Backing review](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/backing-state-review.json) inspects existing local CVar key/value storage, defaults and SetCVar command handling. None contains these literal names. That static observation does not establish runtime absence or prove the absence of other systems. Persisting arbitrary strings locally does not implement synchronization. No meaningful model closure justified by the literal page; zero runtime changes, shims, retirements or new models. Native publication and all three 0/1 effects remain UNPROVEN. Linked Iriel forum content is attribution only, never expanded.

## Historical replay

[Own validator](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/validate.py) reads only its evidence directory and archived snapshots. Original accounting/gaps are separate from mutable current ledgers. [Original manifest](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/historical-inputs.json) seals 16 files, including original ledger/gaps/command logs and a 340,952-byte compressed archive containing 93 snapshots: all 101 handoff registry pages through 1.0.0, full frozen manifest, selected parser/extractor and tests, 71 actual later retail registers, queued raw inputs, integrated template sources and existing backing model files. Source and response are sealed separately. Manifest digest is anchored in the validator; seals are integrity controls, not signatures from an external trusted authority.

[Development receipt](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/validator-development-proof.json) and [proof ledger](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/development-proof-ledger.json) record historical RED exit1 at `f7582f153`, then GREEN exit0 2/2 at `e50955ee3`. Fresh copied Python processes use isolated mode, an empty Python path and unusable Git PATH; no Git, target, src or original/current sources are copied or required. Synthetic later current ledgers/closures leave serialized replay stdout identical. All 16 sealed files plus the manifest are tampered serially: 17 reject/restore controls pass with byte equality and clean replay after each. Deleted gap sidecar restores exactly. Independent decoded accounting rejects every omitted original ledger row (8), gap record (6), inventory occurrence (3), plus invented capability and corrected spelling (2). Counts derive from frozen records, not current closures.

The post-implementation GREEN receipt remains separate from the original sealed command ledger, avoiding recursive self-sealing. Documentation-only follow-up does not invalidate either test scope. Python changes were manually formatted before commits; no Python formatter is installed. No Rust/profile test added or needed for this source-only slice. No broad/check/lint/readability/profile/startup/full-suite/final gates. Main owns integration and native acceptance; all six publication/semantic records remain UNPROVEN.

## Sources

- [Exact page](../../../data/patch-api/sources/3.0.3-api-changes.wikitext).
- [Spec](../../specs/patch-3-0-3-source-accounting.md).
- [Ledger](../../../data/patch-api/sources/3.0.3-page-coverage.json).
- [Frozen evidence](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/source-pin.json).

## See Also

- [[patch-3-3-3-api-audit]], [[patch-3-3-5-api-audit]], [[patch-4-0-1-api-audit]] — read-only integrated templates; no acceptance inherited.
