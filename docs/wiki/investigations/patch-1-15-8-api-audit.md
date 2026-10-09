# Patch 1.15.8 literal SOURCE audit

Frozen page **686952 / revision 6778071**, timestamp `2026-07-22T05:34:51Z`, audited offline from `fcdf1d151371e894aa19d800eeaaf17aa0a3674a`. No runtime edits, model or native measurements.

## Source identity

[Own ledger and frozen inputs](../../../data/patch-api/evidence/1.15.8-session-2026-10-09/ledger.json): raw 279 bytes, SHA256 `9ba11d7e93a8763efb4c4d430e5d44a2cf8711096970ff8170caf6cf190aaa45`; response SHA256 `93295622a450cb5faedb68d6ecf65617d987133d1a22ebe71294bbd74a5a92fc`. Manifest and 101-page registry ending 1.0.0 retained offline. Raw bytes and every occurrence remain literal; no malformed-source repair, deduplication, alias/signature/default reconstruction or expansion.

## Literal coverage matrix

| Scope | Literal accounting | Proof / limit |
|---|---|---|
| Entire source | Five physical lines, four nonblank rows: three metadata, one UNPROVEN | No source row omitted; diff row holds two distinct contracts |
| Navigation | `{{apichanges|1.15.8|prev=1.15.7|next=1.15.9}}` | Labels retained; template expansion UNPROVEN, navigation metadata is not behavior |
| Header / TOC | `==Resources==`; `11508` | One nonnumerical header; TOC derives from text, not patch-number arithmetic |
| Linked diffs | wow-ui-source `1.15.7..1.15.8`; BlizzardInterfaceResources `1.15.7...1.15.8` | Two unexpanded contracts UNPROVEN; punctuation/occurrences preserved |
| APIs/events/CVars/widgets/commands/signatures/prose | Zero explicit local declarations or inventory entries in each category | Zero does not prove compatibility, empty linked content or native parity |

**Client line remains UNPROVEN in this literal page:** it names no Classic Era/Anniversary client. Do not infer from 1.15 or 11508, or silently expand apichanges. Separately attributed queued 1.15.9 literal wording names Classic Era, Season of Discovery and Hardcore; it supplies context only, not retroactive 1.15.8 native proof.

## Configuration and successor limits

Copied base Cargo/client-profile/manifests record **Era and Anniversary configured 11507**, their feature dependencies and manifest hashes separately. Static configuration is neither native11508 measurement nor an unsupported-client/API diagnosis. Source specifies no concrete model behavior: no model test justified, zero runtime/model/native credit.

[Queued successor inputs](../../../data/patch-api/evidence/1.15.8-session-2026-10-09/queued-successor/status.json) retain own immutable copy of sibling 1.15.9 pin/raw source. Sibling source audit reports completion at `07c93b3b5`/`308aeda25`; main gates unperformed, awaiting main integration. Placeholder only: no applied successor register, retirement, equivalence or native credit. Never cross TBC2.5, Retail2.x, Wrath3.4 or WowForever1.60 histories. Main owns actual successors/native/integration.

## Development proof

Own eight fixtures: RED eight expected assertion failures against empty accounting, retained with scaffold. Python AST formatting only. Implementation and source ledger present; GREEN not yet recorded. No check/lint/readability/coverage/broad/startup/final gates or runtime calls. Shared generator/extractor unchanged; own frozen copies retain original bytes and default flags `[]` for replay.

## Historical proof

Compact historical sealing and copied replay pending; original source proof and later receipt seals will remain separate. No native/final acceptance claim.

## Sources

- [Spec](../../specs/patch-1-15-8-source-accounting.md).
- [Frozen raw source](../../../data/patch-api/evidence/1.15.8-session-2026-10-09/source.wikitext) — exact cached Warcraft Wiki content, attributed by source-pin request URL; cache README records CC BY-SA 4.0.
- [Own fixtures](../../../data/patch-api/evidence/1.15.8-session-2026-10-09/test_source_accounting.py).

## See Also

- [Client profiles](../systems/client-profiles.md) — configuration is not historical native measurement.
- [TBC 2.5.6 audit](patch-2-5-6-api-audit.md) — separate history, no supersession credit.
