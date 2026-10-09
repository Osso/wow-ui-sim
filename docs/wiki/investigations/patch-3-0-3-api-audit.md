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

`--legacy-cvar-definitions` preserves semicolon definitions only under `New CVars`; malformed definition rows fail explicitly. Defaults and extractor are unchanged. Targeted fixtures were committed before RED; missing opt-in produces two expected failures while template reproduction passes. GREEN pending at this checkpoint.

## Model boundary

[Backing review](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/backing-state-review.json) inspects existing local CVar key/value storage, defaults and SetCVar command handling. None contains these literal names. That static observation does not establish runtime absence or prove the absence of other systems. Persisting arbitrary strings locally does not implement synchronization. No meaningful model closure justified by the literal page; zero runtime changes, shims, retirements or new models. Native publication and all three 0/1 effects remain UNPROVEN. Linked Iriel forum content is attribution only, never expanded.

## Historical replay

Original accounting and gap records are separate from mutable current ledgers. Planned sealed archive retains full registry/manifest, exact input pair, parser/extractor, actual later retail registers, queued raw inputs, backing model sources and logs. Fresh copied-process replay and serialized tamper/restoration development tests are pending. No broad/check/lint/readability/profile/startup/full-suite/final gates.

## Sources

- [Exact page](../../../data/patch-api/sources/3.0.3-api-changes.wikitext).
- [Spec](../../specs/patch-3-0-3-source-accounting.md).
- [Ledger](../../../data/patch-api/sources/3.0.3-page-coverage.json).
- [Frozen evidence](../../../data/patch-api/evidence/3.0.3-session-2026-10-09/source-pin.json).

## See Also

- [[patch-3-3-3-api-audit]], [[patch-3-3-5-api-audit]], [[patch-4-0-1-api-audit]] — read-only integrated templates; no acceptance inherited.
